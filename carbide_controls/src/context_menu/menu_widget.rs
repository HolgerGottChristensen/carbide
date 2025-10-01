use carbide::focus::Focus;
use carbide::state::{ReadState, State};
use carbide::widget::{Action, AnyWidget, Widget};
use crate::button::Button;

pub trait AnyMenuWidget: AnyWidget {

}

impl<F: State<T=Focus>, A: Action + Clone + 'static, E: ReadState<T=bool>, H: State<T=bool>, P: State<T=bool>, L: Widget> AnyMenuWidget for Button<F, A, E, H, P, L> {

}