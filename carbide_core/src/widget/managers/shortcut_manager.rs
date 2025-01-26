use crate::environment::EnvironmentKey;
use crate::widget::WidgetId;

#[derive(Debug)]
pub struct ShortcutManager {
    shortcut_id: Option<WidgetId>
}

impl ShortcutManager {
    pub fn new() -> Self {
        ShortcutManager {
            shortcut_id: None,
        }
    }

    pub fn shortcut(&mut self, id: WidgetId) -> bool {
        if let Some(_) = self.shortcut_id {
            return false;
        }

        self.shortcut_id = Some(id);

        true
    }

    pub fn has_shortcut(&self) -> Option<WidgetId> {
        self.shortcut_id
    }
}

impl EnvironmentKey for ShortcutManager {
    type Value = ShortcutManager;
}

#[derive(Copy, Clone, Debug)]
pub struct ShortcutPressed(pub WidgetId);


#[derive(Copy, Clone, Debug)]
pub struct ShortcutReleased(pub WidgetId);