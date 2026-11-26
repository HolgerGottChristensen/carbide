
fn get_albedo(material: Material, s: sampler, vs_out: VertexOutput, coords: vec2<f32>, ddx: vec2<f32>, ddy: vec2<f32>) -> vec4<f32> {
    var albedo: vec4<f32>;
    //let albedo_has_texture_enabled = bool((material.texture_enable >> 0u) & 0x1u);
    // if (extract_material_flag(material.flags, FLAGS_ALBEDO_ACTIVE)) {
    //if (albedo_has_texture_enabled) {
    //    albedo = textureSampleGrad(albedo_tex, s, coords, ddx, ddy);
    //} else {
        albedo = vec4<f32>(1.0);
    //}

    // Vertex colours, when the material uses them (ALBEDO_BLEND), converted from sRGB when they
    // are in it (ALBEDO_VERTEX_SRGB).
    if ((material.flags & 0x2u) != 0u) {
        var vertex_color = vs_out.color_0;
        if ((material.flags & 0x4u) != 0u) {
            vertex_color = vec4<f32>(srgb_to_linear(vertex_color.rgb), vertex_color.a);
        }
        albedo *= vertex_color;
    }
    // } else {
    //     albedo = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    // }
    albedo *= material.albedo;

    return albedo;
}
// An sRGB colour converted to linear, with the exact sRGB curve.
fn srgb_to_linear(color: vec3<f32>) -> vec3<f32> {
    let low = color / 12.92;
    let high = pow((color + 0.055) / 1.055, vec3<f32>(2.4));
    return select(high, low, color <= vec3<f32>(0.04045));
}
