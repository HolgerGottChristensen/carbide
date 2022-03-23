use carbide_core::widget::menu::menu::Menu;
use crate::event::HotKey;
use crate::widget::Id;

#[derive(Debug, Clone)]
pub enum MenuItem {
    Item {
        id: Id,
        name: String,
        hotkey: Option<HotKey>,
        enabled: bool,
        selected: bool,
    },
    Separator,
    SubMenu {
        menu: Menu
    }
}

impl MenuItem {
    pub fn separator() -> MenuItem {
        MenuItem::Separator
    }

    pub fn new(name: String, hotkey: Option<HotKey>, enabled: bool, selected: bool) -> MenuItem {
        MenuItem::Item {
            id: Id::new_v4(),
            name,
            hotkey,
            enabled,
            selected
        }
    }
}