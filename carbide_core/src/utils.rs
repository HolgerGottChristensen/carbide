//!
//! Various utility functions used throughout carbide.
//!

use std;
use std::borrow::Cow;
use std::f64::consts::{E, PI};
use std::iter::{Chain, once, Once};
use num::{Float, NumCast, PrimInt, ToPrimitive};

/// Compare to PartialOrd values and return the min.
pub fn partial_min<T: PartialOrd>(a: T, b: T) -> T {
    if a <= b {
        a
    } else {
        b
    }
}

/// Compare to PartialOrd values and return the max.
pub fn partial_max<T: PartialOrd>(a: T, b: T) -> T {
    if a >= b {
        a
    } else {
        b
    }
}

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
    let f= 2.0 * PI;
    f * turns
}

/// A type returned by the `iter_diff` function.
///
/// Represents way in which the elements (of type `E`) yielded by the iterator `I` differ to some
/// other iterator yielding borrowed elements of the same type.
///
/// `I` is some `Iterator` yielding elements of type `E`.
pub enum IterDiff<E, I> {
    /// The index of the first non-matching element along with the iterator's remaining elements
    /// starting with the first mis-matched element.
    FirstMismatch(usize, Chain<Once<E>, I>),
    /// The remaining elements of the iterator.
    Longer(Chain<Once<E>, I>),
    /// The total number of elements that were in the iterator.
    Shorter(usize),
}

/// Compares every element yielded by both elems and new_elems in lock-step.
///
/// If the number of elements yielded by `b` is less than the number of elements yielded by `a`,
/// the number of `b` elements yielded will be returned as `IterDiff::Shorter`.
///
/// If the two elements of a step differ, the index of those elements along with the remaining
/// elements are returned as `IterDiff::FirstMismatch`.
///
/// If `a` becomes exhausted before `b` becomes exhausted, the remaining `b` elements will be
/// returned as `IterDiff::Longer`.
///
/// This function is useful when comparing a non-`Clone` `Iterator` of elements to some existing
/// collection. If there is any difference between the elements yielded by the iterator and those
/// of the collection, a suitable `IterDiff` is returned so that the existing collection may be
/// updated with the difference using elements from the very same iterator.
pub fn iter_diff<'a, A, B>(a: A, b: B) -> Option<IterDiff<B::Item, B::IntoIter>>
    where
        A: IntoIterator<Item=&'a B::Item>,
        B: IntoIterator,
        B::Item: PartialEq + 'a,
{
    let mut b = b.into_iter();
    for (i, a_elem) in a.into_iter().enumerate() {
        match b.next() {
            None => return Some(IterDiff::Shorter(i)),
            Some(b_elem) => {
                if *a_elem != b_elem {
                    return Some(IterDiff::FirstMismatch(i, once(b_elem).chain(b)));
                }
            }
        }
    }
    b.next().map(|elem| IterDiff::Longer(once(elem).chain(b)))
}

/// Returns `Borrowed` `elems` if `elems` contains the same elements as yielded by `new_elems`.
///
/// Allocates a new `Vec<T>` and returns `Owned` if either the number of elements or the elements
/// themselves differ.
pub fn write_if_different<T, I>(elems: &[T], new_elems: I) -> Cow<[T]>
    where
        T: PartialEq + Clone,
        I: IntoIterator<Item=T>,
{
    match iter_diff(elems.iter(), new_elems.into_iter()) {
        Some(IterDiff::FirstMismatch(i, mismatch)) => {
            Cow::Owned(elems[0..i].iter().cloned().chain(mismatch).collect())
        }
        Some(IterDiff::Longer(remaining)) => {
            Cow::Owned(elems.iter().cloned().chain(remaining).collect())
        }
        Some(IterDiff::Shorter(num_new_elems)) => {
            Cow::Owned(elems.iter().cloned().take(num_new_elems).collect())
        }
        None => Cow::Borrowed(elems),
    }
}

/// Compares two iterators to see if they yield the same thing.
pub fn iter_eq<A, B>(mut a: A, mut b: B) -> bool
    where
        A: Iterator,
        B: Iterator<Item=A::Item>,
        A::Item: PartialEq,
{
    loop {
        match (a.next(), b.next()) {
            (None, None) => return true,
            (maybe_a, maybe_b) => {
                if maybe_a != maybe_b {
                    return false;
                }
            }
        }
    }
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
