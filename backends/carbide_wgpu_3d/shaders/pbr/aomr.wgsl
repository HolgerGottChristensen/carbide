
// --- AO, Metallic, and Roughness ---
fn get_aomr(material: Material) -> vec3<f32> {
    var ambient_occlusion: f32;
    var metallic: f32;
    var perceptual_roughness: f32;

    //if (extract_material_flag(material.flags, FLAGS_AOMR_COMBINED)) {
        // In roughness texture:
        // Red: AO
        // Green: Roughness
        // Blue: Metallic
        //if (has_roughness_texture(&material)) {
            //let aomr = roughness_texture(&material, s, coords, uvdx, uvdy);
            //pixel.ambient_occlusion = material.ambient_occlusion * aomr[0];
            //pixel.perceptual_roughness = material.roughness * aomr[1];
            //pixel.metallic = material.metallic * aomr[2];
        //} else {
            ambient_occlusion = material.ambient_occlusion;
            perceptual_roughness = material.roughness;
            metallic = material.metallic;
        //}
    //} else if (extract_material_flag(material.flags, FLAGS_AOMR_BW_SPLIT)) {
        // In ao texture:
        // Red: AO
        // In metallic texture:
        // Red: Metallic
        // In roughness texture:
        // Red: Roughness
        //if (has_roughness_texture(&material)) {
            //pixel.perceptual_roughness = material.roughness * roughness_texture(&material, s, coords, uvdx, uvdy).r;
        //} else {
            //pixel.perceptual_roughness = material.roughness;
        //}

        //if (has_metallic_texture(&material)) {
            //pixel.metallic = material.metallic * metallic_texture(&material, s, coords, uvdx, uvdy).r;
        //} else {
            //pixel.metallic = material.metallic;
        //}

        //if (has_ambient_occlusion_texture(&material)) {
            //pixel.ambient_occlusion = material.ambient_occlusion * ambient_occlusion_texture(&material, s, coords, uvdx, uvdy).r;
        //} else {
            //pixel.ambient_occlusion = material.ambient_occlusion;
        //}
    //} else {
        // In ao texture:
        // Red: AO
        //
        // In roughness texture (FLAGS_AOMR_SPLIT):
        // Red: Roughness
        // Green: Metallic
        //
        // In roughness texture (FLAGS_AOMR_SWIZZLED_SPLIT):
        // Green: Roughness
        // Blue: Metallic
        //if (has_roughness_texture(&material)) {
            //let texture_read = roughness_texture(&material, s, coords, uvdx, uvdy);
            //var rm: vec2<f32>;
            //if (extract_material_flag(material.flags, FLAGS_AOMR_SWIZZLED_SPLIT)) {
                //rm = texture_read.gb;
            //} else {
                //rm = texture_read.rg;
            //}
            //pixel.perceptual_roughness = material.roughness * rm[0];
            //pixel.metallic = material.metallic * rm[1];
        //} else {
            //pixel.perceptual_roughness = material.roughness;
            //pixel.metallic = material.metallic;
        //}

        //if (has_ambient_occlusion_texture(&material)) {
            //let texture_read = ambient_occlusion_texture(&material, s, coords, uvdx, uvdy);
            //pixel.ambient_occlusion = material.ambient_occlusion * texture_read.r;
        //} else {
            //pixel.ambient_occlusion = material.ambient_occlusion;
        //}
    //}

    return vec3<f32>(ambient_occlusion, metallic, perceptual_roughness);
}