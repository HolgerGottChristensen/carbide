use std::f64::consts::PI;
use carbide::state::AnyReadState;
use crate::state::{ConvertIntoRead, Map1, RMap1};

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq)]
pub enum Angle {
    Degrees(f64),
    Turns(f64),
    Radians(f64),
    Gradians(f64),
}

impl Angle {
    pub fn turns(&self) -> f64 {
        match self {
            Angle::Degrees(d) => *d / 360.0,
            Angle::Turns(t) => *t,
            Angle::Radians(r) => *r / 2.0 * PI,
            Angle::Gradians(g) => *g / 400.0,
        }
    }

    pub fn degrees(&self) -> f64 {
        match self {
            Angle::Degrees(d) => *d,
            Angle::Turns(t) => *t * 360.0,
            Angle::Radians(r) => (*r / 2.0 * PI) * 360.0,
            Angle::Gradians(g) => (*g / 400.0) * 360.0,
        }
    }
}

impl ConvertIntoRead<Angle> for f64 {
    type Output<G: AnyReadState<T=Self> + Clone> = RMap1<fn(&f64)->Angle, f64, Angle, G>;

    fn convert<F: AnyReadState<T=Self> + Clone>(f: F) -> Self::Output<F> {
        Map1::read_map(f, |a| Angle::Degrees(*a))
    }
}

impl ConvertIntoRead<Angle> for f32 {
    type Output<G: AnyReadState<T=Self> + Clone> = RMap1<fn(&f32)->Angle, f32, Angle, G>;

    fn convert<F: AnyReadState<T=Self> + Clone>(f: F) -> Self::Output<F> {
        Map1::read_map(f, |a| Angle::Degrees(*a as f64))
    }
}