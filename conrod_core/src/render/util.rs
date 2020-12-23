use crate::{Rect, widget};
use crate::graph;
use crate::graph::Graph;
use crate::render::primitive::Primitive;
use crate::render::primitive_kind::PrimitiveKind;

/// Simplify the constructor for a `Primitive`.
pub fn new_primitive(id: widget::Id, kind: PrimitiveKind, scizzor: Rect, rect: Rect) -> Primitive {
    Primitive {
        id: id,
        kind: kind,
        scizzor: scizzor,
        rect: rect,
    }
}

/// Retrieves the next visible widget from the `depth_order`, updating the `crop_stack` as
/// necessary.
pub fn next_widget<'a>(depth_order: &mut std::slice::Iter<widget::Id>,
                   graph: &'a Graph,
                   crop_stack: &mut Vec<(widget::Id, Rect)>,
                   window_rect: Rect) -> Option<(widget::Id, Rect, &'a graph::Container)>
{
    while let Some(&id) = depth_order.next() {
        let container = match graph.widget(id) {
            Some(container) => container,
            None => continue,
        };

        // If we're currently using a cropped context and the current `crop_parent_idx` is
        // *not* a depth-wise parent of the widget at the current `idx`, we should pop that
        // cropped context from the stack as we are done with it.
        while let Some(&(crop_parent_idx, _)) = crop_stack.last() {
            if graph.does_recursive_depth_edge_exist(crop_parent_idx, id) {
                break;
            } else {
                crop_stack.pop();
            }
        }

        // Check the stack for the current Context.
        let scizzor = crop_stack.last().map(|&(_, scizzor)| scizzor).unwrap_or(window_rect);

        // If the current widget should crop its children, we need to add a rect for it to
        // the top of the crop stack.
        if container.crop_kids {
            let scizzor_rect = container.kid_area.rect.overlap(scizzor)
                .unwrap_or_else(|| Rect::from_xy_dim([0.0, 0.0], [0.0, 0.0]));
            crop_stack.push((id, scizzor_rect));
        }

        // We only want to return primitives that are actually visible.
        let is_visible = container.rect.overlap(window_rect).is_some()
            && graph::algo::cropped_area_of_widget(graph, id).is_some();
        if !is_visible {
            continue;
        }

        return Some((id, scizzor, container));
    }

    None
}
