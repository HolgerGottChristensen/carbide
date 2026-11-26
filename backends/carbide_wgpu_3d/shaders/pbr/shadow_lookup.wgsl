// How much of a light reaches a point, from the light's shadow map: 1 lit, 0 in shadow, and in
// between at shadow edges. The point is moved `bias` metres along its normal first, so surfaces
// don't shadow themselves. Shadow maps are reverse-Z: greater depth is nearer the light. A map
// fills the top-left `scale` of its layer.
fn shadow_visibility(view_proj: mat4x4<f32>, layer: i32, bias: f32, scale: f32, world_position: vec3<f32>, world_normal: vec3<f32>) -> f32 {
    if (layer < 0) {
        return 1.0;
    }

    let clip = view_proj * vec4<f32>(world_position + world_normal * bias, 1.0);
    if (clip.w <= 0.0) {
        return 1.0;
    }

    let ndc = clip.xyz / clip.w;
    if (abs(ndc.x) > 1.0 || abs(ndc.y) > 1.0 || ndc.z < 0.0 || ndc.z > 1.0) {
        // Outside the map: nothing was drawn there to cast a shadow.
        return 1.0;
    }

    let uv = vec2<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5) * scale;

    // Percentage-closer filtering: five comparisons, each blended 2x2 by the comparison sampler.
    var lit = textureSampleCompareLevel(shadow_maps, shadow_sampler, uv, layer, ndc.z);
    lit += textureSampleCompareLevel(shadow_maps, shadow_sampler, uv, layer, ndc.z, vec2<i32>(0, 1));
    lit += textureSampleCompareLevel(shadow_maps, shadow_sampler, uv, layer, ndc.z, vec2<i32>(0, -1));
    lit += textureSampleCompareLevel(shadow_maps, shadow_sampler, uv, layer, ndc.z, vec2<i32>(1, 0));
    lit += textureSampleCompareLevel(shadow_maps, shadow_sampler, uv, layer, ndc.z, vec2<i32>(-1, 0));
    return lit * 0.2;
}
