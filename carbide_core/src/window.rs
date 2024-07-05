use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

use image::DynamicImage;

use crate::draw::ImageId;
use crate::locate_folder::Search;
use crate::text::{FontId};
use crate::widget::Menu;
use crate::widget::AnyWidget;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WindowId(pub usize);

impl WindowId {
    pub fn new() -> WindowId {
        static WINDOW_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);
        WindowId(WINDOW_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}
