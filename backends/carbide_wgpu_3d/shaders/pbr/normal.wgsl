
fn get_normal(material: Material, s: sampler, vs_out: VertexOutput, coords: vec2<f32>, ddx: vec2<f32>, ddy: vec2<f32>) -> vec3<f32>{
    var normal: vec3<f32>;
    let normal_has_texture_enabled = bool((material.texture_enable >> 1u) & 0x1u);
    if (normal_has_texture_enabled) {
        var normal_inner: vec3<f32>;
        let texture_read = textureSampleGrad(normal_tex, s, coords, ddx, ddy);
    //    if (extract_material_flag(material.flags, FLAGS_BICOMPONENT_NORMAL)) {
    //        var bicomp: vec2<f32>;
    //        if (extract_material_flag(material.flags, FLAGS_SWIZZLED_NORMAL)) {
    //            bicomp = texture_read.ag;
    //        } else {
    //            bicomp = texture_read.rg;
    //        }
    //        bicomp = bicomp * 2.0 - 1.0;
    //        let bicomp_sq = bicomp * bicomp;

    //        normal_inner = vec3<f32>(bicomp, sqrt(1.0 - bicomp_sq.r - bicomp_sq.g));
    //    } else {
            normal_inner = normalize(texture_read.rgb * 2.0 - 1.0);
            //normal_inner = (texture_read.rgb * 2.0) - 1.0;
            //normal_inner = texture_read.rgb;
    //    }
        //if (extract_material_flag(material.flags, FLAGS_YDOWN_NORMAL)) {
        //    normal_inner.y = -normal_inner.y;
        //}
        let normal_norm = normalize(vs_out.normal);
        let tangent_norm = normalize(vs_out.tangent);
        let bitangent = cross(normal_norm, tangent_norm);

        let tbn = mat3x3(tangent_norm, bitangent, normal_norm);

        normal = tbn * normal_inner;
        //normal = normal_inner;
    } else {
        normal = vs_out.normal;
    }
    return normalize(normal);
    //return normal;
}