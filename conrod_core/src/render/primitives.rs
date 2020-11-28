use ::{widget, Rect};
use graph::{Graph, UniqueWidgetState};
use ::{Theme, text};
use widget::triangles::Triangle;
use ::{Point};
use render::primitive_walker::PrimitiveWalker;
use render::primitive::Primitive;
use ::{color, OldWidget};
use render::primitive_kind::PrimitiveKind;
use render::util::{new_primitive, next_widget};
use widget::render::Render;
use widget::{Oval, Rectangle};
use widget::envelope_editor::EnvelopePoint;
use position::{Align, Dimensions};
use render::text::Text;
use render::owned_primitives::OwnedPrimitives;
use render::owned_primitive_kind::OwnedPrimitiveKind;
use render::owned_primitive::OwnedPrimitive;
use render::owned_text::OwnedText;
use Color;

/// An iterator-like type that yields a reference to each primitive in order of depth for
/// rendering.
///
/// This type is produced by the `Ui::draw` and `Ui::draw_if_changed` methods.
///
/// This type borrows data from the `Ui` in order to lazily produce each `Primitive`. If you
/// require ownership over the sequence of primitives, consider using the `OwnedPrimitives` type.
/// The `OwnedPrimitives` type can be produced by calling the `Primitives::owned` method.
pub struct Primitives<'a> {
    crop_stack: Vec<(widget::Id, Rect)>,
    depth_order: std::slice::Iter<'a, widget::Id>,
    graph: &'a Graph,
    theme: &'a Theme,
    fonts: &'a text::font::Map,
    window_rect: Rect,
    /// A buffer to use for triangulating polygons and lines for the `Triangles`.
    triangles: Vec<Triangle<Point>>,
    /// The slice of rusttype `PositionedGlyph`s to re-use for the `Text` primitive.
    positioned_glyphs: Vec<text::PositionedGlyph>,
}

impl<'a> PrimitiveWalker for Primitives<'a> {
    fn next_primitive(&mut self) -> Option<Primitive> {
        self.next()
    }
}

impl<'a> Primitives<'a> {

    /// Constructor for the `Primitives` iterator.
    pub fn new(graph: &'a Graph,
               depth_order: &'a [widget::Id],
               theme: &'a Theme,
               fonts: &'a text::font::Map,
               window_dim: Dimensions) -> Self
    {
        Primitives {
            crop_stack: Vec::new(),
            depth_order: depth_order.iter(),
            graph: graph,
            theme: theme,
            fonts: fonts,
            window_rect: Rect::from_xy_dim([0.0, 0.0], window_dim),
            triangles: Vec::new(),
            positioned_glyphs: Vec::new(),
        }
    }

    /// Yield the next `Primitive` for rendering.
    pub fn next(&mut self) -> Option<Primitive> {
        let Primitives {
            ref mut crop_stack,
            ref mut depth_order,
            ref mut triangles,
            ref mut positioned_glyphs,
            graph,
            theme,
            fonts,
            window_rect,
        } = *self;

        while let Some(widget) = next_widget(depth_order, graph, crop_stack, window_rect) {
            use widget::primitive::point_path::{State as PointPathState, Style as PointPathStyle};
            use widget::primitive::shape::polygon::{State as PolygonState};
            use widget::primitive::shape::Style as ShapeStyle;

            type TrianglesSingleColorState =
                widget::triangles::State<Vec<widget::triangles::Triangle<Point>>>;
            type TrianglesMultiColorState =
                widget::triangles::State<Vec<widget::triangles::Triangle<(Point, color::Rgba)>>>;

            let (id, clip, container) = widget;
            let rect = container.rect;

            fn state_type_id<W>() -> std::any::TypeId
                where W: OldWidget,
            {
                std::any::TypeId::of::<W::State>()
            }



            // Extract the unique state and style from the container.
            if container.type_id == state_type_id::<widget::Rectangle>() {
                if let Some(rectangle) = container.unique_widget_state::<widget::Rectangle>() {
                    let UniqueWidgetState { ref style, .. } = *rectangle;
                    let color = style.get_color(theme);
                    let r = Rectangle::initialize(vec![]);
                    return r.render(id, clip, container);
                    match *style {
                        ShapeStyle::Fill(_) => {
                            let kind = PrimitiveKind::Rectangle { color: color };
                            return Some(new_primitive(id, kind, clip, rect));
                        },
                        ShapeStyle::Outline(ref line_style) => {
                            let (l, r, b, t) = rect.l_r_b_t();
                            let array = [
                                [l, b],
                                [l, t],
                                [r, t],
                                [r, b],
                                [l, b],
                            ];
                            let cap = line_style.get_cap(theme);
                            let thickness = line_style.get_thickness(theme);
                            let points = array.iter().cloned();
                            let triangles = match widget::point_path::triangles(points, cap, thickness) {
                                None => &[],
                                Some(iter) => {
                                    triangles.extend(iter);
                                    &triangles[..]
                                },
                            };
                            let kind = PrimitiveKind::TrianglesSingleColor {
                                color: color.to_rgb(),
                                triangles: triangles.to_vec(),
                            };
                            return Some(new_primitive(id, kind, clip, rect));
                        },
                    }
                }

            } else if container.type_id == std::any::TypeId::of::<TrianglesSingleColorState>() {
                type Style = widget::triangles::SingleColor;
                if let Some(tris) = container.state_and_style::<TrianglesSingleColorState, Style>() {
                    let UniqueWidgetState { ref state, ref style } = *tris;
                    let widget::triangles::SingleColor(color) = *style;
                    let kind = PrimitiveKind::TrianglesSingleColor {
                        color: color,
                        triangles: state.triangles.to_vec(),
                    };
                    return Some(new_primitive(id, kind, clip, rect));
                }

            } else if container.type_id == std::any::TypeId::of::<TrianglesMultiColorState>() {
                type Style = widget::triangles::MultiColor;
                if let Some(tris) = container.state_and_style::<TrianglesMultiColorState, Style>() {
                    let UniqueWidgetState { ref state, .. } = *tris;
                    let kind = PrimitiveKind::TrianglesMultiColor { triangles: state.triangles.clone() };
                    return Some(new_primitive(id, kind, clip, rect));
                }

            } else if container.type_id == state_type_id::<widget::Oval<widget::oval::Full>>() {
                if let Some(oval) = container.unique_widget_state::<widget::Oval<widget::oval::Full>>() {
                    let UniqueWidgetState { ref style, ref state } = *oval;
                    triangles.clear();
                    let points = widget::oval::circumference(rect, state.resolution);
                    let color = style.get_color(theme);
                    match *style {

                        ShapeStyle::Fill(_) => {
                            let r = Oval::fill_old(Dimensions::new(10.0, 10.0));
                            return r.render(id, clip, container);
                        },

                        ShapeStyle::Outline(ref line_style) => {
                            let cap = line_style.get_cap(theme);
                            let thickness = line_style.get_thickness(theme);
                            let triangles = match widget::point_path::triangles(points, cap, thickness) {
                                None => &[],
                                Some(iter) => {
                                    triangles.extend(iter);
                                    &triangles[..]
                                },
                            };
                            let kind = PrimitiveKind::TrianglesSingleColor {
                                color: color.to_rgb(),
                                triangles: triangles.to_vec(),
                            };
                            return Some(new_primitive(id, kind, clip, rect));
                        },
                    }
                }

            // Oval subsection.
            } else if container.type_id == state_type_id::<widget::Oval<widget::oval::Section>>() {
                if let Some(oval) = container.unique_widget_state::<widget::Oval<widget::oval::Section>>() {
                    let UniqueWidgetState { ref style, ref state } = *oval;
                    triangles.clear();
                    let points = widget::oval::circumference(rect, state.resolution)
                        .section(state.section.radians)
                        .offset_radians(state.section.offset_radians);
                    let color = style.get_color(theme);
                    match *style {

                        ShapeStyle::Fill(_) => {
                            let triangles = {
                                triangles.extend(points.triangles());
                                &triangles[..]
                            };
                            let kind = PrimitiveKind::TrianglesSingleColor {
                                color: color.to_rgb(),
                                triangles: triangles.to_vec(),
                            };
                            return Some(new_primitive(id, kind, clip, rect));
                        },

                        ShapeStyle::Outline(ref line_style) => {
                            use std::iter::once;
                            let cap = line_style.get_cap(theme);
                            let thickness = line_style.get_thickness(theme);
                            let middle = rect.xy();
                            let points = once(middle).chain(points).chain(once(middle));
                            let triangles = match widget::point_path::triangles(points, cap, thickness) {
                                None => &[],
                                Some(iter) => {
                                    triangles.extend(iter);
                                    &triangles[..]
                                },
                            };
                            let kind = PrimitiveKind::TrianglesSingleColor {
                                color: color.to_rgb(),
                                triangles: triangles.to_vec(),
                            };
                            return Some(new_primitive(id, kind, clip, rect));
                        },
                    }
                }

            } else if container.type_id == std::any::TypeId::of::<PolygonState>() {
                use widget::primitive::shape::Style;
                if let Some(polygon) = container.state_and_style::<PolygonState, Style>() {
                    let UniqueWidgetState { ref state, ref style } = *polygon;
                    triangles.clear();

                    let color = style.get_color(theme);
                    let points = state.points.iter().cloned();
                    match *style {

                        ShapeStyle::Fill(_) => {
                            let triangles = match widget::polygon::triangles(points) {
                                None => &[],
                                Some(iter) => {
                                    triangles.extend(iter);
                                    &triangles[..]
                                },
                            };
                            let kind = PrimitiveKind::TrianglesSingleColor {
                                color: color.to_rgb(),
                                triangles: triangles.to_vec(),
                            };
                            return Some(new_primitive(id, kind, clip, rect));
                        },

                        ShapeStyle::Outline(ref line_style) => {
                            let cap = line_style.get_cap(theme);
                            let thickness = line_style.get_thickness(theme);
                            let triangles = match widget::point_path::triangles(points, cap, thickness) {
                                None => &[],
                                Some(iter) => {
                                    triangles.extend(iter);
                                    &triangles[..]
                                },
                            };
                            let kind = PrimitiveKind::TrianglesSingleColor {
                                color: color.to_rgb(),
                                triangles: triangles.to_vec(),
                            };
                            return Some(new_primitive(id, kind, clip, rect));
                        },
                    }
                }

            } else if container.type_id == state_type_id::<widget::Line>() {
                if let Some(line) = container.unique_widget_state::<widget::Line>() {
                    let UniqueWidgetState { ref state, ref style } = *line;
                    triangles.clear();
                    let color = style.get_color(theme);
                    let cap = style.get_cap(theme);
                    let thickness = style.get_thickness(theme);
                    let points = std::iter::once(state.start).chain(std::iter::once(state.end));
                    let triangles = match widget::point_path::triangles(points, cap, thickness) {
                        None => &[],
                        Some(iter) => {
                            triangles.extend(iter);
                            &triangles[..]
                        },
                    };
                    let kind = PrimitiveKind::TrianglesSingleColor {
                        color: color.to_rgb(),
                        triangles: triangles.to_vec(),
                    };
                    return Some(new_primitive(id, kind, clip, rect));
                }

            } else if container.type_id == std::any::TypeId::of::<PointPathState>() {
                if let Some(point_path) = container.state_and_style::<PointPathState, PointPathStyle>() {
                    let UniqueWidgetState { ref state, ref style } = *point_path;
                    triangles.clear();
                    let color = style.get_color(theme);
                    let cap = style.get_cap(theme);
                    let thickness = style.get_thickness(theme);
                    let points = state.points.iter().map(|&t| t);
                    let triangles = match widget::point_path::triangles(points, cap, thickness) {
                        None => &[],
                        Some(iter) => {
                            triangles.extend(iter);
                            &triangles[..]
                        },
                    };
                    let kind = PrimitiveKind::TrianglesSingleColor {
                        color: color.to_rgb(),
                        triangles: triangles.to_vec(),
                    };
                    return Some(new_primitive(id, kind, clip, rect));
                }

            } else if container.type_id == state_type_id::<widget::Text>() {
                if let Some(text) = container.unique_widget_state::<widget::Text>() {
                    let UniqueWidgetState { ref state, ref style } = *text;
                    let font_id = match style.font_id(theme).or_else(|| fonts.ids().next()) {
                        Some(id) => id,
                        None => continue,
                    };
                    let font = match fonts.get(font_id) {
                        Some(font) => font,
                        None => continue,
                    };

                    // Retrieve styling.
                    let color = style.color(theme);
                    let font_size = style.font_size(theme);
                    let line_spacing = style.line_spacing(theme);
                    let justify = style.justify(theme);
                    let y_align = Align::End;

                    let text = Text {
                        positioned_glyphs: (*positioned_glyphs).clone(),
                        window_dim: window_rect.dim(),
                        text: state.string.clone(),
                        line_infos: state.line_infos.to_vec(),
                        font: font.clone(),
                        font_size: font_size,
                        rect: rect,
                        justify: justify,
                        y_align: y_align,
                        line_spacing: line_spacing,
                    };

                    let kind = PrimitiveKind::Text {
                        color: color,
                        text: text,
                        font_id: font_id,
                    };
                    return Some(new_primitive(id, kind, clip, rect));
                }

            } else if container.type_id == state_type_id::<widget::Image>() {
                use widget::primitive::image::{State, Style};
                if let Some(image) = container.state_and_style::<State, Style>() {
                    let UniqueWidgetState { ref state, ref style } = *image;
                    let color = style.maybe_color(theme);
                    let kind = PrimitiveKind::Image {
                        color: color,
                        image_id: state.image_id,
                        source_rect: state.src_rect,
                    };
                    return Some(new_primitive(id, kind, clip, rect));
                }

            // Return an `Other` variant for all non-primitive widgets.
            } else {
                let kind = PrimitiveKind::Rectangle {color: Color::random()};
                return Some(new_primitive(id, kind, clip, rect));
            }
        }

        None
    }

    /// Collect the `Primitives` list into an owned collection.
    ///
    /// This is useful for sending `Ui` rendering data across threads in an efficient manner.
    pub fn owned(mut self) -> OwnedPrimitives {
        let mut primitives = Vec::with_capacity(self.depth_order.len());
        let mut primitive_triangles_multi_color = Vec::new();
        let mut primitive_triangles_single_color = Vec::new();
        let mut primitive_line_infos = Vec::new();
        let mut texts_string = String::new();
        let mut max_glyphs = 0;

        while let Some(Primitive { id, rect, scizzor, kind }) = self.next() {
            let new = |kind| OwnedPrimitive {
                id: id,
                rect: rect,
                scizzor: scizzor,
                kind: kind,
            };

            match kind {

                PrimitiveKind::Rectangle { color } => {
                    let kind = OwnedPrimitiveKind::Rectangle { color: color };
                    primitives.push(new(kind));
                },

                PrimitiveKind::TrianglesSingleColor { color, triangles } => {
                    let start = primitive_triangles_single_color.len();
                    primitive_triangles_single_color.extend(triangles.iter().cloned());
                    let end = primitive_triangles_single_color.len();
                    let kind = OwnedPrimitiveKind::TrianglesSingleColor {
                        color: color,
                        triangle_range: start..end,
                    };
                    primitives.push(new(kind));
                },

                PrimitiveKind::TrianglesMultiColor { triangles } => {
                    let start = primitive_triangles_multi_color.len();
                    primitive_triangles_multi_color.extend(triangles.iter().cloned());
                    let end = primitive_triangles_multi_color.len();
                    let kind = OwnedPrimitiveKind::TrianglesMultiColor {
                        triangle_range: start..end,
                    };
                    primitives.push(new(kind));
                },

                PrimitiveKind::Image { image_id, color, source_rect } => {
                    let kind = OwnedPrimitiveKind::Image {
                        image_id: image_id,
                        color: color,
                        source_rect: source_rect,
                    };
                    primitives.push(new(kind));
                },

                PrimitiveKind::Text { color, font_id, text } => {
                    let Text {
                        window_dim,
                        text,
                        line_infos,
                        font,
                        font_size,
                        rect,
                        justify,
                        y_align,
                        line_spacing,
                        ..
                    } = text;

                    // Keep a rough estimate of the maximum number of glyphs so that we know what
                    // capacity we should allocate the `PositionedGlyph` buffer with.
                    max_glyphs = std::cmp::max(max_glyphs, text.len());

                    // Pack the `texts_string`.
                    let start_str_byte = texts_string.len();
                    texts_string.push_str(&text);
                    let end_str_byte = texts_string.len();

                    // Pack the `line_infos`.
                    let start_line_info_idx = primitive_line_infos.len();
                    primitive_line_infos.extend(line_infos.iter().cloned());
                    let end_line_info_idx = primitive_line_infos.len();

                    let owned_text = OwnedText {
                        str_byte_range: start_str_byte..end_str_byte,
                        line_infos_range: start_line_info_idx..end_line_info_idx,
                        window_dim: window_dim,
                        font: font.clone(),
                        font_size: font_size,
                        rect: rect,
                        justify: justify,
                        y_align: y_align,
                        line_spacing: line_spacing,
                    };

                    let kind = OwnedPrimitiveKind::Text {
                        color: color,
                        font_id: font_id,
                        text: owned_text,
                    };
                    primitives.push(new(kind));
                },

                // TODO: Not sure how we should handle this yet.
                PrimitiveKind::Other(_) => (),

            }
        }

        OwnedPrimitives {
            primitives: primitives,
            triangles_single_color: primitive_triangles_single_color,
            triangles_multi_color: primitive_triangles_multi_color,
            max_glyphs: max_glyphs,
            line_infos: primitive_line_infos,
            texts_string: texts_string,
        }
    }

}


