// Khronos PBR Neutral tone mapping: keeps base colours as they are under normal lighting, and
// compresses only highlights that would clip, desaturating them towards white.
// https://github.com/KhronosGroup/ToneMapping/tree/main/PBR_Neutral
fn tonemap_pbr_neutral(color_in: vec3<f32>) -> vec3<f32> {
    let start_compression = 0.8 - 0.04;
    let desaturation = 0.15;

    var color = color_in;
    let x = min(color.r, min(color.g, color.b));
    let offset = select(0.04, x - 6.25 * x * x, x < 0.08);
    color -= offset;

    let peak = max(color.r, max(color.g, color.b));
    if (peak < start_compression) {
        return color;
    }

    let d = 1.0 - start_compression;
    let new_peak = 1.0 - d * d / (peak + d - start_compression);
    color *= new_peak / peak;

    let g = 1.0 - 1.0 / (desaturation * (peak - new_peak) + 1.0);
    return mix(color, new_peak * vec3<f32>(1.0, 1.0, 1.0), g);
}
