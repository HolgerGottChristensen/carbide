use crate::widget::menu::menu_item::MenuItem;
//use carbide_core::platform::mac::menu::set_application_menu;

pub type ContextMenu = Menu;

#[derive(Debug, Clone)]
pub struct Menu {
    name: String,
    items: Vec<MenuItem>,
    pub(crate) hint: Option<MenuHint>,
}

#[derive(Debug, Clone)]
pub enum MenuHint {
    /// This is not really recommended to be used for now, since the menu it creates is not looking
    /// like the one from swiftui. It for example is missing minimize, zoom and bring all to front.
    /// It will also contain tab things, but I think that is unstable in winit and the app keeps
    /// crashing with this.
    Window,
    /// Inserts the mac specific help search thing at the start of the menu if this is applied.
    Help,
}

impl Menu {
    pub fn new(title: &str) -> Menu {
        Menu {
            name: title.to_string(),
            items: vec![],
            hint: None,
        }
    }

    pub fn hint(mut self, kind: MenuHint) -> Menu {
        self.hint = Some(kind);
        self
    }

    pub fn item(mut self, item: MenuItem) -> Menu {
        self.items.push(item);
        self
    }

    pub fn separator(mut self) -> Menu {
        self.items.push(MenuItem::separator());
        self
    }

    pub fn items(&self) -> &Vec<MenuItem> {
        &self.items
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn sub_menu(mut self, menu: Menu) -> Menu {
        self.items.push(MenuItem::SubMenu { menu });
        self
    }
}
