use std::ops::Range;
use carbide_core::draw::{BoundingBox, Position};
use carbide_core::draw::image::ImageId;
use carbide_core::draw::shape::triangle::Triangle;
use carbide_core::layout::BasicLayouter;
use carbide_core::mesh::{MODE_GEOMETRY};
use carbide_core::render::{CarbideTransform, InnerRenderContext, Style};
use carbide_core::text::Glyph;
use carbide_core::widget::FilterId;
use crate::render_pass_command::{RenderPass, RenderPassCommand, WGPUBindGroup};
use crate::vertex::Vertex;

#[derive(Debug)]
pub struct WGPURenderContext {
    style: Vec<WGPUStyle>,
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
        self.state = State::Plain { start: 0 };
        self.vertices.clear();
        self.current_bind_group = None;
    }

    pub fn vertices(&self) -> &Vec<Vertex> {
        &self.vertices
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

        let cmd = RenderPassCommand::Draw {
            vertex_range: vertices.start as u32..vertices.end as u32,
        };
        self.render_pass_inner.push(cmd);
    }
}

impl InnerRenderContext for WGPURenderContext {
    fn transform(&mut self, transform: CarbideTransform, anchor: BasicLayouter) {
        todo!()
    }

    fn de_transform(&mut self) {
        todo!()
    }

    fn clip(&mut self, bounding_box: BoundingBox) {
        todo!()
    }

    fn de_clip(&mut self) {
        todo!()
    }

    fn filter(&mut self, id: FilterId) {
        todo!()
    }

    fn stencil(&mut self, geometry: &Vec<Triangle<Position>>) {
        todo!()
    }

    fn de_stencil(&mut self) {
        todo!()
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
        }

    }

    fn de_style(&mut self) {
        assert!(self.style.pop().is_some(), "A style was popped, when no style is present.")
    }

    fn image(&mut self, id: ImageId, bounding_box: BoundingBox) {
        todo!()
    }

    fn text(&mut self, text: &Vec<Glyph>) {
        todo!()
    }
}