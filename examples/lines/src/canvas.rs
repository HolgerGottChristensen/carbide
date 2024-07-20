use carbide::draw::{Dimension, Position, Rect};
use carbide::environment::{Environment, EnvironmentColor};
use carbide::state::{LocalState, StateSync, State, ValueRefMut};
use carbide::widget::canvas::{CanvasContext, Context};

use crate::editing_mode::{CreateWallState, EditingMode, SelectedState};
use crate::graph::Graph;
use crate::guide::Guide;
use crate::line::Line;

#[derive(Clone)]
pub struct GraphCanvas(pub LocalState<Graph>);

impl CanvasContext for GraphCanvas {
    fn call(&mut self, area: Rect, context: &mut Context, env: &mut Environment) {
        self.0.sync(env);
        let mut graph = self.0.value_mut();
        context.set_line_width(1.0);

        graph.calculate_lines();

        draw_edges(context, &mut graph);

        draw_nodes(context, &mut graph);

        draw_guides(&area, context, &mut graph);

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

                context.circle(mouse_position.x - 4.5, mouse_position.y - 4.5, 9.0);
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
                context.circle(pos.x - 4.5, pos.y - 4.5, 9.0);
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

                context.circle(mouse_position.x - 4.5, mouse_position.y - 4.5, 9.0);
                context.fill();
            }
            EditingMode::Selection { hovered, selected } => {
                draw_selection_hovered(context, &mut graph, hovered);
                draw_selection_selected(context, graph, selected);
            }
        }
    }
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
            context.circle(node.position.x - 4.5, node.position.y - 4.5, 9.0);
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
            context.circle(node.position.x - 4.5, node.position.y - 4.5, 9.0);
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
            context.circle(node.position.x - 4.5, node.position.y - 4.5, 9.0);

            context.fill();
            context.begin_path();
            context.set_fill_style(EnvironmentColor::DarkText);
        } else {
            context.circle(node.position.x - 4.5, node.position.y - 4.5, 9.0);
        }
    }

    context.fill();
}

fn line_between(context: &mut Context, line: &Line, offset: Position) {
    if line.len().is_normal() {
        context.move_to(offset.x + line.start.x, offset.y + line.start.y);
        context.line_to(offset.x + line.end.x, offset.y + line.end.y);
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
                point_context.circle(position.x - 2.5, position.y - 2.5, 5.0);
            }
        }
    }

    context.stroke();

    point_context.fill();

    context.append(point_context);
}
