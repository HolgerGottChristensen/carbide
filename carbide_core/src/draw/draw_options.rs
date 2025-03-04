use carbide::draw::stroke::StrokeAlignment;
use crate::draw::fill::FillOptions;
use crate::draw::stroke::StrokeOptions;
use crate::widget::ShapeStyle;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DrawOptions {
    Fill(FillOptions),
    Stroke(StrokeOptions)
}

impl From<StrokeOptions> for DrawOptions {
    fn from(value: StrokeOptions) -> Self {
        DrawOptions::Stroke(value)
    }
}

impl From<FillOptions> for DrawOptions {
    fn from(value: FillOptions) -> Self {
        DrawOptions::Fill(value)
    }
}

impl From<ShapeStyle> for DrawOptions {
    fn from(value: ShapeStyle) -> Self {
        match value {
            ShapeStyle::Default |
            ShapeStyle::Fill |
            ShapeStyle::FillAndStroke { .. } => DrawOptions::Fill(FillOptions::default()),
            ShapeStyle::Stroke { line_width } => DrawOptions::Stroke(
                StrokeOptions::default()
                    .with_stroke_width(line_width)
                    .with_alignment(StrokeAlignment::Positive)
            )
        }
    }
}