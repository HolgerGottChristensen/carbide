use std::collections::{HashMap, HashSet};
use std::ops::Range;

use wgpu::BindGroup;

use carbide_core::color::{Color, ColorExt, WHITE};
use carbide_lyon::stroke_vertex::StrokeVertex;
use carbide_lyon::triangle::Triangle;
use carbide_core::draw::{Dimension, DrawStyle, ImageId, Position, Rect, Scalar, MODE_GEOMETRY, MODE_GEOMETRY_DASH, MODE_GEOMETRY_DASH_FAST, MODE_GRADIENT_GEOMETRY, MODE_GRADIENT_GEOMETRY_DASH, MODE_GRADIENT_GEOMETRY_DASH_FAST, MODE_GRADIENT_ICON, MODE_GRADIENT_TEXT, MODE_ICON, MODE_IMAGE, MODE_TEXT};
use carbide_core::draw::stroke::{StrokeAlignment, StrokeDashMode, StrokeDashPattern};
use carbide_core::math::{Matrix4, SquareMatrix};
use carbide_core::render::{CarbideTransform, InnerRenderContext, Layer, LayerId};
use carbide_core::text::{TextContext, TextId};
use carbide_core::widget::{AnyShape, FilterId, ImageFilter, ShapeStyle};
use carbide_lyon::Tesselator;
use crate::gradient::{Dashes, Gradient};
use crate::render_context::TargetState::{Free, Used};
use crate::render_pass_command::{RenderPass, RenderPassCommand, WGPUBindGroup};
use crate::render_target::RenderTarget;
use crate::vertex::Vertex;

#[derive(Debug)]
pub struct WGPURenderContext {
    style_stack: Vec<WGPUStyle>,
    stroke_dash_stack: Vec<Option<StrokeDashPattern>>,
    stencil_stack: Vec<Range<u32>>,
    scissor_stack: Vec<Rect>,
    uniform_stack: Vec<(Uniform, usize)>,

    // (mask, target)
    mask_target_stack: Vec<(usize, usize)>,
    // target, old_target
    filter_target_stack: Vec<(usize, usize)>,

    uniforms: Vec<Uniform>,
    gradients: Vec<Gradient>,
    filters: HashMap<FilterId, ImageFilter>,
    vertices: Vec<Vertex>,

    render_pass: Vec<RenderPass>,
    render_pass_inner: Vec<RenderPassCommand>,
    current_bind_group: Option<WGPUBindGroup>,
    current_gradient: Option<Gradient>,
    current_stroke_dash: Option<StrokeDashPattern>,
    current_frame_filters: HashSet<FilterId>,

    finished: bool,
    targets: TargetStates,
    window_bounding_box: Rect,
    frame_count: usize,
    skip_rendering: bool,
    masked: bool,
    current_target: usize,

    layers: HashMap<LayerId, RenderTarget>,

    tesselator: Tesselator,
}

#[derive(Debug, Clone)]
enum WGPUStyle {
    Color([f32; 4]),
    Gradient(Gradient),
}

#[derive(Debug, PartialEq)]
enum TargetState {
    Free,
    Used
}

#[derive(Debug)]
struct TargetStates {
    inner: Vec<TargetState>,
}

impl TargetStates {
    fn new() -> TargetStates {
        TargetStates {
            inner: vec![
                Used
            ],
        }
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    /// Get the index of the next free target and a bool indicating if the target needs clearing.
    fn get(&mut self) -> (usize, bool) {
        if let Some((index, target)) = self.inner.iter_mut().enumerate().filter(|(_, a)| **a != Used).next() {
            let needs_free = *target == Free;
            *target = Used;
            (index, needs_free)
        } else {
            self.inner.push(Used);
            (self.inner.len() - 1, false)
        }
    }

    fn free(&mut self, index: usize) {
        self.inner[index] = Free;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Uniform {
    pub transform: Matrix4<f32>,
    pub hue_rotation: f32,
    pub saturation_shift: f32,
    pub luminance_shift: f32,
    pub color_invert: bool,
}

impl WGPURenderContext {
    pub fn new() -> WGPURenderContext {
        WGPURenderContext {
            style_stack: vec![],
            stroke_dash_stack: vec![],
            stencil_stack: vec![],
            scissor_stack: vec![],
            uniform_stack: vec![(Uniform {
                transform: Matrix4::identity(),
                hue_rotation: 0.0,
                saturation_shift: 0.0,
                luminance_shift: 0.0,
                color_invert: false,
            }, 0)],
            mask_target_stack: vec![],
            filter_target_stack: vec![],
            uniforms: vec![Uniform {
                transform: Matrix4::identity(),
                hue_rotation: 0.0,
                saturation_shift: 0.0,
                luminance_shift: 0.0,
                color_invert: false,
            }],
            gradients: vec![],
            finished: false,
            render_pass: vec![],
            render_pass_inner: vec![],
            vertices: vec![],
            current_bind_group: None,
            window_bounding_box: Rect::default(),
            frame_count: 0,
            skip_rendering: false,
            current_gradient: None,
            masked: false,
            targets: TargetStates::new(),
            current_target: 0,
            current_stroke_dash: None,
            layers: Default::default(),
            filters: HashMap::new(),
            current_frame_filters: HashSet::new(),
            tesselator: Tesselator::new(),
        }
    }

    pub fn layer_bind_group(&self, layer_id: LayerId) -> &BindGroup {
        &self.layers[&layer_id].bind_group
    }

    pub fn target_count(&self) -> usize {
        self.targets.len()
    }

    pub fn clear(&mut self) {
        assert!(self.style_stack.is_empty());
        self.render_pass.clear();
        self.render_pass_inner.clear();
        self.scissor_stack.clear();
        self.mask_target_stack.clear();
        self.filter_target_stack.clear();
        self.stroke_dash_stack.clear();

        self.uniform_stack.clear();
        self.uniforms.clear();
        self.gradients.clear();

        self.uniforms.push(Uniform {
            transform: Matrix4::identity(),
            hue_rotation: 0.0,
            saturation_shift: 0.0,
            luminance_shift: 0.0,
            color_invert: false,
        });
        self.uniform_stack.push((Uniform {
            transform: Matrix4::identity(),
            hue_rotation: 0.0,
            saturation_shift: 0.0,
            luminance_shift: 0.0,
            color_invert: false,
        }, 0));
        self.stroke_dash_stack.push(None);

        self.stencil_stack.clear();
        self.finished = false;
        self.vertices.clear();
        self.current_bind_group = None;
        self.current_gradient = None;
        self.current_stroke_dash = None;
        self.skip_rendering = false;
        self.masked = false;
        self.current_target = 0;
    }

    pub fn vertices(&self) -> &Vec<Vertex> {
        &self.vertices
    }

    pub fn uniforms(&self) -> &Vec<Uniform> {
        &self.uniforms
    }
    pub fn filters(&self) -> &HashMap<FilterId, ImageFilter> {
        &self.filters
    }

    pub fn gradients(&self) -> &Vec<Gradient> {
        &self.gradients
    }

    pub fn start(&mut self, window_bounding_box: Rect) {
        self.frame_count += 1;
        //println!("Start render frame: {}", self.frame_count);
        self.window_bounding_box = window_bounding_box;
        self.clear()
    }

    pub fn finish(&mut self) -> Vec<RenderPass> {
        if self.finished {
            panic!("Trying to finish a render context that is already in a finished state.");
        }

        self.finished = true;

        self.filters.retain(|a, _| self.current_frame_filters.contains(a));
        self.current_frame_filters.clear();

        let mut swap = vec![];
        std::mem::swap(&mut swap, &mut self.render_pass);

        swap
    }

    fn draw(&mut self, start: u32, end: u32) {
        if let Some(RenderPass::Normal { commands, .. }) = self.render_pass.last_mut() {
            if let Some(RenderPassCommand::Draw { vertex_range }) = commands.last_mut() {
                vertex_range.end = end;
            } else {
                commands.push(RenderPassCommand::Draw { vertex_range: start..end })
            }
        } else {
            self.render_pass.push(RenderPass::Normal { commands: vec![
                RenderPassCommand::Draw { vertex_range: start..end }
            ], target_index: self.current_target });
            self.current_bind_group = None;
        }
    }

    fn draw_image(&mut self, id: Option<WGPUBindGroup>, bounding_box: Rect, source_rect: Rect, mut mode: u32) {
        if self.skip_rendering {
            return;
        }

        if let Some(id) = id {
            let command = if let Some(current) = &mut self.current_bind_group {
                if current != &id {
                    *current = id.clone();
                    Some(RenderPassCommand::SetBindGroup {
                        bind_group: id,
                    })
                } else {
                    None
                }
            } else {
                self.current_bind_group = Some(id.clone());
                Some(RenderPassCommand::SetBindGroup {
                    bind_group: id,
                })
            };

            if let Some(command) = command {
                self.push_command(command);
            }
        }

        let (color, is_gradient) = match self.style_stack.last().unwrap_or(&WGPUStyle::Color(WHITE.gamma_srgb_to_linear().to_fsa())).clone() {
            WGPUStyle::Color(c) => {
                (c, false)
            },
            WGPUStyle::Gradient(gradient) => {
                self.ensure_state_gradient(&gradient);
                ([0.0, 0.0, 0.0, 1.0], true)
            }
        };

        let (uv_l, uv_r, uv_b, uv_t) = source_rect.l_r_b_t();

        if mode == MODE_ICON && is_gradient {
            mode = MODE_GRADIENT_ICON;
        }
        if mode == MODE_TEXT && is_gradient {
            mode = MODE_GRADIENT_TEXT;
        }

        let mode = if self.masked { mode | 0b100000 } else { mode };

        let create_vertex = |x, y, tx, ty| Vertex {
            position: [x as f32, y as f32, 0.0],
            tex_coords: [tx as f32, ty as f32],
            rgba: color,
            mode,
            attributes0: [0.0, 0.0, 0.0, 0.0],
            attributes1: [0.0, 0.0, 0.0, 0.0],
        };


        let (l, r, b, t) = bounding_box.l_r_b_t();

        let start = self.vertices.len();
        // Bottom left triangle.
        self.vertices.push(create_vertex(l, t, uv_l, uv_t));
        self.vertices.push(create_vertex(r, b, uv_r, uv_b));
        self.vertices.push(create_vertex(l, b, uv_l, uv_b));

        // Top right triangle.
        self.vertices.push(create_vertex(l, t, uv_l, uv_t));
        self.vertices.push(create_vertex(r, t, uv_r, uv_t));
        self.vertices.push(create_vertex(r, b, uv_r, uv_b));

        self.draw(start as u32, self.vertices.len() as u32);
    }

    fn push_command(&mut self, command: RenderPassCommand) {
        if let Some(RenderPass::Normal { commands, .. }) = self.render_pass.last_mut() {
            commands.push(command);
        } else {
            self.render_pass.push(RenderPass::Normal { commands: vec![
                command
            ], target_index: self.current_target });
        }
    }

    fn start_render_pass(&mut self, index: usize) {
        self.current_bind_group = None;
        self.render_pass.push(RenderPass::Normal { commands: vec![], target_index: index });
    }

    fn ensure_state_gradient(&mut self, gradient: &Gradient) {
        let needs_update = if let Some(current) = &mut self.current_gradient {
            current != gradient
        } else {
            true
        };

        if needs_update {
            self.current_gradient = Some(gradient.clone());

            self.push_command(RenderPassCommand::Gradient(gradient.clone()));
        }
    }
}

impl WGPURenderContext {
    fn fill(&mut self, geometry: impl Iterator<Item=Triangle<Position>>) {
        if self.skip_rendering {
            return;
        }
        //println!("draw geometry: {}", geometry.len());

        let style = self.style_stack.last().unwrap().clone();

        let (color, gradient) = match style {
            WGPUStyle::Color(c) => {
                (c, false)
            },
            WGPUStyle::Gradient(g) => {
                self.ensure_state_gradient(&g);
                ([0.0, 0.0, 0.0, 1.0], true)
            },
        };

        let mode = if gradient { MODE_GRADIENT_GEOMETRY } else { MODE_GEOMETRY };
        let mode = if self.masked { mode | 0b100000 } else { mode };

        let start = self.vertices.len();
        self.vertices.extend(
            geometry
                .flat_map(|triangle| {
                    [
                        Vertex::new_from_2d(
                            triangle.0[0].x as f32,
                            triangle.0[0].y as f32,
                            color,
                            [0.0, 0.0],
                            mode
                        ),
                        Vertex::new_from_2d(
                            triangle.0[1].x as f32,
                            triangle.0[1].y as f32,
                            color,
                            [0.0, 0.0],
                            mode
                        ),
                        Vertex::new_from_2d(
                            triangle.0[2].x as f32,
                            triangle.0[2].y as f32,
                            color,
                            [0.0, 0.0],
                            mode
                        )
                    ]
                })
        );

        self.draw(start as u32, self.vertices.len() as u32);
    }

    fn stroke(&mut self, stroke: impl Iterator<Item=Triangle<StrokeVertex>>) {
        if self.skip_rendering {
            return;
        }
        //println!("draw geometry: {}", geometry.len());

        let style = self.style_stack.last().unwrap().clone();

        let (mut color, gradient) = match style {
            WGPUStyle::Color(c) => {
                (c, false)
            },
            WGPUStyle::Gradient(g) => {
                self.ensure_state_gradient(&g);
                ([0.0, 0.0, 0.0, 1.0], true)
            },
        };

        let new_stroke_dash = self.stroke_dash_stack.last().unwrap();

        if &self.current_stroke_dash != new_stroke_dash {
            if let Some(new_stroke_dash) = new_stroke_dash {
                assert_ne!(new_stroke_dash.pattern.len(), 0);
                let repeat = new_stroke_dash.pattern.len() % 2 == 1;

                let pattern_count = new_stroke_dash.pattern.len();
                let count = (if repeat { pattern_count * 2 } else { pattern_count }).min(32);

                let mut dashes = [0.0f32; 32];
                let mut total_dash_width = 0.0;

                for i in 0..count {
                    dashes[i] = (new_stroke_dash.pattern[i % pattern_count] as f32).abs();
                    total_dash_width += dashes[i];
                }

                let offset = total_dash_width - new_stroke_dash.offset as f32 % total_dash_width;

                self.push_command(RenderPassCommand::StrokeDashing (Dashes {
                    dashes,
                    dash_count: count as u32,
                    start_cap: new_stroke_dash.start_cap as u32,
                    end_cap: new_stroke_dash.end_cap as u32,
                    total_dash_width,
                    dash_offset: offset,
                }));
            }
            self.current_stroke_dash = self.stroke_dash_stack.last().unwrap().clone();
        }

        let mode = if let Some(stroke_dash) = &self.current_stroke_dash {
            if stroke_dash.dash_type == StrokeDashMode::Fast {
                if gradient { MODE_GRADIENT_GEOMETRY_DASH_FAST } else { MODE_GEOMETRY_DASH_FAST }
            } else {
                if gradient { MODE_GRADIENT_GEOMETRY_DASH } else { MODE_GEOMETRY_DASH }
            }
        } else {
            if gradient { MODE_GRADIENT_GEOMETRY } else { MODE_GEOMETRY }
        };

        let mode = if self.masked { mode | 0b100000 } else { mode };

        let start = self.vertices.len();
        self.vertices.extend(
            stroke
                .flat_map(|triangle| triangle.0)
                .map(|v| {
                    Vertex {
                        position: [
                            v.position.x,
                            v.position.y,
                            0.0
                        ],
                        tex_coords: [0.0, 0.0],
                        rgba: if gradient { Color::random().to_fsa() } else { color },
                        mode,
                        attributes0: [
                            v.start.x,
                            v.start.y,
                            v.end.x,
                            v.end.y,
                        ],
                        attributes1: [
                            v.start_angle.radians,
                            v.end_angle.radians,
                            v.width,
                            v.offset
                        ],
                    }
                })
        );

        self.draw(start as u32, self.vertices.len() as u32);
    }
}

impl InnerRenderContext for WGPURenderContext {
    fn transform(&mut self, transform: CarbideTransform) {
        let (latest_uniform, _) = &self.uniform_stack[self.uniform_stack.len() - 1];

        let new_uniform = Uniform {
            transform: latest_uniform.transform * transform,
            ..*latest_uniform
        };

        let index = self.uniforms.len();
        self.uniform_stack.push((new_uniform, index));
        self.uniforms.push(new_uniform);

        self.push_command(RenderPassCommand::Uniform { uniform_bind_group_index: index });
    }

    fn pop_transform(&mut self) {
        self.uniform_stack.pop();
        self.push_command(RenderPassCommand::Uniform {
            uniform_bind_group_index: self.uniform_stack[self.uniform_stack.len() - 1].1
        });
    }

    fn color_filter(&mut self, hue_rotation: f32, saturation_shift: f32, luminance_shift: f32, color_invert: bool) {
        let (latest_uniform, _) = &self.uniform_stack[self.uniform_stack.len() - 1];

        let new_uniform = Uniform {
            transform: latest_uniform.transform,
            hue_rotation: latest_uniform.hue_rotation + hue_rotation,
            saturation_shift: latest_uniform.saturation_shift + saturation_shift,
            luminance_shift: latest_uniform.luminance_shift + luminance_shift,
            color_invert: if color_invert { !latest_uniform.color_invert } else { latest_uniform.color_invert },
        };

        let index = self.uniforms.len();
        self.uniform_stack.push((new_uniform, index));
        self.uniforms.push(new_uniform);

        self.push_command(RenderPassCommand::Uniform { uniform_bind_group_index: index });
    }

    fn pop_color_filter(&mut self) {
        self.uniform_stack.pop();
        self.push_command(RenderPassCommand::Uniform {
            uniform_bind_group_index: self.uniform_stack[self.uniform_stack.len() - 1].1
        });
    }

    fn clip(&mut self, bounding_box: Rect) {
        let corrected = if let Some(outer) = self.scissor_stack.last() {
            bounding_box.within_bounding_box(outer)
        } else {
            bounding_box.within_bounding_box(&self.window_bounding_box)
        };

        if corrected.height() > 0.0 && corrected.width() > 0.0 {
            self.push_command(RenderPassCommand::SetScissor {
                rect: corrected
            });
        } else {
            self.skip_rendering = true;
        }

        self.scissor_stack.push(corrected);
    }

    fn pop_clip(&mut self) {
        self.scissor_stack.pop();

        match self.scissor_stack.last() {
            Some(n) => {
                if n.height() > 0.0 && n.width() > 0.0 {
                    self.skip_rendering = false;
                    self.push_command(RenderPassCommand::SetScissor {
                        rect: *n
                    })
                }
            }
            None => {
                self.skip_rendering = false;
                self.push_command(RenderPassCommand::SetScissor {
                    rect: self.window_bounding_box
                })
            }
        }
    }

    fn filter(&mut self, filter: &ImageFilter, bounding_box: Rect) {
        if self.skip_rendering {
            return;
        }

        self.filters.entry(filter.id).or_insert_with(|| filter.clone());
        self.current_frame_filters.insert(filter.id);

        let create_vertex = |x, y| Vertex {
            position: [x as f32, y as f32, 0.0],
            tex_coords: [
                (x / self.window_bounding_box.dimension.width) as f32,
                (y / self.window_bounding_box.dimension.height) as f32,
            ],
            rgba: [1.0, 1.0, 1.0, 1.0],
            mode: MODE_TEXT,
            attributes0: [0.0, 0.0, 0.0, 0.0],
            attributes1: [0.0, 0.0, 0.0, 0.0],
        };


        let (l, r, b, t) = bounding_box.l_r_b_t();
        let (new_target, _needs_clear) = self.targets.get();

        let vertices_start = self.vertices.len() as u32;

        // Bottom left triangle.
        self.vertices.push(create_vertex(l, t));
        self.vertices.push(create_vertex(r, b));
        self.vertices.push(create_vertex(l, b));

        // Top right triangle.
        self.vertices.push(create_vertex(l, t));
        self.vertices.push(create_vertex(r, t));
        self.vertices.push(create_vertex(r, b));

        self.render_pass.push(RenderPass::Filter {
            vertex_range: vertices_start..self.vertices.len() as u32,
            filter_id: filter.id,
            source_id: new_target,
            target_id: self.current_target,
            mask_id: None,
            initial_copy: true,
        });

        self.targets.free(new_target);
    }

    fn filter2d(&mut self, filter1: &ImageFilter, bounding_box1: Rect, filter2: &ImageFilter, bounding_box2: Rect) {
        if self.skip_rendering {
            return;
        }

        self.filters.entry(filter1.id).or_insert_with(|| filter1.clone());
        self.current_frame_filters.insert(filter1.id);
        self.filters.entry(filter2.id).or_insert_with(|| filter2.clone());
        self.current_frame_filters.insert(filter2.id);

        let create_vertex = |x, y| Vertex {
            position: [x as f32, y as f32, 0.0],
            tex_coords: [
                (x / self.window_bounding_box.dimension.width) as f32,
                (y / self.window_bounding_box.dimension.height) as f32,
            ],
            rgba: [1.0, 1.0, 1.0, 1.0],
            mode: MODE_TEXT,
            attributes0: [0.0, 0.0, 0.0, 0.0],
            attributes1: [0.0, 0.0, 0.0, 0.0],
        };

        let (l, r, b, t) = bounding_box1.l_r_b_t();

        let vertices_start1 = self.vertices.len() as u32;

        // Bottom left triangle.
        self.vertices.push(create_vertex(l, t));
        self.vertices.push(create_vertex(r, b));
        self.vertices.push(create_vertex(l, b));

        // Top right triangle.
        self.vertices.push(create_vertex(l, t));
        self.vertices.push(create_vertex(r, t));
        self.vertices.push(create_vertex(r, b));

        let (l, r, b, t) = bounding_box2.l_r_b_t();

        let vertices_start2 = self.vertices.len() as u32;

        // Bottom left triangle.
        self.vertices.push(create_vertex(l, t));
        self.vertices.push(create_vertex(r, b));
        self.vertices.push(create_vertex(l, b));

        // Top right triangle.
        self.vertices.push(create_vertex(l, t));
        self.vertices.push(create_vertex(r, t));
        self.vertices.push(create_vertex(r, b));

        let (new_target, needs_clear) = self.targets.get();

        if needs_clear {
            self.render_pass.push(RenderPass::Clear { target_index: new_target });
        }

        let range = vertices_start1..vertices_start2;
        self.render_pass.push(RenderPass::Filter {
            vertex_range: range,
            filter_id: filter1.id,
            source_id: 0,
            target_id: 1,
            mask_id: None,
            initial_copy: false,
        });

        let range = vertices_start2..self.vertices.len() as u32;
        self.render_pass.push(RenderPass::Filter {
            vertex_range: range,
            filter_id: filter2.id,
            source_id: 1,
            target_id: 0,
            mask_id: None,
            initial_copy: false,
        });

        self.current_bind_group = None;

        self.targets.free(new_target);
    }

    fn stencil(&mut self, geometry: &dyn AnyShape) {
        if self.skip_rendering {
            return;
        }

        let start = self.vertices.len();

        /*self.vertices.extend(
            geometry.iter()
                .flat_map(|triangle| &triangle.0)
                .map(|position| Vertex::new_from_2d(
                    position.x as f32,
                    position.y as f32,
                    [1.0, 1.0, 1.0, 1.0],
                    [0.0, 0.0],
                    MODE_GEOMETRY
                ))
        );*/

        todo!();

        let range = start as u32..self.vertices.len() as u32;

        self.stencil_stack.push(range.clone());

        self.push_command(RenderPassCommand::Stencil { vertex_range: range });
    }

    fn pop_stencil(&mut self) {
        if self.skip_rendering {
            return;
        }

        if let Some(range) = self.stencil_stack.pop() {
            self.push_command(RenderPassCommand::DeStencil { vertex_range: range });
        } else {
            panic!("Trying to pop from empty stencil stack")
        }
    }

    fn fill_shape(&mut self, shape: &dyn AnyShape) {
        if self.skip_rendering {
            return;
        }

        let triangles = self.tesselator.fill(shape.description());
        self.fill(triangles);
    }

    fn stroke_shape(&mut self, shape: &dyn AnyShape, stroke_width: Scalar, stroke_alignment: StrokeAlignment) {
        if self.skip_rendering {
            return;
        }

        let triangles = self.tesselator.stroke(shape.description(), stroke_width, stroke_alignment);

        self.stroke(triangles);
    }

    fn style(&mut self, style: DrawStyle) {
        match style {
            DrawStyle::Color(color) => {
                let color = color.gamma_srgb_to_linear()
                    .pre_multiply()
                    .to_fsa();

                self.style_stack.push(WGPUStyle::Color(color));
            }
            DrawStyle::Gradient(g) => {
                self.style_stack.push(WGPUStyle::Gradient(Gradient::convert(&g)))
            }
            DrawStyle::MultiGradient(_) => {
                todo!()
            }
        }

    }

    fn pop_style(&mut self) {
        assert!(self.style_stack.pop().is_some(), "A style was popped, when no style is present.")
    }

    fn stroke_dash_pattern(&mut self, pattern: Option<StrokeDashPattern>) {
        self.stroke_dash_stack.push(pattern);
    }

    fn pop_stroke_dash_pattern(&mut self) {
        self.stroke_dash_stack.pop();
    }

    fn image(&mut self, id: Option<ImageId>, bounding_box: Rect, source_rect: Rect, mode: u32) {
        self.draw_image(id.map(|id| WGPUBindGroup::Image(id)), bounding_box, source_rect, mode)
    }

    fn text(&mut self, text: TextId, ctx: &mut dyn TextContext) {
        if self.skip_rendering {
            return;
        }

        ctx.render(text, self);
    }

    fn filter_new(&mut self) {
        let (new_target, needs_clear) = self.targets.get();

        if needs_clear {
            self.render_pass.push(RenderPass::Clear { target_index: new_target });
        }

        self.filter_target_stack.push((new_target, self.current_target));

        self.current_target = new_target;
    }

    fn filter_new_pop(&mut self, filter: &ImageFilter, color: Color, post_draw: bool) {
        let (target, old_target) = self.filter_target_stack.pop().unwrap();

        let create_vertex = |x, y, tx, ty| Vertex {
            position: [x as f32, y as f32, 0.0],
            tex_coords: [tx as f32, ty as f32],
            rgba: color
                .gamma_srgb_to_linear()
                .pre_multiply()
                .to_fsa(),
            mode: if post_draw { MODE_IMAGE } else { MODE_TEXT },
            attributes0: [0.0, 0.0, 0.0, 0.0],
            attributes1: [0.0, 0.0, 0.0, 0.0],

        };

        self.filters.entry(filter.id).or_insert_with(|| filter.clone());
        self.current_frame_filters.insert(filter.id);

        let (l, r, b, t) = self.window_bounding_box.l_r_b_t();

        let vertices_start = self.vertices.len() as u32;

        // Bottom left triangle.
        self.vertices.push(create_vertex(l, t, 0.0, 1.0));
        self.vertices.push(create_vertex(r, b, 1.0, 0.0));
        self.vertices.push(create_vertex(l, b, 0.0, 0.0));

        // Top right triangle.
        self.vertices.push(create_vertex(l, t, 0.0, 1.0));
        self.vertices.push(create_vertex(r, t, 1.0, 1.0));
        self.vertices.push(create_vertex(r, b, 1.0, 0.0));

        let range = vertices_start..self.vertices.len() as u32;
        self.render_pass.push(RenderPass::Filter {
            vertex_range: range.clone(),
            filter_id: filter.id,
            source_id: target,
            target_id: old_target,
            mask_id: None,
            initial_copy: false,
        });

        if post_draw {
            self.render_pass.push(RenderPass::Normal { commands: vec![
                RenderPassCommand::SetBindGroup { bind_group: WGPUBindGroup::Target(target) },
                RenderPassCommand::Draw { vertex_range: range }
            ], target_index: old_target });
        }

        self.current_target = old_target;
        self.targets.free(target);
    }

    fn filter_new_pop2d(&mut self, filter: &ImageFilter, filter2: &ImageFilter, color: Color, post_draw: bool) {
        let (target, old_target) = self.filter_target_stack.pop().unwrap();
        let (new_target, needs_clear) = self.targets.get();

        if needs_clear {
            self.render_pass.push(RenderPass::Clear { target_index: new_target });
        }

        self.filters.entry(filter.id).or_insert_with(|| filter.clone());
        self.current_frame_filters.insert(filter.id);
        self.filters.entry(filter2.id).or_insert_with(|| filter2.clone());
        self.current_frame_filters.insert(filter2.id);

        let create_vertex = |x, y, tx, ty| Vertex {
            position: [x as f32, y as f32, 0.0],
            tex_coords: [tx as f32, ty as f32],
            rgba: color
                .gamma_srgb_to_linear()
                .pre_multiply()
                .to_fsa(),
            mode: if post_draw { MODE_IMAGE } else { MODE_TEXT },
            attributes0: [0.0, 0.0, 0.0, 0.0],
            attributes1: [0.0, 0.0, 0.0, 0.0],
        };

        let (l, r, b, t) = self.window_bounding_box.l_r_b_t();


        let vertices_start = self.vertices.len() as u32;

        // Bottom left triangle.
        self.vertices.push(create_vertex(l, t, 0.0, 1.0));
        self.vertices.push(create_vertex(r, b, 1.0, 0.0));
        self.vertices.push(create_vertex(l, b, 0.0, 0.0));

        // Top right triangle.
        self.vertices.push(create_vertex(l, t, 0.0, 1.0));
        self.vertices.push(create_vertex(r, t, 1.0, 1.0));
        self.vertices.push(create_vertex(r, b, 1.0, 0.0));

        let range = vertices_start..self.vertices.len() as u32;

        self.render_pass.push(RenderPass::Filter {
            vertex_range: range.clone(),
            filter_id: filter.id,
            source_id: target,
            target_id: new_target,
            mask_id: None,
            initial_copy: false,
        });

        self.render_pass.push(RenderPass::Filter {
            vertex_range: range.clone(),
            filter_id: filter2.id,
            source_id: new_target,
            target_id: old_target,
            mask_id: None,
            initial_copy: false,
        });

        if post_draw {
            self.render_pass.push(RenderPass::Normal { commands: vec![
                RenderPassCommand::SetBindGroup { bind_group: WGPUBindGroup::Target(target) },
                RenderPassCommand::Draw { vertex_range: range }
            ], target_index: old_target });
        }

        self.current_target = old_target;
        self.targets.free(new_target);
        self.targets.free(target);
    }

    fn mask_start(&mut self) {
        let (index, need_clear) = self.targets.get();
        self.mask_target_stack.push((index, self.current_target));

        if need_clear {
            self.render_pass.push(RenderPass::Clear { target_index: index });
        }

        self.start_render_pass(index);
        self.current_target = index;
    }

    fn mask_in(&mut self) {
        let (mask, target) = *self.mask_target_stack.last().unwrap();
        self.start_render_pass(target);
        self.push_command(RenderPassCommand::SetMaskBindGroup {
            bind_group: WGPUBindGroup::Target(mask),
        });
        self.current_target = target;

        self.masked = true;
    }

    fn mask_end(&mut self) {
        let (mask, _) = self.mask_target_stack.pop().unwrap();
        self.masked = false;
        self.targets.free(mask);
    }

    fn layer(&mut self, layer_id: LayerId, requested_size: Dimension) -> Layer {
        let width = requested_size.width.floor().max(1.0) as u32;
        let height = requested_size.height.floor().max(1.0) as u32;

        if self.layers.contains_key(&layer_id) {
            let target = self.layers.get_mut(&layer_id).unwrap();

            if target.texture.width() != width || target.texture.height() != height {
                *target = RenderTarget::new(width, height);
            }

            Layer {
                inner: target,
                inner2: target,
            }
        } else {
            let new_layer = RenderTarget::new(width, height);

            self.layers.insert(layer_id, new_layer);

            let inner = self.layers.get_mut(&layer_id).unwrap();

            Layer {
                inner,
                inner2: inner
            }
        }
    }

    fn render_layer(&mut self, layer_id: LayerId, bounding_box: Rect) {
        let bind_group = WGPUBindGroup::Layer(layer_id);

        self.draw_image(Some(bind_group), bounding_box, Rect::from_corners(Position::new(0.0, 0.0), Position::new(1.0, 1.0)), MODE_IMAGE)
    }
}