// See BRDF section of https://learnopengl.com/PBR/Theory

// Isotropic: https://github.com/google/filament/blob/3728f0660395d04d5735d85831a43480501f1c63/shaders/src/brdf.fs#L54
fn brdf_d_ggx(roughness: f32, noh: f32) -> f32 {
    /*let a2 = a * a;
    let f = (noh * a2 - noh) * noh + 1.0;
    return a2 / (PI * f * f);*/

    let oneMinusNoHSquared = 1.0 - noh * noh;
    let a = noh * roughness;
    let k = roughness / (oneMinusNoHSquared + a * a);
    let d = k * k * (1.0 / PI);
    //return saturateMediump(d);
    return d;

}

fn brdf_f_schlick_vec3(u: f32, f0: vec3<f32>, f90: f32) -> vec3<f32> {
    return f0 + (f90 - f0) * pow5(1.0 - u);
}

fn brdf_f_schlick_f32(u: f32, f0: f32, f90: f32) -> f32 {
    return f0 + (f90 - f0) * pow5(1.0 - u);
}

fn brdf_fd_burley(nov: f32, nol: f32, loh: f32, roughness: f32) -> f32 {
    let f90 = 0.5 + 2.0 * roughness * loh * loh;
    let light_scatter = brdf_f_schlick_f32(nol, 1.0, f90);
    let view_scatter = brdf_f_schlick_f32(nov, 1.0, f90);
    return light_scatter * view_scatter * (1.0 / PI);
}

fn brdf_fd_lambert() -> f32 {
    return 1.0 / PI;
}

fn brdf_v_smith_ggx_correlated(nov: f32, nol: f32, roughness: f32) -> f32 {
    let a2 = roughness * roughness;
    let ggxv = nol * sqrt((nov - a2 * nov) * nov + a2);
    let ggxl = nov * sqrt((nol - a2 * nol) * nol + a2);
    return 0.5 / (ggxl + ggxv);
}

fn diffuse(roughness: f32, nov: f32, nol: f32, loh: f32) -> f32 {
    return brdf_fd_burley(nov, nol, loh, roughness);
    //return brdf_fd_lambert();
}