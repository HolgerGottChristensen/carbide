use carbide_core::focus::Focus;
use carbide_core::state::State;
use carbide_core::widget::{Action, MouseArea, Widget};

pub type PlainButton<I: Action + Clone + 'static, O: Action + Clone + 'static, F: State<T=Focus> + Clone, C: Widget + Clone, H: State<T=bool> + Clone, P: State<T=bool> + Clone> = MouseArea<I, O, F, C, H, P>;
