use carbide_core::CommonWidgetImpl;
use carbide_core::render::RenderContext;
use carbide_macro::carbide_default_builder2;

use crate::draw::{Dimension, Position, Rect};
use crate::environment::Environment;
use crate::render::Render;
use crate::utils::gaussian;
use crate::widget::{BlurType, CommonWidget, FilterId, ImageFilter, ImageFilterValue, Widget, WidgetExt, WidgetId};

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render)]
pub struct Blur {
    id: WidgetId,
    position: Position,
    dimension: Dimension,
    blur_type: BlurType,
    /// The first is the filter id and the second is the filters radius
    filter_horizontal_has_been_inserted: Option<(FilterId, u32)>,
    /// The first is the filter id and the second is the filters radius
    filter_vertical_has_been_inserted: Option<(FilterId, u32)>,
}

impl Blur {

    #[carbide_default_builder2]
    pub fn gaussian(sigma: f32) -> Self {
        Blur {
            id: WidgetId::new(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            blur_type: BlurType::Gaussian(sigma),
            filter_horizontal_has_been_inserted: None,
            filter_vertical_has_been_inserted: None,
        }
    }

    pub fn mean(radius: u32) -> Self {
        Blur {
            id: WidgetId::new(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            blur_type: BlurType::Mean(radius),
            filter_horizontal_has_been_inserted: None,
            filter_vertical_has_been_inserted: None,
        }
    }

    fn gaussian_blur(sigma: f32) -> ImageFilter {
        assert!(sigma > 0.0);
        let mut entries = vec![];
        let radius = (3.0 * sigma).round() as i32;

        for x in -radius..=radius {
            entries.push(ImageFilterValue::new(
                x,
                0,
                gaussian(sigma as f64, x as f64) as f32,
            ))
        }

        let mut filter = ImageFilter { filter: entries };

        filter.normalize();
        filter
    }

    /// This is also known as box blur or linear blur.
    fn mean_blur(radius: u32) -> ImageFilter {
        let radius = radius as i32;
        let div = 2 * radius + 1;

        let mut entries = vec![];

        for radius_x in -radius..=radius {
            entries.push(ImageFilterValue::new(radius_x, 0, 1.0 / div as f32))
        }

        ImageFilter { filter: entries }
    }
}

impl CommonWidget for Blur {
    CommonWidgetImpl!(self, id: self.id, child: (), position: self.position, dimension: self.dimension, flexibility: 0);
}

impl Render for Blur {
    fn render(&mut self, context: &mut RenderContext, env: &mut Environment) {
        if self.filter_horizontal_has_been_inserted == None {
            let (filter_id, radius) = match self.blur_type {
                BlurType::Mean(radius) => (env.insert_filter(Blur::mean_blur(radius)), radius),
                BlurType::Gaussian(sigma) => {
                    let filter = Blur::gaussian_blur(sigma);
                    let radius = filter.radius_x();
                    (env.insert_filter(filter), radius)
                }
            };
            self.filter_horizontal_has_been_inserted = Some((filter_id, radius));
        }
        if self.filter_vertical_has_been_inserted == None {
            let (filter_id, radius) = match self.blur_type {
                BlurType::Mean(radius) => {
                    (env.insert_filter(Blur::mean_blur(radius).flipped()), radius)
                }
                BlurType::Gaussian(sigma) => {
                    let filter = Blur::gaussian_blur(sigma).flipped();
                    let radius = filter.radius_y();
                    (env.insert_filter(filter), radius)
                }
            };
            self.filter_vertical_has_been_inserted = Some((filter_id, radius));
        }

        if let Some((filter_id1, radius)) = self.filter_horizontal_has_been_inserted {
            let position = self.position - Position::new(0.0, radius as f64);
            let dimension = self.dimension + Dimension::new(0.0, radius as f64 * 2.0);

            if let Some((filter_id2, _)) = self.filter_vertical_has_been_inserted {
                context.filter2d(
                    filter_id1,
                    Rect::new(position, dimension),
                    filter_id2,
                    Rect::new(self.position, self.dimension), |_| {}
                );
            }
        }
    }
}

impl WidgetExt for Blur {}
