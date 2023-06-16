
use carbide_core::widget::popup_menu::PopupMenu;

use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position};

use crate::state::{TState};
use crate::widget::{CommonWidget, Menu, Widget, WidgetExt, WidgetId};

#[derive(Debug, Clone, Widget)]
pub struct MenuBar {
    id: WidgetId,
    child: Box<dyn Widget>,
    menus: TState<Vec<Menu>>,
    position: Position,
    dimension: Dimension,
}

impl MenuBar {
    pub fn new(_menus: Vec<Menu>, _child: Box<dyn Widget>) -> Box<Self> {
        /*let menus = ValueState::new(menus);
        let child = VStack::new(vec![
            HStack::new(vec![
                ForEach::new(menus.clone(), Self::menu_delegate),
                Spacer::new(),
            ])
            .spacing(1.0)
            .background(*Rectangle::new().fill(EnvironmentColor::Green)),
            Spacer::new(),
            child,
            Spacer::new(),
        ]);

        Box::new(MenuBar {
            id: WidgetId::new(),
            child,
            menus,
            position: Default::default(),
            dimension: Default::default(),
        })*/
        todo!()
    }

    fn menu_delegate(item: TState<Menu>, _index: TState<usize>) -> Box<dyn Widget> {
        PopupMenu::new(item, true)
    }
}

impl CommonWidget for MenuBar {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension);
}

impl WidgetExt for MenuBar {}
