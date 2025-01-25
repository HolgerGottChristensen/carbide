use carbide_controls::button::{BorderedProminentStyle, Button};
use carbide_controls::ControlsExt;
use carbide_core::closure;
use carbide_core::cursor::MouseCursor;
use carbide_core::draw::Dimension;
use carbide_core::state::{ReadState, State};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {

    let mut application = Application::new()
        .with_asset_fonts();

    let cursors1 = vec![
        MouseCursor::Default,
        MouseCursor::Crosshair,
        MouseCursor::Pointer,
        MouseCursor::Move,
        MouseCursor::Text,
        MouseCursor::Wait,
        MouseCursor::Help,
        MouseCursor::Progress,
        MouseCursor::NotAllowed,
        MouseCursor::ContextMenu,
        MouseCursor::Cell,
        MouseCursor::VerticalText,
        MouseCursor::Alias,
        MouseCursor::Copy,
        MouseCursor::NoDrop,
        MouseCursor::Grab,
        MouseCursor::Grabbing,
    ];

    let cursors2 = vec![
        MouseCursor::AllScroll,
        MouseCursor::ZoomIn,
        MouseCursor::ZoomOut,
        MouseCursor::EResize,
        MouseCursor::NResize,
        MouseCursor::NeResize,
        MouseCursor::NwResize,
        MouseCursor::SResize,
        MouseCursor::SeResize,
        MouseCursor::SwResize,
        MouseCursor::WResize,
        MouseCursor::EwResize,
        MouseCursor::NsResize,
        MouseCursor::NeswResize,
        MouseCursor::NwseResize,
        MouseCursor::ColResize,
        MouseCursor::RowResize,
    ];

    fn delegate(item: impl State<T=MouseCursor>, _: impl ReadState<T=usize>) -> impl Widget {
        Button::new(format!("{:?}", *item.value()), closure!(|_|{}))
            .cursor(item.value().clone())
            .frame(100.0, 22.0)
    }

    application.set_scene(Window::new(
        "Cursors example - Carbide",
        Dimension::new(400.0, 600.0),
        HStack::new((
            VStack::new(ForEach::new(cursors1, delegate)),
            VStack::new(ForEach::new(cursors2, delegate)),
        )).cross_axis_alignment(CrossAxisAlignment::Start)
            .button_style(BorderedProminentStyle)
    ));

    application.launch();
}
