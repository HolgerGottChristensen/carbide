use wgpu::{BindGroupLayout, BlendState, ColorTargetState, CompareFunction, DepthBiasState, DepthStencilState, Device, FragmentState, FrontFace, MultisampleState, PipelineLayout, PrimitiveState, PrimitiveTopology, RenderPipeline, ShaderModule, StencilFaceState, StencilOperation, TextureFormat, VertexState};

use crate::vertex::Vertex;
use crate::globals::{FILTER_RENDER_PIPELINE_LAYOUT, FILTER_SHADER, MAIN_SHADER, RENDER_PIPELINE_LAYOUT};
use crate::msaa::Msaa;

pub struct RenderPipelines {
    pub(crate) final_render_pipeline: RenderPipeline,
    pub(crate) render_pipeline_no_mask: RenderPipeline,
    pub(crate) render_pipeline_add_mask: RenderPipeline,
    pub(crate) render_pipeline_in_mask: RenderPipeline,
    pub(crate) render_pipeline_remove_mask: RenderPipeline,

    /// This is used when applying normal filter, or in the second pass of the of the two pass filter
    pub(crate) render_pipeline_in_mask_filter: RenderPipeline,
}


pub(crate) fn main_pipeline_layout(
    device: &Device,
    main_bind_group_layout: &BindGroupLayout,
    uniform_bind_group_layout: &BindGroupLayout,
    gradient_bind_group_layout: &BindGroupLayout,
    atlas_bind_group_layout: &BindGroupLayout,
) -> PipelineLayout {
    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("carbide_main_pipeline_layout"),
        bind_group_layouts: &[
            main_bind_group_layout,
            uniform_bind_group_layout,
            gradient_bind_group_layout,
            atlas_bind_group_layout,
            main_bind_group_layout,
        ],
        push_constant_ranges: &[],
    })
}

pub(crate) fn filter_pipeline_layout(
    device: &Device,
    filter_texture_bind_group_layout: &BindGroupLayout,
    filter_buffer_bind_group_layout: &BindGroupLayout,
    uniform_bind_group_layout: &BindGroupLayout,
) -> PipelineLayout {
    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("carbide_filter_pipeline_layout"),
        bind_group_layouts: &[
            &filter_texture_bind_group_layout,
            &filter_buffer_bind_group_layout,
            &uniform_bind_group_layout,
            &filter_texture_bind_group_layout,
        ],
        push_constant_ranges: &[],
    })
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum MaskType {
    NoMask,
    AddMask,
    InMask,
    RemoveMask,
}

pub(crate) fn create_pipelines(device: &Device, preferred_format: TextureFormat, msaa: Msaa) -> RenderPipelines {
    let render_pipeline_no_mask_no_msaa = create_final_render_pipeline(
        device,
        &RENDER_PIPELINE_LAYOUT,
        &MAIN_SHADER,
        preferred_format,
    );

    let render_pipeline_no_mask = create_render_pipeline(
        device,
        &RENDER_PIPELINE_LAYOUT,
        &MAIN_SHADER,
        preferred_format,
        MaskType::NoMask,
        msaa
    );

    let render_pipeline_add_mask = create_render_pipeline(
        device,
        &RENDER_PIPELINE_LAYOUT,
        &MAIN_SHADER,
        preferred_format,
        MaskType::AddMask,
        msaa
    );

    let render_pipeline_in_mask = create_render_pipeline(
        device,
        &RENDER_PIPELINE_LAYOUT,
        &MAIN_SHADER,
        preferred_format,
        MaskType::InMask,
        msaa
    );

    let render_pipeline_remove_mask = create_render_pipeline(
        device,
        &RENDER_PIPELINE_LAYOUT,
        &MAIN_SHADER,
        preferred_format,
        MaskType::RemoveMask,
        msaa
    );

    let render_pipeline_in_mask_filter = create_render_pipeline(
        device,
        &FILTER_RENDER_PIPELINE_LAYOUT,
        &FILTER_SHADER,
        preferred_format,
        MaskType::InMask,
        msaa
    );

    RenderPipelines {
        final_render_pipeline: render_pipeline_no_mask_no_msaa,
        render_pipeline_no_mask,
        render_pipeline_add_mask,
        render_pipeline_in_mask,
        render_pipeline_remove_mask,
        render_pipeline_in_mask_filter,
    }
}

pub(crate) fn create_render_pipeline(
    device: &Device,
    render_pipeline_layout: &PipelineLayout,
    shader: &ShaderModule,
    preferred_format: TextureFormat,
    mask_type: MaskType,
    msaa: Msaa
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
        multisample: MultisampleState {
            count: msaa.to_samples(),
            ..Default::default()
        },
        fragment: Some(FragmentState {
            module: &shader,
            entry_point: "main_fs",
            targets: &[Some(ColorTargetState {
                format: preferred_format,
                blend: Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                write_mask: col,
            })],
        }),
        multiview: None,
    })
}

pub(crate) fn create_final_render_pipeline(
    device: &Device,
    render_pipeline_layout: &PipelineLayout,
    shader: &ShaderModule,
    preferred_format: TextureFormat,
) -> RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("carbide_final_render_pipeline"),
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
        depth_stencil: None,
        multisample: Default::default(),
        fragment: Some(FragmentState {
            module: &shader,
            entry_point: "main_fs",
            targets: &[Some(ColorTargetState {
                format: preferred_format,
                blend: Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
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
