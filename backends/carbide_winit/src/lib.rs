//! A function for converting a `winit::Event` to a `carbide::event::Input`.
use winit::dpi::{LogicalPosition, LogicalSize};
use winit::event::{
    ElementState,
    MouseButton as WinitMouseButton,
    MouseScrollDelta,
    Touch as WinitTouch,
    TouchPhase as WinitTouchPhase,
    WindowEvent
};
use winit::keyboard::{Key as WinitKey, NamedKey};
use winit::window::CursorIcon;

use carbide_core::cursor::MouseCursor;
use carbide_core::draw::Position;
use carbide_core::event::{Button, Gesture, Input, Key, Motion, MouseButton, Touch, TouchId, TouchPhase};
pub use custom_event_loop::*;

pub use winit::*;

mod custom_event_loop;

const ARBITRARY_POINTS_PER_LINE_FACTOR: f64 = 10.0;

/// Types that have access to a `winit::Window` and can provide the necessary dimensions and hidpi
/// factor for converting `winit::Event`s to `carbide::event::Input`, as well as set the mouse
/// cursor.
///
/// This allows users to pass references to window render like `glium::Display`,
/// `glium::glutin::Window` or `winit::Window`
pub trait WinitWindow {
    /// Return the inner size of the window in logical pixels.
    fn get_inner_size(&self) -> Option<(u32, u32)>;
    /// Return the window's DPI factor so that we can convert from pixel values to scalar values.
    fn hidpi_factor(&self) -> f32;
}

fn convert_named_key(named: &NamedKey) -> Key {
    match named {
        NamedKey::Alt => Key::Alt,
        NamedKey::AltGraph => Key::AltGraph,
        NamedKey::CapsLock => Key::CapsLock,
        NamedKey::Control => Key::Control,
        NamedKey::Fn => Key::Fn,
        NamedKey::FnLock => Key::FnLock,
        NamedKey::NumLock => Key::NumLock,
        NamedKey::ScrollLock => Key::ScrollLock,
        NamedKey::Shift => Key::Shift,
        NamedKey::Symbol => Key::Symbol,
        NamedKey::SymbolLock => Key::SymbolLock,
        NamedKey::Meta => Key::Meta,
        NamedKey::Hyper => Key::Hyper,
        NamedKey::Super => Key::Super,
        NamedKey::Enter => Key::Enter,
        NamedKey::Tab => Key::Tab,
        NamedKey::Space => Key::Space,
        NamedKey::ArrowDown => Key::ArrowDown,
        NamedKey::ArrowLeft => Key::ArrowLeft,
        NamedKey::ArrowRight => Key::ArrowRight,
        NamedKey::ArrowUp => Key::ArrowUp,
        NamedKey::End => Key::End,
        NamedKey::Home => Key::Home,
        NamedKey::PageDown => Key::PageDown,
        NamedKey::PageUp => Key::PageUp,
        NamedKey::Backspace => Key::Backspace,
        NamedKey::Clear => Key::Clear,
        NamedKey::Copy => Key::Copy,
        NamedKey::CrSel => Key::CrSel,
        NamedKey::Cut => Key::Cut,
        NamedKey::Delete => Key::Delete,
        NamedKey::EraseEof => Key::EraseEof,
        NamedKey::ExSel => Key::ExSel,
        NamedKey::Insert => Key::Insert,
        NamedKey::Paste => Key::Paste,
        NamedKey::Redo => Key::Redo,
        NamedKey::Undo => Key::Undo,
        NamedKey::Accept => Key::Accept,
        NamedKey::Again => Key::Again,
        NamedKey::Attn => Key::Attn,
        NamedKey::Cancel => Key::Cancel,
        NamedKey::ContextMenu => Key::ContextMenu,
        NamedKey::Escape => Key::Escape,
        NamedKey::Execute => Key::Execute,
        NamedKey::Find => Key::Find,
        NamedKey::Help => Key::Help,
        NamedKey::Pause => Key::Pause,
        NamedKey::Play => Key::Play,
        NamedKey::Props => Key::Props,
        NamedKey::Select => Key::Select,
        NamedKey::ZoomIn => Key::ZoomIn,
        NamedKey::ZoomOut => Key::ZoomOut,
        NamedKey::BrightnessDown => Key::BrightnessDown,
        NamedKey::BrightnessUp => Key::BrightnessUp,
        NamedKey::Eject => Key::Eject,
        NamedKey::LogOff => Key::LogOff,
        NamedKey::Power => Key::Power,
        NamedKey::PowerOff => Key::PowerOff,
        NamedKey::PrintScreen => Key::PrintScreen,
        NamedKey::Hibernate => Key::Hibernate,
        NamedKey::Standby => Key::Standby,
        NamedKey::WakeUp => Key::WakeUp,
        NamedKey::AllCandidates => Key::AllCandidates,
        NamedKey::Alphanumeric => Key::Alphanumeric,
        NamedKey::CodeInput => Key::CodeInput,
        NamedKey::Compose => Key::Compose,
        NamedKey::Convert => Key::Convert,
        NamedKey::FinalMode => Key::FinalMode,
        NamedKey::GroupFirst => Key::GroupFirst,
        NamedKey::GroupLast => Key::GroupLast,
        NamedKey::GroupNext => Key::GroupNext,
        NamedKey::GroupPrevious => Key::GroupPrevious,
        NamedKey::ModeChange => Key::ModeChange,
        NamedKey::NextCandidate => Key::NextCandidate,
        NamedKey::NonConvert => Key::NonConvert,
        NamedKey::PreviousCandidate => Key::PreviousCandidate,
        NamedKey::Process => Key::Process,
        NamedKey::SingleCandidate => Key::SingleCandidate,
        NamedKey::HangulMode => Key::HangulMode,
        NamedKey::HanjaMode => Key::HanjaMode,
        NamedKey::JunjaMode => Key::JunjaMode,
        NamedKey::Eisu => Key::Eisu,
        NamedKey::Hankaku => Key::Hankaku,
        NamedKey::Hiragana => Key::Hiragana,
        NamedKey::HiraganaKatakana => Key::HiraganaKatakana,
        NamedKey::KanaMode => Key::KanaMode,
        NamedKey::KanjiMode => Key::KanjiMode,
        NamedKey::Katakana => Key::Katakana,
        NamedKey::Romaji => Key::Romaji,
        NamedKey::Zenkaku => Key::Zenkaku,
        NamedKey::ZenkakuHankaku => Key::ZenkakuHankaku,
        NamedKey::Soft1 => Key::Soft1,
        NamedKey::Soft2 => Key::Soft2,
        NamedKey::Soft3 => Key::Soft3,
        NamedKey::Soft4 => Key::Soft4,
        NamedKey::ChannelDown => Key::ChannelDown,
        NamedKey::ChannelUp => Key::ChannelUp,
        NamedKey::Close => Key::Close,
        NamedKey::MailForward => Key::MailForward,
        NamedKey::MailReply => Key::MailReply,
        NamedKey::MailSend => Key::MailSend,
        NamedKey::MediaClose => Key::MediaClose,
        NamedKey::MediaFastForward => Key::MediaFastForward,
        NamedKey::MediaPause => Key::MediaPause,
        NamedKey::MediaPlay => Key::MediaPlay,
        NamedKey::MediaPlayPause => Key::MediaPlayPause,
        NamedKey::MediaRecord => Key::MediaRecord,
        NamedKey::MediaRewind => Key::MediaRewind,
        NamedKey::MediaStop => Key::MediaStop,
        NamedKey::MediaTrackNext => Key::MediaTrackNext,
        NamedKey::MediaTrackPrevious => Key::MediaTrackPrevious,
        NamedKey::New => Key::New,
        NamedKey::Open => Key::Open,
        NamedKey::Print => Key::Print,
        NamedKey::Save => Key::Save,
        NamedKey::SpellCheck => Key::SpellCheck,
        NamedKey::Key11 => Key::Key11,
        NamedKey::Key12 => Key::Key12,
        NamedKey::AudioBalanceLeft => Key::AudioBalanceLeft,
        NamedKey::AudioBalanceRight => Key::AudioBalanceRight,
        NamedKey::AudioBassBoostDown => Key::AudioBassBoostDown,
        NamedKey::AudioBassBoostToggle => Key::AudioBassBoostToggle,
        NamedKey::AudioBassBoostUp => Key::AudioBassBoostUp,
        NamedKey::AudioFaderFront => Key::AudioFaderFront,
        NamedKey::AudioFaderRear => Key::AudioFaderRear,
        NamedKey::AudioSurroundModeNext => Key::AudioSurroundModeNext,
        NamedKey::AudioTrebleDown => Key::AudioTrebleDown,
        NamedKey::AudioTrebleUp => Key::AudioTrebleUp,
        NamedKey::AudioVolumeDown => Key::AudioVolumeDown,
        NamedKey::AudioVolumeUp => Key::AudioVolumeUp,
        NamedKey::AudioVolumeMute => Key::AudioVolumeMute,
        NamedKey::MicrophoneToggle => Key::MicrophoneToggle,
        NamedKey::MicrophoneVolumeDown => Key::MicrophoneVolumeDown,
        NamedKey::MicrophoneVolumeUp => Key::MicrophoneVolumeUp,
        NamedKey::MicrophoneVolumeMute => Key::MicrophoneVolumeMute,
        NamedKey::SpeechCorrectionList => Key::SpeechCorrectionList,
        NamedKey::SpeechInputToggle => Key::SpeechInputToggle,
        NamedKey::LaunchApplication1 => Key::LaunchApplication1,
        NamedKey::LaunchApplication2 => Key::LaunchApplication2,
        NamedKey::LaunchCalendar => Key::LaunchCalendar,
        NamedKey::LaunchContacts => Key::LaunchContacts,
        NamedKey::LaunchMail => Key::LaunchMail,
        NamedKey::LaunchMediaPlayer => Key::LaunchMediaPlayer,
        NamedKey::LaunchMusicPlayer => Key::LaunchMusicPlayer,
        NamedKey::LaunchPhone => Key::LaunchPhone,
        NamedKey::LaunchScreenSaver => Key::LaunchScreenSaver,
        NamedKey::LaunchSpreadsheet => Key::LaunchSpreadsheet,
        NamedKey::LaunchWebBrowser => Key::LaunchWebBrowser,
        NamedKey::LaunchWebCam => Key::LaunchWebCam,
        NamedKey::LaunchWordProcessor => Key::LaunchWordProcessor,
        NamedKey::BrowserBack => Key::BrowserBack,
        NamedKey::BrowserFavorites => Key::BrowserFavorites,
        NamedKey::BrowserForward => Key::BrowserForward,
        NamedKey::BrowserHome => Key::BrowserHome,
        NamedKey::BrowserRefresh => Key::BrowserRefresh,
        NamedKey::BrowserSearch => Key::BrowserSearch,
        NamedKey::BrowserStop => Key::BrowserStop,
        NamedKey::AppSwitch => Key::AppSwitch,
        NamedKey::Call => Key::Call,
        NamedKey::Camera => Key::Camera,
        NamedKey::CameraFocus => Key::CameraFocus,
        NamedKey::EndCall => Key::EndCall,
        NamedKey::GoBack => Key::GoBack,
        NamedKey::GoHome => Key::GoHome,
        NamedKey::HeadsetHook => Key::HeadsetHook,
        NamedKey::LastNumberRedial => Key::LastNumberRedial,
        NamedKey::Notification => Key::Notification,
        NamedKey::MannerMode => Key::MannerMode,
        NamedKey::VoiceDial => Key::VoiceDial,
        NamedKey::TV => Key::TV,
        NamedKey::TV3DMode => Key::TV3DMode,
        NamedKey::TVAntennaCable => Key::TVAntennaCable,
        NamedKey::TVAudioDescription => Key::TVAudioDescription,
        NamedKey::TVAudioDescriptionMixDown => Key::TVAudioDescriptionMixDown,
        NamedKey::TVAudioDescriptionMixUp => Key::TVAudioDescriptionMixUp,
        NamedKey::TVContentsMenu => Key::TVContentsMenu,
        NamedKey::TVDataService => Key::TVDataService,
        NamedKey::TVInput => Key::TVInput,
        NamedKey::TVInputComponent1 => Key::TVInputComponent1,
        NamedKey::TVInputComponent2 => Key::TVInputComponent2,
        NamedKey::TVInputComposite1 => Key::TVInputComposite1,
        NamedKey::TVInputComposite2 => Key::TVInputComposite2,
        NamedKey::TVInputHDMI1 => Key::TVInputHDMI1,
        NamedKey::TVInputHDMI2 => Key::TVInputHDMI2,
        NamedKey::TVInputHDMI3 => Key::TVInputHDMI3,
        NamedKey::TVInputHDMI4 => Key::TVInputHDMI4,
        NamedKey::TVInputVGA1 => Key::TVInputVGA1,
        NamedKey::TVMediaContext => Key::TVMediaContext,
        NamedKey::TVNetwork => Key::TVNetwork,
        NamedKey::TVNumberEntry => Key::TVNumberEntry,
        NamedKey::TVPower => Key::TVPower,
        NamedKey::TVRadioService => Key::TVRadioService,
        NamedKey::TVSatellite => Key::TVSatellite,
        NamedKey::TVSatelliteBS => Key::TVSatelliteBS,
        NamedKey::TVSatelliteCS => Key::TVSatelliteCS,
        NamedKey::TVSatelliteToggle => Key::TVSatelliteToggle,
        NamedKey::TVTerrestrialAnalog => Key::TVTerrestrialAnalog,
        NamedKey::TVTerrestrialDigital => Key::TVTerrestrialDigital,
        NamedKey::TVTimer => Key::TVTimer,
        NamedKey::AVRInput => Key::AVRInput,
        NamedKey::AVRPower => Key::AVRPower,
        NamedKey::ColorF0Red => Key::ColorF0Red,
        NamedKey::ColorF1Green => Key::ColorF1Green,
        NamedKey::ColorF2Yellow => Key::ColorF2Yellow,
        NamedKey::ColorF3Blue => Key::ColorF3Blue,
        NamedKey::ColorF4Grey => Key::ColorF4Grey,
        NamedKey::ColorF5Brown => Key::ColorF5Brown,
        NamedKey::ClosedCaptionToggle => Key::ClosedCaptionToggle,
        NamedKey::Dimmer => Key::Dimmer,
        NamedKey::DisplaySwap => Key::DisplaySwap,
        NamedKey::DVR => Key::DVR,
        NamedKey::Exit => Key::Exit,
        NamedKey::FavoriteClear0 => Key::FavoriteClear0,
        NamedKey::FavoriteClear1 => Key::FavoriteClear1,
        NamedKey::FavoriteClear2 => Key::FavoriteClear2,
        NamedKey::FavoriteClear3 => Key::FavoriteClear3,
        NamedKey::FavoriteRecall0 => Key::FavoriteRecall0,
        NamedKey::FavoriteRecall1 => Key::FavoriteRecall1,
        NamedKey::FavoriteRecall2 => Key::FavoriteRecall2,
        NamedKey::FavoriteRecall3 => Key::FavoriteRecall3,
        NamedKey::FavoriteStore0 => Key::FavoriteStore0,
        NamedKey::FavoriteStore1 => Key::FavoriteStore1,
        NamedKey::FavoriteStore2 => Key::FavoriteStore2,
        NamedKey::FavoriteStore3 => Key::FavoriteStore3,
        NamedKey::Guide => Key::Guide,
        NamedKey::GuideNextDay => Key::GuideNextDay,
        NamedKey::GuidePreviousDay => Key::GuidePreviousDay,
        NamedKey::Info => Key::Info,
        NamedKey::InstantReplay => Key::InstantReplay,
        NamedKey::Link => Key::Link,
        NamedKey::ListProgram => Key::ListProgram,
        NamedKey::LiveContent => Key::LiveContent,
        NamedKey::Lock => Key::Lock,
        NamedKey::MediaApps => Key::MediaApps,
        NamedKey::MediaAudioTrack => Key::MediaAudioTrack,
        NamedKey::MediaLast => Key::MediaLast,
        NamedKey::MediaSkipBackward => Key::MediaSkipBackward,
        NamedKey::MediaSkipForward => Key::MediaSkipForward,
        NamedKey::MediaStepBackward => Key::MediaStepBackward,
        NamedKey::MediaStepForward => Key::MediaStepForward,
        NamedKey::MediaTopMenu => Key::MediaTopMenu,
        NamedKey::NavigateIn => Key::NavigateIn,
        NamedKey::NavigateNext => Key::NavigateNext,
        NamedKey::NavigateOut => Key::NavigateOut,
        NamedKey::NavigatePrevious => Key::NavigatePrevious,
        NamedKey::NextFavoriteChannel => Key::NextFavoriteChannel,
        NamedKey::NextUserProfile => Key::NextUserProfile,
        NamedKey::OnDemand => Key::OnDemand,
        NamedKey::Pairing => Key::Pairing,
        NamedKey::PinPDown => Key::PinPDown,
        NamedKey::PinPMove => Key::PinPMove,
        NamedKey::PinPToggle => Key::PinPToggle,
        NamedKey::PinPUp => Key::PinPUp,
        NamedKey::PlaySpeedDown => Key::PlaySpeedDown,
        NamedKey::PlaySpeedReset => Key::PlaySpeedReset,
        NamedKey::PlaySpeedUp => Key::PlaySpeedUp,
        NamedKey::RandomToggle => Key::RandomToggle,
        NamedKey::RcLowBattery => Key::RcLowBattery,
        NamedKey::RecordSpeedNext => Key::RecordSpeedNext,
        NamedKey::RfBypass => Key::RfBypass,
        NamedKey::ScanChannelsToggle => Key::ScanChannelsToggle,
        NamedKey::ScreenModeNext => Key::ScreenModeNext,
        NamedKey::Settings => Key::Settings,
        NamedKey::SplitScreenToggle => Key::SplitScreenToggle,
        NamedKey::STBInput => Key::STBInput,
        NamedKey::STBPower => Key::STBPower,
        NamedKey::Subtitle => Key::Subtitle,
        NamedKey::Teletext => Key::Teletext,
        NamedKey::VideoModeNext => Key::VideoModeNext,
        NamedKey::Wink => Key::Wink,
        NamedKey::ZoomToggle => Key::ZoomToggle,
        NamedKey::F1 => Key::F1,
        NamedKey::F2 => Key::F2,
        NamedKey::F3 => Key::F3,
        NamedKey::F4 => Key::F4,
        NamedKey::F5 => Key::F5,
        NamedKey::F6 => Key::F6,
        NamedKey::F7 => Key::F7,
        NamedKey::F8 => Key::F8,
        NamedKey::F9 => Key::F9,
        NamedKey::F10 => Key::F10,
        NamedKey::F11 => Key::F11,
        NamedKey::F12 => Key::F12,
        NamedKey::F13 => Key::F13,
        NamedKey::F14 => Key::F14,
        NamedKey::F15 => Key::F15,
        NamedKey::F16 => Key::F16,
        NamedKey::F17 => Key::F17,
        NamedKey::F18 => Key::F18,
        NamedKey::F19 => Key::F19,
        NamedKey::F20 => Key::F20,
        NamedKey::F21 => Key::F21,
        NamedKey::F22 => Key::F22,
        NamedKey::F23 => Key::F23,
        NamedKey::F24 => Key::F24,
        NamedKey::F25 => Key::F25,
        NamedKey::F26 => Key::F26,
        NamedKey::F27 => Key::F27,
        NamedKey::F28 => Key::F28,
        NamedKey::F29 => Key::F29,
        NamedKey::F30 => Key::F30,
        NamedKey::F31 => Key::F31,
        NamedKey::F32 => Key::F32,
        NamedKey::F33 => Key::F33,
        NamedKey::F34 => Key::F34,
        NamedKey::F35 => Key::F35,
        _ => unimplemented!(),
    }
}

/// Maps winit key to carbide core key
pub fn convert_key(key: &WinitKey) -> Key {
    match key {
        WinitKey::Named(n) => convert_named_key(n),
        WinitKey::Character(s) => Key::Character(s.to_string()),
        WinitKey::Unidentified(u) => {
            println!("Warning, unknown keycode: {:?}", u);
            Key::Unknown
        }
        WinitKey::Dead(d) => {
            println!("Warning, dead key: {:?}", d);
            Key::Unknown
        }
    }
}

pub fn convert_mouse_button(button: WinitMouseButton) -> MouseButton {
    match button {
        WinitMouseButton::Left => MouseButton::Left,
        WinitMouseButton::Right => MouseButton::Right,
        WinitMouseButton::Middle => MouseButton::Middle,
        WinitMouseButton::Other(0) => MouseButton::Button4,
        WinitMouseButton::Other(1) => MouseButton::Button5,
        WinitMouseButton::Other(2) => MouseButton::Button6,
        WinitMouseButton::Other(3) => MouseButton::Button7,
        WinitMouseButton::Other(4) => MouseButton::Button8,
        n => {
            println!("Unknown mouse button: {:?}", n);
            MouseButton::Unknown
        },
    }
}

pub fn convert_mouse_cursor(cursor: MouseCursor) -> CursorIcon {
    match cursor {
        MouseCursor::Default => CursorIcon::Default,
        MouseCursor::Crosshair => CursorIcon::Crosshair,
        MouseCursor::Pointer => CursorIcon::Pointer,
        MouseCursor::Move => CursorIcon::Move,
        MouseCursor::Text => CursorIcon::Text,
        MouseCursor::Wait => CursorIcon::Wait,
        MouseCursor::Help => CursorIcon::Help,
        MouseCursor::Progress => CursorIcon::Progress,
        MouseCursor::NotAllowed => CursorIcon::NotAllowed,
        MouseCursor::ContextMenu => CursorIcon::ContextMenu,
        MouseCursor::Cell => CursorIcon::Cell,
        MouseCursor::VerticalText => CursorIcon::VerticalText,
        MouseCursor::Alias => CursorIcon::Alias,
        MouseCursor::Copy => CursorIcon::Copy,
        MouseCursor::NoDrop => CursorIcon::NoDrop,
        MouseCursor::Grab => CursorIcon::Grab,
        MouseCursor::Grabbing => CursorIcon::Grabbing,
        MouseCursor::AllScroll => CursorIcon::AllScroll,
        MouseCursor::ZoomIn => CursorIcon::ZoomIn,
        MouseCursor::ZoomOut => CursorIcon::ZoomOut,
        MouseCursor::EResize => CursorIcon::EResize,
        MouseCursor::NResize => CursorIcon::NResize,
        MouseCursor::NeResize => CursorIcon::NeResize,
        MouseCursor::NwResize => CursorIcon::NwResize,
        MouseCursor::SResize => CursorIcon::SResize,
        MouseCursor::SeResize => CursorIcon::SeResize,
        MouseCursor::SwResize => CursorIcon::SwResize,
        MouseCursor::WResize => CursorIcon::WResize,
        MouseCursor::EwResize => CursorIcon::EwResize,
        MouseCursor::NsResize => CursorIcon::NsResize,
        MouseCursor::NeswResize => CursorIcon::NeswResize,
        MouseCursor::NwseResize => CursorIcon::NwseResize,
        MouseCursor::ColResize => CursorIcon::ColResize,
        MouseCursor::RowResize => CursorIcon::RowResize,
        MouseCursor::Custom(_) => CursorIcon::Default,
    }
}

pub fn convert_window_event(event: &WindowEvent) -> Option<Input> {
    // FIXME: We should not hardcode scale factor. When we convert physical to logical, this
    // should be depending on the current window, because the event can be propagated to more than
    // one window.

    // The window size in points.
    let scale_factor: f64 = 2.0;

    match event {
        WindowEvent::Resized(physical_size) => {
            let LogicalSize { width, height } = physical_size.to_logical(scale_factor);
            Some(Input::Resize(width, height))
        }
        WindowEvent::Focused(focused) => {
            Some(Input::Focus(*focused))
        }
        WindowEvent::KeyboardInput { event: winit::event::KeyEvent { logical_key, state, .. }, .. } => {
            let key = convert_key(logical_key);
            let res = match state {
                ElementState::Pressed => Input::Press(Button::Keyboard(key)),
                ElementState::Released => Input::Release(Button::Keyboard(key)),
            };
            Some(res)
        }
        WindowEvent::Touch(WinitTouch { phase, location, id, .. }) => {
            let LogicalPosition { x, y } = location.to_logical::<f64>(scale_factor);

            let phase = match phase {
                WinitTouchPhase::Started => TouchPhase::Start,
                WinitTouchPhase::Moved => TouchPhase::Move,
                WinitTouchPhase::Cancelled => TouchPhase::Cancel,
                WinitTouchPhase::Ended => TouchPhase::End,
            };

            let id = TouchId::new(id.clone());

            let touch = Touch {
                phase,
                id,
                position: Position::new(x, y)
            };

            Some(Input::Touch(touch))
        }
        WindowEvent::CursorMoved { position, .. } => {
            let LogicalPosition { x, y } = position.to_logical::<f64>(scale_factor);

            Some(Input::Motion(Motion::MouseCursor { x, y }))
        }
        WindowEvent::MouseWheel { delta, .. } => {
            match delta {
                MouseScrollDelta::PixelDelta(delta) => {
                    let LogicalPosition { x, y } = delta.to_logical::<f64>(scale_factor);
                    let x = x;
                    let y = -y;

                    Some(Input::Motion(Motion::Scroll { x, y }))
                }
                MouseScrollDelta::LineDelta(x, y) => {
                    // This should be configurable (we should provide a LineDelta event to allow for this).
                    let x = ARBITRARY_POINTS_PER_LINE_FACTOR * *x as carbide_core::draw::Scalar;
                    let y = ARBITRARY_POINTS_PER_LINE_FACTOR * -*y as carbide_core::draw::Scalar;

                    Some(Input::Motion(Motion::Scroll { x, y }))
                }
            }
        }
        WindowEvent::SmartMagnify { .. } => {
            Some(Input::Gesture(Gesture::SmartScale))
        }
        WindowEvent::TouchpadRotate { delta, phase, .. } => {
            let phase = match phase {
                WinitTouchPhase::Started => TouchPhase::Start,
                WinitTouchPhase::Moved => TouchPhase::Move,
                WinitTouchPhase::Cancelled => TouchPhase::Cancel,
                WinitTouchPhase::Ended => TouchPhase::End,
            };

            Some(Input::Gesture(Gesture::Rotate(
                *delta as f64,
                phase
            )))
        }
        WindowEvent::TouchpadMagnify {delta, phase, .. } => {
            let phase = match phase {
                WinitTouchPhase::Started => TouchPhase::Start,
                WinitTouchPhase::Moved => TouchPhase::Move,
                WinitTouchPhase::Cancelled => TouchPhase::Cancel,
                WinitTouchPhase::Ended => TouchPhase::End,
            };

            Some(Input::Gesture(Gesture::Scale(
                *delta,
                phase
            )))
        }
        WindowEvent::MouseInput { state, button, .. } => {

            let mouse_button = convert_mouse_button(*button);
            match state {
                ElementState::Pressed => Some(Input::Press(Button::Mouse(mouse_button))),
                ElementState::Released => Some(Input::Release(Button::Mouse(mouse_button))),
            }
        }
        WindowEvent::CloseRequested => {
            Some(Input::CloseRequested)
        }
        _ => None,
    }
}