use std::cmp::Ordering;

use carbide::{lens, matches_case, Scalar};
use carbide::{Application, Window};
use carbide::draw::{Dimension, Position, Rect};
use carbide::environment::{Environment, EnvironmentColor, EnvironmentFontSize};
use carbide::state::{IndexableState, LocalState, Map1, ReadState, State, StateExt, TState, ValueRefMut};
use carbide::text::FontFamily;
use carbide::widget::*;
use carbide::widget::canvas::{Canvas, Context};
use carbide_controls::{Button, capture, Slider, TextInput};

use crate::edge::Edge;
use crate::editing_mode::{CreateWallState, EditingMode, SelectedState};
use crate::graph::Graph;
use crate::guide::Guide;
use crate::line::Line;
use crate::node::Node;
use crate::node_editor::NodeEditor;

mod constraints;
mod edge;
mod editing_mode;
mod graph;
mod guide;
mod line;
mod node;
mod node_editor;

fn main() {
    env_logger::init();

    let mut application = Application::new();

    let family =
        FontFamily::new_from_paths("NotoSans", vec!["fonts/NotoSans/NotoSans-Regular.ttf"]);
    application.add_font_family(family);

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

    let editing_mode: TState<EditingMode> = lens!(Graph; state.editing_mode);

    let canvas = Canvas::<Graph>::new_with_state(&state, |s, rect, mut context, _| {
        let mut graph = s.value_mut();
        context.set_line_width(1.0);

        graph.calculate_lines();

        draw_edges(&mut context, &mut graph);

        draw_nodes(&mut context, &mut graph);

        draw_guides(&rect, &mut context, &mut graph);

        match graph.editing_mode {
            EditingMode::Editing => {}
            EditingMode::CreateWallP1 {
                mouse_position,
                state,
            } => {
                context.begin_path();
                match state {
                    CreateWallState::Invalid => {
                        context.set_fill_style(EnvironmentColor::Red);
                    }
                    CreateWallState::ExistingNode => {
                        context.set_fill_style(EnvironmentColor::Blue);
                    }
                    CreateWallState::SplitEdge => {
                        context.set_fill_style(EnvironmentColor::Green);
                    }
                    CreateWallState::Floating => {
                        context.set_fill_style(EnvironmentColor::Red);
                    }
                }

                context.circle(mouse_position.x() - 4.5, mouse_position.y() - 4.5, 9.0);
                context.fill();
            }
            EditingMode::CreateWallP2 {
                mouse_position,
                state,
                first_node_id,
            } => {
                let pos = graph.get_node(first_node_id).position;
                context.begin_path();
                context.set_fill_style(EnvironmentColor::Yellow);
                context.circle(pos.x() - 4.5, pos.y() - 4.5, 9.0);
                context.fill();

                context.begin_path();

                match state {
                    CreateWallState::Invalid => {
                        context.set_fill_style(EnvironmentColor::Red);
                    }
                    CreateWallState::ExistingNode => {
                        context.set_fill_style(EnvironmentColor::Blue);
                    }
                    CreateWallState::SplitEdge => {
                        context.set_fill_style(EnvironmentColor::Green);
                    }
                    CreateWallState::Floating => {
                        context.set_fill_style(EnvironmentColor::Blue);
                    }
                }

                context.circle(mouse_position.x() - 4.5, mouse_position.y() - 4.5, 9.0);
                context.fill();
            }
            EditingMode::Selection { hovered, selected } => {
                draw_selection_hovered(&mut context, &mut graph, hovered);
                draw_selection_selected(&mut context, graph, selected);
            }
        }

        context
    });

    let node_editor = NodeEditor::new(&state);

    let add_wall_button = Button::new("Add Wall")
        .on_click(capture!([editing_mode], |env: &mut Environment| {
            *editing_mode = EditingMode::CreateWallP1 {
                mouse_position: Position::new(0.0, 0.0),
                state: CreateWallState::Invalid,
            };
        }))
        .frame(70.0, 26.0);

    let selection_button = Button::new("Selection")
        .on_click(capture!([editing_mode], |env: &mut Environment| {
            *editing_mode = EditingMode::Selection {
                selected: SelectedState::None,
                hovered: SelectedState::None,
            };
        }))
        .frame(70.0, 26.0);

    let editing_button = Button::new("Editing")
        .on_click(capture!([editing_mode], |env: &mut Environment| {
            *editing_mode = EditingMode::Editing;
        }))
        .frame(70.0, 26.0);

    fn selected_node_view(graph: &TState<Graph>, selected_state: TState<usize>) -> Box<dyn Widget> {
        let nodes: TState<Vec<Node>> = lens!(Graph; graph.nodes);
        let node = nodes.index(&selected_state);

        let height = lens!(Node; node.height);

        VStack::new(vec![
            Text::new(
                Map1::read_map(selected_state, |id: &usize| format!("NodeId: {}", id))
                    .ignore_writes(),
            )
            .font_size(EnvironmentFontSize::Title),
            Rectangle::new().frame_fixed_height(1.0),
            Spacer::fixed(5.0),
            Text::new(
                Map1::read_map(node.clone(), |a: &Node| {
                    format!(
                        "Position: (x: {:.2}, y: {:.2})",
                        a.position.x(),
                        a.position.y()
                    )
                })
                .ignore_writes(),
            ),
            HStack::new(vec![
                Text::new("Height:"),
                TextInput::new(height),
                Text::new("cm"),
            ]),
            Spacer::new(),
        ])
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .padding(10.0)
    }

    fn selected_edge_view(graph: &TState<Graph>, selected_state: TState<usize>) -> Box<dyn Widget> {
        let edges = lens!(Graph; graph.edges);
        let edge = edges.index(&selected_state);

        let offset = lens!(Edge; edge.offset);
        let width = lens!(Edge; edge.width);

        VStack::new(vec![
            Text::new(
                Map1::read_map(selected_state, |id: &usize| format!("EdgeId: {}", id))
                    .ignore_writes(),
            )
            .font_size(EnvironmentFontSize::Title),
            Rectangle::new().frame_fixed_height(1.0),
            Spacer::fixed(5.0),
            HStack::new(vec![
                Text::new("Offset:"),
                Slider::new(offset.clone(), 0.0, 1.0).step(0.05),
                Text::new(
                    Map1::read_map(offset, |o: &Scalar| format!("{:.0} %", o * 100.0))
                        .ignore_writes(),
                ),
            ]),
            HStack::new(vec![
                Text::new("Width:"),
                Slider::new(width.clone(), 5.0, 50.0).step(5.0),
                Text::new(
                    Map1::read_map(width, |w: &Scalar| format!("{:.2} cm", w)).ignore_writes(),
                ),
            ]),
            Spacer::new(),
        ])
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .padding(10.0)
    }

    let selected_id = Match::new(&state)
        .case(matches_case!(state, Graph { editing_mode: EditingMode::Selection { selected: SelectedState::None, .. }, .. }, {
            VStack::new(vec![
                Text::new("Nothing is selected")
                    .font_size(EnvironmentFontSize::Title),
                Spacer::new(),
            ]).cross_axis_alignment(CrossAxisAlignment::Start)
            .padding(10.0)
        }))
        .case(matches_case!(state, Graph { editing_mode: EditingMode::Selection { selected: SelectedState::Node(x), .. }, .. }, x => selected_node_view(&state, x)))
        .case(matches_case!(state, Graph { editing_mode: EditingMode::Selection { selected: SelectedState::Edge(x), .. }, .. }, x => selected_edge_view(&state, x)));

    let status_bar = HStack::new(vec![
        Text::new(
            Map1::read_map(editing_mode, |a: &EditingMode| format!("Mode: {}", a)).ignore_writes(),
        ),
        Spacer::new(),
    ])
    .padding(EdgeInsets::single(0.0, 0.0, 10.0, 10.0))
    .frame_fixed_height(20.0)
    .background(Rectangle::new().fill(EnvironmentColor::SystemFill));

    let tool_bar = HStack::new(vec![
        add_wall_button,
        selection_button,
        editing_button,
        Spacer::new(),
    ])
    .padding(EdgeInsets::single(0.0, 0.0, 10.0, 10.0))
    .frame_fixed_height(35.0)
    .background(Rectangle::new().fill(EnvironmentColor::SystemFill));


    application.set_scene(Window::new(
        "Lines example".to_string(),
        Dimension::new(800.0, 600.0),
        VStack::new(vec![
            tool_bar,
            HSplit::new(
                ZStack::new(vec![node_editor, canvas.clip()]),
                ZStack::new(vec![
                    Rectangle::new().fill(EnvironmentColor::TertiarySystemFill),
                    selected_id,
                ]),
            )
                .relative_to_end(250.0),
            status_bar,
        ])
            .spacing(0.0),
    ).close_application_on_window_close());

    application.launch();
}

fn draw_selection_selected(
    mut context: &mut Context,
    mut graph: ValueRefMut<Graph>,
    selected: SelectedState,
) {
    match selected {
        SelectedState::None => {}
        SelectedState::Node(node_id) => {
            let node = graph.get_node(node_id);

            context.begin_path();
            context.set_fill_style(EnvironmentColor::Yellow);
            context.circle(node.position.x() - 4.5, node.position.y() - 4.5, 9.0);
            context.fill();
        }
        SelectedState::Edge(edge_id) => {
            let edge = graph.get_edge(edge_id);
            let line = Line::new(
                graph.get_node(edge.to).position,
                graph.get_node(edge.from).position,
            );

            context.begin_path();
            context.set_stroke_style(EnvironmentColor::Yellow);
            line_between(&mut context, &line, graph.offset);
            context.stroke();
        }
    }
}

fn draw_selection_hovered(
    mut context: &mut Context,
    mut graph: &mut ValueRefMut<Graph>,
    hovered: SelectedState,
) {
    match hovered {
        SelectedState::None => {}
        SelectedState::Node(node_id) => {
            let node = graph.get_node(node_id);

            context.begin_path();
            context.set_fill_style(EnvironmentColor::Green);
            context.circle(node.position.x() - 4.5, node.position.y() - 4.5, 9.0);
            context.fill();
        }
        SelectedState::Edge(edge_id) => {
            let edge = graph.get_edge(edge_id);
            let line = Line::new(
                graph.get_node(edge.to).position,
                graph.get_node(edge.from).position,
            );

            context.begin_path();
            context.set_stroke_style(EnvironmentColor::Green);
            line_between(&mut context, &line, graph.offset);
            context.stroke();
        }
    }
}

fn draw_edges(mut context: &mut Context, mut graph: &mut ValueRefMut<Graph>) {
    context.begin_path();

    context.set_stroke_style(EnvironmentColor::DarkText);

    for edge in &graph.edges {
        let line = Line::new(
            graph.get_node(edge.from).position,
            graph.get_node(edge.to).position,
        );

        line_between(&mut context, &line, graph.offset);
    }

    context.stroke();

    context.begin_path();
    context.set_stroke_style(EnvironmentColor::Blue);

    for edge in &graph.edges {
        line_between(&mut context, &edge.neg_line, graph.offset);
        line_between(&mut context, &edge.pos_line, graph.offset);
    }

    context.stroke();
}

fn draw_nodes(mut context: &mut Context, graph: &mut ValueRefMut<Graph>) {
    context.set_fill_style(EnvironmentColor::DarkText);
    context.begin_path();

    for node in &graph.nodes {
        if node.hovered {
            context.fill();

            context.begin_path();
            context.set_fill_style(EnvironmentColor::Blue);
            context.circle(node.position.x() - 4.5, node.position.y() - 4.5, 9.0);

            context.fill();
            context.begin_path();
            context.set_fill_style(EnvironmentColor::DarkText);
        } else {
            context.circle(node.position.x() - 4.5, node.position.y() - 4.5, 9.0);
        }
    }

    context.fill();
}

fn line_between(context: &mut Context, line: &Line, offset: Position) {
    if line.len().is_normal() {
        context.move_to(offset.x() + line.start.x(), offset.y() + line.start.y());
        context.line_to(offset.x() + line.end.x(), offset.y() + line.end.y());
    }
}

fn draw_guides(rect: &Rect, mut context: &mut Context, mut graph: &mut ValueRefMut<Graph>) {
    let mut point_context = Context::new();
    point_context.begin_path();
    point_context.set_fill_style(EnvironmentColor::Green);

    context.begin_path();
    context.set_stroke_style(EnvironmentColor::Green);

    for guide in &graph.guides {
        match guide {
            Guide::Vertical(x) => {
                line_between(
                    &mut context,
                    &Line::new(Position::new(*x, 0.0), Position::new(*x, rect.height())),
                    graph.offset,
                );
            }
            Guide::Horizontal(y) => {
                line_between(
                    &mut context,
                    &Line::new(Position::new(0.0, *y), Position::new(rect.width(), *y)),
                    graph.offset,
                );
            }
            Guide::Directional(line) => {
                line_between(
                    &mut context,
                    &line.extend(Rect::new(
                        Position::new(0.0, 0.0),
                        Dimension::new(rect.width(), rect.height()),
                    )),
                    graph.offset,
                );
            }
            Guide::Point(position) => {
                point_context.circle(position.x() - 2.5, position.y() - 2.5, 5.0);
            }
        }
    }

    context.stroke();

    point_context.fill();

    context.append(point_context);
}

// https://math.stackexchange.com/questions/3176543/intersection-point-of-2-lines-defined-by-2-points-each
/// # a = pt 1 on line 1
/// # b = pt 2 on line 1
/// # c = pt 1 on line 2
/// # d = pt 2 on line 2
fn intersect(a: Position, b: Position, c: Position, d: Position) -> Option<Position> {
    // stuff for line 1
    let a1 = b.y() - a.y();
    let b1 = a.x() - b.x();
    let c1 = a1 * a.x() + b1 * a.y();

    // stuff for line 2
    let a2 = d.y() - c.y();
    let b2 = c.x() - d.x();
    let c2 = a2 * c.x() + b2 * c.y();

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
