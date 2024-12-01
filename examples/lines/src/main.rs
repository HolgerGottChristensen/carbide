use std::cmp::Ordering;

use carbide::{closure, lens, ui};
use carbide::{Application, Window};
use carbide::controls::{Button, Slider, TextInput};
use carbide::draw::{Dimension, Position, Scalar};
use carbide::environment::{EnvironmentColor, EnvironmentFontSize};
use carbide::state::{IndexState, LocalState, Map1, ReadState, ReadStateExtNew, State};
use carbide::widget::*;
use carbide::widget::canvas::Canvas;

use crate::canvas::GraphCanvas;
use crate::edge::Edge;
use crate::editing_mode::{CreateWallState, EditingMode, SelectedState};
use crate::graph::Graph;
use crate::line::Line;
use crate::node::Node;
use crate::node_editor::NodeEditor;

mod edge;
mod editing_mode;
mod graph;
mod guide;
mod line;
mod node;
mod node_editor;
mod canvas;

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    let mut graph = Graph::new();
    graph.add_node(Node::new(Position::new(100.0, 100.0)));
    graph.add_node(Node::new(Position::new(300.0, 100.0)));
    graph.add_node(Node::new(Position::new(300.0, 200.0)));
    graph.add_node(Node::new(Position::new(200.0, 200.0)));
    //graph.add_node(Node::new(Position::new(150.0, 300.0)));
    graph.add_edge(0, 1, Edge::new());
    graph.add_edge(1, 2, Edge::new());
    graph.add_edge(2, 3, Edge::new());
    graph.add_edge(3, 0, Edge::new());
    //graph.add_edge(0, 4, Edge::new().offset(0.3));

    let state = LocalState::new(graph);

    let editing_mode = lens!(state.editing_mode);

    let canvas = Canvas::new(GraphCanvas(state.clone()));

    let node_editor = NodeEditor::new(&state);

    let add_wall_button = Button::new_primary("Add Wall", closure!(|_| {
        *$editing_mode = EditingMode::CreateWallP1 {
            mouse_position: Position::new(0.0, 0.0),
            state: CreateWallState::Invalid,
        };
    })).frame(70.0, 26.0);

    let selection_button = Button::new_primary("Selection", closure!(|_| {
        *$editing_mode = EditingMode::Selection {
            selected: SelectedState::None,
            hovered: SelectedState::None,
        };
    })).frame(70.0, 26.0);

    let editing_button = Button::new_primary("Editing", closure!(|_| {
        *$editing_mode = EditingMode::Editing;
    }))
        .frame(70.0, 26.0);

    let selected_panel = ui!(
        match state {
            Graph { editing_mode: EditingMode::Selection { selected: SelectedState::None, .. }, .. } => {
                VStack::new((
                    Text::new("Nothing is selected")
                        .font_size(EnvironmentFontSize::Title),
                    Spacer::new(),
                ))
                .cross_axis_alignment(CrossAxisAlignment::Start)
                .padding(10.0)
                .boxed()
            }
            Graph { editing_mode: EditingMode::Selection { selected: SelectedState::Node(n), .. }, .. } => {
                selected_node_view(state.clone(), n)
            }
            Graph { editing_mode: EditingMode::Selection { selected: SelectedState::Edge(e), .. }, .. } => {
                selected_edge_view(state.clone(), e)
            }
            _ => Empty::new().boxed(),
        }
    );

    let status_bar = HStack::new((
        Text::new(Map1::read_map(editing_mode, |a: &EditingMode| format!("Mode: {}", a))),
        Spacer::new(),
    ))
    .padding(EdgeInsets::single(0.0, 0.0, 10.0, 10.0))
    .frame_fixed_height(20.0)
    .background(Rectangle::new().fill(EnvironmentColor::SystemFill));

    let tool_bar = HStack::new((
        add_wall_button,
        selection_button,
        editing_button,
        Spacer::new(),
    ))
    .padding(EdgeInsets::single(0.0, 0.0, 10.0, 10.0))
    .frame_fixed_height(35.0)
    .background(Rectangle::new().fill(EnvironmentColor::SystemFill));


    let widget = VStack::new((
        tool_bar,
        HSplit::new(
            ZStack::new((
                node_editor,
                canvas.clip()
            )),
            ZStack::new((
                Rectangle::new().fill(EnvironmentColor::TertiarySystemFill),
                selected_panel,
            )),
        ).relative_to_end(250.0),
        status_bar,
    ))
        .spacing(0.0);

    application.set_scene(Window::new(
        "Lines example".to_string(),
        Dimension::new(800.0, 600.0),
        widget,
    ));

    application.launch();
}

fn selected_node_view(graph: impl State<T=Graph>, selected_state: impl ReadState<T=usize>) -> Box<dyn AnyWidget> {
    let nodes = lens!(graph.nodes);
    let node = IndexState::new(nodes, selected_state.clone());

    let height = lens!(node.height);

    VStack::new((
        Text::new(Map1::read_map(selected_state, |id: &usize| format!("NodeId: {}", id)).ignore_writes())
            .font_size(EnvironmentFontSize::Title),
        Rectangle::new()
            .frame_fixed_height(1.0),
        Spacer::fixed(5.0),
        Text::new(
            Map1::read_map(node.clone(), |a: &Node| {
                format!(
                    "Position: (x: {:.2}, y: {:.2})",
                    a.position.x,
                    a.position.y
                )
            })
                .ignore_writes(),
        ),
        HStack::new((
            Text::new("Height:"),
            TextInput::new(height),
            Text::new("cm"),
        )),
        Spacer::new(),
    ))
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .padding(10.0)
        .boxed()
}

fn selected_edge_view(graph: impl State<T=Graph>, selected_state: impl ReadState<T=usize>) -> Box<dyn AnyWidget> {
    let edges = lens!(graph.edges);
    let edge = IndexState::new(edges, selected_state.clone());

    let offset = lens!(edge.offset);
    let width = lens!(edge.width);

    VStack::new((
        Text::new(Map1::read_map(selected_state, |id: &usize| format!("EdgeId: {}", id)).ignore_writes())
            .font_size(EnvironmentFontSize::Title),
        Rectangle::new()
            .frame_fixed_height(1.0),
        Spacer::fixed(5.0),
        HStack::new((
            Text::new("Offset:"),
            Slider::new(offset.clone(), 0.0, 1.0).step(0.05),
            Text::new(
                Map1::read_map(offset, |o: &Scalar| format!("{:.0} %", o * 100.0))
                    .ignore_writes(),
            ),
        )),
        HStack::new((
            Text::new("Width:"),
            Slider::new(width.clone(), 5.0, 50.0).step(5.0),
            Text::new(Map1::read_map(width, |w: &Scalar| format!("{:.2} cm", w)).ignore_writes()),
        )),
        Spacer::new(),
    ))
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .padding(10.0)
        .boxed()
}

// https://math.stackexchange.com/questions/3176543/intersection-point-of-2-lines-defined-by-2-points-each
/// # a = pt 1 on line 1
/// # b = pt 2 on line 1
/// # c = pt 1 on line 2
/// # d = pt 2 on line 2
fn intersect(a: Position, b: Position, c: Position, d: Position) -> Option<Position> {
    // stuff for line 1
    let a1 = b.y - a.y;
    let b1 = a.x - b.x;
    let c1 = a1 * a.x + b1 * a.y;

    // stuff for line 2
    let a2 = d.y - c.y;
    let b2 = c.x - d.x;
    let c2 = a2 * c.x + b2 * c.y;

    let determinant = a1 * b2 - a2 * b1;

    if determinant == 0.0 {
        None
    } else {
        let x = (b2 * c1 - b1 * c2) / determinant;
        let y = (a1 * c2 - a2 * c1) / determinant;
        Some(Position::new(x, y))
    }
}

fn total_cmp(one: f64, other: f64) -> Ordering {
    let mut left = one.to_bits() as i64;
    let mut right = other.to_bits() as i64;

    left ^= (((left >> 63) as u64) >> 1) as i64;
    right ^= (((right >> 63) as u64) >> 1) as i64;

    left.cmp(&right)
}