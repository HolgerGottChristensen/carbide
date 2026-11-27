// Draws a level of the filtered reflection cube: the environment convolved with GGX for the
// level's roughness, with the view along the normal, as the split-sum approximation assumes
// (Karis, "Real Shading in Unreal Engine 4"). Samples read the source's mip level whose texels
// cover about the sample's solid angle, one level further, so few samples don't alias, and a
// small, very bright sun doesn't show as blotches ("filtered importance sampling": Colbert and
// Křivánek, GPU Gems 3, chapter 20, with its +1; Bevy's `environment_filter.wesl` leaves it out).
//!include environment/face.wgsl

struct Params {
    target_size: f32,
    // The level's perceptual roughness.
    roughness: f32,
    sample_count: u32,
    // The size of the source's first level.
    source_size: f32,
}

@group(0) @binding(0)
var source: texture_cube<f32>;
@group(0) @binding(1)
var source_sampler: sampler;
@group(0) @binding(2)
var<uniform> params: Params;

@fragment
fn radiance_fs(in: FaceOutput) -> @location(0) vec4<f32> {
    let n = face_direction(in.face, in.position.xy / params.target_size);
    let alpha = params.roughness * params.roughness;
    let to_world = frame(n);
    // The solid angle of a texel of the source's first level.
    let texel_angle = 4.0 * ENV_PI / (6.0 * params.source_size * params.source_size);

    var sum = vec3<f32>(0.0);
    var weight = 0.0;
    for (var i = 0u; i < params.sample_count; i++) {
        let h_local = importance_sample_ggx(hammersley(i, params.sample_count), alpha);
        let h = to_world * h_local;
        // The view is the normal, so the light is the normal reflected about the half vector.
        let l = 2.0 * dot(n, h) * h - n;
        let nol = dot(n, l);
        if (nol > 0.0) {
            // The probability of `l`: D · N·H / (4 V·H), and V·H = N·H here.
            let pdf = d_ggx(h_local.z, alpha) / 4.0;
            let sample_angle = 1.0 / (f32(params.sample_count) * pdf + 0.0001);
            let level = max(0.5 * log2(sample_angle / texel_angle) + 1.0, 0.0);
            sum += textureSampleLevel(source, source_sampler, l, level).rgb * nol;
            weight += nol;
        }
    }

    return vec4<f32>(sum / max(weight, 0.0001), 1.0);
}
