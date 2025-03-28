use std::ops::Deref;
use std::time::Duration;
use carbide::color::ColorExt;
use carbide::widget::WidgetSync;

use carbide_core::color::BLACK;
use carbide_core::CommonWidgetImpl;
use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::{EnvironmentColor};
use carbide_core::event::{Key, KeyboardEvent, KeyboardEventContext, KeyboardEventHandler, MouseEvent, MouseEventContext, MouseEventHandler};
use carbide_core::layout::{Layout, LayoutContext};
use carbide_core::render::{Render, RenderContext};
use carbide_core::state::{AnyState, IntoReadState, LocalState, Map2, ReadState, ReadStateExtNew, State, StateExtNew, StateSync};
use carbide_core::widget::{AnyWidget, CommonWidget, HSplit, HStack, IfElse, Image, ProgressView, Rectangle, Spacer, VStack, WidgetExt, WidgetId, ZStack};
use carbide_derive::Widget;

use crate::{Video, VideoId};

const ICON_SIZE: f64 = 48.0;
const SKIP_ICON_SIZE: f64 = 32.0;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Layout, Render, MouseEvent, KeyboardEvent)]
pub struct VideoPlayer<Id> where Id: ReadState<T=Option<VideoId>> + Clone {
    #[id] id: WidgetId,
    position: Position,
    dimension: Dimension,

    video: Video<Id>,

    video_overlay: Box<dyn AnyWidget>,
    video_overlay_visible: bool,

    #[state] playing: Box<dyn AnyState<T=bool>>,
    #[state] duration: Box<dyn AnyState<T=Option<Duration>>>,
    #[state] current_time: Box<dyn AnyState<T=Duration>>,
    #[state] buffering: Box<dyn AnyState<T=bool>>,
}

impl VideoPlayer<Option<VideoId>> {
    pub fn new<Id: IntoReadState<Option<VideoId>>>(id: Id) -> VideoPlayer<Id::Output> {

        let duration = LocalState::new(None);
        let current_time = LocalState::new(Duration::new(0, 0));

        let percent_played = Map2::map(
            current_time.clone(),
            duration.clone(),
            |current_time, duration: &Option<Duration>| {
                if let Some(duration) = duration {
                    current_time.as_secs_f64() / duration.as_secs_f64()
                } else {
                    1.0
                }
            }, |new, mut current_time, duration| {
                if let Some(duration) = duration.deref() {
                    *current_time = Duration::from_secs_f64(duration.as_secs_f64() * new);
                }
            }
        );

        let buffering = LocalState::new(false);


        let current_time_forward = current_time.clone();
        let current_time_replay = current_time.clone();

        let playing = LocalState::new(true);
        let playing_play = playing.clone();
        let playing_pause = playing.clone();

        let play_button = Image::new("icons/play-fill.png")
            .scaled_to_fit()
            .on_click(move |_| {
                let mut playing = playing_play.clone();
                playing.set_value(true);
            })
            .frame(ICON_SIZE, ICON_SIZE);

        let pause_button = Image::new("icons/pause-fill.png")
            .scaled_to_fit()
            .on_click(move |_| {
                let mut playing = playing_pause.clone();
                playing.set_value(false);
            })
            .frame(ICON_SIZE, ICON_SIZE);

        let forward_button = Image::new("icons/forward-10-fill.png")
            .scaled_to_fit()
            .on_click(move |_| {
                let mut current_time = current_time_forward.clone();
                let current = *current_time.value();
                current_time.set_value(current + Duration::new(10, 0));
            })
            .frame(SKIP_ICON_SIZE, SKIP_ICON_SIZE);

        let replay_button = Image::new("icons/replay-10-fill.png")
            .scaled_to_fit()
            .on_click(move |_| {
                let mut current_time = current_time_replay.clone();
                let current = *current_time.value();
                if current >= Duration::new(10, 0) {
                    current_time.set_value(current - Duration::new(10, 0));
                } else {
                    current_time.set_value(Duration::new(0, 0));
                }
            })
            .frame(SKIP_ICON_SIZE, SKIP_ICON_SIZE);

        let video_overlay = ZStack::new((
            Rectangle::new().fill(BLACK.with_opacity(0.3)),
            HStack::new((
                Spacer::new(),
                replay_button,
                Spacer::new(),
                IfElse::new(buffering.clone())
                    .when_true(ProgressView::new().size(ICON_SIZE))
                    .when_false(
                        IfElse::new(playing.clone())
                        .when_true(pause_button)
                        .when_false(play_button)
                    ),
                Spacer::new(),
                forward_button,
                Spacer::new(),
            )),
            VStack::new((
                Spacer::new(),
                HSplit::new(
                    Rectangle::new()
                        .fill(EnvironmentColor::Accent)
                        .frame_fixed_height(4.0),
                    Rectangle::new()
                        .fill(EnvironmentColor::OpaqueSeparator)
                        .frame_fixed_height(4.0)
                ).percent(percent_played)
                    //.non_draggable()
            ))
        )).boxed();


        VideoPlayer {
            id: Default::default(),
            position: Default::default(),
            dimension: Default::default(),
            video: Video::new(id)
                .playing(playing.clone())
                .current_time(current_time.clone())
                .duration(duration.clone())
                .buffering(buffering.clone())
                .scaled_to_fit(),
            video_overlay,
            video_overlay_visible: false,
            playing: playing.as_dyn(),
            duration: duration.as_dyn(),
            current_time: current_time.as_dyn(),
            buffering: buffering.as_dyn(),
        }
    }
}

impl<Id: ReadState<T=Option<VideoId>> + Clone> MouseEventHandler for VideoPlayer<Id> {
    fn handle_mouse_event(&mut self, event: &MouseEvent, ctx: &mut MouseEventContext) {
        if !matches!(event, MouseEvent::Left | MouseEvent::Entered) {
            self.video_overlay_visible = self.is_inside(event.get_current_mouse_position());
        }
    }

    fn process_mouse_event(&mut self, event: &MouseEvent, ctx: &mut MouseEventContext) {
        if *ctx.is_current {
            if !*ctx.consumed {
                self.sync(ctx.env);
                self.handle_mouse_event(event, ctx);
            }
        }

        self.video_overlay.process_mouse_event(event, ctx);
    }
}

impl<Id: ReadState<T=Option<VideoId>> + Clone> KeyboardEventHandler for VideoPlayer<Id> {
    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, ctx: &mut KeyboardEventContext) {
        match event {
            KeyboardEvent::Press { key: Key::ArrowLeft, .. } => {
                let current = *self.current_time.value();
                if current >= Duration::new(10, 0) {
                    self.current_time.set_value(current - Duration::new(10, 0));
                } else {
                    self.current_time.set_value(Duration::new(0, 0));
                }
            }
            KeyboardEvent::Press { key: Key::ArrowRight, .. } => {
                let current = *self.current_time.value();
                self.current_time.set_value(current + Duration::new(10, 0));
            }
            _ => ()
        }
    }
}


impl<Id: ReadState<T=Option<VideoId>> + Clone> Layout for VideoPlayer<Id> {
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
        let res = self.video.calculate_size(requested_size, ctx);
        self.video_overlay.calculate_size(res, ctx);

        self.dimension = res;
        res
    }

    fn position_children(&mut self, ctx: &mut LayoutContext) {
        let alignment = self.alignment();
        let position = self.position();
        let dimension = self.dimension();

        self.video.set_position(alignment.position(position, dimension, self.video.dimension()));
        self.video_overlay.set_position(alignment.position(position, dimension, self.video_overlay.dimension()));

        self.video.position_children(ctx);
        self.video_overlay.position_children(ctx);
    }
}

impl<Id: ReadState<T=Option<VideoId>> + Clone> Render for VideoPlayer<Id> {
    fn render(&mut self, context: &mut RenderContext) {
        self.sync(context.env);
        self.video.render(context);

        if self.video_overlay_visible || *self.buffering.value() {
            self.video_overlay.render(context);
        }
    }
}

impl<Id: ReadState<T=Option<VideoId>> + Clone> CommonWidget for VideoPlayer<Id> {
    CommonWidgetImpl!(self, child: self.video, position: self.position, dimension: self.dimension, flexibility: 10);
}