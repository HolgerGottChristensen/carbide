use std::fmt::Debug;

pub trait SliderValue: Debug + Clone + PartialEq + PartialOrd + 'static {
    fn interpolate(&self, other: &Self, percentage: f64) -> Self;
    fn stepped_interpolate(&self, other: &Self, step_size: &Self, percentage: f64) -> Self {
        let stepped = Self::stepped_percent(self, other, step_size, percentage);
        Self::interpolate(self, other, stepped)
    }
    fn percent(&self, start: &Self, end: &Self) -> f64;
    fn stepped_percent(&self, other: &Self, step_size: &Self, percentage: f64) -> f64;
}

impl SliderValue for f64 {
    fn interpolate(&self, other: &Self, percentage: f64) -> Self {
        percentage * (*other - *self) + *self
    }

    fn percent(&self, start: &Self, end: &Self) -> f64 {
        (*self - *start) / (*end - *start)
    }

    fn stepped_percent(&self, other: &Self, step_size: &Self, percentage: f64) -> f64 {
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
    fn interpolate(&self, other: &Self, percentage: f64) -> Self {
        (percentage * ((*other - *self) as f64)) as f32 + *self
    }

    fn percent(&self, start: &Self, end: &Self) -> f64 {
        (*self - *start) as f64 / (*end - *start) as f64
    }

    fn stepped_percent(&self, other: &Self, step_size: &Self, percentage: f64) -> f64 {
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
            fn interpolate(&self, other: &Self, percentage: f64) -> Self {
                (percentage * (*other - *self) as f64).round() as $typ + *self
            }

            fn percent(&self, start: &Self, end: &Self) -> f64 {
                self.saturating_sub(*start) as f64 / (*end - *start) as f64
            }

            fn stepped_percent(&self, other: &Self, step_size: &Self, percentage: f64) -> f64 {
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
        )*
    };
}

impl_slider_value!(u8, u16, u32, u64, u128, usize);
impl_slider_value!(i8, i16, i32, i64, i128, isize);