use std::ops::Range;

use cgmath::Matrix4;

use crate::draw::draw_gradient::DrawGradient;
use crate::draw::image::ImageId;
use crate::draw::Rect;
use crate::widget::FilterId;

pub type VertexRange = Range<usize>;

/// Draw commands are produced by mesh.rs and should be easily interpretable by
/// any backend implementations. Everything is specified in carbide points.
#[derive(Debug)]
pub enum DrawCommand {
    /// Geometry is the simplest draw command. It represents simple
    /// geometry such as rectangles, lines, strokes, text and more.
    /// Things like color, and such is defined on the vertices themselves.
    Geometry(VertexRange),

    /// Image consists of a vertex range and an image id. The corresponding
    /// image for the image id can be looked up in the environment image_map.
    Image(VertexRange, ImageId),

    /// Gradient consists of a vertex range and a draw gradient. The gradient
    /// stores things such as start and end position, gradient type and
    /// colors to be used. The vertex range relates to the vertexes that
    /// specifies the positions of the geometry to be rendered.
    Gradient(VertexRange, DrawGradient),

    /// The scissor rect specifies a region on the window that should be rendered within
    /// Anything outside the scissor rectangle should not be rendered.
    /// The rectangle is guarantied to be bounded within the window.
    Scissor(Rect),

    /// The stencil consists of a vertex range. This range specifies
    /// the shape of the stencil. The stencil should be seen as plain geometry
    /// that places a mask on everything drawn after it, until the same vertex
    /// range is provided in the DeStencil command.
    /// Stencils a stackable and if two stencils are provided after one another,
    /// the union of the two should be used as the resulting stencil.
    Stencil(VertexRange),

    /// The de-stencil consists of a vertex range specifying what should be removed from
    /// the stencil stack. This is the inverse of Stencil.
    DeStencil(VertexRange),

    /// Specifies a new matrix that should be used as the transform.
    /// Transforms are "pre stacked", meaning they will contain the full
    /// transform that should be used, and no calculations should be needed.
    /// Transform will also be used to set when something is "de-transformed"
    Transform(Matrix4<f32>),

    /// A filter represented by a filter id, and a vertex range specifying the geometry
    /// within the filter should be applied. Filters are things like gaussian blurs,
    /// and other convolution filters.
    Filter(VertexRange, FilterId),

    /// To have more efficient filters, we can apply them in two steps, first in one axis and
    /// then in the other. These are defined on FilterSplitPt1 and FilterSplitPt2
    FilterSplitPt1(VertexRange, FilterId),
    FilterSplitPt2(VertexRange, FilterId),
}