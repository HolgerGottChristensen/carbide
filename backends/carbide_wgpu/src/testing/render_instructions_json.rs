use crate::render_pass_command::{RenderPass, RenderPassCommand, WGPUBindGroup};

/// Hand-built `serde_json::Value` conversion of the draw-call plan.
///
/// `RenderPass`/`RenderPassCommand`/`WGPUBindGroup` don't derive `Serialize` (adding it would
/// mean adding it across their whole nested type graph — `Rect`, `FilterId`, `ImageId`,
/// `LayerId`, `WgpuGradient`, `WgpuDashes` — including several `carbide_core` types, for a
/// benefit specific to this one snapshot), so this builds the `serde_json::Value` tree by hand
/// instead, falling back to each inner type's existing `Debug` output for the handful of fields
/// it doesn't break out explicitly.
pub(crate) fn render_passes_to_json(passes: &[RenderPass]) -> serde_json::Value {
    fn bind_group_to_json(bind_group: &WGPUBindGroup) -> serde_json::Value {
        match bind_group {
            WGPUBindGroup::Image(id) => serde_json::json!({ "type": "Image", "id": format!("{id:?}") }),
            WGPUBindGroup::Target(index) => serde_json::json!({ "type": "Target", "index": index }),
            WGPUBindGroup::Layer(id) => serde_json::json!({ "type": "Layer", "id": format!("{id:?}") }),
        }
    }

    fn command_to_json(command: &RenderPassCommand) -> serde_json::Value {
        match command {
            RenderPassCommand::SetScissor { rect } => serde_json::json!({
                "type": "SetScissor", "rect": format!("{rect:?}"),
            }),
            RenderPassCommand::Draw { vertex_range } => serde_json::json!({
                "type": "Draw", "vertex_range": [vertex_range.start, vertex_range.end],
            }),
            RenderPassCommand::Stencil { vertex_range } => serde_json::json!({
                "type": "Stencil", "vertex_range": [vertex_range.start, vertex_range.end],
            }),
            RenderPassCommand::DeStencil { vertex_range } => serde_json::json!({
                "type": "DeStencil", "vertex_range": [vertex_range.start, vertex_range.end],
            }),
            RenderPassCommand::Uniform { uniform_bind_group_index } => serde_json::json!({
                "type": "Uniform", "uniform_bind_group_index": uniform_bind_group_index,
            }),
            RenderPassCommand::Gradient(gradient) => serde_json::json!({
                "type": "Gradient", "value": format!("{gradient:?}"),
            }),
            RenderPassCommand::StrokeDashing(dashes) => serde_json::json!({
                "type": "StrokeDashing", "value": format!("{dashes:?}"),
            }),
            RenderPassCommand::SetBindGroup { bind_group } => serde_json::json!({
                "type": "SetBindGroup", "bind_group": bind_group_to_json(bind_group),
            }),
            RenderPassCommand::SetMaskBindGroup { bind_group } => serde_json::json!({
                "type": "SetMaskBindGroup", "bind_group": bind_group_to_json(bind_group),
            }),
        }
    }

    let passes: Vec<serde_json::Value> = passes.iter().map(|pass| match pass {
        RenderPass::Normal { commands, target_index } => serde_json::json!({
            "type": "Normal",
            "target_index": target_index,
            "commands": commands.iter().map(command_to_json).collect::<Vec<_>>(),
        }),
        RenderPass::Clear { target_index } => serde_json::json!({
            "type": "Clear", "target_index": target_index,
        }),
        RenderPass::Filter { vertex_range, filter_id, source_id, target_id, mask_id, initial_copy } => serde_json::json!({
            "type": "Filter",
            "vertex_range": [vertex_range.start, vertex_range.end],
            "filter_id": format!("{filter_id:?}"),
            "source_id": source_id,
            "target_id": target_id,
            "mask_id": mask_id,
            "initial_copy": initial_copy,
        }),
    }).collect();

    serde_json::Value::Array(passes)
}
