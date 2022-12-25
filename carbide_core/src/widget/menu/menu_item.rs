use std::fmt::{Debug, Formatter};

use dyn_clone::DynClone;

use carbide_core::widget::menu::menu::Menu;

use crate::environment::Environment;
use crate::event::HotKey;
use crate::widget::WidgetId;

pub trait MenuAction: Fn(&mut Environment) + DynClone {}

impl<I> MenuAction for I where I: Fn(&mut Environment) + Clone {}

dyn_clone::clone_trait_object!(MenuAction);

impl Debug for Box<dyn MenuAction> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MenuAction").finish()
    }
}

#[derive(Debug, Clone)]
pub enum MenuItem {
    Item {
        id: WidgetId,
        name: String,
        hotkey: Option<HotKey>,
        enabled: bool,
        selected: bool,
        action: Box<dyn MenuAction>,
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

    pub fn new(name: &str, selected: bool) -> MenuItem {
        MenuItem::Item {
            id: WidgetId::new(),
            name: name.to_string(),
            hotkey: None,
            enabled: true,
            selected,
            action: Box::new(|_| {})
        }
    }

    pub fn hotkey(mut self, hotkey: HotKey) -> Self {
        let hk = hotkey;
        match &mut self {
            MenuItem::Item { hotkey, .. } => {
                *hotkey = Some(hk);
            }
            MenuItem::Separator => {}
            MenuItem::SubMenu { .. } => {}
        }

        self
    }

    pub fn disabled(mut self) -> Self {
        match &mut self {
            MenuItem::Item { enabled, .. } => {
                *enabled = false;
            }
            MenuItem::Separator => {}
            MenuItem::SubMenu { .. } => {}
        }

        self
    }

    pub fn action(mut self, action: Box<dyn MenuAction>) -> Self {
        let a = action;
        match &mut self {
            MenuItem::Item { action, .. } => {
                *action = a;
            }
            MenuItem::Separator => {}
            MenuItem::SubMenu { .. } => {}
        }

        self
    }
}
