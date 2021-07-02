use std::collections::HashMap;
use std::fmt::Debug;

use bitflags::_core::fmt::Formatter;
use fxhash::{FxBuildHasher, FxHashMap};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::{Color, from_bin, to_bin};
use crate::focus::Refocus;
use crate::prelude::EnvironmentVariable;
use crate::state::global_state::GlobalState;
use crate::state::state::State;
use crate::state::state_key::StateKey;
use crate::text::{Font, FontFamily, FontId};
use crate::widget::Dimensions;
use crate::widget::primitive::Widget;
use crate::widget::types::image_information::ImageInformation;

pub struct Environment<GS> where GS: GlobalState {
    /// This stack should be used to scope the environment. This contains information such as
    /// current foreground color, text colors and more. This means a parent can choose some
    /// styling that is applied to all of its children, unless some child overrides that style.
    // TODO: Consider switching to a map, so we dont need to search through the vec for better performance
    stack: Vec<EnvironmentVariable>,

    /// Keep the loaded fonts in a map from font id to font. This is used when
    /// calculating the size of rendered strings.
    fonts: Vec<Font>,

    /// Font families. The fonts are still kept as seperate fonts in the above vec, but references
    /// are kept in families for better lookup.
    font_families: FxHashMap<String, FontFamily>,

    /// This map contains the widths and heights for loaded images.
    /// This is used to make the static size of the Image widget its
    /// required size.
    images_information: FxHashMap<crate::image_map::Id, ImageInformation>,

    /// A map from String to a widget.
    /// This key should correspond to the targeted overlay_layer
    overlay_map: FxHashMap<String, Box<dyn Widget<GS>>>,

    /// Keep local state as a map from String, to a vector of bytes.
    /// The vector is used as a serializing target for the state value.
    /// bin-code is used to serialize the state.
    /// Keys should be unique to avoid trying to deserialize state into
    /// different state.
    pub(crate) local_state: FxHashMap<StateKey, Vec<u8>>,

    /// This field holds the requests for refocus. If Some we need to check the refocus
    /// reason and apply that focus change after the event is done. This also means that
    /// the focus change is not instant, but updates after each run event.
    pub(crate) focus_request: Option<Refocus>,

    /// The size of the drawing area in actual pixels.
    pixel_dimensions: Dimensions,

    /// The pixel density, or scale factor.
    /// On windows this is the settable factor in desktop settings.
    /// On retina displays for macos this is 2 and otherwise 1.
    scale_factor: f64,
}

impl<GS: GlobalState> std::fmt::Debug for Environment<GS> {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl<GS: GlobalState> Environment<GS> {
    pub fn new(env_stack: Vec<EnvironmentVariable>, pixel_dimensions: Dimensions, scale_factor: f64) -> Self {
        Environment {
            stack: env_stack,
            fonts: vec![],
            font_families: HashMap::with_hasher(FxBuildHasher::default()),
            images_information: HashMap::with_hasher(FxBuildHasher::default()),
            overlay_map: HashMap::with_hasher(FxBuildHasher::default()),
            local_state: HashMap::with_hasher(FxBuildHasher::default()),
            focus_request: None,
            pixel_dimensions,
            scale_factor,
        }
    }

    pub fn set_pixel_width(&mut self, new_pixel_width: f64) {
        self.pixel_dimensions[0] = new_pixel_width;
    }

    pub fn set_pixel_height(&mut self, new_pixel_height: f64) {
        self.pixel_dimensions[1] = new_pixel_height;
    }

    pub fn set_scale_factor(&mut self, new_scale_factor: f64) {
        self.scale_factor = new_scale_factor;
    }

    pub fn get_corrected_width(&self) -> f64 {
        self.pixel_dimensions[0] / self.scale_factor
    }

    pub fn get_corrected_height(&self) -> f64 {
        self.pixel_dimensions[1] / self.scale_factor
    }

    pub fn get_corrected_dimensions(&self) -> Dimensions {
        [self.pixel_dimensions[0] / self.scale_factor, self.pixel_dimensions[1] / self.scale_factor]
    }

    pub fn get_pixel_width(&self) -> f64 {
        self.pixel_dimensions[0] / self.scale_factor
    }

    pub fn get_pixel_height(&self) -> f64 {
        self.pixel_dimensions[1] / self.scale_factor
    }

    pub fn get_pixel_dimensions(&self) -> Dimensions {
        self.pixel_dimensions
    }

    pub fn get_scale_factor(&self) -> f64 {
        self.scale_factor
    }

    pub fn request_focus(&mut self, request_type: Refocus) {
        self.focus_request = Some(request_type);
    }

    pub fn get_image_information(&self, id: &crate::image_map::Id) -> Option<&ImageInformation> {
        self.images_information.get(id)
    }

    pub fn insert_image(&mut self, id: crate::image_map::Id, image: ImageInformation) {
        self.images_information.insert(id, image);
    }

    pub fn get_overlay(&mut self, id: &String) -> Option<Box<dyn Widget<GS>>> {
        self.overlay_map.remove(id)
    }

    pub fn add_overlay(&mut self, id: &str, overlay: Box<dyn Widget<GS>>) {
        self.overlay_map.insert(id.to_string(), overlay);
    }

    pub fn clear(&mut self) {
        self.clear_local_state();
        self.overlay_map.clear();
    }

    fn clear_local_state(&mut self) {
        self.local_state.clear()
    }

    pub fn update_local_state<T: Serialize + Clone + Debug + DeserializeOwned>(&self, local_state: &mut dyn State<T, GS>) {
        local_state.update_dependent_states(self);
        if let Some(key) = local_state.get_key() {
            let local_value: &Vec<u8> = match self.local_state.get(key) {
                Some(n) => n,
                None => return,
            };
            *local_state.get_latest_value_mut() = from_bin::<T>(&local_value).unwrap();
        }
    }

    pub fn insert_local_state<T: Serialize + Clone + Debug>(&mut self, local_state: &dyn State<T, GS>) {
        local_state.insert_dependent_states(self);
        if let Some(key) = local_state.get_key() {
            let value = local_state.get_latest_value();
            self.local_state.insert(key.clone(), to_bin(value).unwrap());
        }
    }

    pub fn insert_local_state_from_key_value<T: Serialize + Clone + Debug>(&mut self, key: &StateKey, value: &T) {
        self.local_state.insert(key.clone(), to_bin(value).unwrap());
    }

    pub fn insert_font_from_file<P>(&mut self, path: P) -> FontId
        where P: AsRef<std::path::Path>,
    {
        let font = Font::from_file(path).unwrap();
        let font_id = self.fonts.len();
        self.fonts.push(font);
        font_id
    }

    pub fn insert_bitmap_font_from_file<P>(&mut self, path: P) -> FontId
        where P: AsRef<std::path::Path>,
    {
        let font = Font::from_file_bitmap(path).unwrap();
        let font_id = self.fonts.len();
        self.fonts.push(font);
        font_id
    }

    pub fn get_font(&self, id: FontId) -> &Font {
        &self.fonts[id]
    }

    pub fn get_font_mut(&mut self, id: FontId) -> &mut Font {
        &mut self.fonts[id]
    }

    /// Adds the given `rusttype::Font` to the `Map` and returns a unique `Id` for it.
    pub fn insert_font(&mut self, font: Font) -> FontId {
        let font_id = self.fonts.len();
        self.fonts.push(font);
        font_id
    }

    pub fn add_font_family(&mut self, mut family: FontFamily) {
        for font in &mut family.fonts {
            let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
            let font_path = assets.join(&font.path);
            if font.is_bitmap {
                self.insert_bitmap_font_from_file(font_path);
            } else {
                self.insert_font_from_file(font_path);
            }
        }
        let key = family.name.clone();
        self.font_families.insert(key, family);
    }

    pub fn get_first_font_family(&self) -> &FontFamily {
        self.font_families.iter().next().unwrap().1
    }

    pub fn get_font_family(&self, name: &String) -> &FontFamily {
        self.font_families.get(name).unwrap()
    }


    pub fn push_vec(&mut self, value: Vec<EnvironmentVariable>) {
        for v in value {
            self.push(v);
        }
    }

    pub fn push(&mut self, value: EnvironmentVariable) {
        self.stack.push(value);
    }

    pub fn pop(&mut self) {
        self.stack.pop();
    }

    pub fn get_color(&self, color: &StateKey) -> Option<Color> {
        if let StateKey::Color(col) = color {
            for item in self.stack.iter().rev() {
                match item {
                    EnvironmentVariable::Color { key, value } => {
                        if key == col {
                            return Some(value.clone())
                        }
                    }
                    _ => ()
                }
            }
        }

        None
    }

    pub fn get_font_size(&self, font_size: &StateKey) -> Option<u32> {
        if let StateKey::FontSize(size) = font_size {
            for item in self.stack.iter().rev() {
                match item {
                    EnvironmentVariable::FontSize { key, value } => {
                        if key == size {
                            return Some(*value)
                        }
                    }
                    _ => ()
                }
            }
        }

        None
    }
}
