// The clear coat's strength and perceptual roughness, each multiplied by its map when one is
// loaded, read through the linear view, laid out as the material's `ClearcoatTextures` says:
// - CC_GLTF_COMBINED: one map, in the clear coat slot: strength in red, roughness in green.
// - CC_GLTF_SPLIT: strength in its map's red, roughness in its map's green, as glTF's
//   KHR_materials_clearcoat reads them.
// - CC_BW_SPLIT: two greyscale maps, each read from red.
fn get_clear_coat(material: Material, uv: TexCoords) -> vec3<f32> {
    var clear_coat = material.clear_coat;
    var clear_coat_perceptual_roughness = material.clear_coat_roughness;

    let has_strength = (material.texture_enable & 0x20u) != 0u;
    let has_roughness = (material.texture_enable & 0x40u) != 0u;

    if ((material.flags & 0x400u) != 0u) {
        if (has_strength) {
            let texel = sample_material_texture(clear_coat_tex, material, SLOT_CLEAR_COAT, uv);
            clear_coat *= texel.r;
            clear_coat_perceptual_roughness *= texel.g;
        }
    } else {
        if (has_strength) {
            clear_coat *= sample_material_texture(clear_coat_tex, material, SLOT_CLEAR_COAT, uv).r;
        }
        if (has_roughness) {
            let texel = sample_material_texture(clear_coat_roughness_tex, material, SLOT_CLEAR_COAT_ROUGHNESS, uv);
            if ((material.flags & 0x800u) != 0u) {
                clear_coat_perceptual_roughness *= texel.g;
            } else {
                clear_coat_perceptual_roughness *= texel.r;
            }
        }
    }

    // 0.045 from https://github.com/google/filament/blob/3728f0660395d04d5735d85831a43480501f1c63/shaders/src/common_material.fs#L4
    clear_coat_perceptual_roughness = clamp(clear_coat_perceptual_roughness, 0.045, 1.0);
    let clear_coat_roughness = perceptual_roughness_to_roughness(clear_coat_perceptual_roughness);

    return vec3<f32>(clear_coat, clear_coat_perceptual_roughness, clear_coat_roughness);
}