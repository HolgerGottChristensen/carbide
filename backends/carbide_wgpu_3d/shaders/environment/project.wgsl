// Projects an environment cube onto 9 spherical harmonics, for diffuse light: a 9 × 1 target,
// one fragment per coefficient, each summing every texel of a small level of the cube, weighted
// by the texel's solid angle.
//
// Each coefficient is convolved with the cosine lobe (Ramamoorthi and Hanrahan, "An Efficient
// Representation for Irradiance Environment Maps": π, 2π/3 and π/4 for the three bands) and
// divided by π, so evaluating them for a normal gives the irradiance over π: the light a white
// Lambertian surface reflects, as Filament stores them.
//!include environment/face.wgsl
//!include environment/harmonics.wgsl

// The level to sum, read texel by texel, its faces as layers.
@group(0) @binding(0)
var source: texture_2d_array<f32>;

@fragment
fn project_fs(in: FaceOutput) -> @location(0) vec4<f32> {
    let index = u32(in.position.x);
    let size = textureDimensions(source).x;

    var sum = vec3<f32>(0.0);
    for (var face = 0u; face < 6u; face++) {
        for (var y = 0u; y < size; y++) {
            for (var x = 0u; x < size; x++) {
                let uv = (vec2<f32>(f32(x), f32(y)) + 0.5) / f32(size);
                let c = 2.0 * uv - 1.0;
                // The solid angle of the texel: its area on the face, (2 / size)², over the cube
                // of its distance from the centre.
                let solid_angle = 4.0 / (f32(size * size) * pow(1.0 + dot(c, c), 1.5));
                let radiance = textureLoad(source, vec2<u32>(x, y), face, 0).rgb;
                sum += radiance * sh_basis(face_direction(face, uv), index) * solid_angle;
            }
        }
    }

    let band = select(select(2.0, 1.0, index < 4u), 0.0, index == 0u);
    let lobe = select(select(0.25, 2.0 / 3.0, band == 1.0), 1.0, band == 0.0);
    return vec4<f32>(sum * lobe, 1.0);
}
