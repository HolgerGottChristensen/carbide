//! A mesh type dedicated to converting sequences of `render::Primitive`s to a representation in
//! vertices ready for uploading to the GPU.
//!
//! While populating the vertices buffer ready for uploading to the GPU, the `Mesh` will also
//! produce a sequence of commands describing the order in which draw commands should occur and
//! whether or not the `Scissor` should be updated between draws.


use cgmath::{Matrix4, SquareMatrix, Vector3};
use image::GenericImageView;

use crate::color;
use crate::draw::{Position, Rect, Scalar};
use crate::draw::image::ImageId;
use crate::environment::Environment;
use crate::layout::BasicLayouter;
use crate::mesh::{MODE_GEOMETRY, MODE_TEXT, MODE_TEXT_COLOR};
use crate::mesh::draw_command::DrawCommand;
use crate::mesh::vertex::Vertex;
use crate::render::{Primitive, PrimitiveKind};

/// A mesh whose vertices may be populated by a list of render primitives.
///
/// This is a convenience type for simplifying backend implementations.
#[derive(Debug)]
pub struct Mesh {
    commands: Vec<DrawCommand>,
    vertices: Vec<Vertex>,
}

impl Mesh {
    pub fn new() -> Self {
        Mesh {
            commands: vec![],
            vertices: vec![],
        }
    }

    pub fn commands(&self) -> &[DrawCommand] {
        &self.commands
    }

    pub fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }

    /// Fill the inner vertex buffer from the given primitives.
    ///
    /// - `viewport`: the window in which the UI is drawn. The width and height should be the
    ///   physical size (pixels).
    /// - `scale_factor`: the factor for converting from carbide's DPI agnostic point space to the
    ///   pixel space of the viewport.
    /// - `image_map`: a map from image IDs to images.
    /// - `primitives`: the sequence of UI primitives in order of depth to be rendered.
    pub fn fill(
        &mut self,
        viewport: Rect,
        env: &mut Environment,
        primitives: Vec<Primitive>,
    ) {
        let scale_factor = env.scale_factor();

        let Mesh {
            ref mut commands,
            ref mut vertices,
        } = *self;

        commands.clear();
        vertices.clear();

        enum State {
            Image { image_id: ImageId, start: usize },
            Plain { start: usize },
        }

        let mut current_state = State::Plain { start: 0 };

        // Keep track of the scissor as it changes.
        let mut scissor_stack = vec![viewport];
        let mut stencil_stack = vec![];
        let mut transform_stack = vec![Matrix4::identity()];

        commands.push(DrawCommand::Scissor(scissor_stack[0]));

        // Draw each primitive in order of depth.
        for primitive in primitives {
            let rectangle = primitive.bounding_box;
            match primitive.kind {
                PrimitiveKind::Stencil(triangles) => {
                    if triangles.len() == 0 {
                        continue;
                    }
                    match &current_state {
                        State::Plain { start } => {
                            commands.push(DrawCommand::Geometry(*start..vertices.len()))
                        }
                        State::Image { image_id, start } => {
                            commands.push(DrawCommand::Image(*start..vertices.len(), image_id.clone()))
                        }
                    }

                    let start_index_for_stencil = vertices.len();

                    fn position_to_vertex(p: Position) -> Vertex {
                        Vertex {
                            position: [p.x as f32, p.y as f32, 0.0],
                            tex_coords: [0.0, 0.0],
                            rgba: [1.0, 1.0, 1.0, 1.0],
                            mode: MODE_GEOMETRY,
                        }
                    }

                    for triangle in &triangles {
                        vertices.push(position_to_vertex(triangle[0]));
                        vertices.push(position_to_vertex(triangle[1]));
                        vertices.push(position_to_vertex(triangle[2]));
                    }

                    stencil_stack.push(triangles);

                    commands.push(DrawCommand::Stencil(
                        start_index_for_stencil..vertices.len(),
                    ));

                    current_state = State::Plain {
                        start: vertices.len(),
                    };
                }
                PrimitiveKind::DeStencil => {
                    if let Some(triangles) = stencil_stack.pop() {
                        if triangles.len() == 0 {
                            continue
                        }

                        match &current_state {
                            State::Plain { start } => {
                                commands.push(DrawCommand::Geometry(*start..vertices.len()))
                            }
                            State::Image { image_id, start } => {
                                commands.push(DrawCommand::Image(*start..vertices.len(), image_id.clone()))
                            }
                        }


                        let start_index_for_de_stencil = vertices.len();

                        fn position_to_vertex(p: Position) -> Vertex {
                            Vertex {
                                position: [p.x as f32, p.y as f32, 0.0],
                                tex_coords: [0.0, 0.0],
                                rgba: [1.0, 1.0, 1.0, 1.0],
                                mode: MODE_GEOMETRY,
                            }
                        }

                        for triangle in &triangles {
                            vertices.push(position_to_vertex(triangle[0]));
                            vertices.push(position_to_vertex(triangle[1]));
                            vertices.push(position_to_vertex(triangle[2]));
                        }

                        commands.push(DrawCommand::DeStencil(
                            start_index_for_de_stencil..vertices.len(),
                        ));
                    } else {
                        panic!("Mesh tried to DeStencil when no stencil were present.");
                    }

                    current_state = State::Plain {
                        start: vertices.len(),
                    };
                }
                PrimitiveKind::Filter(filter_id) => {
                    match &current_state {
                        State::Plain { start } => {
                            commands.push(DrawCommand::Geometry(*start..vertices.len()))
                        }
                        State::Image { image_id, start } => {
                            commands.push(DrawCommand::Image(*start..vertices.len(), image_id.clone()))
                        }
                    }

                    let start_index_for_filter = vertices.len();

                    let v = |p: Position| Vertex {
                        position: [p.x as f32, p.y as f32, 0.0],
                        tex_coords: [
                            (p.x / viewport.dimension.width * scale_factor) as f32,
                            (p.y / viewport.dimension.height * scale_factor) as f32,
                        ],
                        rgba: [1.0, 1.0, 1.0, 1.0],
                        mode: MODE_GEOMETRY,
                    };

                    let (l, r, t, b) = rectangle.l_r_b_t();

                    let mut push_v = |x, y| vertices.push(v(Position::new(x, y)));

                    // Bottom left triangle.
                    push_v(l, t);
                    push_v(r, b);
                    push_v(l, b);
                    // Top right triangle.
                    push_v(l, t);
                    push_v(r, b);
                    push_v(r, t);

                    commands.push(DrawCommand::Filter(
                        start_index_for_filter..vertices.len(),
                        filter_id,
                    ));

                    current_state = State::Plain {
                        start: vertices.len(),
                    };
                }
                PrimitiveKind::FilterSplitPt1(filter_id) => {
                    match &current_state {
                        State::Plain { start } => {
                            commands.push(DrawCommand::Geometry(*start..vertices.len()))
                        }
                        State::Image { image_id, start } => {
                            commands.push(DrawCommand::Image(*start..vertices.len(), image_id.clone()))
                        }
                    }

                    let start_index_for_filter = vertices.len();

                    let v = |p: Position| Vertex {
                        position: [p.x as f32, p.y as f32, 0.0],
                        tex_coords: [
                            (p.x / viewport.dimension.width * scale_factor) as f32,
                            (p.y / viewport.dimension.height * scale_factor) as f32,
                        ],
                        rgba: [1.0, 1.0, 1.0, 1.0],
                        mode: MODE_GEOMETRY,
                    };

                    let (l, r, t, b) = rectangle.l_r_b_t();

                    let mut push_v = |x, y| vertices.push(v(Position::new(x, y)));

                    // Bottom left triangle.
                    push_v(l, t);
                    push_v(r, b);
                    push_v(l, b);
                    // Top right triangle.
                    push_v(l, t);
                    push_v(r, b);
                    push_v(r, t);

                    commands.push(DrawCommand::FilterSplitPt1(
                        start_index_for_filter..vertices.len(),
                        filter_id,
                    ));

                    current_state = State::Plain {
                        start: vertices.len(),
                    };
                }
                PrimitiveKind::FilterSplitPt2(filter_id) => {
                    match &current_state {
                        State::Plain { start } => {
                            commands.push(DrawCommand::Geometry(*start..vertices.len()))
                        }
                        State::Image { image_id, start } => {
                            commands.push(DrawCommand::Image(*start..vertices.len(), image_id.clone()))
                        }
                    }

                    let start_index_for_filter = vertices.len();

                    let v = |p: Position| Vertex {
                        position: [p.x as f32, p.y as f32, 0.0],
                        tex_coords: [
                            (p.x / viewport.dimension.width * scale_factor) as f32,
                            (p.y / viewport.dimension.height * scale_factor) as f32,
                        ],
                        rgba: [1.0, 1.0, 1.0, 1.0],
                        mode: MODE_GEOMETRY,
                    };

                    let (l, r, t, b) = rectangle.l_r_b_t();

                    let mut push_v = |x, y| vertices.push(v(Position::new(x, y)));

                    // Bottom left triangle.
                    push_v(l, t);
                    push_v(r, b);
                    push_v(l, b);
                    // Top right triangle.
                    push_v(l, t);
                    push_v(r, b);
                    push_v(r, t);

                    commands.push(DrawCommand::FilterSplitPt2(
                        start_index_for_filter..vertices.len(),
                        filter_id,
                    ));

                    current_state = State::Plain {
                        start: vertices.len(),
                    };
                }
                PrimitiveKind::Transform(matrix, alignment) => {
                    match &current_state {
                        State::Plain { start } => {
                            commands.push(DrawCommand::Geometry(*start..vertices.len()))
                        }
                        State::Image { image_id, start } => {
                            commands.push(DrawCommand::Image(*start..vertices.len(), image_id.clone()))
                        }
                    }

                    let latest_transform = &transform_stack[transform_stack.len() - 1];

                    let new_transform = match alignment {
                        BasicLayouter::TopLeading => {
                            let center_x = (rectangle.position.x) as f32;
                            let center_y = (rectangle.position.y) as f32;
                            latest_transform
                                * Matrix4::from_translation(Vector3::new(center_x, center_y, 0.0))
                                * matrix
                                * Matrix4::from_translation(Vector3::new(-center_x, -center_y, 0.0))
                        }
                        BasicLayouter::Top => {
                            let center_x =
                                (rectangle.position.x + rectangle.dimension.width / 2.0) as f32;
                            let center_y = (rectangle.position.y) as f32;
                            latest_transform
                                * Matrix4::from_translation(Vector3::new(center_x, center_y, 0.0))
                                * matrix
                                * Matrix4::from_translation(Vector3::new(-center_x, -center_y, 0.0))
                        }
                        BasicLayouter::TopTrailing => {
                            let center_x =
                                (rectangle.position.x + rectangle.dimension.width) as f32;
                            let center_y = (rectangle.position.y) as f32;
                            latest_transform
                                * Matrix4::from_translation(Vector3::new(center_x, center_y, 0.0))
                                * matrix
                                * Matrix4::from_translation(Vector3::new(-center_x, -center_y, 0.0))
                        }
                        BasicLayouter::Leading => {
                            let center_x = (rectangle.position.x) as f32;
                            let center_y =
                                (rectangle.position.y + rectangle.dimension.height / 2.0) as f32;
                            latest_transform
                                * Matrix4::from_translation(Vector3::new(center_x, center_y, 0.0))
                                * matrix
                                * Matrix4::from_translation(Vector3::new(-center_x, -center_y, 0.0))
                        }
                        BasicLayouter::Center => {
                            let center_x =
                                (rectangle.position.x + rectangle.dimension.width / 2.0) as f32;
                            let center_y =
                                (rectangle.position.y + rectangle.dimension.height / 2.0) as f32;
                            latest_transform
                                * Matrix4::from_translation(Vector3::new(center_x, center_y, 0.0))
                                * matrix
                                * Matrix4::from_translation(Vector3::new(-center_x, -center_y, 0.0))
                        }
                        BasicLayouter::Trailing => {
                            let center_x =
                                (rectangle.position.x + rectangle.dimension.width) as f32;
                            let center_y =
                                (rectangle.position.y + rectangle.dimension.height / 2.0) as f32;
                            latest_transform
                                * Matrix4::from_translation(Vector3::new(center_x, center_y, 0.0))
                                * matrix
                                * Matrix4::from_translation(Vector3::new(-center_x, -center_y, 0.0))
                        }
                        BasicLayouter::BottomLeading => {
                            let center_x = (rectangle.position.x) as f32;
                            let center_y =
                                (rectangle.position.y + rectangle.dimension.height) as f32;
                            latest_transform
                                * Matrix4::from_translation(Vector3::new(center_x, center_y, 0.0))
                                * matrix
                                * Matrix4::from_translation(Vector3::new(-center_x, -center_y, 0.0))
                        }
                        BasicLayouter::Bottom => {
                            let center_x =
                                (rectangle.position.x + rectangle.dimension.width / 2.0) as f32;
                            let center_y =
                                (rectangle.position.y + rectangle.dimension.height) as f32;
                            latest_transform
                                * Matrix4::from_translation(Vector3::new(center_x, center_y, 0.0))
                                * matrix
                                * Matrix4::from_translation(Vector3::new(-center_x, -center_y, 0.0))
                        }
                        BasicLayouter::BottomTrailing => {
                            let center_x =
                                (rectangle.position.x + rectangle.dimension.width) as f32;
                            let center_y =
                                (rectangle.position.y + rectangle.dimension.height) as f32;
                            latest_transform
                                * Matrix4::from_translation(Vector3::new(center_x, center_y, 0.0))
                                * matrix
                                * Matrix4::from_translation(Vector3::new(-center_x, -center_y, 0.0))
                        }
                    };

                    transform_stack.push(new_transform);

                    commands.push(DrawCommand::Transform(new_transform));

                    current_state = State::Plain {
                        start: vertices.len(),
                    };
                }
                PrimitiveKind::DeTransform => {
                    match &current_state {
                        State::Plain { start } => {
                            commands.push(DrawCommand::Geometry(*start..vertices.len()))
                        }
                        State::Image { image_id, start } => {
                            commands.push(DrawCommand::Image(*start..vertices.len(), image_id.clone()))
                        }
                    }

                    transform_stack.pop();
                    commands.push(DrawCommand::Transform(
                        *&transform_stack[transform_stack.len() - 1],
                    ));

                    current_state = State::Plain {
                        start: vertices.len(),
                    };
                }
                PrimitiveKind::Clip => {
                    match &current_state {
                        State::Plain { start } => {
                            commands.push(DrawCommand::Geometry(*start..vertices.len()))
                        }
                        State::Image { image_id, start } => {
                            commands.push(DrawCommand::Image(*start..vertices.len(), image_id.clone()))
                        }
                    }

                    let (mut l, mut r, mut b, mut t) = primitive.bounding_box.l_r_b_t();

                    l *= scale_factor;
                    r *= scale_factor;
                    t *= scale_factor;
                    b *= scale_factor;

                    let new_rect = Rect::from_corners(Position::new(r, b), Position::new(l, t))
                        .within_bounding_box(&viewport);

                    commands.push(DrawCommand::Scissor(new_rect));

                    scissor_stack.push(new_rect);

                    current_state = State::Plain {
                        start: vertices.len(),
                    };
                }
                PrimitiveKind::UnClip => {
                    match &current_state {
                        State::Plain { start } => {
                            commands.push(DrawCommand::Geometry(*start..vertices.len()))
                        }
                        State::Image { image_id, start } => {
                            commands.push(DrawCommand::Image(*start..vertices.len(), image_id.clone()))
                        }
                    }

                    scissor_stack.pop();

                    let new_scizzor = match scissor_stack.first() {
                        Some(n) => n,
                        None => panic!("Trying to pop scizzor, when there is none on the stack"),
                    };

                    commands.push(DrawCommand::Scissor(*new_scizzor));

                    current_state = State::Plain {
                        start: vertices.len(),
                    };
                }
                PrimitiveKind::RectanglePrim { color } => {
                    match current_state {
                        State::Plain { .. } => (),
                        State::Image { image_id, start } => {
                            commands.push(DrawCommand::Image(start..vertices.len(), image_id));
                            current_state = State::Plain {
                                start: vertices.len(),
                            };
                        }
                    }

                    let color = color.gamma_srgb_to_linear().to_fsa();
                    let (l, r, b, t) = primitive.bounding_box.l_r_b_t();

                    let v = |x, y| {
                        // Convert from carbide Scalar range to GL range -1.0 to 1.0.
                        Vertex {
                            position: [x as f32, y as f32, 0.0],
                            tex_coords: [0.0, 0.0],
                            rgba: color,
                            mode: MODE_GEOMETRY,
                        }
                    };

                    let mut push_v = |x, y| vertices.push(v(x, y));

                    // Bottom left triangle.
                    push_v(l, t);
                    push_v(r, b);
                    push_v(l, b);
                    // Top right triangle.
                    push_v(l, t);
                    push_v(r, b);
                    push_v(r, t);
                }
                PrimitiveKind::Geometry { color, triangles } => {
                    if triangles.is_empty() {
                        continue;
                    }

                    match current_state {
                        State::Plain { .. } => (),
                        State::Image { image_id, start } => {
                            commands.push(DrawCommand::Image(start..vertices.len(), image_id));
                            current_state = State::Plain {
                                start: vertices.len(),
                            };
                        }
                    }

                    let color = color.gamma_srgb_to_linear()
                        .pre_multiply()
                        .to_fsa();

                    let v = |p: Position| Vertex {
                        position: [p.x as f32, p.y as f32, 0.0],
                        tex_coords: [0.0, 0.0],
                        rgba: color,
                        mode: MODE_GEOMETRY,
                    };

                    for triangle in triangles {
                        vertices.push(v(triangle[0]));
                        vertices.push(v(triangle[1]));
                        vertices.push(v(triangle[2]));
                    }
                }
                PrimitiveKind::TrianglesMultiColor { triangles } => {
                    todo!()
                }
                PrimitiveKind::Text { color, text: glyphs, } => {
                    match current_state {
                        State::Plain { .. } => (),
                        State::Image { image_id, start } => {
                            commands.push(DrawCommand::Image(start..vertices.len(), image_id));
                            current_state = State::Plain {
                                start: vertices.len(),
                            };
                        }
                    }

                    let color = color.gamma_srgb_to_linear().to_fsa();
                    //let texture_atlas = env.get_font_atlas();

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
                            vertices.push(v_color(x, y, t));
                        } else {
                            vertices.push(v_normal(x, y, t));
                        }
                    };

                    for glyph in glyphs {
                        let _position = glyph.position();
                        if let Some(bb) = glyph.bb() {
                            let (left, right, bottom, top) = bb.l_r_b_t_scaled(scale_factor);

                            if let Some(index) = glyph.atlas_entry() {
                                if !index.borrow().is_active {
                                    println!(
                                        "Trying to show glyph that is not in the texture atlas."
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
                PrimitiveKind::Image { image_id, color, source_rect, mode} => {
                    let image_ref = match env.image_map.get(&image_id) {
                        None => {
                            println!("Image missing in map: {:?}", image_id);
                            continue
                        },
                        Some(img) => img,
                    };

                    // Switch to the `Image` state for this image if we're not in it already.
                    let new_image_id = image_id;
                    match &current_state {
                        // If we're already in the drawing mode for this image, we're done.
                        State::Image { image_id, .. } if image_id == &new_image_id => (),

                        // If we were in the `Plain` drawing state, switch to Image drawing state.
                        State::Plain { start } => {
                            commands.push(DrawCommand::Geometry(*start..vertices.len()));
                            current_state = State::Image {
                                image_id: new_image_id,
                                start: vertices.len(),
                            };
                        }

                        // If we were drawing a different image, switch state to draw *this* image.
                        State::Image { image_id, start } => {
                            commands.push(DrawCommand::Image(*start..vertices.len(), image_id.clone()));
                            current_state = State::Image {
                                image_id: new_image_id,
                                start: vertices.len(),
                            };
                        }
                    }

                    let color = color.unwrap_or(color::WHITE);
                    let (image_w, image_h) = image_ref.dimensions();
                    let (image_w, image_h) = (image_w as Scalar, image_h as Scalar);

                    // Get the sides of the source rectangle as uv coordinates.
                    //
                    // Texture coordinates range:
                    // - left to right: 0.0 to 1.0
                    // - bottom to top: 1.0 to 0.0
                    let (uv_l, uv_r, uv_b, uv_t) = match source_rect {
                        Some(src_rect) => {
                            let (l, r, b, t) = src_rect.l_r_b_t();
                            (
                                (l / image_w) as f32,
                                (r / image_w) as f32,
                                (b / image_h) as f32,
                                (t / image_h) as f32,
                            )
                        }
                        None => (0.0, 1.0, 0.0, 1.0),
                    };

                    let v = |x, y, t| Vertex {
                        position: [x as f32, y as f32, 0.0],
                        tex_coords: t,
                        rgba: color.gamma_srgb_to_linear().to_fsa(),
                        mode,
                    };

                    let mut push_v = |x, y, t| vertices.push(v(x, y, t));

                    // Swap bottom and top to suit reversed vulkan coords.
                    let (l, r, b, t) = primitive.bounding_box.l_r_b_t();

                    // Bottom left triangle.
                    push_v(l, t, [uv_l, uv_t]);
                    push_v(r, b, [uv_r, uv_b]);
                    push_v(l, b, [uv_l, uv_b]);

                    // Top right triangle.
                    push_v(l, t, [uv_l, uv_t]);
                    push_v(r, t, [uv_r, uv_t]);
                    push_v(r, b, [uv_r, uv_b]);
                }
                PrimitiveKind::Gradient(triangles, gradient) => {
                    match current_state {
                        State::Plain { start } => {
                            commands.push(DrawCommand::Geometry(start..vertices.len()))
                        }
                        State::Image { image_id, start } => {
                            commands.push(DrawCommand::Image(start..vertices.len(), image_id))
                        }
                    }

                    let v = |p: Position| Vertex {
                        position: [p.x as f32, p.y as f32, 0.0],
                        tex_coords: [0.0, 0.0],
                        rgba: [0.0, 0.0, 0.0, 1.0],
                        mode: MODE_GEOMETRY,
                    };

                    let len_before_push = vertices.len();

                    for triangle in triangles {
                        vertices.push(v(triangle[0]));
                        vertices.push(v(triangle[1]));
                        vertices.push(v(triangle[2]));
                    }

                    commands.push(DrawCommand::Gradient(
                        len_before_push..vertices.len(),
                        gradient,
                    ));

                    current_state = State::Plain {
                        start: vertices.len(),
                    };
                }
            }
        }

        // Enter the final command.
        match current_state {
            State::Plain { start } => commands.push(DrawCommand::Geometry(start..vertices.len())),
            State::Image { image_id, start } => {
                commands.push(DrawCommand::Image(start..vertices.len(), image_id))
            }
        }
    }
}