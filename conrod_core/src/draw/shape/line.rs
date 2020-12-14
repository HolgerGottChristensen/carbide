use ::{Point, Scalar};
use ::{graph, Theme};
use widget;
use widget::Line;
use draw::shape::triangle::Triangle;

/// Given two points and half the line thickness, return the four corners of the rectangle
/// describing the line.
pub fn rect_corners(a: Point, b: Point, half_thickness: Scalar) -> [Point; 4] {
    let direction = [b[0] - a[0], b[1] - a[1]];
    let mag = (direction[0] * direction[0] + direction[1] * direction[1]).sqrt();
    let unit = [direction[0] / mag, direction[1] / mag];
    let normal = [-unit[1], unit[0]];
    let n = [normal[0] * half_thickness, normal[1] * half_thickness];
    let r1 = [a[0] + n[0], a[1] + n[1]];
    let r2 = [a[0] - n[0], a[1] - n[1]];
    let r3 = [b[0] + n[0], b[1] + n[1]];
    let r4 = [b[0] - n[0], b[1] - n[1]];
    [r1, r2, r3, r4]
}

/// The function to use for picking whether a given point is over the line.
pub fn is_over_widget(widget: &graph::Container, point: Point, theme: &Theme) -> widget::IsOver {
    widget
        .unique_widget_state::<Line>()
        .map(|widget| {
            let thickness = widget.style.get_thickness(theme);
            let (a, b) = (widget.state.start, widget.state.end);
            is_over(a, b, thickness, point)
        })
        .unwrap_or_else(|| widget.rect.is_over(point))
        .into()
}

/// Given two points and half the line thickness, return the two triangles that describe the line.
pub fn triangles(a: Point, b: Point, half_thickness: Scalar) -> [Triangle<Point>; 2] {
    let r = rect_corners(a, b, half_thickness);
    let t1 = Triangle([r[0], r[3], r[1]]);
    let t2 = Triangle([r[0], r[3], r[2]]);
    [t1, t2]
}

/// Describes whether or not the given point touches the line described by *a -> b* with the given
/// thickness.
pub fn is_over(a: Point, b: Point, thickness: Scalar, point: Point) -> bool {
    let half_thickness = thickness * 0.5;
    let tris = triangles(a, b, half_thickness);
    widget::triangles::is_over(tris.iter().cloned(), point)
}

