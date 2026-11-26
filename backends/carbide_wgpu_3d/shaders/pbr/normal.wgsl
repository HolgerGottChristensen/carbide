// The surface normal, in view space: the vertex normal, bent by the normal map when the material
// has one loaded.
//
// Normal maps are in tangent space: x along the tangent (increasing u), y along "up" in the image
// (decreasing v), z out of the surface. Image-up is cross(normal, tangent) · w, glTF's bitangent,
// where w is the tangent's sign: -1 on the mirrored half of a model whose texture is mirrored.
// That is y for maps whose green points up (OpenGL); maps whose green points down (DirectX,
// YDOWN_NORMAL) have y flipped.
fn get_normal(material: Material, vs_out: VertexOutput, coords: vec2<f32>, ddx: vec2<f32>, ddy: vec2<f32>) -> vec3<f32> {
    let normal = normalize(vs_out.normal);

    // No normal map loaded (bit 1), or a mesh without tangents: the vertex normal.
    if ((material.texture_enable & 0x2u) == 0u || dot(vs_out.tangent.xyz, vs_out.tangent.xyz) < 1e-8) {
        return normal;
    }

    let texel = sample_material_texture(normal_tex, material, coords, ddx, ddy);

    var tangent_space: vec3<f32>;
    if ((material.flags & 0x8u) != 0u) {
        // Two components (BICOMPONENT_NORMAL), from red and green, or alpha and green when
        // swizzled (SWIZZLED_NORMAL); z follows from unit length.
        var xy = texel.rg;
        if ((material.flags & 0x10u) != 0u) {
            xy = texel.ag;
        }
        xy = xy * 2.0 - 1.0;
        tangent_space = vec3<f32>(xy, sqrt(max(1.0 - dot(xy, xy), 0.0)));
    } else {
        tangent_space = texel.rgb * 2.0 - 1.0;
    }

    if ((material.flags & 0x20u) != 0u) {
        tangent_space.y = -tangent_space.y;
    }

    // The material's normal scale steepens or flattens the map (glTF, `normalTexture.scale`); the
    // result is normalised below.
    tangent_space = vec3<f32>(tangent_space.xy * material.normal_scale, tangent_space.z);

    // The tangent, made perpendicular to the interpolated normal again.
    let tangent = normalize(vs_out.tangent.xyz - normal * dot(normal, vs_out.tangent.xyz));
    let image_up = cross(normal, tangent) * select(1.0, -1.0, vs_out.tangent.w < 0.0);

    return normalize(tangent * tangent_space.x + image_up * tangent_space.y + normal * tangent_space.z);
}
