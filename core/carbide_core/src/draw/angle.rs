use std::f64::consts::PI;
use crate::state::AnyReadState;
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
            Angle::Radians(r) => *r / (2.0 * PI),
            Angle::Gradians(g) => *g / 400.0,
        }
    }

    pub fn degrees(&self) -> f64 {
        match self {
            Angle::Degrees(d) => *d,
            Angle::Turns(t) => *t * 360.0,
            Angle::Radians(r) => *r * 180.0 / PI,
            Angle::Gradians(g) => (*g / 400.0) * 360.0,
        }
    }

    pub fn radians(&self) -> f64 {
        match self {
            Angle::Degrees(d) => *d * PI / 180.0,
            Angle::Turns(t) => *t * 2.0 * PI,
            Angle::Radians(r) => *r,
            Angle::Gradians(g) => *g * PI / 200.0,
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

#[cfg(test)]
mod tests {
    use std::f64::consts::PI;
    use super::Angle;

    /// A quarter turn is the same angle in every unit, read in every unit.
    #[test]
    fn every_unit_converts_to_every_other() {
        let quarter = [Angle::Degrees(90.0), Angle::Turns(0.25), Angle::Radians(PI / 2.0), Angle::Gradians(100.0)];

        for angle in quarter {
            assert!((angle.degrees() - 90.0).abs() < 1e-9, "{:?} in degrees", angle);
            assert!((angle.turns() - 0.25).abs() < 1e-9, "{:?} in turns", angle);
            assert!((angle.radians() - PI / 2.0).abs() < 1e-9, "{:?} in radians", angle);
        }
    }
}
