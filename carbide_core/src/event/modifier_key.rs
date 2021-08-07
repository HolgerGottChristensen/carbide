use bitflags::bitflags;
use serde::{Deserialize, Serialize};

// Defining every combination to allow assignment in static expressions.
bitflags!(
    #[allow(missing_docs)]
    #[derive(Deserialize, Serialize)]
    pub struct ModifierKey: u8 {
        /// No modifier.
        const NO_MODIFIER           = 0b00000000;
        /// Ctrl.
        const CTRL                  = 0b00000001;
        /// Shift.
        const SHIFT                 = 0b00000010;
        /// Alt.
        const ALT                   = 0b00000100;
        /// Gui.
        const GUI                   = 0b00001000;
        /// Ctrl + Shift.
        const CTRL_SHIFT            = ModifierKey::CTRL.bits
                                    | ModifierKey::SHIFT.bits;
        /// Ctrl + Alt.
        const CTRL_ALT              = ModifierKey::CTRL.bits
                                    | ModifierKey::ALT.bits;
        /// Ctrl + Gui.
        const CTRL_GUI              = ModifierKey::CTRL.bits
                                    | ModifierKey::GUI.bits;
        /// Ctrl + Shift + Alt.
        const CTRL_SHIFT_ALT        = ModifierKey::CTRL.bits
                                    | ModifierKey::SHIFT.bits
                                    | ModifierKey::ALT.bits;
        /// Ctrl + Shift + Gui.
        const CTRL_SHIFT_GUI        = ModifierKey::CTRL.bits
                                    | ModifierKey::SHIFT.bits
                                    | ModifierKey::GUI.bits;
        /// Ctrl + Shift + Alt + Gui.
        const CTRL_SHIFT_ALT_GUI    = ModifierKey::CTRL.bits
                                    | ModifierKey::SHIFT.bits
                                    | ModifierKey::ALT.bits
                                    | ModifierKey::GUI.bits;
        /// Shift + Alt.
        const SHIFT_ALT             = ModifierKey::SHIFT.bits
                                    | ModifierKey::ALT.bits;
        /// Shift + Gui.
        const SHIFT_GUI             = ModifierKey::SHIFT.bits
                                    | ModifierKey::GUI.bits;
        /// Shift + Alt + Gui.
        const SHIFT_ALT_GUI         = ModifierKey::SHIFT.bits
                                    | ModifierKey::ALT.bits
                                    | ModifierKey::GUI.bits;
        /// Alt + Gui.
        const ALT_GUI               = ModifierKey::ALT.bits
                                    | ModifierKey::GUI.bits;
    }
);

impl Default for ModifierKey {
    fn default() -> ModifierKey {
        ModifierKey::NO_MODIFIER
    }
}