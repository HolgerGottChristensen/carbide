// The 9 real spherical-harmonic basis functions of bands 0 to 2, in Sloan's order ("Stupid
// Spherical Harmonics Tricks"), shared by the pass that projects an environment onto them and the
// shading that evaluates them.

fn sh_basis(d: vec3<f32>, index: u32) -> f32 {
    switch index {
        case 0u: { return 0.282095; }
        case 1u: { return 0.488603 * d.y; }
        case 2u: { return 0.488603 * d.z; }
        case 3u: { return 0.488603 * d.x; }
        case 4u: { return 1.092548 * d.x * d.y; }
        case 5u: { return 1.092548 * d.y * d.z; }
        case 6u: { return 0.315392 * (3.0 * d.z * d.z - 1.0); }
        case 7u: { return 1.092548 * d.x * d.z; }
        default: { return 0.546274 * (d.x * d.x - d.y * d.y); }
    }
}
