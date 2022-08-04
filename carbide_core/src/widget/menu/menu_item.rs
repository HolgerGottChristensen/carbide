use crate::event::HotKey;
use crate::widget::WidgetId;
use carbide_core::widget::menu::menu::Menu;

#[derive(Debug, Clone)]
pub enum MenuItem {
    Item {
        id: WidgetId,
        name: String,
        hotkey: Option<HotKey>,
        enabled: bool,
        selected: bool,
    },
    Separator,
    SubMenu {
        menu: Menu,
    },
}

impl MenuItem {
    pub fn separator() -> MenuItem {
        MenuItem::Separator
    }

    pub fn new(name: &str, hotkey: Option<HotKey>, enabled: bool, selected: bool) -> MenuItem {
        MenuItem::Item {
            id: WidgetId::new(),
            name: name.to_string(),
            hotkey,
            enabled,
            selected,
        }
    }
}
