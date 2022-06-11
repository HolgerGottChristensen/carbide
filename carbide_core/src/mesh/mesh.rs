//! A mesh type dedicated to converting sequences of `render::Primitive`s to a representation in
//! vertices ready for uploading to the GPU.
//!
//! While populating the vertices buffer ready for uploading to the GPU, the `Mesh` will also
//! produce a sequence of commands describing the order in which draw commands should occur and
//! whether or not the `Scizzor` should be updated between draws.

use std::{fmt, ops};

use cgmath::{Matrix4, SquareMatrix, Vector3};
use image::{DynamicImage, GenericImage, GenericImageView};
use rusttype::gpu_cache::Cache as RustTypeGlyphCache;
use rusttype::gpu_cache::CacheWriteErr as RustTypeCacheWriteError;

use crate::{color, image_map};
use crate::draw::{Position, Rect, Scalar};
use crate::draw::draw_gradient::DrawGradient;
use crate::environment::Environment;
use crate::layout::BasicLayouter;
use crate::mesh::{
    DEFAULT_GLYPH_CACHE_DIMS, GLYPH_CACHE_POSITION_TOLERANCE, GLYPH_CACHE_SCALE_TOLERANCE,
    MODE_GEOMETRY, MODE_TEXT, MODE_TEXT_COLOR,
};
use crate::mesh::atlas::texture_atlas::TextureAtlas;
use crate::mesh::vertex::Vertex;
use crate::render::{PrimitiveKind, PrimitiveWalker};

/// Images within the given image map must know their dimensions in pixels.
pub trait ImageDimensions {
    /// The dimensions of the image in pixels.
    fn dimensions(&self) -> [u32; 2];
}

/// A mesh whose vertices may be populated by a list of render primitives.
///
/// This is a convenience type for simplifying backend implementations.
#[derive(Debug)]
pub struct Mesh {
    // TODO: Consider mooving glyphcache and atlas to env, such that we can cache texture coords.
    glyph_cache: GlyphCache,
    glyph_cache_pixel_buffer: Vec<u8>,
    texture_atlas: TextureAtlas,
    texture_atlas_image: DynamicImage,
    commands: Vec<PreparedCommand>,
    vertices: Vec<Vertex>,
}

/// Represents the scissor in pixel coordinates.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Scissor {
    /// The top left of the scissor rectangle, where the top-left corner of the viewport is [0, 0].
    pub top_left: [i32; 2],
    /// The dimensions of the `Scissor` rect.
    pub dimensions: [u32; 2],
}

/// A `Command` describing a step in the drawing process.
#[derive(Clone, Debug)]
pub enum Command {
    /// Draw to the target.
    Draw(Draw),
    /// Update the scizzor within the pipeline.
    Scissor(Scissor),
    Stencil(std::ops::Range<usize>),
    DeStencil(std::ops::Range<usize>),
    Transform(Matrix4<f32>),
    Filter(std::ops::Range<usize>, u32),
    FilterSplitPt1(std::ops::Range<usize>, u32),
    FilterSplitPt2(std::ops::Range<usize>, u32),
}

/// An iterator yielding `Command`s, produced by the `Renderer::commands` method.
pub struct Commands<'a> {
    commands: std::slice::Iter<'a, PreparedCommand>,
}

/// A `Command` for drawing to the target.
///
/// Each variant describes how to draw the contents of the vertex buffer.
#[derive(Clone, Debug)]
pub enum Draw {
    /// A range of vertices representing triangles textured with the image in the
    /// image_map at the given `widget::Id`.
    Image(image_map::Id, std::ops::Range<usize>),
    /// A range of vertices representing plain triangles.
    Plain(std::ops::Range<usize>),
    /// A range of vertices that should be drawn as a gradient
    Gradient(std::ops::Range<usize>, DrawGradient),
}

/// The result of filling the mesh.
///
/// Provides information on whether or not the glyph cache has been updated and requires
/// re-uploading to the GPU.
#[allow(missing_copy_implementations)]
pub struct Fill {
    /// Whether or not the glyph cache pixel data should be written to the GPU.
    pub glyph_cache_requires_upload: bool,
    /// Whether or not the atlas pixel data should be written to the GPU.
    pub atlas_requires_upload: bool,
}

// A wrapper around an owned glyph cache, providing `Debug` and `Deref` impls.
struct GlyphCache(RustTypeGlyphCache<'static>);

#[derive(Debug)]
enum PreparedCommand {
    Image(image_map::Id, std::ops::Range<usize>),
    Plain(std::ops::Range<usize>),
    Gradient(std::ops::Range<usize>, DrawGradient),
    Scissor(Scissor),
    Stencil(std::ops::Range<usize>),
    DeStencil(std::ops::Range<usize>),
    Transform(Matrix4<f32>),
    Filter(std::ops::Range<usize>, u32),
    FilterSplitPt1(std::ops::Range<usize>, u32),
    FilterSplitPt2(std::ops::Range<usize>, u32),
}

impl Mesh {
    /// Construct a new empty `Mesh` with default glyph cache dimensions.
    pub fn new() -> Self {
        Self::with_glyph_cache_dimensions(DEFAULT_GLYPH_CACHE_DIMS)
    }

    /// Construct a `Mesh` with the given glyph cache dimensions.
    pub fn with_glyph_cache_dimensions(glyph_cache_dims: [u32; 2]) -> Self {
        let [gc_width, gc_height] = glyph_cache_dims;

        let glyph_cache = RustTypeGlyphCache::builder()
            .dimensions(gc_width, gc_height)
            .scale_tolerance(GLYPH_CACHE_SCALE_TOLERANCE)
            .position_tolerance(GLYPH_CACHE_POSITION_TOLERANCE)
            .build()
            .into();
        let glyph_cache_pixel_buffer = vec![0u8; gc_width as usize * gc_height as usize];
        let commands = vec![];
        let vertices = vec![];
        Mesh {
            glyph_cache,
            glyph_cache_pixel_buffer,
            texture_atlas: TextureAtlas::new(512, 512),
            texture_atlas_image: DynamicImage::new_rgba8(512, 512),
            commands,
            vertices,
        }
    }

    /// Fill the inner vertex buffer from the given primitives.
    ///
    /// - `viewport`: the window in which the UI is drawn. The width and height should be the
    ///   physical size (pixels).
    /// - `scale_factor`: the factor for converting from carbide's DPI agnostic point space to the
    ///   pixel space of the viewport.
    /// - `image_map`: a map from image IDs to images.
    /// - `primitives`: the sequence of UI primitives in order of depth to be rendered.
    pub fn fill<P, I>(
        &mut self,
        viewport: Rect,
        env: &mut Environment,
        image_map: &image_map::ImageMap<I>,
        mut primitives: P,
    ) -> Result<Fill, RustTypeCacheWriteError>
        where
            P: PrimitiveWalker,
            I: ImageDimensions,
    {
        let scale_factor = env.get_scale_factor();

        let Mesh {
            ref mut glyph_cache,
            glyph_cache_pixel_buffer: _,
            ref mut commands,
            ref mut vertices,
            texture_atlas: _,
            ref mut texture_atlas_image,
        } = *self;

        commands.clear();
        vertices.clear();

        // Keep track of whether or not the glyph cache texture needs to be updated.
        let glyph_cache_requires_upload = false;
        let mut atlas_requires_upload = false;

        let texture_atlas = env.get_font_atlas_mut();
        texture_atlas.cache_queued(|x, y, image_data| {

            //println!("Insert the image at: {}, {} with size {}, {}", x, y, image_data.width(), image_data.height());
            for (ix, iy, pixel) in image_data.pixels() {
                texture_atlas_image.put_pixel(x + ix, y + iy, pixel);
            }

            atlas_requires_upload = true;
        });

        enum State {
            Image {
                image_id: image_map::Id,
                start: usize,
            },
            Plain {
                start: usize,
            },
        }

        let mut current_state = State::Plain { start: 0 };

        // Viewport dimensions and the "dots per inch" factor.
        let _half_viewport_w = viewport.width() / 2.0;
        let _half_viewport_h = viewport.height() / 2.0;

        // Width of the glyph cache is useful when writing to the pixel buffer.
        let (glyph_cache_w, _) = glyph_cache.dimensions();
        let _glyph_cache_w = glyph_cache_w as usize;

        // Functions for converting for carbide scalar coords to normalised vertex coords (-1.0 to 1.0).
        let vx = |x: Scalar| x as f32;//(x * scale_factor / half_viewport_w - 1.0) as f32;
        let vy = |y: Scalar| y as f32;//-1.0 * (y * scale_factor / half_viewport_h - 1.0) as f32;

        let rect_to_scissor = |rect: Rect| {
            // We need to restrict the scissor x and y to [0, ~].
            // This means we might need to subtract from the width and height.

            let mut x = rect.position.x;
            let mut y = rect.position.y;
            let mut width = rect.dimension.width;
            let mut height = rect.dimension.height;

            // Make the x in range
            if x < 0.0 {
                width -= x;
                x = 0.0;
            }

            // Make the y in range
            if y < 0.0 {
                height -= y;
                y = 0.0;
            }

            // Make the width in range
            if x + width > viewport.width() {
                width = viewport.width() - x
            }

            // Make the height in range
            if y + height > viewport.height() {
                height = viewport.height() - y
            }

            Scissor {
                top_left: [x as i32, y as i32],
                dimensions: [width as u32, height as u32],
            }
        };

        // Keep track of the scissor as it changes.
        let mut scissor_stack = vec![rect_to_scissor(viewport)];
        let mut stencil_stack = vec![];
        let mut transform_stack = vec![Matrix4::identity()];

        commands.push(PreparedCommand::Scissor(*scissor_stack.first().unwrap()));

        // Switches to the `Plain` state and completes the previous `Command` if not already in the
        // `Plain` state.
        macro_rules! switch_to_plain_state {
            () => {
                match current_state {
                    State::Plain { .. } => (),
                    State::Image { image_id, start } => {
                        commands.push(PreparedCommand::Image(image_id, start..vertices.len()));
                        current_state = State::Plain {
                            start: vertices.len(),
                        };
                    }
                }
            };
        }

        // Draw each primitive in order of depth.
        while let Some(primitive) = primitives.next_primitive() {
            let rectangle = primitive.bounding_box;
            match primitive.kind {
                PrimitiveKind::Stencil(triangles) => {
                    match current_state {
                        State::Plain { start } => {
                            commands.push(PreparedCommand::Plain(start..vertices.len()))
                        }
                        State::Image { image_id, start } => {
                            commands.push(PreparedCommand::Image(image_id, start..vertices.len()))
                        }
                    }

                    let start_index_for_stencil = vertices.len();

                    let v = |p: Position| Vertex {
                        position: [vx(p.x), vy(p.y), 0.0],
                        tex_coords: [0.0, 0.0],
                        rgba: [1.0, 1.0, 1.0, 1.0],
                        mode: MODE_GEOMETRY,
                    };

                    for triangle in &triangles {
                        vertices.push(v(triangle[0]));
                        vertices.push(v(triangle[1]));
                        vertices.push(v(triangle[2]));
                    }

                    stencil_stack.push(triangles);

                    commands.push(PreparedCommand::Stencil(
                        start_index_for_stencil..vertices.len(),
                    ));

                    current_state = State::Plain {
                        start: vertices.len(),
                    };
                }
                PrimitiveKind::DeStencil => {
                    match current_state {
                        State::Plain { start } => {
                            commands.push(PreparedCommand::Plain(start..vertices.len()))
                        }
                        State::Image { image_id, start } => {
                            commands.push(PreparedCommand::Image(image_id, start..vertices.len()))
                        }
                    }

                    if let Some(triangles) = stencil_stack.pop() {
                        let start_index_for_de_stencil = vertices.len();

                        let v = |p: Position| Vertex {
                            position: [vx(p.x), vy(p.y), 0.0],
                            tex_coords: [0.0, 0.0],
                            rgba: [1.0, 1.0, 1.0, 1.0],
                            mode: MODE_GEOMETRY,
                        };

                        for triangle in &triangles {
                            vertices.push(v(triangle[0]));
                            vertices.push(v(triangle[1]));
                            vertices.push(v(triangle[2]));
                        }

                        commands.push(PreparedCommand::DeStencil(
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
                    match current_state {
                        State::Plain { start } => {
                            commands.push(PreparedCommand::Plain(start..vertices.len()))
                        }
                        State::Image { image_id, start } => {
                            commands.push(PreparedCommand::Image(image_id, start..vertices.len()))
                        }
                    }

                    let start_index_for_filter = vertices.len();

                    let v = |p: Position| Vertex {
                        position: [vx(p.x), vy(p.y), 0.0],
                        tex_coords: [(p.x / viewport.dimension.width * scale_factor) as f32, (p.y / viewport.dimension.height * scale_factor) as f32],
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

                    commands.push(PreparedCommand::Filter(
                        start_index_for_filter..vertices.len(), filter_id,
                    ));

                    current_state = State::Plain {
                        start: vertices.len(),
                    };
                }
                PrimitiveKind::FilterSplitPt1(filter_id) => {
                    match current_state {
                        State::Plain { start } => {
                            commands.push(PreparedCommand::Plain(start..vertices.len()))
                        }
                        State::Image { image_id, start } => {
                            commands.push(PreparedCommand::Image(image_id, start..vertices.len()))
                        }
                    }

                    let start_index_for_filter = vertices.len();

                    let v = |p: Position| Vertex {
                        position: [vx(p.x), vy(p.y), 0.0],
                        tex_coords: [(p.x / viewport.dimension.width * scale_factor) as f32, (p.y / viewport.dimension.height * scale_factor) as f32],
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

                    commands.push(PreparedCommand::FilterSplitPt1(
                        start_index_for_filter..vertices.len(), filter_id,
                    ));

                    current_state = State::Plain {
                        start: vertices.len(),
                    };
                }
                PrimitiveKind::FilterSplitPt2(filter_id) => {
                    match current_state {
                        State::Plain { start } => {
                            commands.push(PreparedCommand::Plain(start..vertices.len()))
                        }
                        State::Image { image_id, start } => {
                            commands.push(PreparedCommand::Image(image_id, start..vertices.len()))
                        }
                    }

                    let start_index_for_filter = vertices.len();

                    let v = |p: Position| Vertex {
                        position: [vx(p.x), vy(p.y), 0.0],
                        tex_coords: [(p.x / viewport.dimension.width * scale_factor) as f32, (p.y / viewport.dimension.height * scale_factor) as f32],
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

                    commands.push(PreparedCommand::FilterSplitPt2(
                        start_index_for_filter..vertices.len(), filter_id,
                    ));

                    current_state = State::Plain {
                        start: vertices.len(),
                    };
                }
                PrimitiveKind::Transform(matrix, alignment) => {
                    match current_state {
                        State::Plain { start } => {
                            commands.push(PreparedCommand::Plain(start..vertices.len()))
                        }
                        State::Image { image_id, start } => {
                            commands.push(PreparedCommand::Image(image_id, start..vertices.len()))
                        }
                    }

                    let latest_transform = &transform_stack[transform_stack.len() - 1];

                    let new_transform = match alignment {
                        BasicLayouter::TopLeading => {
                            let center_x = (rectangle.position.x) as f32;
                            let center_y = (rectangle.position.y) as f32;
                            latest_transform * Matrix4::from_translation(Vector3::new(center_x, center_y, 0.0)) * matrix * Matrix4::from_translation(Vector3::new(-center_x, -center_y, 0.0))
                        }
                        BasicLayouter::Top => {
                            let center_x = (rectangle.position.x + rectangle.dimension.width / 2.0) as f32;
                            let center_y = (rectangle.position.y) as f32;
                            latest_transform * Matrix4::from_translation(Vector3::new(center_x, center_y, 0.0)) * matrix * Matrix4::from_translation(Vector3::new(-center_x, -center_y, 0.0))
                        }
                        BasicLayouter::TopTrailing => {
                            let center_x = (rectangle.position.x + rectangle.dimension.width) as f32;
                            let center_y = (rectangle.position.y) as f32;
                            latest_transform * Matrix4::from_translation(Vector3::new(center_x, center_y, 0.0)) * matrix * Matrix4::from_translation(Vector3::new(-center_x, -center_y, 0.0))
                        }
                        BasicLayouter::Leading => {
                            let center_x = (rectangle.position.x) as f32;
                            let center_y = (rectangle.position.y + rectangle.dimension.height / 2.0) as f32;
                            latest_transform * Matrix4::from_translation(Vector3::new(center_x, center_y, 0.0)) * matrix * Matrix4::from_translation(Vector3::new(-center_x, -center_y, 0.0))
                        }
                        BasicLayouter::Center => {
                            let center_x = (rectangle.position.x + rectangle.dimension.width / 2.0) as f32;
                            let center_y = (rectangle.position.y + rectangle.dimension.height / 2.0) as f32;
                            latest_transform * Matrix4::from_translation(Vector3::new(center_x, center_y, 0.0)) * matrix * Matrix4::from_translation(Vector3::new(-center_x, -center_y, 0.0))
                        }
                        BasicLayouter::Trailing => {
                            let center_x = (rectangle.position.x + rectangle.dimension.width) as f32;
                            let center_y = (rectangle.position.y + rectangle.dimension.height / 2.0) as f32;
                            latest_transform * Matrix4::from_translation(Vector3::new(center_x, center_y, 0.0)) * matrix * Matrix4::from_translation(Vector3::new(-center_x, -center_y, 0.0))
                        }
                        BasicLayouter::BottomLeading => {
                            let center_x = (rectangle.position.x) as f32;
                            let center_y = (rectangle.position.y + rectangle.dimension.height) as f32;
                            latest_transform * Matrix4::from_translation(Vector3::new(center_x, center_y, 0.0)) * matrix * Matrix4::from_translation(Vector3::new(-center_x, -center_y, 0.0))
                        }
                        BasicLayouter::Bottom => {
                            let center_x = (rectangle.position.x + rectangle.dimension.width / 2.0) as f32;
                            let center_y = (rectangle.position.y + rectangle.dimension.height) as f32;
                            latest_transform * Matrix4::from_translation(Vector3::new(center_x, center_y, 0.0)) * matrix * Matrix4::from_translation(Vector3::new(-center_x, -center_y, 0.0))
                        }
                        BasicLayouter::BottomTrailing => {
                            let center_x = (rectangle.position.x + rectangle.dimension.width) as f32;
                            let center_y = (rectangle.position.y + rectangle.dimension.height) as f32;
                            latest_transform * Matrix4::from_translation(Vector3::new(center_x, center_y, 0.0)) * matrix * Matrix4::from_translation(Vector3::new(-center_x, -center_y, 0.0))
                        }
                    };

                    transform_stack.push(new_transform);

                    commands.push(PreparedCommand::Transform(new_transform));

                    current_state = State::Plain {
                        start: vertices.len(),
                    };
                }
                PrimitiveKind::DeTransform => {
                    match current_state {
                        State::Plain { start } => {
                            commands.push(PreparedCommand::Plain(start..vertices.len()))
                        }
                        State::Image { image_id, start } => {
                            commands.push(PreparedCommand::Image(image_id, start..vertices.len()))
                        }
                    }

                    transform_stack.pop();
                    commands.push(PreparedCommand::Transform(*&transform_stack[transform_stack.len() - 1]));

                    current_state = State::Plain {
                        start: vertices.len(),
                    };
                }
                PrimitiveKind::Clip => {
                    match current_state {
                        State::Plain { start } => {
                            commands.push(PreparedCommand::Plain(start..vertices.len()))
                        }
                        State::Image { image_id, start } => {
                            commands.push(PreparedCommand::Image(image_id, start..vertices.len()))
                        }
                    }

                    let (mut l, mut r, mut b, mut t) = primitive.bounding_box.l_r_b_t();

                    l *= scale_factor;
                    r *= scale_factor;
                    t *= scale_factor;
                    b *= scale_factor;

                    let new_rect = Rect::from_corners(Position::new(r, b), Position::new(l, t));

                    commands.push(PreparedCommand::Scissor(rect_to_scissor(new_rect)));

                    scissor_stack.push(rect_to_scissor(new_rect));

                    current_state = State::Plain {
                        start: vertices.len(),
                    };
                }
                PrimitiveKind::UnClip => {
                    match current_state {
                        State::Plain { start } => {
                            commands.push(PreparedCommand::Plain(start..vertices.len()))
                        }
                        State::Image { image_id, start } => {
                            commands.push(PreparedCommand::Image(image_id, start..vertices.len()))
                        }
                    }

                    scissor_stack.pop();

                    let new_scizzor = match scissor_stack.first() {
                        Some(n) => n,
                        None => panic!("Trying to pop scizzor, when there is none on the stack"),
                    };

                    commands.push(PreparedCommand::Scissor(*new_scizzor));

                    current_state = State::Plain {
                        start: vertices.len(),
                    };
                }
                PrimitiveKind::RectanglePrim { color } => {
                    switch_to_plain_state!();

                    let color = gamma_srgb_to_linear(color.to_fsa());
                    let (l, r, b, t) = primitive.bounding_box.l_r_b_t();

                    let v = |x, y| {
                        // Convert from carbide Scalar range to GL range -1.0 to 1.0.
                        Vertex {
                            position: [vx(x), vy(y), 0.0],
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
                PrimitiveKind::TrianglesSingleColor { color, triangles } => {
                    if triangles.is_empty() {
                        continue;
                    }

                    switch_to_plain_state!();

                    let color = gamma_srgb_to_linear(color.into());
                    let pre_multiplied_color = [color[0] * color[3], color[1] * color[3], color[2] * color[3], color[3]];

                    let v = |p: Position| Vertex {
                        position: [vx(p.x), vy(p.y), 0.0],
                        tex_coords: [0.0, 0.0],
                        rgba: pre_multiplied_color,
                        mode: MODE_GEOMETRY,
                    };

                    for triangle in triangles {
                        vertices.push(v(triangle[0]));
                        vertices.push(v(triangle[1]));
                        vertices.push(v(triangle[2]));
                    }
                }
                PrimitiveKind::TrianglesMultiColor { triangles } => {
                    if triangles.is_empty() {
                        continue;
                    }

                    switch_to_plain_state!();

                    let v = |(p, c): (Position, color::Rgba)| Vertex {
                        position: [vx(p.x), vy(p.y), 0.0],
                        tex_coords: [0.0, 0.0],
                        rgba: gamma_srgb_to_linear(c.into()),
                        mode: MODE_GEOMETRY,
                    };

                    for triangle in triangles {
                        vertices.push(v(triangle[0]));
                        vertices.push(v(triangle[1]));
                        vertices.push(v(triangle[2]));
                    }
                }
                PrimitiveKind::Text { color, text: glyphs, } => {
                    switch_to_plain_state!();
                    let color = gamma_srgb_to_linear(color.to_fsa());
                    //let texture_atlas = env.get_font_atlas();

                    let v_normal = |x, y, t| Vertex {
                        position: [vx(x), vy(y), 0.0],
                        tex_coords: t,
                        rgba: color,
                        mode: MODE_TEXT,
                    };

                    let v_color = |x, y, t| Vertex {
                        position: [vx(x), vy(y), 0.0],
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
                                    println!("Trying to show glyph that is not in the texture atlas.");
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
                PrimitiveKind::Image { image_id, color, source_rect, mode, } => {
                    let image_ref = match image_map.get(&image_id) {
                        None => continue,
                        Some(img) => img,
                    };

                    // Switch to the `Image` state for this image if we're not in it already.
                    let new_image_id = image_id;
                    match current_state {
                        // If we're already in the drawing mode for this image, we're done.
                        State::Image { image_id, .. } if image_id == new_image_id => (),

                        // If we were in the `Plain` drawing state, switch to Image drawing state.
                        State::Plain { start } => {
                            commands.push(PreparedCommand::Plain(start..vertices.len()));
                            current_state = State::Image {
                                image_id: new_image_id,
                                start: vertices.len(),
                            };
                        }

                        // If we were drawing a different image, switch state to draw *this* image.
                        State::Image { image_id, start } => {
                            commands.push(PreparedCommand::Image(image_id, start..vertices.len()));
                            current_state = State::Image {
                                image_id: new_image_id,
                                start: vertices.len(),
                            };
                        }
                    }

                    let color = color.unwrap_or(color::WHITE).to_fsa();
                    let [image_w, image_h] = image_ref.dimensions();
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

                    let v = |x, y, t| {
                        Vertex {
                            position: [x as f32, y as f32, 0.0],
                            tex_coords: t,
                            rgba: gamma_srgb_to_linear(color),
                            mode,
                        }
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
                            commands.push(PreparedCommand::Plain(start..vertices.len()))
                        }
                        State::Image { image_id, start } => {
                            commands.push(PreparedCommand::Image(image_id, start..vertices.len()))
                        }
                    }


                    let v = |p: Position| Vertex {
                        position: [vx(p.x), vy(p.y), 0.0],
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

                    commands.push(PreparedCommand::Gradient(len_before_push..vertices.len(), gradient));

                    current_state = State::Plain {
                        start: vertices.len(),
                    };
                }
            }
        }

        // Enter the final command.
        match current_state {
            State::Plain { start } => commands.push(PreparedCommand::Plain(start..vertices.len())),
            State::Image { image_id, start } => {
                commands.push(PreparedCommand::Image(image_id, start..vertices.len()))
            }
        }

        let fill = Fill {
            glyph_cache_requires_upload,
            atlas_requires_upload,
        };

        Ok(fill)
    }

    pub fn texture_atlas(&self) -> &TextureAtlas {
        &self.texture_atlas
    }

    pub fn texture_atlas_image(&self) -> &DynamicImage {
        &self.texture_atlas_image
    }

    pub fn texture_atlas_image_as_bytes(&self) -> &[u8] {
        /*println!(
            "Number of bytes: {}",
            &self.texture_atlas_image.as_bytes().len()
        );*/
        &self.texture_atlas_image.as_bytes()
    }

    /// The rusttype glyph cache used for managing caching of glyphs into the pixel buffer.
    pub fn glyph_cache(&self) -> &RustTypeGlyphCache {
        &self.glyph_cache.0
    }

    /// The CPU-side of the glyph cache, storing all necessary pixel data in a single slice.
    pub fn glyph_cache_pixel_buffer(&self) -> &[u8] {
        &self.glyph_cache_pixel_buffer
    }

    /// Produce an `Iterator` yielding `Command`s.
    ///
    /// These commands describe the order in which unique draw commands and scizzor updates should
    /// occur.
    pub fn commands(&self) -> Commands {
        let Mesh { ref commands, .. } = *self;
        Commands {
            commands: commands.iter(),
        }
    }

    /// The slice containing all `vertices` produced by the `fill` function.
    ///
    /// Note that these vertices may be represent geometry across multiple `Command`s.
    pub fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }
}

impl<'a> Iterator for Commands<'a> {
    type Item = Command;
    fn next(&mut self) -> Option<Self::Item> {
        let Commands { ref mut commands } = *self;
        commands.next().map(|command| match *command {
            PreparedCommand::Scissor(scizzor) => Command::Scissor(scizzor),
            PreparedCommand::Plain(ref range) => Command::Draw(Draw::Plain(range.clone())),
            PreparedCommand::Gradient(ref range, ref gradient) => Command::Draw(Draw::Gradient(range.clone(), gradient.clone())),
            PreparedCommand::Image(id, ref range) => Command::Draw(Draw::Image(id, range.clone())),
            PreparedCommand::Stencil(ref range) => Command::Stencil(range.clone()),
            PreparedCommand::DeStencil(ref range) => Command::DeStencil(range.clone()),
            PreparedCommand::Transform(ref transform) => Command::Transform(*transform),
            PreparedCommand::Filter(ref range, filter_id) => Command::Filter(range.clone(), filter_id),
            PreparedCommand::FilterSplitPt1(ref range, filter_id) => Command::FilterSplitPt1(range.clone(), filter_id),
            PreparedCommand::FilterSplitPt2(ref range, filter_id) => Command::FilterSplitPt2(range.clone(), filter_id),
        })
    }
}

impl ops::Deref for GlyphCache {
    type Target = RustTypeGlyphCache<'static>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for GlyphCache {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl fmt::Debug for GlyphCache {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "GlyphCache")
    }
}

impl From<RustTypeGlyphCache<'static>> for GlyphCache {
    fn from(gc: RustTypeGlyphCache<'static>) -> Self {
        GlyphCache(gc)
    }
}

fn gamma_srgb_to_linear(c: [f32; 4]) -> [f32; 4] {
    fn component(f: f32) -> f32 {
        // Taken from https://github.com/PistonDevelopers/graphics/src/color.rs#L42
        if f <= 0.04045 {
            f / 12.92
        } else {
            ((f + 0.055) / 1.055).powf(2.4)
        }
    }
    [component(c[0]), component(c[1]), component(c[2]), c[3]]
}
