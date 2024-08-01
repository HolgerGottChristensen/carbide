
fn get_albedo(material: Material, s: sampler, vs_out: VertexOutput, coords: vec2<f32>, ddx: vec2<f32>, ddy: vec2<f32>) -> vec4<f32> {
    var albedo: vec4<f32>;
    let albedo_has_texture_enabled = bool((material.texture_enable >> 0u) & 0x1u);
    // if (extract_material_flag(material.flags, FLAGS_ALBEDO_ACTIVE)) {
        if (albedo_has_texture_enabled) {
            albedo = textureSampleGrad(albedo_tex, s, coords, ddx, ddy);
        } else {
            albedo = vec4<f32>(1.0);
        }
        //if (extract_material_flag(material.flags, FLAGS_ALBEDO_BLEND)) {
        //    if (extract_material_flag(material.flags, FLAGS_ALBEDO_VERTEX_SRGB)) {
        //        pixel.albedo *= vec4<f32>(srgb_display_to_scene(vs_out.color.rgb), vs_out.color.a);
        //    } else {
                //albedo *= vs_out.color_0;
        //    }
        //}
    // } else {
    //     albedo = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    // }
    albedo *= material.albedo;

    return albedo;
}