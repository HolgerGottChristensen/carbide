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

// Narkowicz's fit of the ACES filmic curve: more contrast, and shifts hues in bright areas.
// https://knarkowicz.wordpress.com/2016/01/06/aces-filmic-tone-mapping-curve/
fn tonemap_aces(x: vec3<f32>) -> vec3<f32> {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    return clamp((x * (a * x + b)) / (x * (c * x + d) + e), vec3<f32>(0.0), vec3<f32>(1.0));
}

// The tone mapping operator `kind` selects: 0 PBR Neutral, 1 ACES, 2 none (clamp).
fn tonemap(color: vec3<f32>, kind: u32) -> vec3<f32> {
    switch kind {
        case 1u: {
            return tonemap_aces(color);
        }
        case 2u: {
            return clamp(color, vec3<f32>(0.0), vec3<f32>(1.0));
        }
        default: {
            return tonemap_pbr_neutral(color);
        }
    }
}
