use std::cell::RefCell;
use std::sync::Arc;
use crate::color_dialog::style::ColorDialogStyle;
use carbide::color::{Color, ColorExt, RED};
use carbide::environment::Environment;
use carbide::event::{CoreEvent, EventSink, NoopEventSink};
use carbide::state::{AnyReadState, AnyState, LocalState, ReadState, State, StateExtNew, StateSync};
use objc2::rc::Retained;
use objc2::runtime::NSObject;
use objc2::ClassType;
use objc2::DeclaredClass;
use objc2::{declare_class, msg_send_id, mutability, sel};
use objc2_app_kit::{NSColor, NSColorPanel, NSColorSpace};
use objc2_foundation::{CGFloat, MainThreadMarker};

thread_local! {
    static RECEIVER: Retained<ColorDialogReceiver> = ColorDialogReceiver::new();
    static COLOR: RefCell<Box<dyn AnyState<T=Color>>> = RefCell::new(LocalState::new(RED).as_dyn());
    static SINK: RefCell<Arc<dyn EventSink>> = RefCell::new(Arc::new(NoopEventSink));
}

#[derive(Copy, Clone, Debug)]
pub struct MacOSNativeColorDialogStyle;

impl ColorDialogStyle for MacOSNativeColorDialogStyle {
    fn open(&self, mut color: Box<dyn AnyState<T=Color>>, mut show_alpha: Box<dyn AnyReadState<T=bool>>, env: &mut Environment) {
        let main_thead_marker = MainThreadMarker::new().expect("To be run in the main thread");

        color.sync(env);
        show_alpha.sync(env);

        let initial_color = *color.value();

        let initial_color = unsafe {
            NSColor::colorWithRed_green_blue_alpha(
                initial_color.red() as CGFloat,
                initial_color.green() as CGFloat,
                initial_color.blue() as CGFloat,
                initial_color.opacity() as CGFloat
            )
        };

        COLOR.set(color);

        let panel = unsafe { NSColorPanel::sharedColorPanel(main_thead_marker) };

        //panel.close();
        unsafe { panel.setColor(&initial_color) };
        unsafe { panel.setContinuous(true) };
        unsafe { panel.setShowsAlpha(*show_alpha.value()) };
        unsafe { panel.setAction(Some(sel!(colorChanged:))) };

        let sink = env.get::<dyn EventSink>()
            .expect("Event sink is required to open this color dialog")
            .clone();

        SINK.set(sink);

        RECEIVER.with(|receiver| {
            unsafe { panel.setTarget(Some(&receiver)); }
        });

        panel.orderFront(None);
    }
}

declare_class!(
    pub struct ColorDialogReceiver;
    // SAFETY:
    // - The superclass NSObject does not have any subclassing requirements.
    // - InteriorMutable is a safe default.
    // - `ColorDialogReceiver` does not implement `Drop`.
    unsafe impl ClassType for ColorDialogReceiver {
        type Super = NSObject;
        type Mutability = mutability::InteriorMutable;
        const NAME: &'static str = "ColorDialogReceiver";
    }

    impl DeclaredClass for ColorDialogReceiver {}

    unsafe impl ColorDialogReceiver {
        #[method(colorChanged:)]
        fn __color_changed(&self, sender: &NSColorPanel) {
            let color = unsafe { sender.color() };
            ColorDialogReceiver::color_changed(&color);
        }
    }
);

impl ColorDialogReceiver {
    pub fn new() -> Retained<Self> {
        let allocated_instance = ColorDialogReceiver::alloc();
        unsafe { msg_send_id![allocated_instance, init] }
    }

    fn color_changed(color: &NSColor) {
        println!("Called: {:?}", color);
        let converted_color = unsafe { color.colorUsingColorSpace(&NSColorSpace::sRGBColorSpace()) }
            .expect("The color to be representable in generic RGB color space");

        COLOR.with_borrow_mut(|state| unsafe {
            state.set_value(Color::Rgba(
                converted_color.redComponent() as f32,
                converted_color.greenComponent() as f32,
                converted_color.blueComponent() as f32,
                converted_color.alphaComponent() as f32
            ))
        });
        SINK.with_borrow(|sink| {
            sink.send(CoreEvent::AsyncStream);
        })
    }
}