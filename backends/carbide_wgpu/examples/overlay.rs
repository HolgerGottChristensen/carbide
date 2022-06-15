#[macro_use]
extern crate carbide_derive;

use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::*;
use carbide_core::event::{Key, KeyboardEvent, KeyboardEventHandler};
use carbide_core::layout::Layout;
use carbide_core::state::{BoolState, LocalState, StateExt};
use carbide_core::text::FontFamily;
use carbide_core::widget::*;
use carbide_wgpu::window::*;

fn main() {
    env_logger::init();

    let icon_path = Window::relative_path_to_assets("images/rust_press.png");

    let mut window = Window::new(
        "Overlay example - Carbide".to_string(),
        1200,
        900,
        Some(icon_path.clone()),
    );

    let family = FontFamily::new_from_paths("NotoSans", vec![
        "fonts/NotoSans/NotoSans-Regular.ttf"
    ]);
    window.add_font_family(family);

    let showing_state: BoolState = LocalState::new(false).into();

    window.set_widgets(
        OverlaidLayer::new(
            "overlay",
            VStack::new(vec![
                Text::new(showing_state.mapped(|a: &bool| format!("Currently showing overlay: {}", *a))),
                ZStack::new(vec![
                    Over::new(showing_state)
                        .frame(100.0, 100.0),
                    Rectangle::new()
                        .fill(EnvironmentColor::Green)
                        .frame(200.0, 200.0),
                    Rectangle::new()
                        .fill(EnvironmentColor::Red)
                        .frame(100.0, 100.0),
                    Text::new("Test")
                        .foreground_color(EnvironmentColor::Blue),
                ]),
                Text::new("Press space to toggle the overlay (yellow rectangle)"),
            ]))
    );

    window.launch();
}

#[derive(Clone, Debug, Widget)]
#[carbide_exclude(KeyboardEvent, Layout)]
struct Over {
    id: WidgetId,
    position: Position,
    dimension: Dimension,
    overlay_widget: Overlay,
}

impl Over {
    pub fn new(showing: BoolState) -> Box<Over> {
        Box::new(
            Over {
                id: WidgetId::new_v4(),
                position: Position::new(0.0, 0.0),
                dimension: Dimension::new(100.0, 100.0),
                overlay_widget: Overlay::new(
                    ZStack::new(vec![
                        Rectangle::new().fill(EnvironmentColor::Yellow),
                        Text::new("Over")
                            .foreground_color(EnvironmentColor::Red),
                    ]).frame(50.0, 50.0)
                ).showing(showing),
            }
        )
    }
}

impl KeyboardEventHandler for Over {
    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment) {
        match event {
            KeyboardEvent::Press(k, _) => {
                if *k == Key::Space {
                    if !self.overlay_widget.is_showing() {
                        self.overlay_widget.set_showing(true);
                        env.add_overlay("overlay", Some(self.overlay_widget.clone()))
                    } else {
                        env.add_overlay("overlay", None)
                    }
                }
            }
            _ => ()
        }
    }
}

impl Layout for Over {
    fn calculate_size(&mut self, requested_size: Dimension, env: &mut Environment) -> Dimension {
        self.set_dimension(requested_size);
        if self.overlay_widget.is_showing() {
            self.overlay_widget.calculate_size(requested_size, env);
        }
        requested_size
    }

    fn position_children(&mut self) {
        if self.overlay_widget.is_showing() {
            let positioning = self.alignment().positioner();
            let position = self.position();
            let dimension = self.dimension();
            positioning(position, dimension, &mut self.overlay_widget as &mut dyn Widget);
            self.overlay_widget.position_children();
        }
    }
}

/*impl Render for Over {
    fn process_get_primitives(&mut self, _: &mut Vec<Primitive>, env: &mut Environment) {
        if self.overlay_widget.is_showing() {
            env.add_overlay("overlay", OverlayValue::Update(self.overlay_widget.position(), self.overlay_widget.dimension()));
        }
    }
}*/

impl CommonWidget for Over {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn set_id(&mut self, id: WidgetId) {
        self.id = id;
    }

    fn children(&self) -> WidgetIter {
        WidgetIter::Empty
    }

    fn children_mut(&mut self) -> WidgetIterMut {
        WidgetIterMut::Empty
    }

    fn children_direct(&mut self) -> WidgetIterMut {
        WidgetIterMut::Empty
    }

    fn children_direct_rev(&mut self) -> WidgetIterMut {
        WidgetIterMut::Empty
    }

    fn position(&self) -> Position {
        self.position
    }

    fn set_position(&mut self, position: Position) {
        self.position = position
    }

    fn dimension(&self) -> Dimension {
        self.dimension
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        self.dimension = dimension
    }
}

impl WidgetExt for Over {}
