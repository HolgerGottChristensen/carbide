//! Helpers shared by the rendering tests in this folder. Each test file is its own crate and
//! declares `mod common;`, so not every file uses every helper.
#![allow(dead_code)]

use carbide_3d::{Mesh, Vertex};
use carbide_core::color::WHITE;
use carbide_core::math::{InnerSpace, Vector3};

// Linking the wgpu 3D backend registers its plugin, so scenes in these tests render with it.
use carbide_wgpu_3d as _;

/// A cube centred on the origin with sides of `size`, with flat normals: four vertices per face,
/// wound clockwise as seen from outside the cube, the front face of the renderer. Vertex colours
/// are white, so snapshots don't depend on `Vertex::new`'s random colours.
pub fn cube(size: f32) -> Mesh {
    let half = size / 2.0;
    let normals: [Vector3<f32>; 6] = [
        Vector3::unit_x(), -Vector3::unit_x(),
        Vector3::unit_y(), -Vector3::unit_y(),
        Vector3::unit_z(), -Vector3::unit_z(),
    ];

    let mut vertices = vec![];
    let mut indices = vec![];

    for normal in normals {
        // Seen from outside, looking along -normal: pick an up direction, and the right
        // direction follows (left-handed: right = up × view direction).
        let up = if normal.y.abs() > 0.5 { Vector3::unit_z() } else { Vector3::unit_y() };
        let right = up.cross(-normal).normalize();

        let center = normal * half;
        let corners = [
            center + (-right + up) * half,
            center + (right + up) * half,
            center + (right - up) * half,
            center + (-right - up) * half,
        ];

        let first = vertices.len() as u32;
        for corner in corners {
            let mut vertex = Vertex::new(corner).normal(normal);
            vertex.color_0 = WHITE;
            vertex.color_1 = WHITE;
            vertices.push(vertex);
        }

        indices.extend([first, first + 1, first + 2, first, first + 2, first + 3]);
    }

    Mesh::new(vertices, indices)
}
