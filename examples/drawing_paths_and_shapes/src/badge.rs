use carbide::draw::{Dimension, Rect};
use carbide::layout::BasicLayouter;
use carbide::state::{LocalState, Map1};
use carbide::widget::{ForEach, GeometryReader, Widget, WidgetExt, ZStack};
use crate::badge_background::BadgeBackground;
use crate::badge_symbol::BadgeSymbol;

pub struct Badge;

impl Badge {
    pub fn new() -> impl Widget {
        let geometry = LocalState::new(Rect::default());
        GeometryReader::new(
            geometry.clone(),
            ZStack::new((
                BadgeBackground::new(),
                ForEach::new(0..8, move |_, index| {
                    BadgeSymbol::new(Map1::read_map(index, |index| {
                        *index as f64 / 8.0 * 360.0
                    })).aspect_ratio(Dimension::new(1.0, 1.0)).scale_to_fit().scale_effect(0.25)
                        .with_anchor(BasicLayouter::Top)
                        .offset(
                            0.0,
                            Map1::read_map(geometry.clone(), |geometry| (1.0 / 4.0) * geometry.height()),
                        )
                })
            ))
        )
    }
}