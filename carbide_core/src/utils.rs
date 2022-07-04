//!
//! Various utility functions used throughout carbide.
//!

use std;
use std::borrow::Cow;
use std::f64::consts::{E, PI};
use std::iter::{once, Chain, Once};

/// Clamp a value between some range.
pub fn clamp<T: PartialOrd>(n: T, start: T, end: T) -> T {
    if start <= end {
        if n < start {
            start
        } else if n > end {
            end
        } else {
            n
        }
    } else {
        if n < end {
            end
        } else if n > start {
            start
        } else {
            n
        }
    }
}

/// Get the closest index in a sorted vec of f32
// function binary_search_rightmost(A, n, T):
//     L := 0
//     R := n
//     while L < R:
//         m := floor((L + R) / 2)
//         if A[m] > T:
//             R := m
//         else:
//             L := m + 1
//     return R - 1
pub fn binary_search(value: f32, vec: &Vec<f32>) -> usize {
    let mut left = 0;
    let mut right = vec.len();
    while left < right {
        let m = (left + right) / 2;
        if vec[m] > value {
            right = m;
        } else {
            left = m + 1;
        }
    }
    if right != 0 {
        right - 1
    } else {
        0
    }
}

/// Modulo float.
pub fn fmod(f: f32, n: i32) -> f32 {
    let i = f.floor() as i32;
    let modulo = match i % n {
        r if (r > 0 && n < 0) || (r < 0 && n > 0) => r + n,
        r => r,
    };
    modulo as f32 + f - i as f32
}

pub fn turns_to_radians(turns: f32) -> f32 {
    use std::f32::consts::PI;
    let f = 2.0 * PI;
    f * turns
}

pub fn gaussian(sigma: f64, x: f64) -> f64 {
    (1.0 / f64::sqrt(2.0 * PI * sigma)) * E.powf(-(x.powi(2) / (2.0 * f64::powi(sigma, 2))))
}

#[test]
fn gaussian_test() {
    println!("{}", gaussian(1.0, 0.0));
    println!("{}", gaussian(1.0, 0.5));
    assert_eq!(gaussian(1.0, -0.5), gaussian(1.0, 0.5))
}
