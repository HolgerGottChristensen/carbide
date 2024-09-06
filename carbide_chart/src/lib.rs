use carbide_core::draw::{Dimension, Position};
use carbide_core::widget::WidgetId;

mod scale;
mod element;
mod controller;
mod chart;

pub use chart::Chart;

extern crate carbide_core as carbide;