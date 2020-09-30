//! Contains all render used to describe the input events that `Widget`s may handle.
//!
//! The two primary render of this module are:
//!
//! - `Input`: conrod's input type passed by the user to `Ui::handle_event` in order to drive the
//! `Ui`.
//! - `Event`: enumerates all possible events interpreted by conrod that may be propagated to
//! widgets.
//!
//! The Event System
//! ----------------
//!
//! Conrod's event system looks like this:
//!
//! *Input -> Ui -> Event -> Widget*
//!
//! The **Ui** receives **Input**s such as `Press` and `Release` via the `Ui::handle_event` method.
//! It interprets these **Input**s to create higher-level **Event**s such as `DoubleClick`,
//! `WidgetCapturesKeyboard`, etc. These **Event**s are stored and then fed to each **Widget** when
//! `Ui::set_widgets` is called. At the end of `Ui::set_widgets` the stored **Event**s are flushed
//! ready for the next incoming **Input**s.
//!
//! Conrod uses the `pistoncore-input` crate's `Input` type. There are a few reasons for this:
//!
//! 1. This `Input` type already provides a number of useful variants of events that we wish to
//!    provide and handle within conrod, and we do not yet see any great need to re-write it and
//!    duplicate code.
//! 2. The `Input` type is already compatible with all `pistoncore-window` backends including
//!    `glfw_window`, `sdl2_window` and `glutin_window`. That said, co-ordinates and scroll
//!    directions may need to be translated to conrod's orientation.
//! 3. The `pistoncore-input` crate also provides a `GenericEvent` trait which allows us to easily
//!    provide a blanket implementation of `ToRawEvent` for all event render that already implement
//!    this trait.
//!
//! Because we use the `pistoncore-input` `Event` type, we also re-export its associated data
//! render (`Button`, `ControllerAxisArgs`, `Key`, etc).

pub mod release;
pub mod key_press;
pub mod mouse_press;
pub mod press;
pub mod button;
pub mod motion;
pub mod text;
pub mod widget;
pub mod ui;
pub mod event;
pub mod mouse_release;
pub mod input;
pub mod key_release;
pub mod drag;
pub mod click;
pub mod double_click;
pub mod tap;
pub mod scroll;