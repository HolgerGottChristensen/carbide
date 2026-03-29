
pub trait WidgetProperties {
    type Kind: WidgetKind;
}

pub trait WidgetKind: sealed::Sealed {
    fn kind() -> Kind;
}

#[derive(Copy, Clone, Debug)]
pub enum Kind {
    Ignore,
    Simple,
    Proxy,
    Dynamic
}

pub struct WidgetKindIgnore;
pub struct WidgetKindSimple;
pub struct WidgetKindProxy;
pub struct WidgetKindDynamic;

impl WidgetKind for WidgetKindIgnore {
    fn kind() -> Kind { Kind::Ignore }
}

impl WidgetKind for WidgetKindSimple {
    fn kind() -> Kind { Kind::Simple }
}

impl WidgetKind for WidgetKindProxy {
    fn kind() -> Kind { Kind::Proxy }
}

impl WidgetKind for WidgetKindDynamic {
    fn kind() -> Kind { Kind::Dynamic }
}

mod sealed {
    use crate::widget::common::widget_properties::*;

    pub trait Sealed {}

    impl Sealed for WidgetKindIgnore {}
    impl Sealed for WidgetKindSimple {}
    impl Sealed for WidgetKindProxy {}
    impl Sealed for WidgetKindDynamic {}
}