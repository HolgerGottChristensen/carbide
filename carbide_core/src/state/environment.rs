use std::collections::HashMap;
use std::fmt::Debug;

use bitflags::_core::fmt::Formatter;
use serde::Serialize;

use crate::{Color, from_bin, to_bin, text};
use crate::text::font::{Error, Id};
use crate::widget::primitive::Widget;
use crate::widget::types::image_information::ImageInformation;
use crate::state::global_state::GlobalState;
use crate::state::state::State;
use serde::de::DeserializeOwned;
use crate::focus::Refocus;
use crate::state::environment_variable::EnvironmentVariable;
use fxhash::{FxHashMap, FxBuildHasher};
use crate::state::state_key::StateKey;
use crate::widget::Dimensions;

pub struct Environment<GS> where GS: GlobalState {

    /// This stack should be used to scope the environment. This contains information such as
    /// current foreground color, text colors and more. This means a parent can choose some
    /// styling that is applied to all of its children, unless some child overrides that style.
    // TODO: Consider switching to a map, so we dont need to search through the vec for better performance
    stack: Vec<EnvironmentVariable>,

    /// Keep the loaded fonts in a map from font id to font. This is used when
    /// calculating the size of rendered strings.
    fonts: text::font::Map,

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

    pub window_dimension: Dimensions,
}

impl<GS: GlobalState> std::fmt::Debug for Environment<GS> {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl<GS: GlobalState> Environment<GS> {

    pub fn new(env_stack: Vec<EnvironmentVariable>, dimensions: Dimensions) -> Self {
        Environment {
            stack: env_stack,
            fonts: text::font::Map::new(),
            images_information: HashMap::with_hasher(FxBuildHasher::default()),
            overlay_map: HashMap::with_hasher(FxBuildHasher::default()),
            local_state: HashMap::with_hasher(FxBuildHasher::default()),
            focus_request: None,
            window_dimension: dimensions
        }
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

    pub fn get_fonts_map(&self) -> &text::font::Map {
        &self.fonts
    }

    pub fn insert_font_from_file<P>(&mut self, path: P) -> Result<Id, Error>
        where P: AsRef<std::path::Path>,
    {
        self.fonts.insert_from_file(path)

    }

    pub fn get_font(&self, id: Id) -> &rusttype::Font<'static> {
        self.fonts.get(id).expect("No font was found with the id")
    }

    /// Adds the given `rusttype::Font` to the `Map` and returns a unique `Id` for it.
    pub fn insert_font(&mut self, font: rusttype::Font<'static>) -> Id {
        self.fonts.insert(font)
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
