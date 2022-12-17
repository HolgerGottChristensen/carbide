use carbide::cursor::MouseCursor;
use carbide::state::{ReadState, TState};
use carbide_controls::Button;
use carbide::draw::Dimension;
use carbide::widget::*;
use carbide::{Application, Window};

fn main() {
    env_logger::init();

    let mut application = Application::new()
        .with_asset_fonts();

    let cursors1 = vec![
        MouseCursor::Default,
        MouseCursor::Crosshair,
        MouseCursor::Hand,
        MouseCursor::Arrow,
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

    fn delegate(item: TState<MouseCursor>, _: TState<usize>) -> Box<dyn Widget> {
        Button::new(format!("{:?}", *item.value()))
            .hover_cursor(*item.value())
            .frame(100.0, 22.0)
    }

    application.set_scene(Window::new(
        "Mouse cursors example".to_string(),
        Dimension::new(400.0, 600.0),
        HStack::new(vec![
            VStack::new(vec![ForEach::new(cursors1, delegate)]),
            VStack::new(vec![ForEach::new(cursors2, delegate)]),
        ])
            .cross_axis_alignment(CrossAxisAlignment::Start)
    ).close_application_on_window_close());

    application.launch();
}
