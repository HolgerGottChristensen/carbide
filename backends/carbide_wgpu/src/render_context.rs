use std::ops::Range;
use cgmath::{Matrix4, SquareMatrix, Vector3};
use carbide_core::color::WHITE;
use carbide_core::draw::{BoundingBox, Position, Rect};
use carbide_core::draw::image::ImageId;
use carbide_core::draw::shape::triangle::Triangle;
use carbide_core::layout::BasicLayouter;
use carbide_core::mesh::{MODE_GEOMETRY, MODE_TEXT, MODE_TEXT_COLOR};
use carbide_core::render::{CarbideTransform, InnerRenderContext, Style};
use carbide_core::Scalar;
use carbide_core::text::Glyph;
use carbide_core::widget::FilterId;
use crate::render_pass_command::{RenderPass, RenderPassCommand, WGPUBindGroup};
use crate::vertex::Vertex;

#[derive(Debug)]
pub struct WGPURenderContext {
    style: Vec<WGPUStyle>,
    stencil_stack: Vec<Range<u32>>,
    scissor_stack: Vec<BoundingBox>,
    transform_stack: Vec<(Matrix4<f32>, usize)>,
    transforms: Vec<Matrix4<f32>>,
    state: State,
    render_pass: Vec<RenderPass>,
    render_pass_inner: Vec<RenderPassCommand>,
    vertices: Vec<Vertex>,
    current_bind_group: Option<WGPUBindGroup>,
}

#[derive(Debug)]
enum WGPUStyle {
    Color([f32; 4]),
}

#[derive(Debug)]
enum State {
    Image { id: ImageId, start: usize },
    Plain { start: usize },
    Finished,
}

impl WGPURenderContext {
    pub fn new() -> WGPURenderContext {
        WGPURenderContext {
            style: vec![],
            stencil_stack: vec![],
            scissor_stack: vec![],
            transform_stack: vec![(Matrix4::identity(), 0)],
            transforms: vec![Matrix4::identity()],
            state: State::Plain { start: 0 },
            render_pass: vec![],
            render_pass_inner: vec![],
            vertices: vec![],
            current_bind_group: None,
        }
    }

    pub fn clear(&mut self) {
        assert!(self.style.is_empty());
        self.render_pass.clear();
        self.render_pass_inner.clear();
        self.scissor_stack.clear();

        self.transform_stack.clear();
        self.transforms.clear();
        self.transforms.push(Matrix4::identity());
        self.transform_stack.push((Matrix4::identity(), 0));

        self.stencil_stack.clear();
        self.state = State::Plain { start: 0 };
        self.vertices.clear();
        self.current_bind_group = None;
    }

    pub fn vertices(&self) -> &Vec<Vertex> {
        &self.vertices
    }
    pub fn transforms(&self) -> &Vec<Matrix4<f32>> {
        &self.transforms
    }

    pub fn finish(&mut self) -> Vec<RenderPass> {
        if let State::Finished = self.state {
            panic!("Trying to finish a render context that is already in a finished state.");
        }

        match &self.state {
            State::Plain { start } => {
                self.push_geometry_command(*start..self.vertices.len());
            },
            State::Image { id, start } => {
                self.push_image_command(id.clone(), *start..self.vertices.len());
            }
            State::Finished => {}
        }

        let mut swap = vec![];
        std::mem::swap(&mut swap, &mut self.render_pass_inner);

        self.render_pass.push(RenderPass::Normal(swap));
        self.state = State::Finished;

        let mut swap = vec![];
        std::mem::swap(&mut swap, &mut self.render_pass);

        swap

    }

    fn freshen_state(&mut self) {
        match &self.state {
            State::Image { id, start } => {
                self.push_image_command(id.clone(), *start..self.vertices.len());

            }
            State::Plain { start } => {
                self.push_geometry_command(*start..self.vertices.len());
            }
            State::Finished => {}
        }

        self.state = State::Plain {
            start: self.vertices.len(),
        };
    }

    fn ensure_state_plain(&mut self) {
        if let State::Image { id, start } = &self.state {
            self.push_image_command(id.clone(), *start..self.vertices.len());

            self.state = State::Plain {
                start: self.vertices.len(),
            };
        }
    }

    fn ensure_state_image(&mut self, id: &ImageId) {
        let new_image_id = id.clone();

        match &self.state {
            // If we're already in the drawing mode for this image, we're done.
            State::Image { id, .. } if id == &new_image_id => (),

            // If we were in the `Plain` drawing state, switch to Image drawing state.
            State::Plain { start } => {
                self.push_geometry_command(*start..self.vertices.len());
                self.state = State::Image {
                    id: new_image_id,
                    start: self.vertices.len(),
                };
            }

            // If we were drawing a different image, switch state to draw *this* image.
            State::Image { id, start } => {
                self.push_image_command(id.clone(), *start..self.vertices.len());
                self.state = State::Image {
                    id: new_image_id,
                    start: self.vertices.len(),
                };
            }
            State::Finished => {}
        }
    }

    fn ensure_current_bind_group_is_some(&mut self) {
        if self.current_bind_group.is_none() {
            self.current_bind_group = Some(WGPUBindGroup::Default);
            let cmd = RenderPassCommand::SetBindGroup {
                bind_group: WGPUBindGroup::Default,
            };
            self.render_pass_inner.push(cmd);
        }
    }

    fn push_image_command(&mut self, id: ImageId, vertices: Range<usize>) {
        let new_group = WGPUBindGroup::Image(id.clone());
        let expected_bind_group = Some(WGPUBindGroup::Image(id.clone()));

        if self.current_bind_group != expected_bind_group {
            // Now update the bind group and add the new bind group command.
            self.current_bind_group = expected_bind_group;
            let cmd = RenderPassCommand::SetBindGroup {
                bind_group: new_group,
            };
            self.render_pass_inner.push(cmd);
        }

        let cmd = RenderPassCommand::Draw {
            vertex_range: vertices.start as u32..vertices.end as u32,
        };

        self.render_pass_inner.push(cmd);
    }

    fn push_geometry_command(&mut self, vertices: Range<usize>) {
        self.ensure_current_bind_group_is_some();

        if vertices.len() == 0 {
            return;
        }

        let cmd = RenderPassCommand::Draw {
            vertex_range: vertices.start as u32..vertices.end as u32,
        };
        self.render_pass_inner.push(cmd);
    }
}

impl InnerRenderContext for WGPURenderContext {
    fn transform(&mut self, transform: CarbideTransform) {
        self.freshen_state();

        let (latest_transform, _) = &self.transform_stack[self.transform_stack.len() - 1];

        let new_transform = latest_transform * transform;

        let index = self.transforms.len();
        self.transform_stack.push((new_transform, index));
        self.transforms.push(new_transform);

        self.render_pass_inner.push(RenderPassCommand::Transform { uniform_bind_group_index: index });
    }

    fn pop_transform(&mut self) {
        self.freshen_state();

        self.transform_stack.pop();
        self.render_pass_inner.push(RenderPassCommand::Transform {
            uniform_bind_group_index: self.transform_stack[self.transform_stack.len() - 1].1
        });
    }

    // TODO: clip is broken atm. And no color on hacker news list example...
    fn clip(&mut self, bounding_box: BoundingBox) {
        /*self.freshen_state();

        let corrected = if let Some(outer) = self.scissor_stack.first() {
            bounding_box.within_bounding_box(outer)
        } else {
            bounding_box
        };


        self.render_pass_inner.push(RenderPassCommand::SetScissor {
            rect: corrected
        });

        self.scissor_stack.push(corrected);*/
    }

    fn pop_clip(&mut self) {
        /*self.freshen_state();

        self.scissor_stack.pop();

        match self.scissor_stack.last() {
            Some(n) => {
                self.render_pass_inner.push(RenderPassCommand::SetScissor {
                    rect: *n
                })
            },
            None => (),
        }*/
    }

    fn filter(&mut self, id: FilterId) {
        todo!()
    }

    fn stencil(&mut self, geometry: &Vec<Triangle<Position>>) {
        self.freshen_state();

        let start_index_for_stencil = self.vertices.len();

        self.vertices.extend(
            geometry.iter()
                .flat_map(|triangle| &triangle.0)
                .map(|position| Vertex::new_from_2d(
                    position.x() as f32,
                    position.y() as f32,
                    [1.0, 1.0, 1.0, 1.0],
                    [0.0, 0.0],
                    MODE_GEOMETRY
                ))
        );

        let range = start_index_for_stencil as u32..self.vertices.len() as u32;

        self.stencil_stack.push(range.clone());

        self.render_pass_inner.push(RenderPassCommand::Stencil { vertex_range: range });
    }

    fn pop_stencil(&mut self) {
        self.freshen_state();

        if let Some(range) = self.stencil_stack.pop() {
            self.render_pass_inner.push(RenderPassCommand::DeStencil { vertex_range: range });
        } else {
            panic!("Trying to pop from empty stencil stack")
        }
    }

    fn geometry(&mut self, geometry: &Vec<Triangle<Position>>) {
        self.ensure_state_plain();

        let color = match self.style.last().unwrap() {
            WGPUStyle::Color(c) => *c,
        };

        self.vertices.extend(
            geometry.iter()
                .flat_map(|triangle| &triangle.0)
                .map(|position| Vertex::new_from_2d(
                    position.x() as f32,
                    position.y() as f32,
                    color,
                    [0.0, 0.0],
                    MODE_GEOMETRY
                ))
        );
    }

    fn style(&mut self, style: Style) {
        match style {
            Style::Color(color) => {
                let color = color.gamma_srgb_to_linear()
                    .pre_multiply()
                    .to_fsa();

                self.style.push(WGPUStyle::Color(color));
            }
            Style::Gradient(_) => {
                todo!()
            }
            Style::MultiGradient(_) => {
                todo!()
            }
        }

    }

    fn pop_style(&mut self) {
        assert!(self.style.pop().is_some(), "A style was popped, when no style is present.")
    }

    fn image(&mut self, id: ImageId, bounding_box: Rect, source_rect: Rect, mode: u32) {
        self.ensure_state_image(&id);

        let color = match self.style.last().unwrap_or(&WGPUStyle::Color(WHITE.gamma_srgb_to_linear().to_fsa())) {
            WGPUStyle::Color(c) => *c,
        };

        let (uv_l, uv_r, uv_b, uv_t) = source_rect.l_r_b_t();

        let create_vertex = |x, y, tx, ty| Vertex {
            position: [x as f32, y as f32, 0.0],
            tex_coords: [tx as f32, ty as f32],
            rgba: color,
            mode,
        };


        let (l, r, b, t) = bounding_box.l_r_b_t();

        // Bottom left triangle.
        self.vertices.push(create_vertex(l, t, uv_l, uv_t));
        self.vertices.push(create_vertex(r, b, uv_r, uv_b));
        self.vertices.push(create_vertex(l, b, uv_l, uv_b));

        // Top right triangle.
        self.vertices.push(create_vertex(l, t, uv_l, uv_t));
        self.vertices.push(create_vertex(r, t, uv_r, uv_t));
        self.vertices.push(create_vertex(r, b, uv_r, uv_b));
    }

    fn text(&mut self, text: &Vec<Glyph>) {
        self.ensure_state_plain();

        let color = match self.style.last().unwrap_or(&WGPUStyle::Color(WHITE.gamma_srgb_to_linear().to_fsa())) {
            WGPUStyle::Color(c) => *c,
        };

        let v_normal = |x, y, t| Vertex {
            position: [x as f32, y as f32, 0.0],
            tex_coords: t,
            rgba: color,
            mode: MODE_TEXT,
        };

        let v_color = |x, y, t| Vertex {
            position: [x as f32, y as f32, 0.0],
            tex_coords: t,
            rgba: color,
            mode: MODE_TEXT_COLOR,
        };

        let mut push_v = |x: Scalar, y: Scalar, t: [f32; 2], is_bitmap: bool| {
            if is_bitmap {
                self.vertices.push(v_color(x, y, t));
            } else {
                self.vertices.push(v_normal(x, y, t));
            }
        };

        for glyph in text {
            if let Some(bb) = glyph.bb() {
                let (left, right, bottom, top) = bb.l_r_b_t();

                if let Some(index) = glyph.atlas_entry() {
                    if !index.borrow().is_active {
                        println!(
                            "Trying to show glyph that is not in the texture atlas 11111."
                        );
                    }
                    let coords = index.borrow().tex_coords;

                    push_v(left, top, [coords.min.x, coords.max.y], glyph.is_bitmap());
                    push_v(
                        right,
                        bottom,
                        [coords.max.x, coords.min.y],
                        glyph.is_bitmap(),
                    );
                    push_v(
                        left,
                        bottom,
                        [coords.min.x, coords.min.y],
                        glyph.is_bitmap(),
                    );
                    push_v(left, top, [coords.min.x, coords.max.y], glyph.is_bitmap());
                    push_v(
                        right,
                        bottom,
                        [coords.max.x, coords.min.y],
                        glyph.is_bitmap(),
                    );
                    push_v(right, top, [coords.max.x, coords.max.y], glyph.is_bitmap());
                } else {
                    println!("Trying to show glyph that is not in the texture atlas.");
                }
            }
        }
    }
}