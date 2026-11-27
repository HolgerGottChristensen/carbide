// Image-based lighting: the light from the scene's environment image, which replaces the
// hemisphere's ambient light while set. Split-sum, as Filament and Bevy: diffuse light from the
// environment's spherical harmonics times the diffuse colour, plus the environment's filtered
// reflections times the DFG term, F0 · scale + bias. The maps are made by `EnvironmentMaps`.
//!include environment/harmonics.wgsl

// From a direction in view space to where to read the environment: world space, turned back by
// the environment's rotation around the vertical axis (cosine and sine in the uniforms).
fn environment_direction(view_direction: vec3<f32>) -> vec3<f32> {
    return environment_turn((camera.inv_view * vec4<f32>(view_direction, 0.0)).xyz);
}

// From a direction in world space to where to read the environment.
fn environment_turn(d: vec3<f32>) -> vec3<f32> {
    let c = uniforms.environment_rotation.x;
    let s = uniforms.environment_rotation.y;
    return vec3<f32>(c * d.x - s * d.z, d.y, s * d.x + c * d.z);
}

// The light a white Lambertian surface facing `n` (environment space) reflects: the harmonics
// evaluated for it. Nine coefficients ring around very bright spots such as the sun, and can go
// below zero; light can't.
fn environment_diffuse(n: vec3<f32>) -> vec3<f32> {
    var sum = vec3<f32>(0.0);
    for (var i = 0u; i < 9u; i++) {
        sum += textureLoad(environment_harmonics, vec2<u32>(i, 0u), 0).rgb * sh_basis(n, i);
    }
    return max(sum, vec3<f32>(0.0));
}

// The filtered reflection along `r` (environment space) for `perceptual_roughness`: level
// roughness × (levels - 1), as the levels were filtered.
fn environment_reflection(r: vec3<f32>, perceptual_roughness: f32) -> vec3<f32> {
    let level = perceptual_roughness * (uniforms.environment_levels - 1.0);
    return textureSampleLevel(environment_radiance, primary_sampler, r, level).rgb;
}

// The DFG table's scale and bias for N·V and perceptual roughness, read between the outer texels'
// middles, as the sampler repeats.
fn environment_dfg(nov: f32, perceptual_roughness: f32) -> vec2<f32> {
    let half_texel = 0.5 / f32(textureDimensions(environment_dfg_table).x);
    let uv = clamp(vec2<f32>(nov, perceptual_roughness), vec2<f32>(half_texel), vec2<f32>(1.0 - half_texel));
    return textureSampleLevel(environment_dfg_table, primary_sampler, uv, 0.0).rg;
}

// The environment's light on `pixel`, seen along `v` (view space, towards the eye), before
// occlusion. A clear coat reflects the environment at its own roughness, and dims what is under
// it by what it reflects, as Filament's `evaluateClearCoatIBL`.
fn environment_light(pixel: PixelData, v: vec3<f32>) -> vec3<f32> {
    let nov = max(dot(pixel.normal, v), 0.0001);
    let r = environment_direction(reflect(-v, pixel.normal));

    let dfg = environment_dfg(nov, pixel.perceptual_roughness);
    var specular = (pixel.f0 * dfg.x + dfg.y) * environment_reflection(r, pixel.perceptual_roughness);
    var diffuse = environment_diffuse(environment_direction(pixel.normal)) * pixel.diffuse_color;

    if (pixel.clear_coat > 0.0) {
        let fresnel = brdf_f_schlick_f32(nov, 0.04, 1.0) * pixel.clear_coat;
        diffuse *= 1.0 - fresnel;
        specular *= 1.0 - fresnel;
        specular += environment_reflection(r, pixel.clear_coat_perceptual_roughness) * fresnel;
    }

    return (diffuse + specular) * uniforms.environment_intensity;
}

// The hemisphere's light in a direction (view space): the sky's facing `up`, the ground's facing
// down, blended in between.
fn hemisphere(direction: vec3<f32>, up: vec3<f32>) -> vec3<f32> {
    return mix(uniforms.ground.rgb, uniforms.sky.rgb, dot(direction, up) * 0.5 + 0.5);
}

// The hemisphere's light on `pixel`, as a two-colour environment, as Bevy's ambient light
// (`ambient.wgsl`): diffuse light from the normal's direction times the diffuse colour, and
// reflected light times F0 · scale + bias from the DFG table, so metals reflect it in their colour
// instead of being lit as if matte. The reflection is read along R leaned towards the normal as
// roughness grows, as Filament's `getSpecularDominantDirection`: a blurred reflection of a smooth
// gradient averages it around the normal. A clear coat as in `environment_light`.
fn hemisphere_light(pixel: PixelData, v: vec3<f32>, up: vec3<f32>) -> vec3<f32> {
    let nov = max(dot(pixel.normal, v), 0.0001);
    let r = reflect(-v, pixel.normal);
    let dominant = normalize(mix(r, pixel.normal, pixel.roughness));

    let dfg = environment_dfg(nov, pixel.perceptual_roughness);
    var specular = (pixel.f0 * dfg.x + dfg.y) * hemisphere(dominant, up);
    var diffuse = hemisphere(pixel.normal, up) * pixel.diffuse_color;

    if (pixel.clear_coat > 0.0) {
        let fresnel = brdf_f_schlick_f32(nov, 0.04, 1.0) * pixel.clear_coat;
        diffuse *= 1.0 - fresnel;
        specular *= 1.0 - fresnel;
        specular += hemisphere(r, up) * fresnel;
    }

    return diffuse + specular;
}

// The background, when a view shows the environment: a triangle covering the view at the far
// plane (depth 0, reverse-Z), so it is drawn only where nothing else was, after the opaque meshes
// and before the blended ones.
struct BackgroundOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) clip: vec2<f32>,
}

@vertex
fn background_vs(@builtin(vertex_index) vertex: u32) -> BackgroundOutput {
    let clip = vec2<f32>(f32((vertex << 1u) & 2u), f32(vertex & 2u)) * 2.0 - 1.0;
    var out: BackgroundOutput;
    out.position = vec4<f32>(clip, 0.0, 1.0);
    out.clip = clip;
    return out;
}

// The environment in the pixel's direction: from a point on the near plane to one further along,
// both finite with a perspective camera's infinite far plane. With an orthographic camera every
// pixel looks the same way. At the background's blur, the environment's intensity, exposed and
// tone mapped as the scene, and opaque.
@fragment
fn background_fs(in: BackgroundOutput) -> @location(0) vec4<f32> {
    let near = camera.inv_view_proj * vec4<f32>(in.clip, 1.0, 1.0);
    let further = camera.inv_view_proj * vec4<f32>(in.clip, 0.5, 1.0);
    let direction = normalize(further.xyz / further.w - near.xyz / near.w);
    let light = textureSampleLevel(environment_radiance, primary_sampler, environment_turn(direction), uniforms.background_level).rgb
        * uniforms.environment_intensity;
    return vec4<f32>(tonemap(light * uniforms.exposure, uniforms.tonemapping), 1.0);
}
