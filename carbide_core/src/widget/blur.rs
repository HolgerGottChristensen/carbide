use carbide_core::CommonWidgetImpl;
use carbide_core::render::RenderContext;
use carbide_macro::carbide_default_builder2;

use crate::draw::{Dimension, Position, Rect};
use crate::render::Render;
use crate::widget::{BlurType, CommonWidget, FilterId, ImageFilter, ImageFilterValue, Widget, WidgetExt, WidgetId};

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render)]
pub struct Blur {
    #[id] id: WidgetId,
    position: Position,
    dimension: Dimension,

    blur_type: BlurType,

    horizontal: ImageFilter,
    vertical: ImageFilter,
}

impl Blur {

    #[carbide_default_builder2]
    pub fn gaussian(sigma: f32) -> Self {
        Blur {
            id: WidgetId::new(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            blur_type: BlurType::Gaussian(sigma),
            horizontal: ImageFilter::gaussian_blur_1d(sigma),
            vertical: ImageFilter::gaussian_blur_1d(sigma).flipped(),
        }
    }

    pub fn mean(radius: u32) -> Self {
        Blur {
            id: WidgetId::new(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            blur_type: BlurType::Mean(radius),
            horizontal: Blur::mean_blur(radius),
            vertical: Blur::mean_blur(radius).flipped(),
        }
    }

    /// This is also known as box blur or linear blur.
    pub fn mean_blur(radius: u32) -> ImageFilter {
        let radius = radius as i32;
        let div = 2 * radius + 1;

        let mut entries = vec![];

        for radius_x in -radius..=radius {
            entries.push(ImageFilterValue::new(radius_x, 0, 1.0 / div as f32))
        }

        ImageFilter {
            id: FilterId::new(),
            filter: entries
        }
    }
}

impl CommonWidget for Blur {
    CommonWidgetImpl!(self, child: (), position: self.position, dimension: self.dimension, flexibility: 0);
}

impl Render for Blur {
    fn render(&mut self, context: &mut RenderContext) {
        let radius = self.horizontal.radius_x();
        let position = self.position - Position::new(0.0, radius as f64);
        let dimension = self.dimension + Dimension::new(0.0, radius as f64 * 2.0);

        context.filter2d(
            &self.horizontal,
            Rect::new(position, dimension),
            &self.vertical,
            Rect::new(self.position, self.dimension), |_| {}
        );
    }
}