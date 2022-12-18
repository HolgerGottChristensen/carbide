use carbide_controls::{List, TreeDisclosure};
use carbide_core::draw::Dimension;
use carbide_core::environment::EnvironmentColor;
use carbide_core::lens;
use carbide_core::state::{LocalState, StateExt, TState};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

use crate::Tree::{Leaf, SubTree};

#[derive(Clone, Debug)]
enum Tree {
    SubTree(String, WidgetId, Vec<Tree>),
    Leaf(String, WidgetId),
}

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    let list_model: Tree = SubTree(
        "Root".to_string(),
        WidgetId::new(),
        vec![
            SubTree(
                "Subtree 1".to_string(),
                WidgetId::new(),
                vec![Leaf("Leaf 1".to_string(), WidgetId::new())],
            ),
            Leaf("Leaf 2".to_string(), WidgetId::new()),
            SubTree(
                "Subtree 2".to_string(),
                WidgetId::new(),
                vec![
                    Leaf("Leaf 3".to_string(), WidgetId::new()),
                    Leaf("Leaf 4".to_string(), WidgetId::new()),
                    SubTree(
                        "Subtree 3".to_string(),
                        WidgetId::new(),
                        vec![
                            Leaf("Leaf 5".to_string(), WidgetId::new()),
                            Leaf("Leaf 6".to_string(), WidgetId::new()),
                        ],
                    ),
                ],
            ),
            Leaf("Leaf 7".to_string(), WidgetId::new()),
        ],
    );

    let list_model_state = LocalState::new(vec![list_model]);

    let delegate = move |item: TState<Tree>, _: TState<usize>| -> Box<dyn Widget> {
        ZStack::new(vec![
            Rectangle::new().fill(EnvironmentColor::SystemFill),
            Text::new(lens!(Tree; |item| {
                match item {
                    SubTree(s, _, _) => s.clone(),
                    Leaf(s, _) => s.clone(),
                }
            })),
        ])
        .frame(0.0, 30.0)
        .expand_width()
        .padding(EdgeInsets::single(0.0, 0.0, 0.0, 10.0))
    };

    fn tree_children(t: TState<Tree>) -> TState<Option<Vec<Tree>>> {
        t.map(|tree| match tree {
            SubTree(_, _, c) => Some(c.clone()),
            Leaf(_, _) => None,
        })
        .ignore_writes()
    }

    application.set_scene(Window::new(
        "Tree List Example - Carbide",
        Dimension::new(400.0, 600.0),
        List::new(list_model_state, delegate)
            .tree(tree_children, TreeDisclosure::Arrow)
            .clip()
            .border()
            .border_width(1)
            .color(EnvironmentColor::OpaqueSeparator)
            .padding(40.0),
    ).close_application_on_window_close());

    application.launch();
}
