use crate::environment::{Environment, EnvironmentKeyable};
use crate::state::*;
use crate::environment::EnvironmentKey;


#[derive(Hash, Eq, PartialEq, Clone, Debug, Copy)]
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
}

impl EnvironmentKeyable for EnvironmentFontSize {
    type Output = u32;

    fn get(&self, stack: &Environment) -> Option<Self::Output> {
        match self {
            EnvironmentFontSize::LargeTitle => stack.get::<EnvironmentFontSizeLargeTitle>().cloned(),
            EnvironmentFontSize::Title => stack.get::<EnvironmentFontSizeTitle>().cloned(),
            EnvironmentFontSize::Title2 => stack.get::<EnvironmentFontSizeTitle2>().cloned(),
            EnvironmentFontSize::Title3 => stack.get::<EnvironmentFontSizeTitle3>().cloned(),
            EnvironmentFontSize::Headline => stack.get::<EnvironmentFontSizeHeadline>().cloned(),
            EnvironmentFontSize::Body => stack.get::<EnvironmentFontSizeBody>().cloned(),
            EnvironmentFontSize::Callout => stack.get::<EnvironmentFontSizeCallout>().cloned(),
            EnvironmentFontSize::Subhead => stack.get::<EnvironmentFontSizeSubhead>().cloned(),
            EnvironmentFontSize::Footnote => stack.get::<EnvironmentFontSizeFootnote>().cloned(),
            EnvironmentFontSize::Caption => stack.get::<EnvironmentFontSizeCaption>().cloned(),
            EnvironmentFontSize::Caption2 => stack.get::<EnvironmentFontSizeCaption2>().cloned(),
        }
    }

    fn with(&self, value: &Self::Output, stack: &mut Environment, f: impl FnOnce(&mut Environment)) {
        match self {
            EnvironmentFontSize::LargeTitle => stack.with::<EnvironmentFontSizeLargeTitle>(value, f),
            EnvironmentFontSize::Title => stack.with::<EnvironmentFontSizeTitle>(value, f),
            EnvironmentFontSize::Title2 => stack.with::<EnvironmentFontSizeTitle2>(value, f),
            EnvironmentFontSize::Title3 => stack.with::<EnvironmentFontSizeTitle3>(value, f),
            EnvironmentFontSize::Headline => stack.with::<EnvironmentFontSizeHeadline>(value, f),
            EnvironmentFontSize::Body => stack.with::<EnvironmentFontSizeBody>(value, f),
            EnvironmentFontSize::Callout => stack.with::<EnvironmentFontSizeCallout>(value, f),
            EnvironmentFontSize::Subhead => stack.with::<EnvironmentFontSizeSubhead>(value, f),
            EnvironmentFontSize::Footnote => stack.with::<EnvironmentFontSizeFootnote>(value, f),
            EnvironmentFontSize::Caption => stack.with::<EnvironmentFontSizeCaption>(value, f),
            EnvironmentFontSize::Caption2 => stack.with::<EnvironmentFontSizeCaption2>(value, f),
        }
    }
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

impl ConvertIntoRead<u32> for EnvironmentFontSize {
    type Output<G: AnyReadState<T=Self> + Clone> = EnvMap1<fn(&mut Environment, &EnvironmentFontSize)->u32, EnvironmentFontSize, u32, G>;

    fn convert<F: AnyReadState<T=EnvironmentFontSize> + Clone>(f: F) -> Self::Output<F> {
        Map1::read_map_env(f, |env, value| {
            env.value(value).unwrap()
        })
    }
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentFontSizeLargeTitle;
impl EnvironmentKey for EnvironmentFontSizeLargeTitle {
    type Value = u32;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentFontSizeTitle;
impl EnvironmentKey for EnvironmentFontSizeTitle {
    type Value = u32;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentFontSizeTitle2;
impl EnvironmentKey for EnvironmentFontSizeTitle2 {
    type Value = u32;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentFontSizeTitle3;
impl EnvironmentKey for EnvironmentFontSizeTitle3 {
    type Value = u32;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentFontSizeHeadline;
impl EnvironmentKey for EnvironmentFontSizeHeadline {
    type Value = u32;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentFontSizeBody;
impl EnvironmentKey for EnvironmentFontSizeBody {
    type Value = u32;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentFontSizeCallout;
impl EnvironmentKey for EnvironmentFontSizeCallout {
    type Value = u32;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentFontSizeSubhead;
impl EnvironmentKey for EnvironmentFontSizeSubhead {
    type Value = u32;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentFontSizeFootnote;
impl EnvironmentKey for EnvironmentFontSizeFootnote {
    type Value = u32;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentFontSizeCaption;
impl EnvironmentKey for EnvironmentFontSizeCaption {
    type Value = u32;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentFontSizeCaption2;
impl EnvironmentKey for EnvironmentFontSizeCaption2 {
    type Value = u32;
}