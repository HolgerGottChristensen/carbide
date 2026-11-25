// A point or spot light. A point light is a spot light whose cone lets all light through: a cone
// scale of 0 and an offset of 1.
struct PunctualLight {
    /// The light's position in world space.
    position: vec4<f32>,
    /// The way a spot light shines in world space.
    direction: vec4<f32>,
    /// The light's linear colour times its intensity in candela.
    color: vec3<f32>,
    /// Where the light has faded to nothing, in metres; 0 for no range.
    range: f32,
    cone_scale: f32,
    cone_offset: f32,
}

struct PunctualLights {
    count: u32,
    data: array<PunctualLight>,
}

// The inverse square law, faded smoothly to nothing at `range` when it is above 0, as glTF's
// KHR_lights_punctual recommends: clamp(1 - (d / range)^4, 0, 1) / d^2.
fn distance_attenuation(distance: f32, range: f32) -> f32 {
    let inverse_square = 1.0 / max(distance * distance, 0.0001);

    if (range <= 0.0) {
        return inverse_square;
    }

    let ratio = distance / range;
    let window = saturate(1.0 - ratio * ratio * ratio * ratio);
    return window * inverse_square;
}

// How much of a spot light reaches a point `cos_angle` off its axis, as glTF's
// KHR_lights_punctual recommends: smooth between the cone's angles, squared.
fn cone_attenuation(cos_angle: f32, scale: f32, offset: f32) -> f32 {
    let t = saturate(cos_angle * scale + offset);
    return t * t;
}
