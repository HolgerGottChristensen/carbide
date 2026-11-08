use crate::DataColor;
use crate::element::Stepped;

#[derive(Debug)]
pub struct DataSetOptions {
    pub color: DataColor,
    pub stepped: Stepped,
}