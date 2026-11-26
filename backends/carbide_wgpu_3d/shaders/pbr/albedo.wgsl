
fn get_albedo(material: Material, vs_out: VertexOutput, uv: TexCoords) -> vec4<f32> {
    var albedo = vec4<f32>(1.0);

    // The albedo texture, when it is loaded. Images are stored with premultiplied alpha, so the
    // colour is divided by alpha to light the surface's own colour. Read through the sRGB view, it
    // is already linear.
    if ((material.texture_enable & 0x1u) != 0u) {
        let texel = sample_material_texture(albedo_tex, material, SLOT_ALBEDO, uv);
        albedo = vec4<f32>(select(vec3<f32>(0.0), texel.rgb / texel.a, texel.a > 0.0), texel.a);
    }

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
