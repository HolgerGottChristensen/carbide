
fn get_albedo(material: Material, s: sampler, vs_out: VertexOutput) -> vec4<f32> {
    var albedo: vec4<f32>;
    // if (extract_material_flag(material.flags, FLAGS_ALBEDO_ACTIVE)) {
        //if (has_albedo_texture(&material)) {
        //    pixel.albedo = albedo_texture(&material, s, coords, uvdx, uvdy);
        //} else {
            albedo = vec4<f32>(1.0);
        //}
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