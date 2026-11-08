use bitflags::{bitflags};

// Defining every combination to allow assignment in static expressions.
bitflags! {
    /// Represents the current state of the keyboard modifiers
    ///
    /// Each flag represents a modifier and is set if this modifier is active.
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct ModifierKey: u32 {
        const EMPTY = 0b000;
        /// The "shift" key.
        const SHIFT = 0b100;
        /// The "control" key.
        const CONTROL = 0b100 << 3;
        /// The "alt" key.
        const ALT = 0b100 << 6;
        /// This is the "windows" key on PC and "command" key on Mac.
        const SUPER = 0b100 << 9;

        const CTRL_SHIFT            = ModifierKey::CONTROL.bits()
                                    | ModifierKey::SHIFT.bits();
        /// Ctrl + Alt.
        const CTRL_ALT              = ModifierKey::CONTROL.bits()
                                    | ModifierKey::ALT.bits();
        /// Ctrl + Gui.
        const CTRL_SUPER              = ModifierKey::CONTROL.bits()
                                    | ModifierKey::SUPER.bits();
        /// Ctrl + Shift + Alt.
        const CTRL_SHIFT_ALT        = ModifierKey::CONTROL.bits()
                                    | ModifierKey::SHIFT.bits()
                                    | ModifierKey::ALT.bits();
        /// Ctrl + Shift + Gui.
        const CTRL_SHIFT_SUPER       = ModifierKey::CONTROL.bits()
                                    | ModifierKey::SHIFT.bits()
                                    | ModifierKey::SUPER.bits();
        /// Ctrl + Shift + Alt + Gui.
        const CTRL_SHIFT_ALT_SUPER    = ModifierKey::CONTROL.bits()
                                    | ModifierKey::SHIFT.bits()
                                    | ModifierKey::ALT.bits()
                                    | ModifierKey::SUPER.bits();
        /// Shift + Alt.
        const SHIFT_ALT             = ModifierKey::SHIFT.bits()
                                    | ModifierKey::ALT.bits();
        /// Shift + Gui.
        const SHIFT_SUPER             = ModifierKey::SHIFT.bits()
                                    | ModifierKey::SUPER.bits();
        /// Shift + Alt + Gui.
        const SHIFT_ALT_SUPER        = ModifierKey::SHIFT.bits()
                                    | ModifierKey::ALT.bits()
                                    | ModifierKey::SUPER.bits();
        /// Alt + Gui.
        const ALT_GUI               = ModifierKey::ALT.bits()
                                    | ModifierKey::SUPER.bits();
    }
}

impl ModifierKey {
    /// Returns `true` if the shift key is pressed.
    pub fn shift_key(&self) -> bool {
        self.intersects(Self::SHIFT)
    }
    /// Returns `true` if the control key is pressed.
    pub fn control_key(&self) -> bool {
        self.intersects(Self::CONTROL)
    }
    /// Returns `true` if the alt key is pressed.
    pub fn alt_key(&self) -> bool {
        self.intersects(Self::ALT)
    }
    /// Returns `true` if the super key is pressed.
    pub fn super_key(&self) -> bool {
        self.intersects(Self::SUPER)
    }
}
