use carbide_controls::{List, Treeable, TreeDisclosure};
use carbide_core::draw::Dimension;
use carbide_core::environment::EnvironmentColor;
use carbide_core::state::{AnyState, FieldState, LocalState, Map1, ReadState, State, StateExtNew};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

#[derive(Clone, Debug)]
struct Tree(String, WidgetId, Vec<Tree>);

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    let list_model: Tree = Tree(
        "Root".to_string(),
        WidgetId::new(),
        vec![
            Tree("Leaf 1".to_string(), WidgetId::new(), vec![]),
            Tree("Leaf 2".to_string(), WidgetId::new(), vec![]),
            Tree(
                "Subtree 1".to_string(),
                WidgetId::new(),
                vec![Tree("Leaf 1".to_string(), WidgetId::new(), vec![])],
            ),
            Tree("Leaf 2".to_string(), WidgetId::new(), vec![]),
            Tree(
                "Subtree 2".to_string(),
                WidgetId::new(),
                vec![
                    Tree("Leaf 3".to_string(), WidgetId::new(), vec![]),
                    Tree("Leaf 4".to_string(), WidgetId::new(), vec![]),
                    Tree(
                        "Subtree 3".to_string(),
                        WidgetId::new(),
                        vec![
                            Tree("Leaf 5".to_string(), WidgetId::new(), vec![]),
                            Tree("Leaf 6".to_string(), WidgetId::new(), vec![]),
                        ],
                    ),
                ],
            ),
            Tree("Leaf 7".to_string(), WidgetId::new(), vec![]),
        ],
    );

    let list_model_state = LocalState::new(vec![list_model]);

    fn delegate(item: impl State<T=Tree>, _: impl ReadState<T=usize>) -> impl Widget {
        let label = Map1::read_map(item, |tree| {
            tree.0.clone()
        });

        ZStack::new((
            Rectangle::new().fill(EnvironmentColor::SystemFill),
            Text::new(label),
        ))
            .frame_fixed_height(30.0)
    }

    application.set_scene(Window::new(
        "Tree List Example - Carbide",
        Dimension::new(400.0, 600.0),
        List::new(list_model_state, delegate)
            .tree(TreeDisclosure::Arrow)
            .clip()
            .border()
            .border_width(1)
            .color(EnvironmentColor::OpaqueSeparator)
            .padding(40.0),
    ));

    application.launch();
}

impl Treeable<Tree> for Box<dyn AnyState<T=Tree>> {
    fn children(&self) -> Box<dyn AnyState<T=Vec<Tree>>> {
        FieldState::new(
            self.clone(),
            |item| { &item.2 },
            |item| { &mut item.2 },
        ).as_dyn()
    }
}