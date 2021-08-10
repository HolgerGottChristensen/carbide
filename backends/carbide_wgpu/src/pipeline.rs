use wgpu::{
    CompareFunction, DepthStencilStateDescriptor, Device, PipelineLayout, RenderPipeline,
    ShaderModule, SwapChainDescriptor, TextureFormat,
};

use crate::vertex::Vertex;

pub(crate) enum MaskType {
    NoMask,
    AddMask,
    InMask,
    RemoveMask,
}

pub(crate) fn create_render_pipeline(
    device: &Device,
    render_pipeline_layout: &PipelineLayout,
    vs_module: &ShaderModule,
    fs_module: &ShaderModule,
    sc_desc: &SwapChainDescriptor,
    mask_type: MaskType,
) -> RenderPipeline {
    let (stencil_desc, col) = mask_render_state(mask_type);

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(render_pipeline_layout),
        vertex_stage: wgpu::ProgrammableStageDescriptor {
            module: &vs_module,
            entry_point: "main", // 1.
        },
        fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
            // 2.
            module: &fs_module,
            entry_point: "main",
        }),
        rasterization_state: Some(wgpu::RasterizationStateDescriptor {
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: wgpu::CullMode::None, // Todo fix mesh to always be CCW, then we can cull backfaces
            depth_bias: 0,
            depth_bias_slope_scale: 0.0,
            depth_bias_clamp: 0.0,
            clamp_depth: false,
        }),
        color_states: &[wgpu::ColorStateDescriptor {
            format: sc_desc.format,
            color_blend: wgpu::BlendDescriptor {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
            alpha_blend: wgpu::BlendDescriptor {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
            write_mask: col,
        }],
        primitive_topology: wgpu::PrimitiveTopology::TriangleList, // 1.
        depth_stencil_state: Some(DepthStencilStateDescriptor {
            format: TextureFormat::Depth24PlusStencil8,
            depth_write_enabled: true,
            depth_compare: CompareFunction::Always,
            stencil: stencil_desc,
        }), // 2.
        vertex_state: wgpu::VertexStateDescriptor {
            index_format: wgpu::IndexFormat::Uint16, // 3.
            vertex_buffers: &[Vertex::desc()],       // 4.
        },
        sample_count: 1,                  // 5.
        sample_mask: !0,                  // 6.
        alpha_to_coverage_enabled: false, // 7.
    })
}

// Inspired by ruffle: https://github.com/ruffle-rs/ruffle/blob/master/render/wgpu/src/pipelines.rs
fn mask_render_state(state: MaskType) -> (wgpu::StencilStateDescriptor, wgpu::ColorWrite) {
    let (stencil_state, color_write) = match state {
        MaskType::NoMask => (
            wgpu::StencilStateFaceDescriptor {
                compare: wgpu::CompareFunction::Always,
                fail_op: wgpu::StencilOperation::Keep,
                depth_fail_op: wgpu::StencilOperation::Keep,
                pass_op: wgpu::StencilOperation::Keep,
            },
            wgpu::ColorWrite::ALL,
        ),
        MaskType::AddMask => (
            wgpu::StencilStateFaceDescriptor {
                compare: wgpu::CompareFunction::Equal,
                fail_op: wgpu::StencilOperation::Keep,
                depth_fail_op: wgpu::StencilOperation::Keep,
                pass_op: wgpu::StencilOperation::IncrementClamp,
            },
            wgpu::ColorWrite::empty(),
        ),
        MaskType::InMask => (
            wgpu::StencilStateFaceDescriptor {
                compare: wgpu::CompareFunction::Equal,
                fail_op: wgpu::StencilOperation::Keep,
                depth_fail_op: wgpu::StencilOperation::Keep,
                pass_op: wgpu::StencilOperation::Keep,
            },
            wgpu::ColorWrite::ALL,
        ),
        MaskType::RemoveMask => (
            wgpu::StencilStateFaceDescriptor {
                compare: wgpu::CompareFunction::Equal,
                fail_op: wgpu::StencilOperation::Keep,
                depth_fail_op: wgpu::StencilOperation::Keep,
                pass_op: wgpu::StencilOperation::DecrementClamp,
            },
            wgpu::ColorWrite::empty(),
        ),
    };

    (
        wgpu::StencilStateDescriptor {
            front: stencil_state.clone(),
            back: stencil_state,
            read_mask: 0xff,
            write_mask: 0xff,
        },
        color_write,
    )
}
