use std::slice::from_raw_parts;
use std::str::from_utf8;
use std::time::Duration;
use bitflags::Flags;
use cocoa::base::{id, nil};
use cocoa::foundation::{NSArray, NSAutoreleasePool, NSDictionary, NSInteger, NSString, NSUInteger};
use objc::{msg_send, class, sel, sel_impl};
use objc::runtime::BOOL;
use carbide::layout::LayoutContext;
use carbide_core::CommonWidgetImpl;
use carbide_core::draw::ImageId;
use carbide_core::draw::{Dimension, Position, Rect, Scalar, Texture, TextureFormat};
use carbide_core::environment::Environment;
use carbide_core::event::{CustomEvent, HasEventSink, Key, KeyboardEvent, KeyboardEventHandler};
use carbide_core::layout::Layout;
use carbide_core::draw::MODE_IMAGE;
use carbide_core::render::{Render, RenderContext};
use carbide_core::state::{AnyReadState, AnyState, IntoReadState, IntoState, LocalState, ReadState, ReadStateExtNew, State, StateExtNew};
use carbide_core::widget::{CommonWidget, WidgetExt, WidgetId, AnyWidget, ScaleMode};
use carbide_derive::Widget;
use crate::{CMTime, CMTimeFlags, CVPixelBufferGetBaseAddress, CVPixelBufferGetBytesPerRow, CVPixelBufferGetDataSize, CVPixelBufferGetHeight, CVPixelBufferGetPixelFormatType, CVPixelBufferGetWidth, CVPixelBufferLockBaseAddress, CVPixelBufferUnlockBaseAddress, kCMTimeIndefinite, kCVPixelBufferPixelFormatTypeKey, NSKeyValueChangeKey};
use crate::macos::{listener, NSKeyValueObservingOptions};

pub type VideoId = ImageId;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render, Layout)]
pub struct Video<Id> where Id: ReadState<T=Option<ImageId>> + Clone {
    id: WidgetId,
    position: Position,
    dimension: Dimension,

    /// The unique identifier for the image that will be drawn.
    #[state] video_id_state: Id,
    video_id: Option<VideoId>,
    #[state] playing: Box<dyn AnyReadState<T=bool>>,
    #[state] rate: Box<dyn AnyReadState<T=f64>>,
    #[state] volume: Box<dyn AnyReadState<T=f64>>,
    #[state] muted: Box<dyn AnyReadState<T=bool>>,
    #[state] duration: Box<dyn AnyState<T=Option<Duration>>>,
    #[state] current_time: Box<dyn AnyState<T=Duration>>,
    last_current_time: Duration,
    #[state] buffering: Box<dyn AnyState<T=bool>>,
    scale_mode: ScaleMode,
    resizeable: bool,

    // MacOS platform specific fields
    player: Option<NativePlayer>,
}

#[derive(Debug, Clone)]
struct NativePlayer {
    player: id,
    player_item: id,
    player_output: id,
    listener: id,
}

impl NativePlayer {
    fn new(player: id, player_item: id, player_output: id, listener: id) -> Self {
        // https://stackoverflow.com/a/43148508
        unsafe {
            let _: () = msg_send![player, retain];
            let _: () = msg_send![player_item, retain];
            let _: () = msg_send![player_output, retain];
            let _: () = msg_send![listener, retain];
        }

        NativePlayer {
            player,
            player_item,
            player_output,
            listener,
        }
    }
}

impl Drop for NativePlayer {
    fn drop(&mut self) {
        unsafe {
            let _: () = msg_send![self.player, release];
            let _: () = msg_send![self.player_item, release];
            let _: () = msg_send![self.player_output, release];
            let _: () = msg_send![self.listener, release];
        }
    }
}

impl Video<Option<VideoId>> {
    pub fn new<Id: IntoReadState<Option<VideoId>>>(id: Id) -> Video<Id::Output> {
        Video {
            id: WidgetId::new(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(0.0, 0.0),
            video_id_state: id.into_read_state(),
            video_id: None,
            playing: LocalState::new(true).as_dyn_read(),
            rate: LocalState::new(1.0).as_dyn_read(),
            volume: LocalState::new(1.0).as_dyn_read(),
            muted: LocalState::new(false).as_dyn_read(),

            duration: LocalState::new(None).as_dyn(),
            current_time: LocalState::new(Duration::new(0, 0)).as_dyn(),
            last_current_time: Duration::new(0, 0),
            buffering: LocalState::new(false).as_dyn(),
            scale_mode: ScaleMode::Fit,
            resizeable: false,
            player: None,
        }
    }
}

impl<Id: ReadState<T=Option<ImageId>> + Clone> Layout for Video<Id> {
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {

        if &*self.video_id_state.value() != &self.video_id {
            self.change_video(ctx.env);
        }

        self.update_rate();
        self.update_volume();
        self.update_muted();
        self.update_current_time();
        self.update_frame(ctx);


        let information = self.video_id.as_ref().and_then(|id| {
            let (width, height) = ctx.image.texture_dimensions(id)?;
            Some(Dimension::new(width as Scalar, height as Scalar) / ctx.env.scale_factor())
        }).unwrap_or(Dimension::new(100.0, 100.0));

        if !self.resizeable {
            self.dimension = information;
        } else {
            let width_factor = requested_size.width / information.width;
            let height_factor = requested_size.height / information.height;

            match self.scale_mode {
                ScaleMode::Fit => {
                    let scale_factor = width_factor.min(height_factor);

                    self.dimension = Dimension::new(
                        information.width * scale_factor,
                        information.height * scale_factor,
                    )
                }
                ScaleMode::Fill => {
                    let scale_factor = width_factor.max(height_factor);

                    self.dimension = Dimension::new(
                        information.width * scale_factor,
                        information.height * scale_factor,
                    )
                }
                ScaleMode::Stretch => self.dimension = requested_size,
            }
        }

        self.dimension
    }
}

impl<Id: ReadState<T=Option<ImageId>> + Clone> Video<Id> {
    pub fn playing<P: IntoReadState<bool>>(mut self, playing: P) -> Self {
        self.playing = playing.into_read_state().as_dyn_read();
        self
    }

    pub fn rate<P: IntoReadState<f64>>(mut self, rate: P) -> Self {
        self.rate = rate.into_read_state().as_dyn_read();
        self
    }

    pub fn volume<P: IntoReadState<f64>>(mut self, volume: P) -> Self {
        self.volume = volume.into_read_state().as_dyn_read();
        self
    }

    pub fn muted<P: IntoReadState<bool>>(mut self, muted: P) -> Self {
        self.muted = muted.into_read_state().as_dyn_read();
        self
    }

    pub fn duration<P: IntoState<Option<Duration>>>(mut self, duration: P) -> Self {
        self.duration = duration.into_state().as_dyn();
        self
    }

    pub fn buffering<P: IntoState<bool>>(mut self, buffering: P) -> Self {
        self.buffering = buffering.into_state().as_dyn();
        self
    }

    pub fn current_time<P: IntoState<Duration>>(mut self, current_time: P) -> Self {
        self.current_time = current_time.into_state().as_dyn();
        self
    }

    pub fn resizeable(mut self) -> Self {
        self.resizeable = true;
        self
    }

    pub fn scaled_to_fit(mut self) -> Self {
        self.resizeable = true;
        self.scale_mode = ScaleMode::Fit;
        self
    }

    pub fn scaled_to_fill(mut self) -> Self {
        self.resizeable = true;
        self.scale_mode = ScaleMode::Fill;
        self
    }

    fn update_rate(&mut self) {
        if let Some(player) = &self.player {
            let playing = *self.playing.value();
            let rate = *self.rate.value() as f32;
            unsafe {
                if playing {
                    let _: () = msg_send![player.player, setRate:rate];
                } else {
                    let _: () = msg_send![player.player, setRate:0.0f32];
                }
            }
        }
    }

    fn update_volume(&mut self) {
        if let Some(player) = &self.player {
            let volume = *self.volume.value() as f32;
            unsafe {
                let _: () = msg_send![player.player, setVolume:volume];
            }
        }
    }

    fn update_muted(&mut self) {
        if let Some(player) = &self.player {
            let muted = *self.muted.value();
            unsafe {
                let _: () = msg_send![player.player, setMuted:muted];
            }
        }
    }

    fn update_current_time(&mut self) {
        if let Some(player) = &self.player {
            let mut current_time = *self.current_time.value();

            if let Some(duration) = *self.duration.value() {
                current_time = current_time.min(duration);
            }

            if self.last_current_time != current_time {
                self.last_current_time = current_time;
                let time = CMTime::from(current_time);

                unsafe {
                    let _: () = msg_send![player.player, seekToTime:time];
                }
            }
        }
    }

    fn update_frame(&mut self, ctx: &mut LayoutContext) {
        if let Some(player) = &self.player {
            unsafe {
                let rate: f32 = msg_send![player.player, rate];
                if rate != 0.0 {
                    ctx.env.request_animation_frame();
                }

                let current_time: CMTime = msg_send![player.player, currentTime];
                self.current_time.set_value(Duration::from(current_time));
                self.last_current_time = Duration::from(current_time);
                if msg_send![player.player_output, hasNewPixelBufferForItemTime:current_time] {
                    let buffer: id = msg_send![player.player_output, copyPixelBufferForItemTime:current_time itemTimeForDisplay:nil];

                    let width = CVPixelBufferGetWidth(buffer);
                    let height = CVPixelBufferGetHeight(buffer);
                    let bytes_per_row = CVPixelBufferGetBytesPerRow(buffer);
                    //println!("Buffer width: {}", width);
                    //println!("Buffer height: {}", height);

                    //println!("Data size: {}", CVPixelBufferGetDataSize(buffer));
                    //println!("Bytes per row: {}", bytes_per_row);
                    //println!("Pixel format type: {:?}", from_utf8(&CVPixelBufferGetPixelFormatType(buffer)));

                    let lockCode = CVPixelBufferLockBaseAddress(buffer, 0);
                    //println!("lockCode: {:?}", lockCode);
                    let address = CVPixelBufferGetBaseAddress(buffer);
                    //println!("address: {:?}", address);

                    ctx.image.update_texture(self.video_id.clone().unwrap(), Texture {
                        width: width as u32,
                        height: height as u32,
                        bytes_per_row: bytes_per_row as u32,
                        format: TextureFormat::BGRA8,
                        data: from_raw_parts(address, width * height * 4),
                    });

                    let lockCode = CVPixelBufferUnlockBaseAddress(buffer, 0);
                }
            }
        }
    }

    fn change_video(&mut self, env: &mut Environment) {
        println!("Change video to: {:?}", &*self.video_id_state.value());

        // add observer
        // [object_to_observe addObserver:observer forKeyPath:string options:o context:nil]

        if let Some(image_id) = &*self.video_id_state.value() {
            let path = image_id.as_ref();

            unsafe {
                let string: id = NSString::alloc(nil).init_str(path.as_os_str().to_str().unwrap()).autorelease();
                let url: id = if path.is_file() {
                    msg_send![class!(NSURL), fileURLWithPath:string]
                } else {
                    msg_send![class!(NSURL), URLWithString:string]
                };

                let player_item: id = msg_send![class!(AVPlayerItem), playerItemWithURL:url];
                let player: id = msg_send![class!(AVPlayer), playerWithPlayerItem:player_item];
                let player_output: id = msg_send![class!(AVPlayerItemVideoOutput), alloc];

                let duration = self.duration.clone();
                let buffering = self.buffering.clone();
                let sink = env.event_sink();
                let listener = listener(move |key_path, map| {
                    let mut duration = duration.clone();
                    let mut buffering = buffering.clone();
                    //println!("KeyPath: {key_path}, Map: {map:?}");

                    match key_path.as_str() {
                        "status" => {
                            if let Some(i) = map.get(&NSKeyValueChangeKey::New) {
                                let val: i64 = msg_send![*i, integerValue];
                                if val == 2 {
                                    dbg!(val);
                                    let error: id = msg_send![player_item, error];
                                    dbg!(error);
                                    dbg!(error == nil);

                                    let error_msg: id = msg_send![error, localizedDescription];

                                    let error_msg = unsafe {
                                        let slice = std::slice::from_raw_parts(error_msg.UTF8String() as *const _, error_msg.len());
                                        let result = std::str::from_utf8_unchecked(slice);
                                        result.to_string()
                                    };

                                    println!("{:?}", error_msg);
                                }

                                let item_duration: CMTime = msg_send![player_item, duration];
                                if !item_duration.flags.contains(CMTimeFlags::Indefinite) {
                                    duration.set_value(Some(Duration::from(item_duration)));
                                }
                                println!("Duration: {:?}", duration.value());
                                sink.send(CustomEvent::Async);
                            }
                        }
                        "playbackBufferEmpty" => {
                            if !*buffering.value() {
                                buffering.set_value(true);
                                sink.send(CustomEvent::Async);
                            }
                        }
                        "playbackLikelyToKeepUp" => {
                            if *buffering.value() {
                                buffering.set_value(false);
                                sink.send(CustomEvent::Async);
                            }
                        }
                        "playbackBufferFull" => {
                            if *buffering.value() {
                                buffering.set_value(false);
                                sink.send(CustomEvent::Async);
                            }
                        }
                        _ => unreachable!()
                    }
                });

                let keyPath: id = NSString::alloc(nil).init_str("status").autorelease();
                let keyPath2: id = NSString::alloc(nil).init_str("playbackBufferEmpty").autorelease();
                let keyPath3: id = NSString::alloc(nil).init_str("playbackLikelyToKeepUp").autorelease();
                let keyPath4: id = NSString::alloc(nil).init_str("playbackBufferFull").autorelease();

                let _: () = msg_send![
                        player_item,
                        addObserver:listener
                        forKeyPath:keyPath
                        options:NSKeyValueObservingOptions::NSKeyValueObservingOptionNew
                        context:nil
                    ];

                let _: () = msg_send![
                        player_item,
                        addObserver:listener
                        forKeyPath:keyPath2
                        options:NSKeyValueObservingOptions::NSKeyValueObservingOptionNew
                        context:nil
                    ];

                let _: () = msg_send![
                        player_item,
                        addObserver:listener
                        forKeyPath:keyPath3
                        options:NSKeyValueObservingOptions::NSKeyValueObservingOptionNew
                        context:nil
                    ];

                let _: () = msg_send![
                        player_item,
                        addObserver:listener
                        forKeyPath:keyPath4
                        options:NSKeyValueObservingOptions::NSKeyValueObservingOptionNew
                        context:nil
                    ];

                let keys = vec![kCVPixelBufferPixelFormatTypeKey];
                let int: ::std::os::raw::c_uint = 1111970369; // Magic number for BGRA format
                let number: id = msg_send![class!(NSNumber), numberWithUnsignedInt:int];
                let objects = vec![number];

                let keys_array = NSArray::arrayWithObjects(nil, &keys);
                let objs_array = NSArray::arrayWithObjects(nil, &objects);

                let dict: id = NSDictionary::dictionaryWithObjects_forKeys_(nil, objs_array, keys_array);

                let _: () = msg_send![player_output, initWithPixelBufferAttributes:dict];
                let _: () = msg_send![player_item, addOutput:player_output];


                self.buffering.set_value(true);
                self.duration.set_value(None);
                self.current_time.set_value(Duration::new(0, 0));
                self.last_current_time = Duration::new(0, 0);
                self.player = Some(NativePlayer::new(player, player_item, player_output, listener))
            }
        }

        self.video_id = self.video_id_state.value().clone();
    }
}

impl<Id: ReadState<T=Option<ImageId>> + Clone> Render for Video<Id> {
    fn render(&mut self, context: &mut RenderContext, env: &mut Environment) {
        if let Some(id) = &self.video_id {
            if context.image.texture_exist(id) {
                context.image(
                    id.clone(),
                    Rect::new(self.position, self.dimension),
                    Rect::from_corners(Position::new(0.0, 1.0), Position::new(1.0, 0.0)),
                    MODE_IMAGE
                );
            }
        }
    }
}

impl<Id: ReadState<T=Option<ImageId>> + Clone> CommonWidget for Video<Id> {
    CommonWidgetImpl!(self, id: self.id, child: (), position: self.position, dimension: self.dimension, flexibility: 10);
}

impl<Id: ReadState<T=Option<ImageId>> + Clone> WidgetExt for Video<Id> {}


// AVPlayer
// timeControlStatus - A value that indicates whether playback is in progress, paused indefinitely, or waiting for network conditions to improve.
// play - Begins playback of the current item.
// pause - Pauses playback of the current item.
// rate - 1.0 is normal play rate.
// seekToTime - move to a specific time
// volume - The audio playback volume for the player.
// muted - A Boolean value that indicates whether the audio output of the player is muted

// AVPlayerItem
// canPlayReverse - A Boolean value that indicates whether the item can play in reverse
// canPlayFastForward - A Boolean value that indicates whether the item can be fast forwarded
// canPlayFastReverse - A Boolean value that indicates whether the item can be quickly reversed
// canPlaySlowForward - A Boolean value that indicates whether the item can play slower than normal
// canPlaySlowReverse - A Boolean value that indicates whether the item can play slowly backward
// duration - The duration of the item.
// currentTime - Returns the current time of the item.
// timebase - The timebase information for the item