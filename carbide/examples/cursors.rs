use carbide::cursor::MouseCursor;
use carbide::state::{State, TState, UsizeState};
use carbide_controls::Button;
use carbide_core::environment::EnvironmentColor;
use carbide_core::state::{LocalState, StateExt};
use carbide_core::text::FontFamily;
use carbide_core::widget::*;
use carbide_wgpu::window::*;

fn main() {
    env_logger::init();

    let icon_path = Window::relative_path_to_assets("images/rust_press.png");

    let mut window = Window::new(
        "Mouse cursors example".to_string(),
        800,
        1200,
        Some(icon_path.clone()),
    );

    let family = FontFamily::new_from_paths("NotoSans", vec![
        "fonts/NotoSans/NotoSans-Regular.ttf"
    ]);
    window.add_font_family(family);

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


    fn delegate(item: TState<MouseCursor>, _: UsizeState) -> Box<dyn Widget> {
        Button::new(format!("{:?}", *item.value()))
            .hover_cursor(*item.value())
            .frame(100.0, 22.0)
    }

    window.set_widgets(
        HStack::new(vec![
            VStack::new(vec![
                ForEach::new(cursors1, delegate)
            ]),
            VStack::new(vec![
                ForEach::new(cursors2, delegate)
            ]),
        ]).cross_axis_alignment(CrossAxisAlignment::Start)
    );

    window.launch();
}
