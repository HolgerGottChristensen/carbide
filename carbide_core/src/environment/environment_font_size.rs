use crate::environment::Environment;
use crate::state::*;
use crate::state::WidgetState;

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum EnvironmentFontSize {
    LargeTitle,
    Title,
    Title2,
    Title3,
    Headline,
    Body,
    Callout,
    Subhead,
    Footnote,
    Caption,
    Caption2,
    Custom(String),
}

impl EnvironmentFontSize {
    pub fn u32(&self) -> impl ReadState<T=u32> {
        <EnvironmentFontSize as IntoReadState<u32>>::into_read_state(self.clone())
    }
}

impl Default for EnvironmentFontSize {
    fn default() -> Self {
        EnvironmentFontSize::Body
    }
}

// ---------------------------------------------------
//  Conversion implementations
// ---------------------------------------------------

impl<T> IntoReadStateHelper<T, EnvironmentFontSize, u32> for T where T: AnyReadState<T=EnvironmentFontSize> + Clone {
    type Output = EnvMap1<fn(&Environment, &EnvironmentFontSize)->u32, EnvironmentFontSize, u32, T>;

    fn into_read_state_helper(self) -> Self::Output {
        Map1::read_map_env(self, |env, value| {
            env.get_font_size(&EnvironmentStateKey::FontSize(value.clone())).unwrap()
        })
    }
}