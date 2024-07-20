

fn get_reflectance(material: Material) -> f32 {
    var reflectance: f32;
    //if (has_reflectance_texture(&material)) {
    //    pixel.reflectance = material.reflectance * reflectance_texture(&material, s, coords, uvdx, uvdy).r;}
    //} else {
        reflectance = material.reflectance;
    //}
    return reflectance;
}