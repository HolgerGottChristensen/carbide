use std;
use std::sync::atomic::{AtomicU32, Ordering};
use crate::draw::image::image_id::ImageId;


/// A type used to map the `widget::Id` of `Image` widgets to their associated `Img` data.
///
/// The `image::Map` type is usually instantiated and loaded during the "setup" stage of the
/// application before the main loop begins.
pub struct ImageMap<Img> {
    map: FHashMap<Img>,
}

/// The type of `std::collections::HashMap` with `fnv::FnvHasher` used within the `image::Map`.
pub type FHashMap<Img> = fxhash::FxHashMap<ImageId, Img>;

impl<Img> std::ops::Deref for ImageMap<Img> {
    type Target = FHashMap<Img>;
    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl<Img> ImageMap<Img> {
    /// Construct a new, empty `image::Map`.
    pub fn new() -> Self {
        ImageMap {
            map: FHashMap::<Img>::default(),
        }
    }

    // Calling any of the following methods will trigger a redraw when using `Ui::draw_if_changed`.

    /// Uniquely borrow the `Img` associated with the given widget.
    ///
    /// Note: Calling this will trigger a redraw the next time `Ui::draw_if_changed` is called.
    pub fn get_mut(&mut self, id: ImageId) -> Option<&mut Img> {
        self.map.get_mut(&id)
    }

    /// Inserts the given image into the map, returning its associated `image::Id`. The user *must*
    /// store the returned `image::Id` in order to use, modify or remove the inserted image.
    ///
    /// Note: Calling this will trigger a redraw the next time `Ui::draw_if_changed` is called.
    pub fn insert(&mut self, img: Img) -> ImageId {
        let id = ImageId::new();
        self.map.insert(id, img);
        id
    }

    pub fn insert_with_id(&mut self, id: ImageId, img: Img) -> ImageId {
        self.map.insert(id, img);
        id
    }

    /// Replaces the given image in the map if it exists. Returns the image or None.
    ///
    /// Note: Calling this will trigger a redraw the next time `Ui::draw_if_changed` is called.
    pub fn replace(&mut self, id: ImageId, img: Img) -> Option<Img> {
        self.map.insert(id, img)
    }

    /// Removes the given image from the map if it exists. Returns the image or None.
    ///
    /// Any future use of the given `image::Id` will be invalid.
    ///
    /// Note: Calling this will trigger a redraw the next time `Ui::draw_if_changed` is called.
    pub fn remove(&mut self, id: ImageId) -> Option<Img> {
        self.map.remove(&id)
    }
}