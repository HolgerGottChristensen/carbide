use std::collections::{HashMap, HashSet};
use std::ops::Range;

use wgpu::BindGroup;

use crate::wgpu_gradient::{WgpuGradient};
use crate::render_context::TargetState::{Free, Used};
use crate::render_pass_command::{RenderPass, RenderPassCommand, WGPUBindGroup};
use crate::wgpu_render_target::WgpuRenderTarget;
use crate::wgpu_vertex::WgpuVertex;
use crate::wgpu_msaa::WgpuMsaa;
use crate::{MODE_GEOMETRY, MODE_GEOMETRY_DASH, MODE_GEOMETRY_DASH_FAST, MODE_GRADIENT_GEOMETRY, MODE_GRADIENT_GEOMETRY_DASH, MODE_GRADIENT_GEOMETRY_DASH_FAST, MODE_GRADIENT_ICON, MODE_GRADIENT_TEXT, MODE_ICON, MODE_IMAGE, MODE_TEXT, MODE_TEXT_COLOR};

use carbide_core::color::{Color, ColorExt, WHITE};
use carbide_core::draw::stroke::{StrokeDashMode, StrokeDashPattern};
use carbide_core::draw::{Dimension, DrawOptions, CompositeDrawShape, DrawStyle, ImageId, ImageMode, ImageOptions, Position, Rect, DrawShape, Scalar};
use carbide_core::environment::Environment;
use carbide_core::math::{Matrix4, SquareMatrix, Vector3};
use carbide_core::render::{InnerRenderContext, Layer, LayerId};
use carbide_core::text::glyph::GlyphRenderMode;
use carbide_core::text::{TextContext, TextId, TextStyle};
use carbide_core::widget::{FilterId, ImageFilter};
use carbide_tesselation::stroke_vertex::StrokeVertex;
use carbide_tesselation::triangle::Triangle;
use carbide_tesselation::{Tesselator, TesselatorKey};
use crate::wgpu_dashes::WgpuDashes;
use crate::window::util::calculate_carbide_to_wgpu_matrix;

/// Look up the tesselator currently in the environment. This is done every time a shape
/// is tesselated, such that the tesselator can be switched while the application is running,
/// or scoped to a part of the widget tree using `Environment::with`.
///
/// # Panics
/// If no tesselator has been registered into the environment.
fn tesselator<'a>(env: &'a Environment) -> &'a dyn Tesselator {
    &**env.get::<TesselatorKey>()
        .expect("No tesselator registered in the environment. Register one using TesselatorKey, for example by depending on carbide_lyon.")
}

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
    gradients: Vec<WgpuGradient>,
    filters: HashMap<FilterId, ImageFilter>,
    // Which target index a given filter samples its source from, so the backend can size that
    // filter's `texture_size` uniform to the source target's *actual* (possibly padded) texture
    // dimensions instead of assuming it's always window-sized.
    filter_source_target: HashMap<FilterId, usize>,
    vertices: Vec<WgpuVertex>,

    render_pass: Vec<RenderPass>,
    render_pass_inner: Vec<RenderPassCommand>,
    current_bind_group: Option<WGPUBindGroup>,
    current_gradient: Option<WgpuGradient>,
    current_stroke_dash: Option<StrokeDashPattern>,
    current_frame_filters: HashSet<FilterId>,

    finished: bool,
    targets: TargetStates,
    window_bounding_box: Rect,
    // Set once per frame in `start()`. Filter kernel radii (`radius_x`/`radius_y` passed to
    // `filter_new`) are physical pixels (see `Shadow::render`), but `window_bounding_box` and
    // everything built from it (`padded_transform`/`padded_bounding_box`) is logical -- this is
    // what converts between the two.
    scale_factor: Scalar,
    frame_count: usize,
    skip_rendering: bool,
    masked: bool,
    current_target: usize,

    layers: HashMap<LayerId, WgpuRenderTarget>,
    msaa: WgpuMsaa,

    tolerance_multiplier: f32,
}

#[derive(Debug, Clone)]
enum WGPUStyle {
    Color([f32; 4]),
    Gradient(WgpuGradient),
}

#[derive(Debug, PartialEq)]
enum TargetState {
    Free,
    Used
}

#[derive(Debug)]
struct TargetStates {
    // (state, (max radius_x, max radius_y) ever requested for this index, in logical pixels)
    inner: Vec<(TargetState, (u32, u32))>,
}

impl TargetStates {
    fn new() -> TargetStates {
        TargetStates {
            inner: vec![
                (Used, (0, 0))
            ],
        }
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    /// Get the index of the next free target and a bool indicating if the target needs clearing.
    /// `radius_x`/`radius_y` request that this target be usable by a filter whose kernel reaches
    /// that far beyond the window on each side; the index's recorded requirement only ever grows
    /// (the max across every call that has ever used this index), since a target already sized
    /// for a past, larger requirement remains valid for a smaller one.
    fn get(&mut self, radius_x: u32, radius_y: u32) -> (usize, bool) {
        if let Some((index, (state, radius))) = self.inner
            .iter_mut()
            .enumerate()
            .filter(|(_, (a, _))| *a != Used)
            .next() {
                let needs_free = *state == Free;
                *state = Used;
                radius.0 = radius.0.max(radius_x);
                radius.1 = radius.1.max(radius_y);
                (index, needs_free)
            } else {
                self.inner.push((Used, (radius_x, radius_y)));
                (self.inner.len() - 1, false)
            }
    }

    fn free(&mut self, index: usize) {
        self.inner[index].0 = Free;
    }

    /// The largest `(radius_x, radius_y)` ever requested via [`Self::get`] for this index —
    /// how far beyond the window (in logical pixels, each side) this target's texture needs to
    /// extend so every filter that has used this index can sample fully within its bounds.
    fn radius(&self, index: usize) -> (u32, u32) {
        self.inner[index].1
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
    pub fn new(msaa: WgpuMsaa) -> WGPURenderContext {
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
            scale_factor: 1.0,
            frame_count: 0,
            skip_rendering: false,
            current_gradient: None,
            masked: false,
            targets: TargetStates::new(),
            current_target: 0,
            current_stroke_dash: None,
            layers: Default::default(),
            msaa,
            filters: HashMap::new(),
            filter_source_target: HashMap::new(),
            current_frame_filters: HashSet::new(),
            tolerance_multiplier: 1.0,
        }
    }

    pub fn layer_bind_group(&self, layer_id: LayerId) -> &BindGroup {
        &self.layers[&layer_id].bind_group
    }

    pub fn target_count(&self) -> usize {
        self.targets.len()
    }

    /// The largest `(radius_x, radius_y)` (in logical pixels, on each side) any filter has ever
    /// requested for this target index via a `filter_new`-family call — how far beyond the
    /// window this target's texture needs to extend so off-screen geometry survives long enough
    /// to be sampled by that filter's blur/effect passes. `(0, 0)` for a target never used that
    /// way (e.g. the main target, or a mask scratch target).
    pub fn target_radius(&self, index: usize) -> (u32, u32) {
        self.targets.radius(index)
    }

    /// Which target index a given filter samples its source from, if it was registered via a
    /// `filter_new`-family call. Lets the caller size that filter's `texture_size` uniform to the
    /// source target's actual (possibly padded) texture dimensions.
    pub fn filter_source_target(&self, filter_id: FilterId) -> Option<usize> {
        self.filter_source_target.get(&filter_id).copied()
    }

    /// Beyond `self.window_bounding_box`, expanded by `radius_x`/`radius_y` (logical pixels) on
    /// every side -- the region a filter with that kernel reach needs its source geometry
    /// rasterized across for every *visible* (within-window) output pixel to sample correctly.
    fn padded_bounding_box(&self, radius_x: u32, radius_y: u32) -> Rect {
        let dimension = self.window_bounding_box.dimension;
        // `radius_x`/`radius_y` are physical pixels (a filter's kernel taps, see `Shadow::render`)
        // but this rect lives in the same logical space as `window_bounding_box`.
        let radius_x = radius_x as Scalar / self.scale_factor;
        let radius_y = radius_y as Scalar / self.scale_factor;
        Rect::new(
            Position::new(-radius_x, -radius_y),
            Dimension::new(dimension.width + 2.0 * radius_x, dimension.height + 2.0 * radius_y),
        )
    }

    /// A transform matrix that, once composed with the frame-wide `carbide_to_wgpu_matrix` (as
    /// every `Uniform.transform` is, in `ensure_uniforms_in_buffer`), makes ordinary
    /// window-absolute pixel coordinates -- including ones outside `[0, window_size]`, up to
    /// `radius_x`/`radius_y` beyond it -- map correctly into a same-radius-padded render target
    /// instead of being clipped at the window's own edge.
    ///
    /// Built by computing the projection a *padded* canvas would use on its own
    /// (`calculate_carbide_to_wgpu_matrix`, the exact function `OffscreenRenderer` uses for the
    /// real window), translating by `(radius_x, radius_y)` so window-pixel `(0, 0)` lands
    /// `radius_x`/`radius_y` texels inside that padded canvas rather than at its edge, and then
    /// cancelling out the frame-wide window matrix that will be multiplied on afterwards (which
    /// is mathematically identical to this same function called with the *unpadded* window
    /// dimensions -- both reduce to depending only on the logical dimension, since the physical
    /// dimension/scale-factor pairing `OffscreenRenderer` calls it with always cancels).
    fn padded_transform(&self, radius_x: u32, radius_y: u32) -> Matrix4<f32> {
        let window_dimension = self.window_bounding_box.dimension;
        // `radius_x`/`radius_y` are physical pixels (a filter's kernel taps, see `Shadow::render`)
        // but the padded canvas this builds a projection for is expressed in the same logical
        // space as `window_dimension`.
        let radius_x = radius_x as Scalar / self.scale_factor;
        let radius_y = radius_y as Scalar / self.scale_factor;
        let padded_dimension = Dimension::new(
            window_dimension.width + 2.0 * radius_x,
            window_dimension.height + 2.0 * radius_y,
        );

        let window_matrix = calculate_carbide_to_wgpu_matrix(window_dimension, 1.0);
        let padded_matrix = calculate_carbide_to_wgpu_matrix(padded_dimension, 1.0);
        let translate = Matrix4::from_translation(Vector3::new(radius_x as f32, radius_y as f32, 0.0));

        let inverse_window_matrix = window_matrix.invert()
            .expect("the window's pixel-to-clip-space projection should always be invertible");

        inverse_window_matrix * padded_matrix * translate
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

    pub fn vertices(&self) -> &Vec<WgpuVertex> {
        &self.vertices
    }

    pub fn uniforms(&self) -> &Vec<Uniform> {
        &self.uniforms
    }
    pub fn filters(&self) -> &HashMap<FilterId, ImageFilter> {
        &self.filters
    }

    pub fn gradients(&self) -> &Vec<WgpuGradient> {
        &self.gradients
    }

    pub fn start(&mut self, window_bounding_box: Rect, scale_factor: Scalar) {
        self.frame_count += 1;
        //println!("Start render frame: {}", self.frame_count);
        self.window_bounding_box = window_bounding_box;
        self.scale_factor = scale_factor;
        self.clear()
    }

    pub fn finish(&mut self) -> Vec<RenderPass> {
        if self.finished {
            panic!("Trying to finish a render context that is already in a finished state.");
        }

        self.finished = true;

        self.filters.retain(|a, _| self.current_frame_filters.contains(a));
        self.filter_source_target.retain(|a, _| self.current_frame_filters.contains(a));
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

    fn draw_raster_image(&mut self, id: Option<WGPUBindGroup>, bounding_box: Rect, source_rect: Rect, mut mode: u32) {
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

        let create_vertex = |x, y, tx, ty| WgpuVertex {
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
            // Dont send multiple uniform commands in a row, avoiding unnecessary bindings swaps.
            if let Some(RenderPassCommand::Uniform {..}) = commands.last() && let RenderPassCommand::Uniform {..} = command {
                commands.pop();
            }
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

    fn ensure_state_gradient(&mut self, gradient: &WgpuGradient) {
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

    fn add_clip_shape_vertices(&mut self, shape: &DrawShape, options: &DrawOptions, tesselator: &dyn Tesselator) {
        match options {
            DrawOptions::Fill(options) => {
                let triangles = tesselator.fill(shape, options, self.tolerance_multiplier).into_iter();
                self.vertices.extend(
                    triangles
                        .flat_map(|triangle| triangle.0)
                        .map(|position| WgpuVertex::new_from_2d(
                            position.x as f32,
                            position.y as f32,
                            [1.0, 1.0, 1.0, 1.0],
                            [0.0, 0.0],
                            MODE_GEOMETRY
                        ))
                );
            }
            DrawOptions::Stroke(options) => {
                let triangles = tesselator.stroke(shape, options, self.tolerance_multiplier).into_iter();
                self.vertices.extend(
                    triangles
                        .flat_map(|triangle| triangle.0)
                        .map(|position| WgpuVertex::new_from_2d(
                            position.position.x as f32,
                            position.position.y as f32,
                            [1.0, 1.0, 1.0, 1.0],
                            [0.0, 0.0],
                            MODE_GEOMETRY
                        ))
                );
            }
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
                        WgpuVertex::new_from_2d(
                            triangle.0[0].x as f32,
                            triangle.0[0].y as f32,
                            color,
                            [0.0, 0.0],
                            mode
                        ),
                        WgpuVertex::new_from_2d(
                            triangle.0[1].x as f32,
                            triangle.0[1].y as f32,
                            color,
                            [0.0, 0.0],
                            mode
                        ),
                        WgpuVertex::new_from_2d(
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

        let (color, gradient) = match style {
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

                self.push_command(RenderPassCommand::StrokeDashing (WgpuDashes {
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
                    WgpuVertex {
                        position: [
                            v.position.x as f32,
                            v.position.y as f32,
                            0.0
                        ],
                        tex_coords: [0.0, 0.0],
                        rgba: if gradient { Color::random().to_fsa() } else { color },
                        mode,
                        attributes0: [
                            v.start.x as f32,
                            v.start.y as f32,
                            v.end.x as f32,
                            v.end.y as f32,
                        ],
                        attributes1: [
                            v.start_angle.radians() as f32,
                            v.end_angle.radians() as f32,
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
    fn transform(&mut self, transform: &Matrix4<f32>) {
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

        let create_vertex = |x, y| WgpuVertex {
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
        let (new_target, _needs_clear) = self.targets.get(0, 0);

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

        let create_vertex = |x, y| WgpuVertex {
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

        let (new_target, needs_clear) = self.targets.get(0, 0);

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

    fn stencil(&mut self, shape: CompositeDrawShape, env: &mut Environment) {
        if self.skip_rendering {
            return;
        }

        let start = self.vertices.len();

        match &shape {
            CompositeDrawShape::Zero => {}
            CompositeDrawShape::One(shape, options) => {
                self.add_clip_shape_vertices(shape, options, tesselator(env));
            }
            CompositeDrawShape::Many(shapes) => {
                for (shape, options) in shapes {
                    self.add_clip_shape_vertices(shape, options, tesselator(env));
                }
            }
        }

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

    fn shape(&mut self, shape: &DrawShape, options: &DrawOptions, env: &mut Environment) {
        if self.skip_rendering {
            return;
        }

        let (latest_uniform, _) = &self.uniform_stack[self.uniform_stack.len() - 1];

        let multiplier = latest_uniform.transform[0][0]
            .max(latest_uniform.transform[1][1]);

        // Cap the tolerance to the div epsilon on lyon
        self.tolerance_multiplier = multiplier.max(1.0);

        let tesselator = tesselator(env);

        match options {
            DrawOptions::Fill(options) => {
                let triangles = tesselator.fill(shape, options, self.tolerance_multiplier).into_iter();
                self.fill(triangles);
            }
            DrawOptions::Stroke(options) => {
                let triangles = tesselator.stroke(shape, options, self.tolerance_multiplier).into_iter();
                self.stroke(triangles);
            }
        }
    }

    fn style(&mut self, style: &DrawStyle) {
        match style {
            DrawStyle::Color(color) => {
                let color = color.gamma_srgb_to_linear()
                    .pre_multiply()
                    .to_fsa();

                self.style_stack.push(WGPUStyle::Color(color));
            }
            DrawStyle::Gradient(g) => {
                self.style_stack.push(WGPUStyle::Gradient(WgpuGradient::convert(g)))
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

    fn raster_image(&mut self, id: &ImageId, bounding_box: Rect, options: ImageOptions) {
        let source_rect = options.source_rect.unwrap_or_else(|| Rect::new(Position::new(0.0, 0.0), Dimension::new(1.0, 1.0)));

        let mode = match options.mode {
            ImageMode::Image => MODE_IMAGE,
            ImageMode::Icon => MODE_ICON,
        };

        self.draw_raster_image(Some(WGPUBindGroup::Image(id.clone())), bounding_box, source_rect, mode)
    }

    fn text(&mut self, text: &str, style: &TextStyle, position: Position, requested_size: Option<Dimension>, env: &mut Environment, ctx: &mut dyn TextContext) {
        if self.skip_rendering {
            return;
        }

        ctx.render_new(text, style, position, requested_size, env, &mut |glyph| {
            let mode = match glyph.mode {
                GlyphRenderMode::Plain => MODE_TEXT,
                GlyphRenderMode::Colored => MODE_TEXT_COLOR,
            };

            self.draw_raster_image(None, glyph.bounding_box, glyph.texture_coords, mode);
        });
    }

    fn text_old(&mut self, text: TextId, ctx: &mut dyn TextContext) {
        if self.skip_rendering {
            return;
        }

        ctx.render(text, &mut |glyph| {
            let mode = match glyph.mode {
                GlyphRenderMode::Plain => MODE_TEXT,
                GlyphRenderMode::Colored => MODE_TEXT_COLOR,
            };

            self.draw_raster_image(None, glyph.bounding_box, glyph.texture_coords, mode);
        });
    }

    fn filter_new(&mut self, radius_x: u32, radius_y: u32) {
        let (new_target, needs_clear) = self.targets.get(radius_x, radius_y);

        if needs_clear {
            self.render_pass.push(RenderPass::Clear { target_index: new_target });
        }

        // `push_command` only extends the render pass sequence's *last* entry when it's already a
        // `RenderPass::Normal` -- with no check that it targets `new_target` specifically. A
        // brand new target index (no `Clear` pushed above) would otherwise leave the child's
        // first draw silently appended to whatever `Normal` pass was last active (a different,
        // wrong target), just like `mask_start` already guards against below.
        self.start_render_pass(new_target);

        self.filter_target_stack.push((new_target, self.current_target));

        self.current_target = new_target;

        // A reused target index's *actual* texture stays sized to the largest radius ever
        // requested for it (`TargetStates::get` only ever grows that record, never shrinks the
        // texture back down) -- so the transform must be built from that same stored maximum,
        // not this call's own `radius_x`/`radius_y`, or it would assume a canvas size smaller
        // than what's really allocated whenever a filter's radius (e.g. an animated shadow's
        // sigma/offset) drops below a previous frame's peak, visibly distorting the content.
        let (actual_radius_x, actual_radius_y) = self.targets.radius(new_target);
        let transform = self.padded_transform(actual_radius_x, actual_radius_y);
        self.transform(&transform);
    }

    fn filter_new_pop(&mut self, filter: &ImageFilter, color: Color, post_draw: bool) {
        let (target, old_target) = self.filter_target_stack.pop().unwrap();
        let (radius_x, radius_y) = self.targets.radius(target);

        // This filter samples the (possibly padded) `target` but writes into `old_target` --
        // the *parent* scope's target, which may be a different size (often window-sized). Pop
        // back to the parent's own transform first so this quad's geometry lands correctly in
        // `old_target`'s own space rather than `target`'s padded one.
        self.pop_transform();

        let create_vertex = |x, y, tx, ty| WgpuVertex {
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
        self.filter_source_target.insert(filter.id, target);

        let (l, r, b, t) = self.padded_bounding_box(radius_x, radius_y).l_r_b_t();

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
        let (radius_x, radius_y) = self.targets.radius(target);
        let (new_target, needs_clear) = self.targets.get(radius_x, radius_y);

        if needs_clear {
            self.render_pass.push(RenderPass::Clear { target_index: new_target });
        }

        self.filters.entry(filter.id).or_insert_with(|| filter.clone());
        self.current_frame_filters.insert(filter.id);
        self.filter_source_target.insert(filter.id, target);

        self.filters.entry(filter2.id).or_insert_with(|| filter2.clone());
        self.current_frame_filters.insert(filter2.id);
        self.filter_source_target.insert(filter2.id, new_target);

        let create_vertex = |x, y, tx, ty| WgpuVertex {
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

        let (l, r, b, t) = self.padded_bounding_box(radius_x, radius_y).l_r_b_t();

        // Pass 1 (`filter`, horizontal) samples the padded `target` and writes into the equally
        // padded `new_target` -- both share the same padded space, so this quad uses the padded
        // transform that's still active from `filter_new`.
        let vertices_start = self.vertices.len() as u32;

        // Bottom left triangle.
        self.vertices.push(create_vertex(l, t, 0.0, 1.0));
        self.vertices.push(create_vertex(r, b, 1.0, 0.0));
        self.vertices.push(create_vertex(l, b, 0.0, 0.0));

        // Top right triangle.
        self.vertices.push(create_vertex(l, t, 0.0, 1.0));
        self.vertices.push(create_vertex(r, t, 1.0, 1.0));
        self.vertices.push(create_vertex(r, b, 1.0, 0.0));

        let range1 = vertices_start..self.vertices.len() as u32;

        self.render_pass.push(RenderPass::Filter {
            vertex_range: range1,
            filter_id: filter.id,
            source_id: target,
            target_id: new_target,
            mask_id: None,
            initial_copy: false,
        });

        // Pass 2 (`filter2`, vertical) and the post-draw composite both write into `old_target`
        // -- the *parent* scope's target, a different (often window-sized) space -- so pop back
        // to the parent's own transform and push a second copy of the same quad under it, rather
        // than reusing pass 1's padded-space vertices.
        self.pop_transform();

        let vertices_start = self.vertices.len() as u32;

        // Bottom left triangle.
        self.vertices.push(create_vertex(l, t, 0.0, 1.0));
        self.vertices.push(create_vertex(r, b, 1.0, 0.0));
        self.vertices.push(create_vertex(l, b, 0.0, 0.0));

        // Top right triangle.
        self.vertices.push(create_vertex(l, t, 0.0, 1.0));
        self.vertices.push(create_vertex(r, t, 1.0, 1.0));
        self.vertices.push(create_vertex(r, b, 1.0, 0.0));

        let range2 = vertices_start..self.vertices.len() as u32;

        self.render_pass.push(RenderPass::Filter {
            vertex_range: range2.clone(),
            filter_id: filter2.id,
            source_id: new_target,
            target_id: old_target,
            mask_id: None,
            initial_copy: false,
        });

        if post_draw {
            self.render_pass.push(RenderPass::Normal { commands: vec![
                RenderPassCommand::SetBindGroup { bind_group: WGPUBindGroup::Target(target) },
                RenderPassCommand::Draw { vertex_range: range2 }
            ], target_index: old_target });
        }

        self.current_target = old_target;
        self.targets.free(new_target);
        self.targets.free(target);
    }

    fn mask_start(&mut self) {
        let (index, need_clear) = self.targets.get(0, 0);
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

    fn layer(&mut self, layer_id: LayerId, requested_size: Dimension, env: &mut Environment) -> Layer {
        let width = requested_size.width.floor().max(1.0) as u32;
        let height = requested_size.height.floor().max(1.0) as u32;

        if self.layers.contains_key(&layer_id) {
            let target = self.layers.get_mut(&layer_id).unwrap();

            if target.texture.width() != width || target.texture.height() != height {
                *target = WgpuRenderTarget::new(width, height, self.msaa, env);
            }

            Layer {
                inner: target,
                inner2: target,
            }
        } else {
            let new_layer = WgpuRenderTarget::new(width, height, self.msaa, env);

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

        self.draw_raster_image(Some(bind_group), bounding_box, Rect::from_corners(Position::new(0.0, 0.0), Position::new(1.0, 1.0)), MODE_IMAGE)
    }
}
