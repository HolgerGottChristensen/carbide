mod node;
mod edge;
mod graph;
mod node_editor;
mod line;

use std::cmp::Ordering;
use std::time::Duration;
use carbide_controls::{Button, capture, TextInput};
use carbide_core::{animate, Scalar};
use carbide_core::draw::Position;
use carbide_core::environment::Environment;
use carbide_core::prelude::EnvironmentColor;
use carbide_core::state::{LocalState, ReadState, State};
use carbide_core::text::FontFamily;
use carbide_core::widget::*;
use carbide_core::widget::canvas::{Canvas, Context};
use carbide_wgpu::window::*;
use crate::edge::Edge;
use crate::graph::Graph;
use crate::line::Line;
use crate::node::Node;
use crate::node_editor::NodeEditor;

fn main() {
    env_logger::init();

    let icon_path = Window::relative_path_to_assets("images/rust_press.png");

    let mut window = Window::new(
        "Lines example".to_string(),
        1200,
        1200,
        Some(icon_path.clone()),
    );

    let family = FontFamily::new_from_paths("NotoSans", vec![
        "fonts/NotoSans/NotoSans-Regular.ttf"
    ]);
    window.add_font_family(family);

    let mut graph = Graph{ nodes: vec![], edges: vec![] };
    graph.add_node(Node::new(Position::new(100.0, 100.0)));
    graph.add_node(Node::new(Position::new(300.0, 100.0)));
    graph.add_node(Node::new(Position::new(300.0, 200.0)));
    graph.add_node(Node::new(Position::new(200.0, 200.0)));
    graph.add_node(Node::new(Position::new(150.0, 300.0)));

    graph.add_edge(0, 1, Edge::new());
    graph.add_edge(1, 2, Edge::new().width(30.0));
    graph.add_edge(2, 3, Edge::new());
    graph.add_edge(3, 0, Edge::new());
    graph.add_edge(0, 4, Edge::new().offset(0.3));


    let state = LocalState::new(graph);

    let canvas = Canvas::<Graph>::new_with_state(&state, |s, rect, mut context, _| {

        fn line_between(context: &mut Context, line: &Line) {
            let center_point = Position::new(0.0, 0.0);
            //let center_point = Position::new(300.0, 300.0);

            context.move_to(center_point.x() + line.start.x(), center_point.y() + line.start.y());
            context.line_to(center_point.x() + line.end.x(), center_point.y() + line.end.y());
        }

        let mut graph = s.value_mut();


        context.set_stroke_style(EnvironmentColor::DarkText);
        context.set_line_width(1.0);

        let width1 = 10.0;

        for node_id in 0..graph.nodes.len() {
            //println!("Nodeid: {:?}", node_id);
            let start_node = graph.get_node(node_id);
            let mut lines = vec![];
            for neighbor in graph.get_outgoing_neighbors_iter(node_id) {
                let end_node = graph.get_node(neighbor.to);

                lines.push((neighbor.id, Line::new(start_node.position, end_node.position), true, neighbor.offset, neighbor.width));
            }

            for neighbor in graph.get_incoming_neighbors_iter(node_id) {
                let end_node = graph.get_node(neighbor.from);

                lines.push((neighbor.id, Line::new(start_node.position, end_node.position), false, neighbor.offset, neighbor.width));
            }

            lines.sort_by(|a, b| {
                total_cmp(a.1.angle(), b.1.angle())
            });

            for (before, after) in lines.iter().zip(lines.iter().skip(1).chain(lines.iter())) {
                line_between(&mut context, &before.1);

                if lines.len() > 1 {

                    let w1 = if before.2 { before.4 * (1.0 - before.3) } else { before.4 * before.3 };
                    let w2 = if !after.2 { after.4 * (1.0 - after.3) } else { after.4 * after.3 };

                    let mut offset1 = before.1.normal_offset(-w1);
                    let mut offset2 = after.1.normal_offset(w2);

                    let intersect1 = offset1.intersect(&offset2);

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
                        edge_after.pos_line.start = intersect1;
                        edge_after.pos_line.flip();
                    } else {
                        edge_after.neg_line.start = intersect1;
                        edge_after.neg_line.flip();
                    }


                } else {
                    let multiplier1 = if before.2 { (1.0 - before.3) } else { before.3 };
                    let multiplier2 = if before.2 { before.3 } else { (1.0 - before.3) };

                    let mut offset1 = before.1.normal_offset(-before.4 * multiplier1);
                    let mut offset2 = before.1.normal_offset(before.4 * multiplier2);

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
            line_between(&mut context, &edge.neg_line);
            line_between(&mut context, &edge.pos_line);
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

        context
    });

    let node_editor = NodeEditor::new(state);

    window.set_widgets(
        ZStack::new(vec![
            node_editor,
            canvas
        ])
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
        println!("The lines are parallel");
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