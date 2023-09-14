use std::slice::from_raw_parts;
use std::str::from_utf8;
use cocoa::base::{id, nil};
use cocoa::foundation::{NSArray, NSAutoreleasePool, NSDictionary, NSInteger, NSString};
use objc::{msg_send, class, sel, sel_impl};
use objc::runtime::BOOL;
use carbide_core::CommonWidgetImpl;
use carbide_core::draw::image::ImageId;
use carbide_core::draw::{Dimension, Position, Rect, Scalar, Texture, TextureFormat};
use carbide_core::environment::Environment;
use carbide_core::layout::Layout;
use carbide_core::mesh::MODE_IMAGE;
use carbide_core::render::{Render, RenderContext};
use carbide_core::state::{IntoReadState, ReadState};
use carbide_core::widget::{CommonWidget, WidgetExt, WidgetId, Widget, ScaleMode};
use crate::{CMTime, CVPixelBufferGetBaseAddress, CVPixelBufferGetBytesPerRow, CVPixelBufferGetDataSize, CVPixelBufferGetHeight, CVPixelBufferGetPixelFormatType, CVPixelBufferGetWidth, CVPixelBufferLockBaseAddress, CVPixelBufferUnlockBaseAddress, kCVPixelBufferPixelFormatTypeKey};

pub type VideoId = ImageId;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render, Layout)]
pub struct Video<Id> where Id: ReadState<T=Option<ImageId>> + Clone {
    id: WidgetId,
    position: Position,
    dimension: Dimension,

    /// The unique identifier for the image that will be drawn.
    #[state]
    pub video_id_state: Id,
    pub video_id: Option<VideoId>,
    scale_mode: ScaleMode,
    resizeable: bool,

    // MacOS platform specific fields
    pub player: id,
    pub player_item: id,
    pub player_output: id,
}

impl Video<Option<VideoId>> {
    pub fn new<Id: IntoReadState<Option<VideoId>>>(id: Id) -> Video<Id::Output> {
        Video {
            id: WidgetId::new(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(0.0, 0.0),
            video_id_state: id.into_read_state(),
            video_id: None,
            scale_mode: ScaleMode::Fit,
            resizeable: false,
            player: nil,
            player_item: nil,
            player_output: nil,
        }
    }
}

impl<Id: ReadState<T=Option<ImageId>> + Clone> Layout for Video<Id> {
    fn calculate_size(&mut self, requested_size: Dimension, env: &mut Environment) -> Dimension {

        if &*self.video_id_state.value() != &self.video_id {
            println!("Change video to: {:?}", &*self.video_id_state.value());

            if let Some(image_id) = &*self.video_id_state.value() {
                let path = image_id.as_ref();
                unsafe {
                    let string: id = NSString::alloc(nil).init_str(path.as_os_str().to_str().unwrap()).autorelease();
                    let url: id = msg_send![class!(NSURL), fileURLWithPath:string];

                    /*let filemanager: id = msg_send![class!(NSFileManager), defaultManager];
                    let accessible: BOOL = msg_send![filemanager, isReadableFileAtPath:url];
                    let isFile: BOOL = msg_send![url, isFileURL];

                    println!("Is accessible: {}", accessible);
                    println!("isFile: {}", isFile);*/

                    self.player_item = msg_send![class!(AVPlayerItem), playerItemWithURL:url];
                    self.player = msg_send![class!(AVPlayer), playerWithPlayerItem:self.player_item];
                    //self.player = msg_send![class!(AVPlayer), playerWithURL:url];
                    self.player_output = msg_send![class!(AVPlayerItemVideoOutput), alloc];

                    let keys = vec![kCVPixelBufferPixelFormatTypeKey];
                    let int: ::std::os::raw::c_uint = 1111970369;
                    let number: id = msg_send![class!(NSNumber), numberWithUnsignedInt:int];
                    let objects = vec![number];

                    let keys_array = NSArray::arrayWithObjects(nil, &keys);
                    let objs_array = NSArray::arrayWithObjects(nil, &objects);

                    let dict: id = NSDictionary::dictionaryWithObjects_forKeys_(nil, objs_array, keys_array);

                    let _: () = msg_send![self.player_output, initWithPixelBufferAttributes:dict];
                    let _: () = msg_send![self.player_item, addOutput: self.player_output];

                    //let status: NSInteger = msg_send![self.player, status];
                    //println!("Player Status: {:?}", status);

                    let _: () = msg_send![self.player, play];
                    //println!("Play");

                    //let status: NSInteger = msg_send![self.player, status];
                    //println!("Player Status: {:?}", status);

                    //let status: NSInteger = msg_send![self.player_item, status];
                    //let isMuted: BOOL = msg_send![self.player, isMuted];
                    //let isExternalPlaybackActive: BOOL = msg_send![self.player, isExternalPlaybackActive];
                    //let volume: f32 = msg_send![self.player, volume];
                    //println!("Player item status: {:?}", status);
                    //println!("isMuted: {:?}", isMuted);
                    //println!("isExternalPlaybackActive: {:?}", isExternalPlaybackActive);
                    //println!("volume: {:?}", volume);

                    // https://stackoverflow.com/a/43148508
                    let _: () = msg_send![self.player, retain];
                    let _: () = msg_send![self.player_item, retain];
                    let _: () = msg_send![self.player_output, retain];
                }
            } else {
                if self.player != nil {
                    unsafe {
                        let _: () = msg_send![self.player, release];
                        let _: () = msg_send![self.player_item, release];
                        let _: () = msg_send![self.player_output, release];
                    }
                }
                println!("All set to nil");
                self.player = nil;
                self.player_item = nil;
                self.player_output = nil;
            }

            self.video_id = self.video_id_state.value().clone();
        }

        let information = self.video_id.as_ref().and_then(|id| {
            let (width, height) = env.image_context.texture_dimensions(id)?;
            Some(Dimension::new(width as Scalar, height as Scalar))
        }).unwrap_or(self.dimension);

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

impl<Id: ReadState<T=Option<ImageId>> + Clone> Render for Video<Id> {
    fn render(&mut self, context: &mut RenderContext, env: &mut Environment) {
        if self.player_output != nil {
            unsafe {
                let rate: f32 = msg_send![self.player, rate];
                //println!("Rate: {:?}", rate);
                if rate != 0.0 {
                    env.request_animation_frame();
                }

                let current_time: CMTime = msg_send![self.player, currentTime];
                if msg_send![self.player_output, hasNewPixelBufferForItemTime:current_time] {
                    //println!("Yes please");
                    let buffer: id = msg_send![self.player_output, copyPixelBufferForItemTime:current_time itemTimeForDisplay:nil];

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

                    env.image_context.update_texture(self.video_id.clone().unwrap(), Texture {
                        width: width as u32,
                        height: height as u32,
                        bytes_per_row: bytes_per_row as u32,
                        format: TextureFormat::BGRA8,
                        data: from_raw_parts(address, width*height*4),
                    });

                    //let slice = Vec::from_raw_parts(address, width*height*4, width*height*4);
                    //println!("Slice: {:?}", &slice[..100]);
                    //println!("len: {}", slice.len());
                    //let mut image = RgbaImage::from_raw(width as u32, height as u32, slice).unwrap();
                    //image.save("test.png").unwrap();
                    let lockCode = CVPixelBufferUnlockBaseAddress(buffer, 0);
                }
            }
        }

        if let Some(id) = &self.video_id {
            if env.image_context.texture_exist(id) {
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
    CommonWidgetImpl!(self, id: self.id, position: self.position, dimension: self.dimension, flexibility: 10);
}

impl<Id: ReadState<T=Option<ImageId>> + Clone> WidgetExt for Video<Id> {}
