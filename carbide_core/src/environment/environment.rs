use std::collections::HashMap;
use std::ffi::c_void;
use std::future::Future;
use std::option::Option::Some;
use std::time::Instant;

use bitflags::_core::fmt::Formatter;
use futures::FutureExt;
use fxhash::{FxBuildHasher, FxHashMap};
use image::DynamicImage;
use oneshot::TryRecvError;

use crate::{Color, image_map};
use crate::animation::Animation;
use crate::cursor::MouseCursor;
use crate::draw::Dimension;
use crate::draw::Scalar;
use crate::environment::WidgetTransferAction;
use crate::focus::Refocus;
use crate::mesh::TextureAtlas;
use crate::prelude::{EnvironmentColor, EnvironmentVariable};
use crate::state::{InnerState, StateContract, StateKey, ValueCell};
use crate::text::{Font, FontFamily, FontId, FontSize, FontStyle, FontWeight, Glyph};
use crate::widget::{ImageFilter, Overlay};
use crate::widget::ImageInformation;

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
    overlay_map: FxHashMap<String, Option<Overlay>>,

    /// A transfer place for widgets. It is a map with key Option<String>. If None it means the
    /// action should be picked up by the closest parent consumer. If Some it will look at the Id
    /// and only consume if the id matches. The consumer should first take out the None key if
    /// exists, and afterwards take out the special keyed value.
    widget_transfer: FxHashMap<Option<String>, WidgetTransferAction>,

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

    /// A map that contains an image filter used for the Filter widget.
    filter_map: FxHashMap<u32, crate::widget::ImageFilter>,
    /// The next id for the filter. This is used when inserting into the filter_map.
    next_filter_id: u32,

    /// A queue of functions that should be evaluated called each frame. This is called from the
    /// main thread, and will return a boolean true if the task is done and should be removed
    /// from the list. Each task in the queue should be fast to call, and non-blocking. We use
    /// oneshot for evaluating if tasks are completed. If they are, we run the continuation and
    /// remove it from the list.
    async_task_queue: Option<Vec<Box<dyn Fn(&mut Environment) -> bool>>>,

    /// The last image index used by the window. This is used when trying to add images from the
    /// environment to the window. We increase this by one when queueing an image, and the value
    /// is updated each frame.
    last_image_index: u32,

    /// A list of queued images. When an image is added to this queue it will be added to the
    /// window the next frame.
    queued_images: Option<Vec<DynamicImage>>,

    cursor: MouseCursor,

    #[cfg(feature = "tokio")]
    tokio_runtime: tokio::runtime::Runtime,

    animations: Option<Vec<Box<dyn Fn(&Instant) -> bool>>>,

    #[cfg(target_os = "macos")]
    macos_window_handle: Option<*mut c_void>,
    #[cfg(target_os = "windows")]
    windows_window_handle: Option<*mut c_void>,
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
        window_handle: Option<*mut c_void>,
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
            widget_transfer: HashMap::with_hasher(FxBuildHasher::default()),
            focus_request: None,
            pixel_dimensions,
            scale_factor,
            frame_start_time: InnerState::new(ValueCell::new(Instant::now())),
            filter_map: filters,
            next_filter_id: 0,
            async_task_queue: Some(vec![]),
            last_image_index: 0,
            queued_images: None,
            cursor: MouseCursor::Arrow,
            #[cfg(feature = "tokio")]
            tokio_runtime: tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build().expect("Could not create a tokio runtime"),
            animations: Some(vec![]),
            #[cfg(target_os = "macos")]
            macos_window_handle: window_handle,
            #[cfg(target_os = "windows")]
            windows_window_handle: window_handle
        }
    }

    #[cfg(target_os = "macos")]
    pub fn ns_window(&self) -> *mut c_void {
        self.macos_window_handle.expect("No window for the environment")
    }

    #[cfg(target_os = "windows")]
    pub fn hwnd(&self) -> *mut c_void {
        self.windows_window_handle.expect("No window for the environment")
    }

    pub fn set_last_image_index(&mut self, next_index: u32) {
        self.last_image_index = next_index;
    }

    pub fn queue_image(&mut self, image: DynamicImage) -> Option<image_map::Id> {
        if let Some(images) = &mut self.queued_images {
            images.push(image)
        } else {
            self.queued_images = Some(vec![image])
        }
        let id = image_map::Id(self.last_image_index);
        self.last_image_index += 1;
        Some(id)
    }

    pub fn queued_images(&mut self) -> Option<Vec<image::DynamicImage>> {
        self.queued_images.take()
    }

    pub fn insert_animation<A: StateContract + 'static>(&mut self, animation: Animation<A>) {
        let poll = move |time: &Instant| {
            let mut animation = animation.clone();
            animation.update(time)
        };
        self.animations.as_mut().expect("No animation queue was present.").push(Box::new(poll));
    }

    pub fn update_animation(&mut self) {
        let mut temp = None;
        std::mem::swap(&mut temp, &mut self.animations);

        let instant = &*self.frame_start_time.borrow();

        temp.as_mut().map(|t| {
            t.retain(|update_animation| {
                !update_animation(instant)
            })
        });

        std::mem::swap(&mut temp, &mut self.animations);
    }

    pub fn check_tasks(&mut self) {
        let mut temp = None;
        std::mem::swap(&mut temp, &mut self.async_task_queue);

        temp.as_mut().map(|t| {
            t.retain(|task| {
                !task(self)
            })
        });

        std::mem::swap(&mut temp, &mut self.async_task_queue);
    }

    /// Starts a listener where next will be called each time something is sent to the channel.
    /// The sender can be cloned and more values can be sent with it. If true is returned from
    /// next, the stream is closed and will no longer receive values.
    pub fn start_stream<T: Send + 'static>(
        &mut self,
        next: impl Fn(T, &mut Environment) -> bool + 'static,
    ) -> std::sync::mpsc::Sender<T> {
        let (sender, receiver) = std::sync::mpsc::channel();

        let poll_message: Box<dyn Fn(&mut Environment) -> bool> = Box::new(
            move |env| -> bool {
                let mut stop = false;
                loop {
                    if stop {
                        break;
                    }
                    match receiver.try_recv() {
                        Ok(message) => {
                            stop = next(message, env);
                        }
                        Err(std::sync::mpsc::TryRecvError::Empty) => {
                            break;
                        }
                        Err(e) => {
                            eprintln!("{:?}", e);
                            stop = true;
                        }
                    }
                }
                stop
            }
        );

        self.async_task_queue.as_mut().expect("No async task queue was present.").push(poll_message);

        sender
    }

    pub fn spawn_task<T: Send + 'static>(
        &mut self,
        task: impl Future<Output=T> + Send + 'static,
        cont: impl Fn(T, &mut Environment) + 'static,
    ) {
        let (sender, receiver) = oneshot::channel();

        let task_with_oneshot = task.then(|message| async move {
            let _ = sender.send(message);
            ()
        });

        let poll_message: Box<dyn Fn(&mut Environment) -> bool> = Box::new(move |env| -> bool {
            match receiver.try_recv() {
                Ok(message) => {
                    cont(message, env);
                    true
                }
                Err(TryRecvError::Empty) => {
                    false
                }
                Err(e) => {
                    eprintln!("{:?}", e);
                    true
                }
            }
        });
        self.async_task_queue.as_mut().expect("No async task queue was present.").push(poll_message);

        #[cfg(feature = "tokio")]
            {
                self.tokio_runtime.spawn(task_with_oneshot);
            }


        #[cfg(all(feature = "async-std", not(feature = "tokio")))]
            {
                async_std::task::spawn(task_with_oneshot);
            }


        #[cfg(not(any(feature = "async-std", feature = "tokio")))]
        println!("Tried to spawn an async task without having any async feature enabled. Try enabling 'async-std' or 'tokio'.")
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

    pub fn cursor(&self) -> MouseCursor {
        self.cursor
    }

    pub fn set_cursor(&mut self, cursor: MouseCursor) {
        self.cursor = cursor;
    }

    pub fn request_focus(&mut self, request_type: Refocus) {
        self.focus_request = Some(request_type);
    }

    pub fn get_image_information(&self, id: &Option<crate::image_map::Id>) -> Option<&ImageInformation> {
        id.as_ref().and_then(|id| {
            self.images_information.get(id)
        })
    }

    pub fn insert_image(&mut self, id: crate::image_map::Id, image: ImageInformation) {
        self.images_information.insert(id, image);
    }

    pub fn overlay(&mut self, id: &String) -> Option<Option<Overlay>> {
        self.overlay_map.remove(id)
    }

    pub fn add_overlay(&mut self, id: &str, overlay: Option<Overlay>) {
        self.overlay_map.insert(id.to_string(), overlay);
    }

    pub fn transferred_widget(&mut self, id: Option<String>) -> Option<WidgetTransferAction> {
        self.widget_transfer.remove(&id)
    }

    pub fn transfer_widget(&mut self, id: Option<String>, widget_transfer: WidgetTransferAction) {
        self.widget_transfer.insert(id, widget_transfer);
    }

    pub fn clear(&mut self) {}

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

    pub fn insert_font_from_file<P>(&mut self, path: P) -> (FontId, FontWeight, FontStyle)
        where
            P: AsRef<std::path::Path>,
    {
        let mut font = Font::from_file(path).unwrap();
        let weight = font.weight();
        let style = font.style();
        let font_id = self.fonts.len();
        font.set_font_id(font_id);
        self.fonts.push(font);
        (font_id, weight, style)
    }

    pub fn insert_bitmap_font_from_file<P>(&mut self, path: P) -> (FontId, FontWeight, FontStyle)
        where
            P: AsRef<std::path::Path>,
    {
        let mut font = Font::from_file_bitmap(path).unwrap();
        let weight = font.weight();
        let style = font.style();
        let font_id = self.fonts.len();
        font.set_font_id(font_id);
        self.fonts.push(font);
        (font_id, weight, style)
    }

    pub fn add_glyphs_to_atlas(&mut self, glyphs: Vec<&mut Glyph>) {
        let scale_factor = self.get_scale_factor();
        for glyph in glyphs {
            let font = &self.fonts[glyph.font_id()];
            if let Some(entry) = self.font_texture_atlas.queue_glyph(glyph, font, scale_factor) {
                glyph.set_texture_index(entry);
            }
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

    // TODO: Add fonts automatically and warn if none could be loaded: https://github.com/RazrFalcon/fontdb/blob/master/src/lib.rs
    pub fn add_font_family(&mut self, mut family: FontFamily) {
        for font in &mut family.fonts {
            let assets = find_folder::Search::KidsThenParents(3, 5)
                .for_folder("assets")
                .unwrap();
            let font_path = assets.join(&font.path);
            let (font_id, weight, style) = if font.is_bitmap {
                self.insert_bitmap_font_from_file(font_path)
            } else {
                self.insert_font_from_file(font_path)
            };
            font.font_id = font_id;
            font.weight_hint = weight;
            font.style_hint = style;
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

    pub fn env_color(&self, color: EnvironmentColor) -> Option<Color> {
        self.get_color(&StateKey::Color(color))
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
