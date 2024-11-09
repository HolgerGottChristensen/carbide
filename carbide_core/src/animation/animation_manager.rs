use std::time::Instant;
use crate::accessibility::{Accessibility, AccessibilityContext};
use crate::color::rgba_bytes;
use crate::draw::theme::Theme;
use crate::draw::Color;
use crate::draw::Dimension;
use crate::environment::{EnvironmentColor, EnvironmentFontSize, Key, Keyable};
use crate::event::{AccessibilityEvent, AccessibilityEventContext, AccessibilityEventHandler, Event, KeyboardEvent, KeyboardEventContext, KeyboardEventHandler, MouseEvent, MouseEventContext, MouseEventHandler, OtherEventContext, OtherEventHandler, WindowEvent, WindowEventContext, WindowEventHandler};
use crate::focus::{FocusContext, Focusable};
use crate::layout::{Layout, LayoutContext};
use crate::lifecycle::{InitializationContext, Initialize, Update, UpdateContext};
use crate::render::Render;
use crate::render::RenderContext;
use crate::widget::{CommonWidget, Widget};
use carbide::ModifierWidgetImpl;

#[derive(Debug, Clone)]
pub struct AnimationManager {
    frame_count: u32,
    frame_time: Instant
}

impl AnimationManager {
    pub fn new() -> AnimationManager {
        AnimationManager {
            frame_count: 0,
            frame_time: Instant::now(),
        }
    }

    pub fn number_of_animation_frames(&self) -> u32 {
        self.frame_count
    }

    pub fn request_animation_frame(&mut self) {
        self.frame_count = self.frame_count.max(1);
    }

    pub fn request_multiple_animation_frames(&mut self, n: u32) {
        self.frame_count = self.frame_count.max(n);
    }

    pub fn take_frame(&mut self) -> bool {
        if self.frame_count > 0 {
            self.frame_count -= 1;
            true
        } else {
            false
        }
    }

    pub fn frame_time(&self) -> Instant {
        self.frame_time
    }

    pub fn update_frame_time(&mut self) {
        self.frame_time = Instant::now();
    }
}

impl Key for AnimationManager {
    type Value = AnimationManager;
}