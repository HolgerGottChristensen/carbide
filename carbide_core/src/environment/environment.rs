use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Formatter;
use std::future::Future;
use std::option::Option::Some;
use std::rc::Rc;
use std::time::Instant;

use fxhash::{FxBuildHasher, FxHashMap};
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

use carbide_core::draw::Position;
use carbide_core::widget::AnyWidget;

use crate::animation::Animation;
use crate::cursor::MouseCursor;
use crate::draw::Dimension;
use crate::draw::Color;
use crate::draw::theme;
use crate::environment::{EnvironmentFontSize, WidgetTransferAction};
use crate::environment::{EnvironmentColor, EnvironmentVariable};
use crate::event::{EventSink, HasEventSink};
use crate::focus::Refocus;
use crate::layout::BasicLayouter;
use crate::state::{InnerState, StateContract};
use crate::widget::{EnvKey, FilterId, ImageFilter, WidgetId};

//type Overlays = Vec<(Box<dyn AnyWidget>, Box<dyn AnyReadState<T=bool>>)>;

pub struct Environment {
    /// This stack should be used to scope the environment. This contains information such as
    /// current foreground color, text colors and more. This means a parent can choose some
    /// styling that is applied to all of its children, unless some child overrides that style.
    // TODO: Consider switching to a map, so we dont need to search through the vec for better performance
    stack: Vec<(&'static str, Box<dyn Any>)>,

    root_alignment: BasicLayouter,

    /// A map from String to a widget.
    /// This key should correspond to the targeted overlay_layer
    overlay_map: FxHashMap<&'static str, Rc<RefCell<Option<Box<dyn AnyWidget>>>>>,

    /// A transfer place for widgets. It is a map with key Option<String>. If None it means the
    /// action should be picked up by the closest parent consumer. If Some it will look at the Id
    /// and only consume if the id matches. The consumer should first take out the None key if
    /// exists, and afterwards take out the special keyed value.
    widget_transfer: FxHashMap<Option<String>, WidgetTransferAction>,

    /// This field holds the requests for refocus. If Some we need to check the refocus
    /// reason and apply that focus change after the event is done. This also means that
    /// the focus change is not instant, but updates after each run event.
    pub focus_request: Option<Refocus>,

    /// The size of the drawing area in actual pixels.
    pixel_dimensions: Dimension,

    /// The pixel density, or scale factor.
    /// On windows this is the settable factor in desktop settings.
    /// On retina displays for macos this is 2 and otherwise 1.
    scale_factor: f64,

    /// The start time of the current frame. This is used to sync the animated states.
    frame_start_time: Rc<RefCell<Instant>>,

    /// A map that contains an image filter used for the Filter widget.
    filter_map: FxHashMap<FilterId, ImageFilter>,

    /// A queue of functions that should be evaluated called each frame. This is called from the
    /// main thread, and will return a boolean true if the task is done and should be removed
    /// from the list. Each task in the queue should be fast to call, and non-blocking. We use
    /// oneshot for evaluating if tasks are completed. If they are, we run the continuation and
    /// remove it from the list.
    async_task_queue: Option<Vec<Box<dyn Fn(&mut Environment) -> bool>>>,

    cursor: MouseCursor,
    mouse_position: Position,

    animations: Option<Vec<Box<dyn Fn(&Instant) -> bool>>>,

    raw_window_handle: Option<RawWindowHandle>,

    event_sink: Box<dyn EventSink>,

    animation_widget_in_frame: usize,

    request_application_close: bool,
}

impl std::fmt::Debug for Environment {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl Environment {
    pub fn new(
        pixel_dimensions: Dimension,
        scale_factor: f64,
        event_sink: Box<dyn EventSink>,
    ) -> Self {
        let font_sizes_large = vec![
            EnvironmentVariable::EnvironmentFontSize {
                key: EnvironmentFontSize::LargeTitle,
                value: 30,
            },
            EnvironmentVariable::EnvironmentFontSize {
                key: EnvironmentFontSize::Title,
                value: 24,
            },
            EnvironmentVariable::EnvironmentFontSize {
                key: EnvironmentFontSize::Title2,
                value: 20,
            },
            EnvironmentVariable::EnvironmentFontSize {
                key: EnvironmentFontSize::Title3,
                value: 18,
            },
            EnvironmentVariable::EnvironmentFontSize {
                key: EnvironmentFontSize::Headline,
                value: 16,
            },
            EnvironmentVariable::EnvironmentFontSize {
                key: EnvironmentFontSize::Body,
                value: 13,
            },
            EnvironmentVariable::EnvironmentFontSize {
                key: EnvironmentFontSize::Callout,
                value: 12,
            },
            EnvironmentVariable::EnvironmentFontSize {
                key: EnvironmentFontSize::Subhead,
                value: 11,
            },
            EnvironmentVariable::EnvironmentFontSize {
                key: EnvironmentFontSize::Footnote,
                value: 9,
            },
            EnvironmentVariable::EnvironmentFontSize {
                key: EnvironmentFontSize::Caption,
                value: 8,
            },
            EnvironmentVariable::EnvironmentFontSize {
                key: EnvironmentFontSize::Caption2,
                value: 7,
            },
        ];

        let env_stack = theme::dark_mode_color_theme()
            .into_iter()
            .chain(font_sizes_large.into_iter())
            .map(|r| {
                match r {
                    EnvironmentVariable::EnvironmentColor { key, value } => {
                        (key.key(), Box::new(value) as Box<dyn Any>)
                    }
                    EnvironmentVariable::EnvironmentFontSize { key, value } => {
                        (key.key(), Box::new(value) as Box<dyn Any>)
                    }
                    EnvironmentVariable::Any { key, value } => {
                        (key, value)
                    }
                }
            })
            .collect::<Vec<_>>();

        let filters = HashMap::with_hasher(FxBuildHasher::default());

        let res = Environment {
            stack: env_stack,
            root_alignment: BasicLayouter::Center,
            overlay_map: HashMap::with_hasher(FxBuildHasher::default()),
            widget_transfer: HashMap::with_hasher(FxBuildHasher::default()),
            focus_request: None,
            pixel_dimensions,
            scale_factor,
            frame_start_time: Rc::new(RefCell::new(Instant::now())),
            filter_map: filters,
            async_task_queue: Some(vec![]),
            cursor: MouseCursor::Default,
            mouse_position: Default::default(),
            animations: Some(vec![]),
            raw_window_handle: None,
            event_sink,
            animation_widget_in_frame: 0,
            request_application_close: false,
        };


        res
    }

    pub fn mouse_position(&self) -> Position {
        self.mouse_position
    }

    pub fn set_mouse_position(&mut self, position: Position) {
        self.mouse_position = position;
    }

    pub fn close_application(&mut self) {
        self.request_application_close = true;
    }

    pub fn should_close_application(&self) -> bool {
        self.request_application_close
    }

    pub fn root_alignment(&self) -> BasicLayouter {
        self.root_alignment
    }

    pub fn set_root_alignment(&mut self, layout: BasicLayouter) {
        self.root_alignment = layout;
    }

    pub fn insert_animation<A: StateContract>(&mut self, animation: Animation<A>) {
        let poll = move |time: &Instant| {
            let mut animation = animation.clone();
            animation.update(time)
        };
        self.animations
            .as_mut()
            .expect("No animation queue was present.")
            .push(Box::new(poll));
    }

    pub fn update_animation(&mut self) {
        let mut temp = None;
        std::mem::swap(&mut temp, &mut self.animations);

        let instant = &*self.frame_start_time.borrow();

        temp.as_mut()
            .map(|t| t.retain(|update_animation| !update_animation(instant)));

        std::mem::swap(&mut temp, &mut self.animations);
    }

    pub fn has_animations(&self) -> bool {
        self.animations
            .as_ref()
            .map(|a| a.len() > 0)
            .unwrap_or(false)
            || self.animation_widget_in_frame > 0
    }

    /// Check if any async tasks have completed and if so call their continuation.
    pub fn check_tasks(&mut self) {
        let mut temp = None;
        std::mem::swap(&mut temp, &mut self.async_task_queue);

        temp.as_mut().map(|t| t.retain(|task| !task(self)));

        std::mem::swap(&mut temp, &mut self.async_task_queue);

        match (temp, &mut self.async_task_queue) {
            (Some(t), Some(queue)) => {
                queue.extend(t);
            }
            (None, _) => (),
            (_, _) => {
                panic!("The async queue was empty, which is not good.")
            }
        }
    }

    /// Starts a listener where next will be called each time something is sent to the channel.
    /// The sender can be cloned and more values can be sent with it. If true is returned from
    /// next, the stream is closed and will no longer receive values.
    pub fn start_stream<T: Send + 'static>(
        &mut self,
        receiver: std::sync::mpsc::Receiver<T>,
        next: impl Fn(T, &mut Environment) -> bool + 'static,
    ) {

        let poll_message: Box<dyn Fn(&mut Environment) -> bool> = Box::new(move |env| -> bool {
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
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        stop = true;
                    }
                }
            }
            stop
        });

        self.async_task_queue
            .as_mut()
            .expect("No async task queue was present.")
            .push(poll_message);
    }

    #[allow(unused_variables)]
    pub fn spawn_task<T: Send + 'static>(
        &mut self,
        task: impl Future<Output = T> + Send + 'static,
        cont: impl Fn(T, &mut Environment) + 'static,
    ) {
        todo!()
        /*let (sender, receiver) = oneshot::channel();

        let event_sink = self.event_sink.clone();
        let task_with_oneshot = task.then(|message| async move {
            let _ = sender.send(message);
            event_sink.call(CustomEvent::Async);
            ()
        });

        let poll_message: Box<dyn Fn(&mut Environment) -> bool> = Box::new(move |env| -> bool {
            match receiver.try_recv() {
                Ok(message) => {
                    cont(message, env);
                    true
                }
                Err(TryRecvError::Empty) => false,
                Err(e) => {
                    eprintln!("{:?}", e);
                    true
                }
            }
        });


        ASYNC_QUEUE.with(|queue| queue.borrow_mut().push(poll_message));

        spawn(task_with_oneshot)*/
    }

    pub fn capture_time(&mut self) {
        *self.frame_start_time.borrow_mut() = Instant::now();
    }

    pub fn captured_time(&self) -> Rc<RefCell<Instant>> {
        self.frame_start_time.clone()
    }

    pub fn pixel_dimensions(&self) -> Dimension {
        self.pixel_dimensions
    }

    /// This is the dimensions in pixels
    pub fn set_pixel_dimensions(&mut self, dimension: Dimension) {
        self.pixel_dimensions = dimension;
    }

    pub fn set_scale_factor(&mut self, new_scale_factor: f64) {
        self.scale_factor = new_scale_factor;
    }

    /// Get the width in carbide points. (Actual pixels / dpi)
    pub fn current_window_width(&self) -> f64 { self.pixel_dimensions.width / self.scale_factor }

    /// Get the height in carbide points. (Actual pixels / dpi)
    pub fn current_window_height(&self) -> f64 {
        self.pixel_dimensions.height / self.scale_factor
    }

    pub fn scale_factor(&self) -> f64 {
        self.scale_factor
    }

    pub fn cursor(&self) -> MouseCursor {
        self.cursor
    }

    pub fn set_cursor(&mut self, cursor: MouseCursor) {
        self.cursor = cursor;
    }

    /// This method is used to request focus. The focus handling is done after each event is
    /// completed, meaning that this can be called multiple times per frame. The default
    /// behavior is on tab and shift-tab, but you can avoid this by having a widget call
    /// [Self::reset_focus_requests].
    pub fn request_focus(&mut self, request_type: Refocus) {
        self.focus_request = Some(request_type);
    }

    pub fn reset_focus_requests(&mut self) {
        self.focus_request = None;
    }

    pub fn with_overlay_layer<R, F: FnOnce(&mut Environment)->R>(&mut self, id: &'static str, layer: Rc<RefCell<Option<Box<dyn AnyWidget>>>>, f: F) -> R {
        let old = self.overlay_map.insert(id, layer);

        let res = f(self);

        if let Some(old) = old {
            self.overlay_map.insert(id, old);
        } else {
            self.overlay_map.remove(id);
        }

        res
    }

    pub fn add_overlay(&mut self, id: &'static str, widget: Box<dyn AnyWidget>) {
        if let Some(layer) = self.overlay_map.get_mut(&id) {
            *layer.borrow_mut() = Some(widget);
        } else {
            println!("Cannot add an overlay without a layer");
        }
    }

    pub fn contains_overlay(&self, id: &'static str, widget_id: WidgetId) -> bool {
        if let Some(layer) = self.overlay_map.get(&id) {
            layer.borrow().as_ref().map_or(false, |m| m.id() == widget_id)
        } else {
            println!("Cannot add an overlay without a layer");
            false
        }
    }

    pub fn remove_overlay(&mut self, id: &'static str, widget_id: WidgetId) {
        if self.contains_overlay(id, widget_id) {
            if let Some(layer) = self.overlay_map.get_mut(&id) {
                *layer.borrow_mut() = None;
            } else {
                println!("Cannot add an overlay without a layer");
            }
        }
    }

    pub fn transferred_widget(&mut self, id: &Option<String>) -> Option<WidgetTransferAction> {
        self.widget_transfer.remove(id)
    }

    pub fn transfer_widget(&mut self, id: Option<String>, widget_transfer: WidgetTransferAction) {
        self.widget_transfer.insert(id, widget_transfer);
    }

    pub fn clear_animation_frame(&mut self) {
        if self.animation_widget_in_frame > 0 {
            self.animation_widget_in_frame -= 1;
        }
    }

    pub fn number_of_animation_frames(&self) -> usize {
        self.animation_widget_in_frame
    }

    pub fn request_animation_frame(&mut self) {
        self.animation_widget_in_frame = self.animation_widget_in_frame.max(1);
    }

    pub fn request_multiple_animation_frames(&mut self, n: usize) {
        self.animation_widget_in_frame = self.animation_widget_in_frame.max(n);
    }

    pub fn get_global_state<T>(&self) -> InnerState<T> {
        todo!()
    }

    pub fn filters(&self) -> &FxHashMap<FilterId, ImageFilter> {
        &self.filter_map
    }

    pub fn insert_filter(&mut self, filter: ImageFilter) -> FilterId {
        let filter_id = FilterId::next();
        self.filter_map.insert(filter_id, filter);
        filter_id
    }

    pub fn remove_filter(&mut self, id: FilterId) {
        self.filter_map.remove(&id);
    }

    pub fn push(&mut self, key: &'static str, value: Box<dyn Any>) {
        self.stack.push((key, value));
    }

    pub fn pop(&mut self) {
        self.stack.pop();
    }

    pub fn color(&self, color: EnvironmentColor) -> Option<Color> {
        self.value::<EnvironmentColor, Color>(color).cloned()

    }

    pub fn font_size(&self, font_size: EnvironmentFontSize) -> Option<u32> {
        self.value::<EnvironmentFontSize, u32>(font_size).cloned()
    }

    pub fn bool<K: EnvKey>(&self, key: K) -> Option<bool> {
        self.value::<K, bool>(key).cloned()
    }

    pub fn value<K: EnvKey, T: 'static>(&self, key: K) -> Option<&T> {
        let key = key.key();
        for (other_key, value) in self.stack.iter().rev() {
            if key == *other_key && value.is::<T>() {
                return value.downcast_ref();
            }
        }

        None
    }

    pub fn window_handle(&self) -> Option<RawWindowHandle> {
        self.raw_window_handle
    }

    pub fn set_window_handle(&mut self, window_handle: Option<RawWindowHandle>) {
        self.raw_window_handle = window_handle;
    }
}

#[allow(unsafe_code)]
unsafe impl HasRawWindowHandle for Environment {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.raw_window_handle.expect("This can only be called after launch of the application and within handler methods")
    }
}

impl HasEventSink for Environment {
    fn event_sink(&self) -> Box<dyn EventSink> {
        self.event_sink.clone()
    }
}