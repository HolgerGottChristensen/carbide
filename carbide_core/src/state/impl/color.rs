use crate::state::{Map1, Map2, RState, TState};
use crate::Color;

impl TState<Color> {
    /// Get the same color darkened. The color is darkened with percent.
    /// If you set the darkened color to some other color, the original color will
    /// be the new color lightened.
    pub fn darkened(&self, percent: impl Into<TState<f32>>) -> TState<Color> {
        Map2::map(
            self.clone(),
            percent.into(),
            |col: &Color, p: &f32| col.darkened(*p),
            move |new_color, old_color, old_percent| {
                (Some(new_color.lightened(*old_percent)), None)
            },
        )
    }

    /// Get the same color lightened. The color is lightened with percent.
    /// If you set the lightened color to some other color, the original color will
    /// be the new color darkened.
    pub fn lightened(&self, percent: impl Into<TState<f32>>) -> TState<Color> {
        Map2::map(
            self.clone(),
            percent.into(),
            |col: &Color, p: &f32| col.lightened(*p),
            move |new_color, old_color, old_percent| (Some(new_color.darkened(*old_percent)), None),
        )
    }

    /// Produce a complementary color. The two colors will accent each other. This is the same as
    /// rotating the hue by 180 degrees. If you set a value in the new state, the old will be the
    /// complementary of that.
    pub fn compliment(&self) -> TState<Color> {
        fn map(color: &Color) -> Color {
            color.complement()
        }
        fn replace(new: Color, _: &Color) -> Option<Color> {
            Some(new.complement())
        }
        Map1::map(self.clone(), map, replace)
    }

    /// Return either black or white, depending which contrasts the Color the most. This will be
    /// useful for determining a readable color for text on any given background Color.
    /// The returned state is read only, because it can not be determined what color the original
    /// state would be after setting the mapped color to either black or white.
    pub fn plain_contrast(&self) -> RState<Color> {
        Map1::read_map(self.clone(), |color: &Color| color.plain_contrast())
    }

    /// Return the color pre multiplied.
    pub fn pre_multiplied(&self) -> RState<Color> {
        Map1::read_map(self.clone(), |color: &Color| color.pre_multiply())
    }

    /// Return the inverted color in rgb space. When setting the color of the mapped state,
    /// the original state will be the new colors invert.
    pub fn inverted(&self) -> TState<Color> {
        Map1::map(
            self.clone(),
            |color: &Color| (*color).invert(),
            |new, original| Some(new.invert()),
        )
    }

    /// Get the luminance of the color. You will be able to change the value of the returned state
    /// and thereby change the luminance of the original color.
    pub fn luminance(&self) -> TState<f32> {
        Map1::map(
            self.clone(),
            |color: &Color| color.luminance(),
            |new, old_color| Some(old_color.with_luminance(new)),
        )
    }

    /// Get the opacity of the color. You will be able to change the value of the returned state
    /// and thereby change the opacity of the original color.
    pub fn opacity(&self) -> TState<f32> {
        Map1::map(
            self.clone(),
            |color: &Color| color.opacity(),
            |new, old_color| Some(old_color.with_opacity(new)),
        )
    }

    /// Get the red component of the color.
    /// You will be able to change the value of the returned state
    /// and thereby change the red component of the original color.
    pub fn red(&self) -> TState<f32> {
        Map1::map(
            self.clone(),
            |color: &Color| color.red(),
            |new, old_color| Some(old_color.with_red(new)),
        )
    }

    /// Get the green component of the color.
    /// You will be able to change the value of the returned state
    /// and thereby change the green component of the original color.
    pub fn green(&self) -> TState<f32> {
        Map1::map(
            self.clone(),
            |color: &Color| color.green(),
            |new, old_color| Some(old_color.with_green(new)),
        )
    }

    /// Get the blue component of the color.
    /// You will be able to change the value of the returned state
    /// and thereby change the blue component of the original color.
    pub fn blue(&self) -> TState<f32> {
        Map1::map(
            self.clone(),
            |color: &Color| color.blue(),
            |new, old_color| Some(old_color.with_blue(new)),
        )
    }

    /// Get the hue of the color.
    /// You will be able to change the value of the returned state
    /// and thereby change the hue of the original color.
    pub fn hue(&self) -> TState<f32> {
        Map1::map(
            self.clone(),
            |color: &Color| color.hue(),
            |new, old_color| Some(old_color.with_hue(new)),
        )
    }

    /// Get the saturation of the color.
    /// You will be able to change the value of the returned state
    /// and thereby change the saturation of the original color.
    pub fn saturation(&self) -> TState<f32> {
        Map1::map(
            self.clone(),
            |color: &Color| color.saturation(),
            |new, old_color| Some(old_color.with_saturation(new)),
        )
    }

    /// Get the lightness of the color.
    /// You will be able to change the value of the returned state
    /// and thereby change the lightness of the original color.
    pub fn lightness(&self) -> TState<f32> {
        Map1::map(
            self.clone(),
            |color: &Color| color.lightness(),
            |new, old_color| Some(old_color.with_lightness(new)),
        )
    }
}
