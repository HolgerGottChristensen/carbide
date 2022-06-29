mod node;
mod edge;
mod graph;
mod node_editor;
mod line;
mod constraints;
mod guide;
mod editing_mode;

use std::cmp::Ordering;
use std::time::Duration;
use carbide_controls::{Button, capture, TextInput};
use carbide_core::{animate, lens, Scalar, matches_case};
use carbide_core::draw::{Dimension, Position, Rect};
use carbide_core::environment::{Environment, EnvironmentFontSize};
use carbide_core::prelude::EnvironmentColor;
use carbide_core::state::{FieldState, LocalState, Map1, ReadState, State, StateExt, TState};
use carbide_core::text::FontFamily;
use carbide_core::widget::*;
use carbide_core::widget::canvas::{Canvas, Context};
use carbide_wgpu::window::*;
use crate::edge::Edge;
use crate::editing_mode::{CreateWallState, EditingMode, SelectedState};
use crate::graph::Graph;
use crate::guide::Guide;
use crate::line::Line;
use crate::node::Node;
use crate::node_editor::NodeEditor;

fn main() {
    env_logger::init();

    let icon_path = Window::relative_path_to_assets("images/rust_press.png");

    let mut window = Window::new(
        "Lines example".to_string(),
        800,
        600,
        Some(icon_path.clone()),
    );

    let family = FontFamily::new_from_paths("NotoSans", vec![
        "fonts/NotoSans/NotoSans-Regular.ttf"
    ]);
    window.add_font_family(family);

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

        fn line_between(context: &mut Context, line: &Line, offset: Position) {
            if line.len().is_normal() {
                context.move_to(offset.x() + line.start.x(), offset.y() + line.start.y());
                context.line_to(offset.x() + line.end.x(), offset.y() + line.end.y());
            }
        }

        let mut graph = s.value_mut();


        context.set_stroke_style(EnvironmentColor::DarkText);
        context.set_line_width(1.0);

        for node_id in 0..graph.nodes.len() {
            //println!("Nodeid: {:?}", node_id);
            let start_node = graph.get_node(node_id);
            let mut lines = vec![];
            for neighbor in graph.get_outgoing_edges_iter(node_id) {
                let end_node = graph.get_node(neighbor.to);

                lines.push((neighbor.id, Line::new(start_node.position, end_node.position), true, neighbor.offset, neighbor.width));
            }

            for neighbor in graph.get_incoming_edges_iter(node_id) {
                let end_node = graph.get_node(neighbor.from);

                lines.push((neighbor.id, Line::new(start_node.position, end_node.position), false, neighbor.offset, neighbor.width));
            }

            lines.sort_by(|a, b| {
                total_cmp(a.1.angle(), b.1.angle())
            });

            for (before, after) in lines.iter().zip(lines.iter().skip(1).chain(lines.iter())) {
                line_between(&mut context, &before.1, graph.offset);

                if lines.len() > 1 {

                    let w1 = if before.2 { before.4 * (1.0 - before.3) } else { before.4 * before.3 };
                    let w2 = if !after.2 { after.4 * (1.0 - after.3) } else { after.4 * after.3 };

                    let offset1 = before.1.normal_offset(-w1);
                    let offset2 = after.1.normal_offset(w2);

                    let a = offset1.intersect(&offset2);

                    let angle = (after.1.angle() - before.1.angle()).abs() % 180.0;

                    let (intersect1, intersect2) = if let Some(a) = a {
                        if (angle < 15.0 || angle > 165.0) && (
                            offset1.start.dist(&a) > offset1.len() / 10.0 &&
                                offset2.start.dist(&a) > offset2.len() / 10.0
                        ) {
                            (offset1.start, offset2.start)
                        } else {

                            (a, a)
                        }
                    } else {
                        (offset1.start, offset2.start)
                    };


                    let edge_before = graph.get_edge_mut(before.0);
                    if before.2 {
                        edge_before.neg_line.start = intersect1;
                        edge_before.neg_line.flip();
                    } else {
                        edge_before.pos_line.start = intersect1;
                        edge_before.pos_line.flip();
                    }


                    let edge_after = graph.get_edge_mut(after.0);
                    if after.2 {
                        edge_after.pos_line.start = intersect2;
                        edge_after.pos_line.flip();
                    } else {
                        edge_after.neg_line.start = intersect2;
                        edge_after.neg_line.flip();
                    }


                } else {
                    let multiplier1 = if before.2 { (1.0 - before.3) } else { before.3 };
                    let multiplier2 = if before.2 { before.3 } else { (1.0 - before.3) };

                    let offset1 = before.1.normal_offset(-before.4 * multiplier1);
                    let offset2 = before.1.normal_offset(before.4 * multiplier2);

                    let edge_before = graph.get_edge_mut(before.0);
                    edge_before.pos_line.start = offset1.start;
                    edge_before.neg_line.start = offset2.start;
                    edge_before.neg_line.flip();
                    edge_before.pos_line.flip();
                }
            }
        }

        context.stroke();
        context.begin_path();
        context.set_stroke_style(EnvironmentColor::Blue);

        for edge in &graph.edges {
            line_between(&mut context, &edge.neg_line, graph.offset);
            line_between(&mut context, &edge.pos_line, graph.offset);
        }

        context.stroke();
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

        context.begin_path();
        context.set_stroke_style(EnvironmentColor::Green);

        for guide in &graph.guides {
            match guide {
                Guide::Vertical(x) => {
                    line_between(&mut context, &Line::new(
                        Position::new(*x, 0.0),
                        Position::new(*x, rect.height()),
                    ), graph.offset);
                }
                Guide::Horizontal(y) => {
                    line_between(&mut context, &Line::new(
                        Position::new(0.0, *y),
                        Position::new(rect.width(), *y),
                    ), graph.offset);
                }
                Guide::Directional(line) => {
                    line_between(&mut context, &line.extend(Rect::new(
                        Position::new(0.0, 0.0),
                        Dimension::new(rect.width(), rect.height())
                    )), graph.offset);
                }
            }
        }

        context.stroke();


        match graph.editing_mode {
            EditingMode::Editing => {}
            EditingMode::CreateWallP1 { mouse_position, state } => {
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
            EditingMode::CreateWallP2 { mouse_position, state, first_node_id } => {
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
                hovered: SelectedState::None
            };
                    }))
        .frame(70.0, 26.0);

    let editing_button = Button::new("Editing")
        .on_click(capture!([editing_mode], |env: &mut Environment| {
                        *editing_mode = EditingMode::Editing;
                    }))
        .frame(70.0, 26.0);

    /*let selected_node_id = FieldState::new(state.clone(), |a: &Graph| {
        match a {
            Graph { editing_mode: EditingMode::Selection { selected: SelectedState::Node(x), .. }, .. } => {
                x
            }
            _ => panic!("Not matching")
        }
    }, |b: &mut Graph| {
        match b {
            Graph { editing_mode: EditingMode::Selection { selected: SelectedState::Node(x), .. }, .. } => {
                x
            }
            _ => panic!("Not matching")
        }
    });

    let selected_id = Match::<Graph>::new(&state)
        .case(|a| {
            //println!("{:?}", a);

            matches!(a, Graph { editing_mode: EditingMode::Selection { selected: SelectedState::Node(x), .. }, .. })
        }, Text::new(selected_node_id));*/

    let selected_id = Match::new(&state)
        .case(matches_case!(state, Graph { editing_mode: EditingMode::Selection { selected: SelectedState::None, .. }, .. }, {
            VStack::new(vec![
                HStack::new(vec![
                    Text::new("Nothing is selected")
                        .font_size(EnvironmentFontSize::Title),
                    Spacer::new()
                ]),
                Spacer::new(),
            ]).padding(10.0)
        }))
        .case(matches_case!(state, Graph { editing_mode: EditingMode::Selection { selected, hovered }, .. }, selected, hovered => VStack::new(vec![
            Text::new(Map1::read_map(selected, |a: &SelectedState| format!("Selected: {:?}", a)).ignore_writes()),
            Text::new(Map1::read_map(hovered, |a: &SelectedState| format!("Hovered: {:?}", a)).ignore_writes()),
        ])));

    window.set_widgets(
        VStack::new(vec![
            HStack::new(vec![
                add_wall_button,
                selection_button,
                editing_button,
                Spacer::new()
            ]).padding(EdgeInsets::single(0.0, 0.0, 10.0, 10.0)).frame_fixed_height(35.0)
                .background(Rectangle::new().fill(EnvironmentColor::SystemFill)),
            HSplit::new(ZStack::new(vec![
                    node_editor,
                    canvas.clip()
                ]),
                ZStack::new(vec![
                    Rectangle::new().fill(EnvironmentColor::Blue),
                    selected_id,
                ])
            ).relative_to_end(250.0),
            HStack::new(vec![
                Text::new(editing_mode.read_map(|a: &EditingMode| format!("Mode: {}", a)).ignore_writes()),
                Spacer::new()
            ]).padding(EdgeInsets::single(0.0, 0.0, 10.0, 10.0))
                .frame_fixed_height(20.0)
                .background(Rectangle::new().fill(EnvironmentColor::SystemFill)),
        ]).spacing(0.0)
    );

    window.launch();
}



// https://math.stackexchange.com/questions/3176543/intersection-point-of-2-lines-defined-by-2-points-each
/// # a = pt 1 on line 1
/// # b = pt 2 on line 1
/// # c = pt 1 on line 2
/// # d = pt 2 on line 2
fn intersect(a: Position, b: Position, c: Position, d: Position) -> Option<Position> {
    // stuff for line 1
    let a1 = b.y()-a.y();
    let b1 = a.x()-b.x();
    let c1 = a1*a.x() + b1*a.y();

    // stuff for line 2
    let a2 = d.y()-c.y();
    let b2 = c.x()-d.x();
    let c2 = a2*c.x() + b2*c.y();

    let determinant = a1*b2 - a2*b1;

    if determinant == 0.0 {
        None
    } else {
        let x = (b2*c1 - b1*c2) / determinant;
        let y = (a1*c2 - a2*c1) / determinant;
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