
pub trait WidgetProperties {
    type Kind: WidgetKind;
}

pub trait WidgetKind: sealed::Sealed + 'static {
    fn kind() -> Kind;
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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

/// The kind of a widget that is a single widget, not a proxy for others (a `ui!` `match` or `if`,
/// a `ForEach`, a `Group`). Modifiers that describe one widget, such as an accessibility label or
/// a picker tag, require it: on a proxy, they would have nothing single to describe.
///
/// An accessibility label on a single widget compiles:
/// ```
/// use carbide_core::accessibility::AccessibilityExt;
/// use carbide_core::widget::Rectangle;
///
/// let _ = Rectangle::new().accessibility_label("A square");
/// ```
/// On a proxy, such as an `if`, it doesn't:
/// ```compile_fail
/// use carbide_core::accessibility::AccessibilityExt;
/// use carbide_core::widget::{IfElse, Rectangle};
///
/// let _ = IfElse::new(true).when_true(Rectangle::new()).accessibility_label("A square");
/// ```
#[diagnostic::on_unimplemented(
    message = "this modifier describes a single widget, but the widget is a proxy for others or none (`{Self}`)",
    label = "a `ui!` match or if, a `ForEach` or a `Group`, whose children take its place in their parent",
    note = "wrap it in a widget with a single child, such as a `ZStack` or `.padding(0.0)`, and apply the modifier to that"
)]
pub trait SingleWidgetKind: WidgetKind {}

impl SingleWidgetKind for WidgetKindSimple {}

/// Warn, once per modifier and in debug builds only, that `modifier` was applied to a widget that
/// turns out to be a proxy at run time: a widget the type system can't see through, such as a
/// `Box<dyn AnyWidget>`. A proxy's children take its place in their parent, so such a modifier has
/// no effect. Called by the modifiers that require a [`SingleWidgetKind`].
pub fn warn_if_proxy(modifier: &'static str, child: &dyn crate::widget::AnyWidget) {
    #[cfg(debug_assertions)]
    {
        use std::collections::HashSet;
        use std::sync::{Mutex, OnceLock};

        if !child.is_proxy() {
            return;
        }

        static WARNED: OnceLock<Mutex<HashSet<&'static str>>> = OnceLock::new();
        if WARNED.get_or_init(Default::default).lock().unwrap().insert(modifier) {
            eprintln!(
                "carbide: `{modifier}` was applied to a proxy widget (a `ui!` match or if, a `ForEach` or a `Group`), \
                 where it has no effect. Wrap the proxy in a widget with a single child, such as a `ZStack`, and apply \
                 `{modifier}` to that."
            );
        }
    }
    #[cfg(not(debug_assertions))]
    let _ = (modifier, child);
}

mod sealed {
    use crate::widget::common::widget_properties::*;

    pub trait Sealed {}

    impl Sealed for WidgetKindIgnore {}
    impl Sealed for WidgetKindSimple {}
    impl Sealed for WidgetKindProxy {}
    impl Sealed for WidgetKindDynamic {}
}