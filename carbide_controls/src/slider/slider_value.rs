use std::fmt::Debug;

/// A slider value is a value that can be represented and edited by a slider
/// The most common values are f32 and f64, but it is also implemented for all
/// other numeric types.
pub trait SliderValue: Debug + Clone + PartialEq + PartialOrd + 'static {

    /// Convert the value into a percent between the start and end
    /// 0.0 should indicate the value is equal to the start value,
    /// and 1.0 should indicate the value is equal to the end value.
    /// It is allowed to return less than 0.0 or more than 1.0
    /// but those can not be selected using the slider.
    fn value_to_percent(&self, start: &Self, end: &Self) -> f64;

    /// Convert a percent to the slider value.
    /// 0.0 represents the start value of the slider
    /// 1.0 represents the end value of the slider,
    /// Values outside the range 0.0 and 1.0 are possible
    /// and need to be handled
    /// self should be considered the minimum value,
    /// and other should be considered the maximum value.
    fn percent_to_value(&self, other: &Self, percentage: f64) -> Self;

    /// Convert the value rounded to the nearest step into a percent
    /// between a start and end.
    /// 0.0 should indicate self is equal to the start
    /// 1.0 should indicate self is equal to the end
    /// The range from start to end might not be cleanly divisible
    /// by the step value. In these cases, the last step will be
    /// smaller than the step value.
    fn value_to_percent_stepped(&self, start: &Self, end: &Self, step: &Self) -> f64;



    /// Convert a percent to the slider value by the step
    /// The default implementation converts the percentage into
    fn percent_to_stepped_value(&self, other: &Self, percentage: f64, step: &Self) -> Self {
        let stepped = Self::percent_to_stepped_percent(self, other, step, percentage);
        Self::percent_to_value(self, other, stepped)
    }

    fn percent_to_stepped_percent(&self, other: &Self, step_size: &Self, percentage: f64) -> f64;

}

impl SliderValue for f64 {
    fn value_to_percent(&self, start: &Self, end: &Self) -> f64 {
        (*self - *start) / (*end - *start)
    }

    fn value_to_percent_stepped(&self, start: &Self, end: &Self, step: &Self) -> f64 {
        todo!()
    }

    fn percent_to_value(&self, other: &Self, percentage: f64) -> Self {
        percentage * (*other - *self) + *self
    }

    fn percent_to_stepped_percent(&self, other: &Self, step_size: &Self, percentage: f64) -> f64 {
        let range = *other - *self;
        let range_mod = range % step_size;
        let percent_lost = range_mod / range;
        let number_of_steps = range / step_size;
        let percent_per_step = (number_of_steps * percentage).round() / number_of_steps;

        if percentage > 1.0 - percent_lost / 2.0 {
            1.0
        } else {
            percent_per_step
        }
    }
}

impl SliderValue for f32 {
    fn value_to_percent(&self, start: &Self, end: &Self) -> f64 {
        (*self - *start) as f64 / (*end - *start) as f64
    }

    fn value_to_percent_stepped(&self, start: &Self, end: &Self, step: &Self) -> f64 {
        todo!()
    }

    fn percent_to_value(&self, other: &Self, percentage: f64) -> Self {
        (percentage * ((*other - *self) as f64)) as f32 + *self
    }

    fn percent_to_stepped_percent(&self, other: &Self, step_size: &Self, percentage: f64) -> f64 {
        let range = *other - *self;
        let range_mod = (range % step_size) as f64;
        let percent_lost = range_mod / range as f64;
        let number_of_steps = range as f64 / *step_size as f64;
        let percent_per_step = (number_of_steps * percentage).round() / number_of_steps;

        if percentage > 1.0 - percent_lost / 2.0 {
            1.0
        } else {
            percent_per_step
        }
    }
}

macro_rules! impl_slider_value {
    ($($typ: ty),*) => {
        $(
        impl SliderValue for $typ {
            fn percent_to_value(&self, other: &Self, percentage: f64) -> Self {
                (percentage * (*other - *self) as f64).round() as $typ + *self
            }

            fn value_to_percent(&self, start: &Self, end: &Self) -> f64 {
                self.saturating_sub(*start) as f64 / (*end - *start) as f64
            }

            fn percent_to_stepped_percent(&self, other: &Self, step_size: &Self, percentage: f64) -> f64 {
                let range = *other - *self;
                let range_mod = (range % step_size) as f64;
                let percent_lost = range_mod / range as f64;
                let number_of_steps = range as f64 / *step_size as f64;
                let percent_per_step = (number_of_steps * percentage).round() / number_of_steps;

                if percentage > 1.0 - percent_lost / 2.0 {
                    1.0
                } else {
                    percent_per_step
                }
            }

            fn value_to_percent_stepped(&self, start: &Self, end: &Self, step: &Self) -> f64 {
                todo!()
            }
        }

        impl carbide_core::state::ConvertIntoRead<$crate::slider::SliderStepping<$typ>> for $typ {
            type Output<G: carbide_core::state::AnyReadState<T=Self> + Clone> = carbide_core::state::RMap1<fn(&$typ)->$crate::slider::SliderStepping<$typ>, $typ, $crate::slider::SliderStepping<$typ>, G>;

            fn convert<F: carbide_core::state::AnyReadState<T=$typ> + Clone>(f: F) -> Self::Output<F> {
                carbide_core::state::Map1::read_map(f, |c| {
                    $crate::slider::SliderStepping::Stepped(c.clone())
                })
            }
        }
        )*
    };
}

impl_slider_value!(u8, u16, u32, u64, u128, usize);
impl_slider_value!(i8, i16, i32, i64, i128, isize);