// --- Ambient occlusion, metallic and roughness ---
//
// The material's values, each multiplied by its map when one is loaded. The maps are data, read
// through their linear views, and laid out as the material's `AoMRTextures` says:
// - AOMR_COMBINED: one map, in the roughness slot: occlusion in red, roughness in green, metallic
//   in blue, as glTF packs them.
// - AOMR_SPLIT: occlusion in the occlusion map's red; roughness and metallic in the roughness
//   map's red and green.
// - AOMR_SWIZZLED_SPLIT: as split, with roughness and metallic in green and blue.
// - AOMR_BW_SPLIT: three greyscale maps, each read from red.
// The occlusion map is weakened by the material's occlusion strength first.
// Returns occlusion, metallic and perceptual roughness.
// An occlusion map's value with the material's strength: 1 + strength · (texel − 1), so 0 gives no
// occlusion and 1 the map as it is (glTF, `occlusionTexture.strength`).
fn occlusion_with_strength(material: Material, texel: f32) -> f32 {
    return 1.0 + material.occlusion_strength * (texel - 1.0);
}

fn get_aomr(material: Material, uv: TexCoords) -> vec3<f32> {
    var ambient_occlusion = material.ambient_occlusion;
    var metallic = material.metallic;
    var perceptual_roughness = material.roughness;

    let has_ao = (material.texture_enable & 0x4u) != 0u;
    let has_metallic = (material.texture_enable & 0x8u) != 0u;
    let has_roughness = (material.texture_enable & 0x10u) != 0u;

    if ((material.flags & 0x40u) != 0u) {
        if (has_roughness) {
            let orm = sample_material_texture(roughness_tex, material, SLOT_ROUGHNESS, uv);
            ambient_occlusion *= occlusion_with_strength(material, orm.r);
            perceptual_roughness *= orm.g;
            metallic *= orm.b;
        }
    } else if ((material.flags & 0x200u) != 0u) {
        if (has_ao) {
            ambient_occlusion *= occlusion_with_strength(material, sample_material_texture(ambient_occlusion_tex, material, SLOT_OCCLUSION, uv).r);
        }
        if (has_metallic) {
            metallic *= sample_material_texture(metallic_tex, material, SLOT_METALLIC, uv).r;
        }
        if (has_roughness) {
            perceptual_roughness *= sample_material_texture(roughness_tex, material, SLOT_ROUGHNESS, uv).r;
        }
    } else {
        if (has_ao) {
            ambient_occlusion *= occlusion_with_strength(material, sample_material_texture(ambient_occlusion_tex, material, SLOT_OCCLUSION, uv).r);
        }
        if (has_roughness) {
            let texel = sample_material_texture(roughness_tex, material, SLOT_ROUGHNESS, uv);
            var roughness_metallic = texel.rg;
            if ((material.flags & 0x80u) != 0u) {
                roughness_metallic = texel.gb;
            }
            perceptual_roughness *= roughness_metallic.x;
            metallic *= roughness_metallic.y;
        }
    }

    return vec3<f32>(ambient_occlusion, metallic, perceptual_roughness);
}
