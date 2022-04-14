use carbide_core::widget::popup_menu::PopupMenu;
use crate::draw::{Dimension, Position};
use crate::prelude::*;
use crate::CommonWidgetImpl;

#[derive(Debug, Clone, Widget)]
pub struct MenuBar {
    id: Id,
    child: Box<dyn Widget>,
    menus: TState<Vec<Menu>>,
    position: Position,
    dimension: Dimension,
}

impl MenuBar {
    pub fn new(menus: Vec<Menu>, child: Box<dyn Widget>) -> Box<Self> {
        let menus = ValueState::new(menus);
        let child = VStack::new(vec![
            HStack::new(vec![
                ForEach::new(menus.clone(), Self::menu_delegate),
                Spacer::new()
            ]).spacing(1.0)
                .background(Rectangle::new().fill(EnvironmentColor::Green)),
            Spacer::new(),
            child,
            Spacer::new()
        ]);

        Box::new(MenuBar {
            id: Id::new_v4(),
            child,
            menus,
            position: Default::default(),
            dimension: Default::default(),
        })
    }

    fn menu_delegate(item: TState<Menu>, index: UsizeState) -> Box<dyn Widget> {
        PopupMenu::new(item, true)
    }
}

CommonWidgetImpl!(MenuBar, self, id: self.id, child: self.child, position: self.position, dimension: self.dimension);

impl WidgetExt for MenuBar {}
