use std::collections::HashMap;
use std::time::Instant;

use bitflags::_core::fmt::Formatter;
use fxhash::{FxBuildHasher, FxHashMap};

use crate::Color;
use crate::draw::Dimension;
use crate::draw::Scalar;
use crate::focus::Refocus;
use crate::mesh::TextureAtlas;
use crate::prelude::EnvironmentVariable;
use crate::state::{InnerState, StateKey, ValueCell};
use crate::text::{Font, FontFamily, FontId, FontSize, FontStyle, FontWeight, Glyph};
use crate::widget::ImageFilter;
use crate::widget::ImageInformation;
use crate::widget::Widget;

pub struct Environment {
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

    /// Font atlas that keeps track of all the textures of the glyphs to render, along with details
    /// on texture offsets and more.
    font_texture_atlas: TextureAtlas,

    /// System font family name. This is used to get the font family for default text rendering.
    /// This should always be expected to exist.
    default_font_family_name: String,

    /// This map contains the widths and heights for loaded images.
    /// This is used to make the static size of the Image widget its
    /// required size.
    images_information: FxHashMap<crate::image_map::Id, ImageInformation>,

    /// A map from String to a widget.
    /// This key should correspond to the targeted overlay_layer
    overlay_map: FxHashMap<String, Box<dyn Widget>>,

    /// Keep local state as a map from String, to a vector of bytes.
    /// The vector is used as a serializing target for the state value.
    /// bin-code is used to serialize the state.
    /// Keys should be unique to avoid trying to deserialize state into
    /// different state.
    //pub(crate) local_state: FxHashMap<StateKey, Option<Box<dyn Any>>>,

    /// This field holds the requests for refocus. If Some we need to check the refocus
    /// reason and apply that focus change after the event is done. This also means that
    /// the focus change is not instant, but updates after each run event.
    pub(crate) focus_request: Option<Refocus>,

    /// The size of the drawing area in actual pixels.
    pixel_dimensions: Dimension,

    /// The pixel density, or scale factor.
    /// On windows this is the settable factor in desktop settings.
    /// On retina displays for macos this is 2 and otherwise 1.
    scale_factor: f64,

    /// The start time of the current frame. This is used to sync the animated states.
    frame_start_time: InnerState<Instant>,

    filter_map: FxHashMap<u32, crate::widget::ImageFilter>,
    next_filter_id: u32,
}

impl std::fmt::Debug for Environment {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl Environment {
    pub fn new(
        env_stack: Vec<EnvironmentVariable>,
        pixel_dimensions: Dimension,
        scale_factor: f64,
    ) -> Self {
        let default_font_family_name = "NotoSans";

        let filters = HashMap::with_hasher(FxBuildHasher::default());

        Environment {
            stack: env_stack,
            fonts: vec![],
            font_families: HashMap::with_hasher(FxBuildHasher::default()),
            font_texture_atlas: TextureAtlas::new(512, 512),
            default_font_family_name: default_font_family_name.to_string(),
            images_information: HashMap::with_hasher(FxBuildHasher::default()),
            overlay_map: HashMap::with_hasher(FxBuildHasher::default()),
            //local_state: HashMap::with_hasher(FxBuildHasher::default()),
            focus_request: None,
            pixel_dimensions,
            scale_factor,
            frame_start_time: InnerState::new(ValueCell::new(Instant::now())),
            filter_map: filters,
            next_filter_id: 0,
        }
    }

    pub fn capture_time(&mut self) {
        *self.frame_start_time.borrow_mut() = Instant::now();
    }

    pub fn captured_time(&self) -> InnerState<Instant> {
        self.frame_start_time.clone()
    }

    pub fn set_pixel_width(&mut self, new_pixel_width: f64) {
        self.pixel_dimensions.width = new_pixel_width;
    }

    pub fn set_pixel_height(&mut self, new_pixel_height: f64) {
        self.pixel_dimensions.height = new_pixel_height;
    }

    pub fn set_scale_factor(&mut self, new_scale_factor: f64) {
        self.scale_factor = new_scale_factor;
    }

    pub fn get_corrected_width(&self) -> f64 {
        self.pixel_dimensions.width / self.scale_factor
    }

    pub fn get_corrected_height(&self) -> f64 {
        self.pixel_dimensions.height / self.scale_factor
    }

    pub fn get_corrected_dimensions(&self) -> Dimension {
        Dimension::new(
            self.pixel_dimensions.width / self.scale_factor,
            self.pixel_dimensions.height / self.scale_factor,
        )
    }

    pub fn get_pixel_width(&self) -> f64 {
        self.pixel_dimensions.width / self.scale_factor
    }

    pub fn get_pixel_height(&self) -> f64 {
        self.pixel_dimensions.height / self.scale_factor
    }

    pub fn get_pixel_dimensions(&self) -> Dimension {
        self.pixel_dimensions
    }

    pub fn get_scale_factor(&self) -> f64 {
        self.scale_factor
    }

    pub fn get_font_atlas_mut(&mut self) -> &mut TextureAtlas {
        &mut self.font_texture_atlas
    }

    pub fn get_font_atlas(&self) -> &TextureAtlas {
        &self.font_texture_atlas
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

    pub fn get_overlay(&mut self, id: &String) -> Option<Box<dyn Widget>> {
        self.overlay_map.remove(id)
    }

    pub fn add_overlay(&mut self, id: &str, overlay: Box<dyn Widget>) {
        self.overlay_map.insert(id.to_string(), overlay);
    }

    pub fn clear(&mut self) {
        self.overlay_map.clear();
    }

    pub fn get_global_state<T>(&self) -> InnerState<T> {
        todo!()
    }

    pub fn filters(&self) -> &FxHashMap<u32, crate::widget::ImageFilter> {
        &self.filter_map
    }

    pub fn insert_filter(&mut self, filter: ImageFilter) -> u32 {
        let filter_id = self.next_filter_id;
        self.next_filter_id += 1;
        self.filter_map.insert(filter_id, filter);
        filter_id
    }

    /// Swaps the local state between the env and the state requesting it.
    /*pub fn swap_local_state<T: StateContract + 'static>(&mut self, local_state: &mut LocalState<T>) {
        if let Some(state_in_env) = self.local_state.get_mut(local_state.key()) {
            std::mem::swap(state_in_env, local_state.value());
        } else {
            self.local_state.insert(local_state.key().clone(), None);
        }
    }*/

    /*pub fn update_local_state<T: Serialize + Clone + Debug + DeserializeOwned>(&self, local_state: &mut dyn State<T, GS>) {
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
    }*/

    /*pub fn insert_local_state_from_key_value<T: Serialize + Clone + Debug>(&mut self, key: &StateKey, value: &T) {
        self.local_state.insert(key.clone(), to_bin(value).unwrap());
    }*/

    pub fn insert_font_from_file<P>(&mut self, path: P) -> FontId
        where
            P: AsRef<std::path::Path>,
    {
        let mut font = Font::from_file(path).unwrap();
        let font_id = self.fonts.len();
        font.set_font_id(font_id);
        self.fonts.push(font);
        font_id
    }

    pub fn insert_bitmap_font_from_file<P>(&mut self, path: P) -> FontId
        where
            P: AsRef<std::path::Path>,
    {
        let mut font = Font::from_file_bitmap(path).unwrap();
        let font_id = self.fonts.len();
        font.set_font_id(font_id);
        self.fonts.push(font);
        font_id
    }

    pub fn add_glyphs_to_atlas(&mut self, glyphs: Vec<&mut Glyph>) {
        let scale_factor = self.get_scale_factor();
        for glyph in glyphs {
            let font = &self.fonts[glyph.font_id()];
            self.font_texture_atlas
                .queue_glyph(glyph, font, scale_factor);
        }
    }

    pub fn remove_glyphs_from_atlas(&mut self, _glyphs: &Vec<Glyph>) {}

    pub fn get_glyph_from_fallback(
        &mut self,
        c: char,
        font_size: FontSize,
        scale_factor: Scalar,
    ) -> (Scalar, Glyph) {
        // Try all the loaded fonts. We only check the first font in each family.
        // Todo: Consider using weight hints and style hints.
        // Todo: Consider going through a separate list if we have a lot of families loaded.
        for (_, font_family) in &self.font_families {
            println!("Looking up in font family: {:?}", font_family);
            let font_id = font_family.get_best_fit(FontWeight::Normal, FontStyle::Normal);
            if let Some(res) = self.get_font(font_id).get_glyph(c, font_size, scale_factor) {
                return res;
            }
        }

        // Try load fallback fonts until we run out
        // Todo: Implement fallback font list.

        // Get the glyph for the unknown character: �
        if c != '�' {
            self.get_glyph_from_fallback('�', font_size, scale_factor)
        } else {
            panic!("Could not lookup the char in any of the loaded fonts or in any fallback fonts. \
            Further more we could not look up the missing char replacement �. Something is not right.")
        }
    }

    pub fn get_font_ref(&self, id: FontId) -> &Font {
        &self.fonts[id]
    }

    pub fn get_font(&self, id: FontId) -> Font {
        self.fonts[id].clone()
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
            let assets = find_folder::Search::KidsThenParents(3, 5)
                .for_folder("assets")
                .unwrap();
            let font_path = assets.join(&font.path);
            let font_id = if font.is_bitmap {
                self.insert_bitmap_font_from_file(font_path)
            } else {
                self.insert_font_from_file(font_path)
            };
            font.font_id = font_id;
        }
        let key = family.name.clone();
        self.font_families.insert(key, family);
    }

    pub fn get_first_font_family(&self) -> &FontFamily {
        for (_, family) in self.font_families.iter() {
            return family;
        }

        panic!("No font family have been added, so we can not get the first.")
    }

    pub fn get_system_font_family(&self) -> &FontFamily {
        self.get_font_family(&self.default_font_family_name)
    }

    pub fn get_font_family(&self, name: &String) -> &FontFamily {
        if name == "system-font" {
            self.get_system_font_family()
        } else {
            self.font_families.get(name).unwrap()
        }
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
                            return Some(value.clone());
                        }
                    }
                    _ => (),
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
                            return Some(*value);
                        }
                    }
                    _ => (),
                }
            }
        }

        None
    }
}
