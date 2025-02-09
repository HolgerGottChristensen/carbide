use carbide::{impl_state_value, impl_state_value_generic};
use carbide::state::{AnyReadState, ConvertIntoRead, Map1, RMap1};
use crate::slider::slider_value::SliderValue;

/// Different modes slider stepping behavior
#[derive(Debug, Clone)]
pub enum SliderStepping<V> where V: SliderValue {
    /// No stepping. The slider selections will be smooth
    /// and the slider will show the exact value
    Smooth,
    /// Stepping. The slider selections will be determined
    /// by snapping to the closest step. The slider will
    /// show the step nearest to the current value
    Stepped(V),
    /// Stepping with smooth values. The selections will
    /// be determined by snapping to the closest step.
    /// The slider will show the exact value, so if
    /// the value of the slider change by outside means,
    /// the thumb might show inbetween two steps.
    SmoothStepped(V)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SliderSteppingType {
    Smooth,
    Stepped,
    SmoothStepped
}

impl_state_value!(SliderSteppingType);
impl_state_value_generic!(SliderStepping<T: SliderValue>);

impl ConvertIntoRead<SliderStepping<f32>> for f32 {
    type Output<G: AnyReadState<T=Self> + Clone> = RMap1<fn(&f32)->SliderStepping<f32>, f32, SliderStepping<f32>, G>;

    fn convert<F: AnyReadState<T=f32> + Clone>(f: F) -> Self::Output<F> {
        Map1::read_map(f, |c| {
            SliderStepping::Stepped(c.clone())
        })
    }
}

impl ConvertIntoRead<SliderStepping<f64>> for f64 {
    type Output<G: AnyReadState<T=Self> + Clone> = RMap1<fn(&f64)->SliderStepping<f64>, f64, SliderStepping<f64>, G>;

    fn convert<F: AnyReadState<T=f64> + Clone>(f: F) -> Self::Output<F> {
        Map1::read_map(f, |c| {
            SliderStepping::Stepped(c.clone())
        })
    }
}