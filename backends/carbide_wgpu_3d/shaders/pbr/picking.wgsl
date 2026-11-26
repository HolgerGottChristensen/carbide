// The picking pass: every mesh again, depth-tested, at full resolution with no multisampling,
// writing for each pixel the pick id of the nearest mesh (0 for none, or for meshes that aren't
// pickable but still hide what is behind them) and its depth.
//!include pbr/object.wgsl
//!include pbr/camera.wgsl

@group(0) @binding(1)
var<storage> objects: array<Object>;
@group(0) @binding(2)
var<storage> camera: Camera;

struct PickingOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) pick_id: u32,
}

@vertex
fn picking_vs(@location(0) position: vec3<f32>, @builtin(instance_index) object_index: u32) -> PickingOutput {
    let object = objects[object_index];
    var out: PickingOutput;
    out.position = camera.view_proj * object.transform * vec4<f32>(position, 1.0);
    out.pick_id = object.pick_id;
    return out;
}

@fragment
fn picking_fs(in: PickingOutput) -> @location(0) vec2<u32> {
    return vec2<u32>(in.pick_id, bitcast<u32>(in.position.z));
}
