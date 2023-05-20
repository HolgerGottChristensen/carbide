use smallvec::{SmallVec, smallvec};
use crate::draw::{Dimension, Position};
use crate::environment::Environment;
use crate::flags::Flags;
use crate::layout::Layout;
use crate::widget::{CrossAxisAlignment, Widget};

pub(crate) fn calculate_size_vstack(
    widget: &mut dyn Layout,
    spacing: f64,
    requested_size: Dimension,
    env: &mut Environment,
) {
    calculate_size_stack(
        widget,
        height,
        width,
        height_width,
        spacing,
        requested_size,
        env,
    );
}

pub(crate) fn position_children_vstack(
    widget: &mut dyn Layout,
    spacing: f64,
    cross_axis_alignment: CrossAxisAlignment,
    env: &mut Environment,
) {
    position_children_stack(
        widget,
        y,
        height,
        x,
        width,
        y_x,
        cross_axis_alignment,
        spacing,
        env,
    );
}

pub(crate) fn calculate_size_hstack(
    widget: &mut dyn Layout,
    spacing: f64,
    requested_size: Dimension,
    env: &mut Environment,
) {
    calculate_size_stack(
        widget,
        width,
        height,
        width_height,
        spacing,
        requested_size,
        env,
    );
}

pub(crate) fn position_children_hstack(
    widget: &mut dyn Layout,
    spacing: f64,
    cross_axis_alignment: CrossAxisAlignment,
    env: &mut Environment,
) {
    position_children_stack(
        widget,
        x,
        width,
        y,
        height,
        x_y,
        cross_axis_alignment,
        spacing,
        env,
    );
}

fn x(position: Position) -> f64 {
    position.x
}

fn y(position: Position) -> f64 {
    position.y
}

fn x_y(main_axis: f64, cross_axis: f64) -> Position {
    Position::new(main_axis, cross_axis)
}

fn y_x(main_axis: f64, cross_axis: f64) -> Position {
    Position::new(cross_axis, main_axis)
}

fn height(dimension: Dimension) -> f64 {
    dimension.height
}

fn width(dimension: Dimension) -> f64 {
    dimension.width
}

fn height_width(main_axis: f64, cross_axis: f64) -> Dimension {
    Dimension::new(cross_axis, main_axis)
}

fn width_height(main_axis: f64, cross_axis: f64) -> Dimension {
    Dimension::new(main_axis, cross_axis)
}

/// *dimension*(main_axis, cross_axis)
fn calculate_size_stack(
    widget: &mut dyn Layout,
    main_axis: fn(Dimension) -> f64,
    cross_axis: fn(Dimension) -> f64,
    dimension: fn(f64, f64) -> Dimension,
    spacing: f64,
    requested_size: Dimension,
    env: &mut Environment,
) {
    let mut number_of_children_that_needs_sizing = 0;

    widget.foreach_child(&mut |child| {
        if !child.is_spacer() {
            number_of_children_that_needs_sizing += 1;
        }
    });

    // Calculate the number of spaces between elements in a stack.
    // This will be 0 if there are no children, or one child
    // It will be 1 if there are two non spacer children.

    let mut last_was_non_spacer = false;
    let mut is_first = true;
    let mut number_of_spaces = 0;

    widget.foreach_child(&mut |child| {
        if !is_first {
            if last_was_non_spacer {
                number_of_spaces += 1;
            }
        }

        last_was_non_spacer = !child.is_spacer();
        is_first = false;
    });

    let spacing_total = number_of_spaces as f64 * spacing;

    let mut size_for_children = dimension(
        main_axis(requested_size) - spacing_total,
        cross_axis(requested_size),
    );


    let mut children_flexibility_using_max_val: SmallVec<[(u32, &mut dyn Widget); 25]> = smallvec![];
    let mut children_flexibility_rest: SmallVec<[(u32, &mut dyn Widget); 25]> = smallvec![];

    widget.foreach_child_mut(&mut |child| {
        if !child.is_spacer() {
            if child.flag().contains(Flags::USEMAXCROSSAXIS) {
                children_flexibility_using_max_val.push((child.flexibility(), child));
            } else {
                children_flexibility_rest.push((child.flexibility(), child));
            }
        }
    });

    children_flexibility_using_max_val.sort_by(|(a, _), (b, _)| b.cmp(&a));
    children_flexibility_rest.sort_by(|(a, _), (b, _)| b.cmp(&a));

    let mut max_cross_axis = 0.0;

    let mut total_main_axis = 0.0;

    for (_, mut child) in children_flexibility_rest {
        let size_for_child = dimension(
            main_axis(size_for_children) / number_of_children_that_needs_sizing as f64,
            cross_axis(size_for_children),
        );

        let chosen_size = child.calculate_size(size_for_child, env);

        if cross_axis(chosen_size) > max_cross_axis {
            max_cross_axis = cross_axis(chosen_size);
        }

        size_for_children = dimension(
            (main_axis(size_for_children) - main_axis(chosen_size)).max(0.0),
            cross_axis(size_for_children),
        );

        number_of_children_that_needs_sizing -= 1;

        total_main_axis += main_axis(chosen_size);
    }

    for (_, mut child) in children_flexibility_using_max_val {
        let size_for_child = dimension(
            main_axis(size_for_children) / number_of_children_that_needs_sizing as f64,
            max_cross_axis,
        );

        let chosen_size = child.calculate_size(size_for_child, env);

        size_for_children = dimension(
            (main_axis(size_for_children) - main_axis(chosen_size)).max(0.0),
            cross_axis(size_for_children),
        );

        number_of_children_that_needs_sizing -= 1;

        total_main_axis += main_axis(chosen_size);
    }

    let mut spacer_count = 0.0;

    widget.foreach_child(&mut |child| {
        if child.is_spacer() {
            spacer_count += 1.0;
        }
    });

    let rest_space = main_axis(requested_size) - total_main_axis - spacing_total;

    let request_dimension = dimension(rest_space / spacer_count, 0.0);

    widget.foreach_child_mut(&mut |child| {
        if child.is_spacer() {
            let chosen_size = child.calculate_size(request_dimension, env);
            total_main_axis += main_axis(chosen_size);
        }
    });

    widget.set_dimension(dimension(total_main_axis + spacing_total, max_cross_axis));
}

fn position_children_stack(
    widget: &mut dyn Layout,
    main_axis_position: fn(Position) -> f64,
    main_axis_dimension: fn(Dimension) -> f64,
    cross_axis_position: fn(Position) -> f64,
    cross_axis_dimension: fn(Dimension) -> f64,
    position_from_main_and_cross: fn(f64, f64) -> Position,
    cross_axis_alignment: CrossAxisAlignment,
    spacing: f64,
    env: &mut Environment,
) {
    let alignment = cross_axis_alignment;
    let mut main_axis_offset = 0.0;

    let position = widget.position();
    let dimension = widget.dimension();

    widget.foreach_child_mut(&mut |child| {
        let cross = match alignment {
            CrossAxisAlignment::Start => cross_axis_position(position),
            CrossAxisAlignment::Center => {
                cross_axis_position(position) + cross_axis_dimension(dimension) / 2.0
                    - cross_axis_dimension(child.dimension()) / 2.0
            }
            CrossAxisAlignment::End => {
                cross_axis_position(position) + cross_axis_dimension(dimension)
                    - cross_axis_dimension(child.dimension())
            }
        };

        child.set_position(position_from_main_and_cross(
            main_axis_position(position) + main_axis_offset,
            cross,
        ));

        if !child.is_spacer() {
            main_axis_offset += spacing;
        }

        main_axis_offset += main_axis_dimension(child.dimension());
        child.position_children(env);
    });
}
