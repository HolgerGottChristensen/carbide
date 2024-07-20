//!include pbr/brdf.wgsl
//!include pbr/pixel_data.wgsl

fn diffuseLobe(pixel: PixelData, nov: f32, nol: f32, loh: f32) -> vec3<f32> {
   return pixel.diffuse_color * diffuse(pixel.roughness, nov, nol, loh);
}

fn specularLobe(pixel: PixelData, light_dir: vec3<f32>, h: vec3<f32>, nov: f32, nol: f32, noh: f32, loh: f32) -> vec3<f32> {
    let d = brdf_d_ggx(pixel.roughness, noh);
    let v = brdf_v_smith_ggx_correlated(nov, nol, pixel.roughness);

    let f90 = saturate(dot(pixel.f0, vec3(50.0 * 0.33)));
    let f = brdf_f_schlick_vec3(loh, pixel.f0, f90);
    return (d * v) * f;
}

fn surface_shading(light_dir: vec3<f32>, intensity: vec3<f32>, pixel: PixelData, view_pos: vec3<f32>, occlusion: f32) -> vec3<f32> {
    let n = pixel.normal;
    let h = normalize(view_pos + light_dir);

    //let nov = abs(dot(n, view_pos)) + 0.00001;
    let nov = max(dot(n, view_pos), 0.00001);
    let nol = saturate(dot(n, light_dir));
    let noh = saturate(dot(n, h));
    let loh = saturate(dot(light_dir, h));


    // TODO: figure out how they generate their lut
    let energy_comp = 1.0;

    // specular
    let fr = specularLobe(pixel, light_dir, h, nov, nol, noh, loh);
    // diffuse
    let fd = diffuseLobe(pixel, nov, nol, loh);

    let color = fd + fr * energy_comp;

    let light_attenuation = 1.0;

    return (color * intensity) * (light_attenuation * nol * occlusion);
}