use crate::{DataColor, StackId};
use crate::element::{Fill, Interpolation, SpanGaps};
use crate::scale::ScaleId;

#[derive(Debug)]
pub struct DataSetOptions {
    pub color: DataColor,
    /// How the line between the points is drawn, by controllers that draw lines.
    pub interpolation: Interpolation,
    /// What the area under the line is filled to, by controllers that draw lines.
    pub fill: Fill,
    /// Whether the line breaks or joins across gaps, by controllers that draw lines.
    pub span_gaps: SpanGaps,
    /// The horizontal scale the dataset is drawn on.
    pub x_scale: ScaleId,
    /// The vertical scale the dataset is drawn on.
    pub y_scale: ScaleId,
    /// The stack the dataset is stacked in, by controllers that stack (bars and lines); `None`
    /// stands alone.
    pub stack: Option<StackId>,
    /// The dataset's name, shown in the legend; a dataset without one isn't in it.
    pub label: Option<String>,
}
