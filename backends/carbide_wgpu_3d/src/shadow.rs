//! Shadow maps: where each light's map looks from, and the depth-only pass that draws them.
//!
//! Every light that casts shadows gets a layer in one depth texture array. Layers are as large as
//! the largest map asked for; a light with a smaller resolution draws into the top-left corner of
//! its layer, and the lookup scales its coordinates to match.

use carbide_3d::camera::perspective_infinite_reverse_rh;
use carbide_3d::render::LightShadow;
use carbide_core::math::{Deg, EuclideanSpace, InnerSpace, Matrix4, Point3, Rad, Vector3, Vector4};

/// The depth format of the shadow maps.
pub const SHADOW_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

/// How near a spot light's map starts, in metres.
const SPOT_NEAR: f32 = 0.05;

/// The light a shadow map belongs to.
#[derive(Debug, Copy, Clone)]
pub enum ShadowTarget {
    /// The directional light at this index, shining this way in world space.
    Directional { index: usize, direction: Vector3<f32> },
    /// The point or spot light at this index; its map's view and projection are known when it is
    /// recorded.
    Punctual { index: usize },
}

/// A shadow map to draw this frame.
#[derive(Debug, Copy, Clone)]
pub struct ShadowRequest {
    pub target: ShadowTarget,
    pub settings: LightShadow,
}

/// An up vector for looking along `direction`: world up, unless that is nearly parallel.
fn up_for(direction: Vector3<f32>) -> Vector3<f32> {
    if direction.y.abs() > 0.99 { Vector3::unit_z() } else { Vector3::unit_y() }
}

/// From world space to a spot light's map: looking along the light's axis from its position, with
/// twice the outer angle as the field of view, reverse-Z with an infinite far plane as cameras
/// have.
pub fn spot_view_proj(position: Vector3<f32>, direction: Vector3<f32>, outer: Deg<f32>) -> Matrix4<f32> {
    let eye = Point3::from_vec(position);
    let view = Matrix4::look_at_rh(eye, eye + direction, up_for(direction));
    // A little wider than the cone, so its edge isn't cut by the map's.
    let fov = Rad::from(Deg((outer.0 * 2.0 + 2.0).clamp(1.0, 170.0)));
    perspective_infinite_reverse_rh(fov, 1.0, SPOT_NEAR) * view
}

/// From world space to a directional light's map, for a camera with the inverse view matrix
/// `inv_view`.
///
/// The map covers the sphere `distance` across that starts at the camera and lies in front of it,
/// and everything between that sphere and the light, so casters outside the view still cast into
/// it. Its centre is snapped to whole texels in the light's space, so shadows don't shimmer as the
/// camera moves.
pub fn directional_view_proj(direction: Vector3<f32>, inv_view: Matrix4<f32>, settings: &LightShadow) -> Matrix4<f32> {
    let radius = (settings.distance * 0.5).max(0.01);
    let camera_position = inv_view.w.truncate();
    let forward = -inv_view.z.truncate();
    let forward = if forward.magnitude2() > f32::EPSILON { forward.normalize() } else { -Vector3::unit_z() };
    let center = camera_position + forward * radius;

    // The light's orientation, around the world's origin.
    let rotation = Matrix4::look_at_rh(Point3::new(0.0, 0.0, 0.0), Point3::from_vec(direction), up_for(direction));

    // The centre in the light's space, snapped to the texel grid across the light.
    let texel = 2.0 * radius / settings.resolution.max(1) as f32;
    let center = rotation * center.extend(1.0);
    let snapped = Vector3::new((center.x / texel).floor() * texel, (center.y / texel).floor() * texel, center.z);

    let view = Matrix4::from_translation(-snapped) * rotation;

    // Light space looks along -Z. Depth is 1 (near) at `behind` towards the light, and 0 (far) at
    // `radius` past the centre: reverse-Z, as everywhere else.
    let behind = radius + settings.distance * 2.0;
    let depth = 1.0 / (behind + radius);
    let projection = Matrix4::from_cols(
        Vector4::new(1.0 / radius, 0.0, 0.0, 0.0),
        Vector4::new(0.0, 1.0 / radius, 0.0, 0.0),
        Vector4::new(0.0, 0.0, depth, 0.0),
        Vector4::new(0.0, 0.0, radius * depth, 1.0),
    );

    projection * view
}

/// The size of each layer: the largest resolution asked for, and at least one texel.
pub fn layer_size(requests: &[ShadowRequest]) -> u32 {
    requests.iter().map(|request| request.settings.resolution).max().unwrap_or(1).clamp(1, 8192)
}


#[cfg(test)]
mod tests {
    use carbide_core::math::SquareMatrix;

    use super::*;

    fn project(matrix: Matrix4<f32>, point: Vector3<f32>) -> Vector3<f32> {
        let clip = matrix * point.extend(1.0);
        clip.truncate() / clip.w
    }

    #[test]
    fn directional_map_is_reverse_z_and_covers_the_view() {
        let settings = LightShadow::DEFAULT;
        // A camera at (0, 0, 3) looking at the origin.
        let view = Matrix4::look_at_rh(Point3::new(0.0, 0.0, 3.0), Point3::new(0.0, 0.0, 0.0), Vector3::unit_y());
        let matrix = directional_view_proj(Vector3::new(0.0, -1.0, 0.0), view.invert().unwrap(), &settings);

        let high = project(matrix, Vector3::new(0.0, 1.0, 0.0));
        let low = project(matrix, Vector3::new(0.0, -1.0, 0.0));
        assert!(high.z > low.z, "nearer the light is greater depth: {:?} {:?}", high, low);
        for point in [high, low] {
            assert!(point.x.abs() <= 1.0 && point.y.abs() <= 1.0 && (0.0..=1.0).contains(&point.z), "{:?}", point);
        }
    }

    #[test]
    fn spot_map_is_reverse_z() {
        let matrix = spot_view_proj(Vector3::new(0.0, 2.0, 0.0), Vector3::new(0.0, -1.0, 0.0), Deg(30.0));
        let near = project(matrix, Vector3::new(0.0, 1.0, 0.0));
        let far = project(matrix, Vector3::new(0.0, 0.0, 0.0));
        assert!(near.z > far.z && far.z > 0.0 && near.z <= 1.0);
        assert!(near.x.abs() < 1e-5 && near.y.abs() < 1e-5);
    }
}
