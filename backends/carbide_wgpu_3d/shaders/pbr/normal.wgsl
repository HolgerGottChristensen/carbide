
fn get_normal(vs_out: VertexOutput) -> vec3<f32>{
    var normal: vec3<f32>;
    //if (has_normal_texture(&material)) {
    //    let texture_read = normal_texture(&material, s, coords, uvdx, uvdy);
    //    var normal: vec3<f32>;
    //    if (extract_material_flag(material.flags, FLAGS_BICOMPONENT_NORMAL)) {
    //        var bicomp: vec2<f32>;
    //        if (extract_material_flag(material.flags, FLAGS_SWIZZLED_NORMAL)) {
    //            bicomp = texture_read.ag;
    //        } else {
    //            bicomp = texture_read.rg;
    //        }
    //        bicomp = bicomp * 2.0 - 1.0;
    //        let bicomp_sq = bicomp * bicomp;

    //        normal = vec3<f32>(bicomp, sqrt(1.0 - bicomp_sq.r - bicomp_sq.g));
    //    } else {
    //        normal = normalize(texture_read.rgb * 2.0 - 1.0);
    //    }
    //    if (extract_material_flag(material.flags, FLAGS_YDOWN_NORMAL)) {
    //        normal.y = -normal.y;
    //    }
    //    let normal_norm = normalize(vs_out.normal);
    //    let tangent_norm = normalize(vs_out.tangent);
    //    let bitangent = cross(normal_norm, tangent_norm);

    //    let tbn = mat3x3(tangent_norm, bitangent, normal_norm);

    //    pixel.normal = tbn * normal;
    //} else {
        normal = vs_out.normal;
    //}
    return normalize(normal);
}