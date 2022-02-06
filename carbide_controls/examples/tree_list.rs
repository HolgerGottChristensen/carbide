use std::collections::HashSet;
use carbide_controls::{List, TreeDisclosure};
use carbide_core::environment::{EnvironmentColor, EnvironmentFontSize};
use carbide_core::lens;
use carbide_core::state::{LocalState, State, StateExt, StringState, TState, UsizeState};
use carbide_core::text::FontFamily;
use carbide_core::widget::*;
use carbide_core::window::TWindow;
use carbide_wgpu::window::Window;
use crate::Tree::{Leaf, SubTree};

#[derive(Clone, Debug)]
enum Tree {
    SubTree(String, Id, Vec<Tree>),
    Leaf(String, Id)
}

fn main() {
    env_logger::init();

    let icon_path = Window::relative_path_to_assets("images/rust_press.png");

    let mut window = Window::new(
        "Tree List Example - Carbide",
        800,
        1200,
        Some(icon_path),
    );

    let family = FontFamily::new_from_paths("NotoSans", vec!["fonts/NotoSans/NotoSans-Regular.ttf"]);
    window.add_font_family(family);

    let list_model: Tree =
        SubTree("Root".to_string(), Id::new_v4(), vec![
            SubTree("Subtree 1".to_string(), Id::new_v4(), vec![
                    Leaf("Leaf 1".to_string(), Id::new_v4()),
                ]),
            Leaf("Leaf 2".to_string(), Id::new_v4()),
            SubTree("Subtree 2".to_string(), Id::new_v4(), vec![
                Leaf("Leaf 3".to_string(), Id::new_v4()),
                Leaf("Leaf 4".to_string(), Id::new_v4()),
                SubTree("Subtree 3".to_string(), Id::new_v4(), vec![
                    Leaf("Leaf 5".to_string(), Id::new_v4()),
                    Leaf("Leaf 6".to_string(), Id::new_v4()),
                ]),
            ]),
            Leaf("Leaf 7".to_string(), Id::new_v4()),
        ]);

    let list_model_state = LocalState::new(vec![list_model]);

    let delegate = move |item: TState<Tree>, _: UsizeState| -> Box<dyn Widget> {

        ZStack::new(vec![
            Rectangle::new().fill(EnvironmentColor::SystemFill),
            Text::new(lens!(Tree; |item| {
                match item {
                    SubTree(s, _, _) => s.clone(),
                    Leaf(s, _) => s.clone(),
                }
            })),
        ]).frame(0.0, 30.0)
            .expand_width()
            .padding(EdgeInsets::single(0.0, 0.0, 0.0, 10.0))
    };

    fn tree_children(t: TState<Tree>) -> TState<Option<Vec<Tree>>> {
        t.mapped(|tree: &Tree| {
            match tree {
                SubTree(_, _, c) => Some(c.clone()),
                Leaf(_, _) => None,
            }
        })
    }

    window.set_widgets(
        List::new(list_model_state, delegate)
            .tree(tree_children, TreeDisclosure::Arrow)
            .clip()
            .border()
            .border_width(1)
            .color(EnvironmentColor::OpaqueSeparator)
            .padding(40.0),
    );

    window.launch();
}
