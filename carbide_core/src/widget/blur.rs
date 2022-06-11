use crate::draw::{Dimension, Position, Rect};
use crate::prelude::*;
use crate::render::PrimitiveKind;
use crate::utils::gaussian;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render)]
pub struct Blur {
    id: Uuid,
    position: Position,
    dimension: Dimension,
    blur_type: BlurType,
    /// The first is the filter id and the second is the filters radius
    filter_horizontal_has_been_inserted: Option<(u32, u32)>,
    /// The first is the filter id and the second is the filters radius
    filter_vertical_has_been_inserted: Option<(u32, u32)>,
}

impl Blur {
    pub fn gaussian(sigma: f32) -> Box<Self> {
        Box::new(Blur {
            id: Uuid::new_v4(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            blur_type: BlurType::Gaussian(sigma),
            filter_horizontal_has_been_inserted: None,
            filter_vertical_has_been_inserted: None,
        })
    }

    pub fn mean(radius: u32) -> Box<Self> {
        Box::new(Blur {
            id: Uuid::new_v4(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            blur_type: BlurType::Mean(radius),
            filter_horizontal_has_been_inserted: None,
            filter_vertical_has_been_inserted: None,
        })
    }

    fn gaussian_blur(sigma: f32) -> ImageFilter {
        assert!(sigma > 0.0);
        let mut entries = vec![];
        let radius = (3.0 * sigma).round() as i32;

        for x in -radius..=radius {
            entries.push(ImageFilterValue::new(x, 0, gaussian(sigma as f64, x as f64) as f32))
        }

        let mut filter = ImageFilter {
            filter: entries
        };

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
    fn id(&self) -> Id {
        self.id
    }

    fn set_id(&mut self, id: Id) {
        self.id = id;
    }

    fn children(&self) -> WidgetIter {
        WidgetIter::Empty
    }

    fn children_mut(&mut self) -> WidgetIterMut {
        WidgetIterMut::Empty
    }

    fn children_direct(&mut self) -> WidgetIterMut {
        WidgetIterMut::Empty
    }

    fn children_direct_rev(&mut self) -> WidgetIterMut {
        WidgetIterMut::Empty
    }

    fn position(&self) -> Position {
        self.position
    }

    fn set_position(&mut self, position: Position) {
        self.position = position;
    }

    fn flexibility(&self) -> u32 {
        0
    }

    fn dimension(&self) -> Dimension {
        self.dimension
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        self.dimension = dimension
    }
}

impl Render for Blur {
    fn process_get_primitives(&mut self, primitives: &mut Vec<Primitive>, env: &mut Environment) {
        if self.filter_horizontal_has_been_inserted == None {
            let (filter_id, radius) = match self.blur_type {
                BlurType::Mean(radius) => {
                    (env.insert_filter(Blur::mean_blur(radius)), radius)
                }
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

        if let Some((filter_id, radius)) = self.filter_horizontal_has_been_inserted {
            let position = self.position - Position::new(0.0, radius as f64);
            let dimension = self.dimension + Dimension::new(0.0, radius as f64 * 2.0);
            primitives.push(Primitive {
                kind: PrimitiveKind::FilterSplitPt1(filter_id),
                bounding_box: Rect::new(position, dimension),
            });
        }

        if let Some((filter_id, _)) = self.filter_vertical_has_been_inserted {
            primitives.push(Primitive {
                kind: PrimitiveKind::FilterSplitPt2(filter_id),
                bounding_box: Rect::new(self.position, self.dimension),
            });
        }
    }
}

impl WidgetExt for Blur {}
