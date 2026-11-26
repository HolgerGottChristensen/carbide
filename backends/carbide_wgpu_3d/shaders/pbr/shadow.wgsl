// The shadow pass: draws the meshes that cast shadows into a light's shadow map, depth only.
//!include pbr/object.wgsl

@group(0) @binding(0)
var<storage> objects: array<Object>;
@group(0) @binding(1)
var<storage> light_view_proj: mat4x4<f32>;

@vertex
fn shadow_vs(@location(0) position: vec3<f32>, @builtin(instance_index) object_index: u32) -> @builtin(position) vec4<f32> {
    return light_view_proj * objects[object_index].transform * vec4<f32>(position, 1.0);
}
