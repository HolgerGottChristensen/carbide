use crate::{DataColor, StackId};
use crate::element::Interpolation;
use crate::scale::ScaleId;

#[derive(Debug)]
pub struct DataSetOptions {
    pub color: DataColor,
    /// How the line between the points is drawn, by controllers that draw lines.
    pub interpolation: Interpolation,
    /// The horizontal scale the dataset is drawn on.
    pub x_scale: ScaleId,
    /// The vertical scale the dataset is drawn on.
    pub y_scale: ScaleId,
    /// The stack the dataset's bars are stacked in, by controllers that stack; `None` stands
    /// alone.
    pub stack: Option<StackId>,
}
