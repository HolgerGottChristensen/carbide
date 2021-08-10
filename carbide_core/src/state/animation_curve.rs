use num::traits::FloatConst;

use crate::draw::Position;
use crate::Scalar;

/* Animation curves based on https://easings.net/# and https://github.com/flutter/flutter/blob/f4abaa0735/packages/flutter/lib/src/animation/curves.dart#L1681*/

pub fn linear(input: f64) -> f64 {
    input
}

pub fn fast_linear_to_slow_ease_in(input: f64) -> f64 {
    cubic_bezier(input, Position::new(0.18, 1.0), Position::new(0.04, 1.0))
}

pub fn ease(input: f64) -> f64 {
    cubic_bezier(input, Position::new(0.25, 0.1), Position::new(0.25, 1.0))
}

pub fn ease_in(input: f64) -> f64 {
    cubic_bezier(input, Position::new(0.42, 0.0), Position::new(1.0, 1.0))
}

pub fn ease_in_to_linear(input: f64) -> f64 {
    cubic_bezier(input, Position::new(0.67, 0.03), Position::new(0.65, 0.09))
}

pub fn ease_in_sine(input: f64) -> f64 {
    cubic_bezier(input, Position::new(0.47, 0.0), Position::new(0.745, 0.715))
}

pub fn ease_in_quad(input: f64) -> f64 {
    cubic_bezier(input, Position::new(0.55, 0.085), Position::new(0.68, 0.53))
}

pub fn ease_in_cubic(input: f64) -> f64 {
    cubic_bezier(
        input,
        Position::new(0.55, 0.055),
        Position::new(0.675, 0.19),
    )
}

pub fn ease_in_quart(input: f64) -> f64 {
    cubic_bezier(
        input,
        Position::new(0.895, 0.03),
        Position::new(0.685, 0.22),
    )
}

pub fn ease_in_quint(input: f64) -> f64 {
    cubic_bezier(
        input,
        Position::new(0.755, 0.05),
        Position::new(0.855, 0.06),
    )
}

pub fn ease_in_expo(input: f64) -> f64 {
    cubic_bezier(
        input,
        Position::new(0.95, 0.05),
        Position::new(0.795, 0.035),
    )
}

pub fn ease_in_circ(input: f64) -> f64 {
    cubic_bezier(input, Position::new(0.6, 0.04), Position::new(0.98, 0.335))
}

pub fn ease_in_back(input: f64) -> f64 {
    cubic_bezier(
        input,
        Position::new(0.6, -0.28),
        Position::new(0.735, 0.045),
    )
}

pub fn ease_out(input: f64) -> f64 {
    cubic_bezier(input, Position::new(0.0, 0.0), Position::new(0.58, 1.0))
}

pub fn linear_to_ease_out(input: f64) -> f64 {
    cubic_bezier(input, Position::new(0.35, 0.91), Position::new(0.33, 0.97))
}

pub fn ease_out_sine(input: f64) -> f64 {
    cubic_bezier(input, Position::new(0.39, 0.575), Position::new(0.565, 1.0))
}

pub fn ease_out_quad(input: f64) -> f64 {
    cubic_bezier(input, Position::new(0.25, 0.46), Position::new(0.45, 0.94))
}

pub fn ease_out_cubic(input: f64) -> f64 {
    cubic_bezier(input, Position::new(0.215, 0.61), Position::new(0.355, 1.0))
}

pub fn ease_out_quart(input: f64) -> f64 {
    cubic_bezier(input, Position::new(0.165, 0.84), Position::new(0.44, 1.0))
}

pub fn ease_out_quint(input: f64) -> f64 {
    cubic_bezier(input, Position::new(0.23, 1.0), Position::new(0.32, 1.0))
}

pub fn ease_out_expo(input: f64) -> f64 {
    cubic_bezier(input, Position::new(0.19, 1.0), Position::new(0.22, 1.0))
}

pub fn ease_out_circ(input: f64) -> f64 {
    cubic_bezier(input, Position::new(0.075, 0.82), Position::new(0.165, 1.0))
}

pub fn ease_out_back(input: f64) -> f64 {
    cubic_bezier(
        input,
        Position::new(0.175, 0.885),
        Position::new(0.32, 1.275),
    )
}

pub fn ease_in_out(input: f64) -> f64 {
    cubic_bezier(input, Position::new(0.42, 0.0), Position::new(0.58, 1.0))
}

pub fn ease_in_out_sine(input: f64) -> f64 {
    cubic_bezier(input, Position::new(0.445, 0.05), Position::new(0.55, 0.95))
}

pub fn ease_in_out_quad(input: f64) -> f64 {
    cubic_bezier(
        input,
        Position::new(0.455, 0.03),
        Position::new(0.515, 0.955),
    )
}

pub fn ease_in_out_cubic(input: f64) -> f64 {
    cubic_bezier(
        input,
        Position::new(0.645, 0.045),
        Position::new(0.355, 1.0),
    )
}

pub fn ease_in_out_quart(input: f64) -> f64 {
    cubic_bezier(input, Position::new(0.77, 0.0), Position::new(0.175, 1.0))
}

pub fn ease_in_out_quint(input: f64) -> f64 {
    cubic_bezier(input, Position::new(0.86, 0.0), Position::new(0.07, 1.0))
}

pub fn ease_in_out_expo(input: f64) -> f64 {
    cubic_bezier(input, Position::new(1.0, 0.0), Position::new(0.0, 1.0))
}

pub fn ease_in_out_circ(input: f64) -> f64 {
    cubic_bezier(
        input,
        Position::new(0.785, 0.135),
        Position::new(0.15, 0.86),
    )
}

pub fn ease_in_out_back(input: f64) -> f64 {
    cubic_bezier(
        input,
        Position::new(0.68, -0.55),
        Position::new(0.265, 1.55),
    )
}

pub fn fast_out_slow_in(input: f64) -> f64 {
    cubic_bezier(input, Position::new(0.4, 0.0), Position::new(0.2, 1.0))
}

pub fn slow_middle(input: f64) -> f64 {
    cubic_bezier(input, Position::new(0.15, 0.85), Position::new(0.85, 0.15))
}

pub fn elastic_in_hard(input: f64) -> f64 {
    elastic_helper(input, 0.3)
}

pub fn elastic_in(input: f64) -> f64 {
    elastic_helper(input, 0.4)
}

pub fn elastic_in_soft(input: f64) -> f64 {
    elastic_helper(input, 0.5)
}

pub fn elastic_out_hard(input: f64) -> f64 {
    inverse(input, elastic_in_hard)
}

pub fn elastic_out(input: f64) -> f64 {
    inverse(input, elastic_in)
}

pub fn elastic_out_soft(input: f64) -> f64 {
    inverse(input, elastic_in_soft)
}

pub fn elastic_in_out_hard(input: f64) -> f64 {
    elastic_in_out_helper(input, 0.3)
}

pub fn elastic_in_out(input: f64) -> f64 {
    elastic_in_out_helper(input, 0.4)
}

pub fn elastic_in_out_soft(input: f64) -> f64 {
    elastic_in_out_helper(input, 0.5)
}

pub fn bounce_in(input: f64) -> f64 {
    inverse(input, bounce_helper)
}

pub fn bounce_out(input: f64) -> f64 {
    bounce_helper(input)
}

pub fn bounce_in_out(input: f64) -> f64 {
    if input < 0.5 {
        (1.0 - bounce_helper(1.0 - input * 2.0)) * 0.5
    } else {
        bounce_helper(input * 2.0 - 1.0) * 0.5 + 0.5
    }
}

pub fn bounce_helper(input: f64) -> f64 {
    let mut t = input;
    let c1 = 7.5625;
    let c2 = 2.75;
    if t < 1.0 / c2 {
        return c1 * t * t;
    } else if t < 2.0 / c2 {
        t -= 1.5 / c2;
        return c1 * t * t + 0.75;
    } else if t < 2.5 / c2 {
        t -= 2.25 / c2;
        return c1 * t * t + 0.9375;
    }

    t -= 2.625 / c2;
    c1 * t * t + 0.984375
}

pub fn inverse(input: f64, curve: fn(f64) -> f64) -> f64 {
    1.0 - curve(1.0 - input)
}

pub fn elastic_helper(input: f64, period: f64) -> f64 {
    let s = period / 4.0;
    let t = input - 1.0;
    -f64::powf(2.0, 10.0 * t) * f64::sin((t - s) * (f64::PI() * 2.0) / period)
}

pub fn elastic_in_out_helper(input: f64, period: f64) -> f64 {
    let s = period / 4.0;
    let t = 2.0 * input - 1.0;
    if t < 0.0 {
        -0.5 * f64::powf(2.0, 10.0 * t) * f64::sin((t - s) * (f64::PI() * 2.0) / period)
    } else {
        f64::powf(2.0, -10.0 * t) * f64::sin((t - s) * (f64::PI() * 2.0) / period) * 0.5 + 1.0
    }
}

/// Bezier estimation function based on an iterative approach. This function
/// will fail if multiple y values can be had for a single x. This excludes some input,
/// but the input is not validated.
pub fn cubic_bezier(x: f64, p1: Position, p2: Position) -> f64 {
    let precision = 0.001;
    let mut start = 0.0;
    let mut end = 1.0;
    loop {
        let midpoint = (start + end) / 2.0;
        let estimate = cubic_helper(p1.x, p2.x, midpoint);
        if (x - estimate).abs() < precision {
            return cubic_helper(p1.y, p2.y, midpoint);
        }
        if estimate < x {
            start = midpoint;
        } else {
            end = midpoint;
        }
    }
}

fn cubic_helper(a: f64, b: f64, mid: f64) -> f64 {
    3.0 * a * (1.0 - mid) * (1.0 - mid) * mid + 3.0 * b * (1.0 - mid) * mid * mid + mid * mid * mid
}
