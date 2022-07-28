use wgpu::{
    Adapter, BlendComponent, BlendFactor, BlendOperation, BlendState, ColorTargetState,
    CompareFunction, DepthBiasState, DepthStencilState, Device, FragmentState, FrontFace,
    PipelineLayout, PrimitiveState, PrimitiveTopology, RenderPipeline, ShaderModule,
    StencilFaceState, StencilOperation, Surface, TextureFormat, VertexState,
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
    shader: &ShaderModule,
    preferred_format: TextureFormat,
    mask_type: MaskType,
) -> RenderPipeline {
    let (stencil_desc, col) = mask_render_state(mask_type);



    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(render_pipeline_layout),
        vertex: VertexState {
            module: &shader,
            entry_point: "main_vs",
            buffers: &[Vertex::desc()],
        },
        primitive: PrimitiveState {
            topology: PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: FrontFace::Ccw,
            cull_mode: None,
            unclipped_depth: false,
            polygon_mode: Default::default(),
            conservative: false,
        },
        depth_stencil: Some(DepthStencilState {
            format: TextureFormat::Depth24PlusStencil8,
            depth_write_enabled: true,
            depth_compare: CompareFunction::Always,
            stencil: stencil_desc,
            bias: DepthBiasState {
                constant: 0,
                slope_scale: 0.0,
                clamp: 0.0,
            },
        }),
        multisample: Default::default(),
        fragment: Some(FragmentState {
            module: &shader,
            entry_point: "main_fs",
            targets: &[ColorTargetState {
                format: preferred_format,
                blend: Some(BlendState {
                    color: BlendComponent {
                        src_factor: BlendFactor::One,
                        dst_factor: BlendFactor::OneMinusSrcAlpha,
                        operation: BlendOperation::Add,
                    },
                    alpha: BlendComponent {
                        src_factor: BlendFactor::One,
                        dst_factor: BlendFactor::OneMinusSrcAlpha,
                        operation: BlendOperation::Add,
                    },
                }),
                write_mask: col,
            }],
        }),
        multiview: None,
    })
}

// Inspired by ruffle: https://github.com/ruffle-rs/ruffle/blob/master/render/wgpu/src/pipelines.rs
fn mask_render_state(state: MaskType) -> (wgpu::StencilState, wgpu::ColorWrites) {
    let (stencil_state, color_write) = match state {
        MaskType::NoMask => (
            wgpu::StencilState {
                front: StencilFaceState {
                    compare: CompareFunction::Always,
                    fail_op: StencilOperation::Keep,
                    depth_fail_op: StencilOperation::Keep,
                    pass_op: StencilOperation::Keep,
                },
                back: StencilFaceState {
                    compare: CompareFunction::Always,
                    fail_op: StencilOperation::Keep,
                    depth_fail_op: StencilOperation::Keep,
                    pass_op: StencilOperation::Keep,
                },
                read_mask: !0,
                write_mask: !0,
            },
            wgpu::ColorWrites::ALL,
        ),
        MaskType::AddMask => (
            wgpu::StencilState {
                front: StencilFaceState {
                    compare: CompareFunction::Equal,
                    fail_op: StencilOperation::Keep,
                    depth_fail_op: StencilOperation::Keep,
                    pass_op: StencilOperation::IncrementClamp,
                },
                back: StencilFaceState {
                    compare: CompareFunction::Equal,
                    fail_op: StencilOperation::Keep,
                    depth_fail_op: StencilOperation::Keep,
                    pass_op: StencilOperation::IncrementClamp,
                },
                read_mask: !0,
                write_mask: !0,
            },
            wgpu::ColorWrites::empty(),
        ),
        MaskType::InMask => (
            wgpu::StencilState {
                front: StencilFaceState {
                    compare: CompareFunction::Equal,
                    fail_op: StencilOperation::Keep,
                    depth_fail_op: StencilOperation::Keep,
                    pass_op: StencilOperation::Keep,
                },
                back: StencilFaceState {
                    compare: CompareFunction::Equal,
                    fail_op: StencilOperation::Keep,
                    depth_fail_op: StencilOperation::Keep,
                    pass_op: StencilOperation::Keep,
                },
                read_mask: !0,
                write_mask: !0,
            },
            wgpu::ColorWrites::ALL,
        ),
        MaskType::RemoveMask => (
            wgpu::StencilState {
                front: StencilFaceState {
                    compare: CompareFunction::Equal,
                    fail_op: StencilOperation::Keep,
                    depth_fail_op: StencilOperation::Keep,
                    pass_op: StencilOperation::DecrementClamp,
                },
                back: StencilFaceState {
                    compare: CompareFunction::Equal,
                    fail_op: StencilOperation::Keep,
                    depth_fail_op: StencilOperation::Keep,
                    pass_op: StencilOperation::DecrementClamp,
                },
                read_mask: !0,
                write_mask: !0,
            },
            wgpu::ColorWrites::empty(),
        ),
    };

    (stencil_state, color_write)
}
