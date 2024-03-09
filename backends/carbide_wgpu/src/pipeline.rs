use wgpu::{
    BlendComponent, BlendFactor, BlendOperation, BlendState, ColorTargetState,
    CompareFunction, DepthBiasState, DepthStencilState, Device, FragmentState, FrontFace,
    PipelineLayout, PrimitiveState, PrimitiveTopology, RenderPipeline, ShaderModule,
    StencilFaceState, StencilOperation, TextureFormat, VertexState,
};

use crate::render_pipeline_layouts::RenderPipelines;
use crate::vertex::Vertex;
use crate::wgpu_window::{FILTER_RENDER_PIPELINE_LAYOUT, FILTER_SHADER, GRADIENT_RENDER_PIPELINE_LAYOUT, GRADIENT_SHADER, MAIN_SHADER, RENDER_PIPELINE_LAYOUT};

#[derive(Debug, Copy, Clone)]
pub(crate) enum MaskType {
    NoMask,
    AddMask,
    InMask,
    RemoveMask,
}

pub(crate) fn create_pipelines(device: &Device, preferred_format: TextureFormat) -> RenderPipelines {
    let render_pipeline_no_mask =
        RENDER_PIPELINE_LAYOUT.with(|render_pipeline_layout| {
            MAIN_SHADER.with(|main_shader| {
                create_render_pipeline(
                    device,
                    render_pipeline_layout,
                    main_shader,
                    preferred_format,
                    MaskType::NoMask,
                )
            })
        });

    let render_pipeline_add_mask =
        RENDER_PIPELINE_LAYOUT.with(|render_pipeline_layout| {
            MAIN_SHADER.with(|main_shader| {
                create_render_pipeline(
                    device,
                    render_pipeline_layout,
                    main_shader,
                    preferred_format,
                    MaskType::AddMask,
                )
            })
        });

    let render_pipeline_in_mask =
        RENDER_PIPELINE_LAYOUT.with(|render_pipeline_layout| {
            MAIN_SHADER.with(|main_shader| {
                create_render_pipeline(
                    device,
                    render_pipeline_layout,
                    main_shader,
                    preferred_format,
                    MaskType::InMask,
                )
            })
        });

    let render_pipeline_remove_mask =
        RENDER_PIPELINE_LAYOUT.with(|render_pipeline_layout| {
            MAIN_SHADER.with(|main_shader| {
                create_render_pipeline(
                    device,
                    render_pipeline_layout,
                    main_shader,
                    preferred_format,
                    MaskType::RemoveMask,
                )
            })
        });

    let render_pipeline_in_mask_filter =
        FILTER_RENDER_PIPELINE_LAYOUT.with(|filter_render_pipeline_layout| {
            FILTER_SHADER.with(|filter_shader| {
                create_render_pipeline(
                    device,
                    filter_render_pipeline_layout,
                    filter_shader,
                    preferred_format,
                    MaskType::InMask,
                )
            })
        });

    let render_pipeline_no_mask_filter =
        FILTER_RENDER_PIPELINE_LAYOUT.with(|filter_render_pipeline_layout| {
            FILTER_SHADER.with(|filter_shader| {
                create_render_pipeline(
                    device,
                    filter_render_pipeline_layout,
                    filter_shader,
                    preferred_format,
                    MaskType::NoMask,
                )
            })
        });

    let render_pipeline_in_mask_gradient =
        GRADIENT_RENDER_PIPELINE_LAYOUT.with(|gradient_render_pipeline_layout| {
            GRADIENT_SHADER.with(|gradient_shader| {
                create_render_pipeline(
                    device,
                    gradient_render_pipeline_layout,
                    gradient_shader,
                    preferred_format,
                    MaskType::InMask,
                )
            })
        });

    RenderPipelines {
        render_pipeline_no_mask,
        render_pipeline_add_mask,
        render_pipeline_in_mask,
        render_pipeline_remove_mask,
        render_pipeline_in_mask_filter,
        render_pipeline_no_mask_filter,
        render_pipeline_in_mask_gradient,
    }
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
        label: Some(&format!("Render Pipeline, {:?}", mask_type)),
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
            targets: &[Some(ColorTargetState {
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
            })],
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
