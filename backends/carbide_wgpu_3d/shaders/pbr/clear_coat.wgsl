
fn get_clear_coat(material: Material) -> vec3<f32> {
    var clear_coat: f32;
    var clear_coat_perceptual_roughness: f32;
    //if (extract_material_flag(material.flags, FLAGS_CC_GLTF_COMBINED)) {
    //    if (has_clear_coat_texture(&material)) {
    //        let texture_read = clear_coat_texture(&material, s, coords, uvdx, uvdy);
    //        pixel.clear_coat = material.clear_coat * texture_read.r;
    //        pixel.clear_coat_perceptual_roughness = material.clear_coat_roughness * texture_read.g;
    //    } else {
            clear_coat = material.clear_coat;
            clear_coat_perceptual_roughness = material.clear_coat_roughness;
    //    }
    //} else {
    //    if (has_clear_coat_texture(&material)) {
    //        pixel.clear_coat = material.clear_coat * clear_coat_texture(&material, s, coords, uvdx, uvdy).r;
    //    } else {
    //        pixel.clear_coat = material.clear_coat;
    //    }

    //    if (has_clear_coat_roughness_texture(&material)) {
    //        let texture_read = clear_coat_roughness_texture(&material, s, coords, uvdx, uvdy);

    //        if (extract_material_flag(material.flags, FLAGS_CC_GLTF_SPLIT)) {
    //            pixel.clear_coat_perceptual_roughness = material.clear_coat_roughness * texture_read.g;
    //        } else {
    //            pixel.clear_coat_perceptual_roughness = material.clear_coat_roughness * texture_read.r;
    //        }
    //    } else {
    //        pixel.clear_coat_perceptual_roughness = material.clear_coat_roughness;
    //    }
    //}

    // 0.045 from https://github.com/google/filament/blob/3728f0660395d04d5735d85831a43480501f1c63/shaders/src/common_material.fs#L4
    clear_coat_perceptual_roughness = clamp(clear_coat_perceptual_roughness, 0.045, 1.0);
    let clear_coat_roughness = perceptual_roughness_to_roughness(clear_coat_perceptual_roughness);

    return vec3<f32>(clear_coat, clear_coat_perceptual_roughness, clear_coat_roughness);
}