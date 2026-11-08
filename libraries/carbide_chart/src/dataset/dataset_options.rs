use crate::DataColor;
use crate::element::Stepped;
use crate::scale::ScaleId;

#[derive(Debug)]
pub struct DataSetOptions {
    pub color: DataColor,
    pub stepped: Stepped,
    /// The horizontal scale the dataset is drawn on.
    pub x_scale: ScaleId,
    /// The vertical scale the dataset is drawn on.
    pub y_scale: ScaleId,
}
