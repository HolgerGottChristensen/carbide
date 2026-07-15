use std::fmt::Debug;

use crate::animation::Animatable;
use crate::animation::animation_curve::linear;
use crate::animation::keyframes::timeline::{ResolveMode, Timeline, TimelineStatus};
use crate::state::{AnyState, IntoState, State, StateContract, StateExtNew};
use crate::time::Duration;

#[derive(Clone)]
struct KeyframeStop<T> {
    value: T,
    curve: fn(f64) -> f64,
    interpolation: fn(&T, &T, f64) -> T,
}

/// A leaf [`Timeline`] holding an ordered list of `(value, duration, curve,
/// interpolation)` stops for one [`Animatable`] (or custom-interpolated)
/// value, bound to an existing carbide `State<T>` handle. `curve` (timing)
/// and `interpolation` (blending) are independently settable per segment —
/// e.g. one segment can use [`ease_in_out`](crate::animation::ease_in_out)
/// with `Color::hsla_blend` while the next uses linear timing with the
/// default `Color::rgba_blend`. Both also have a per-leaf *default* (`linear`
/// for the curve, `Animatable::interpolate`/a custom fn for interpolation)
/// used by any stop that doesn't explicitly override it — set the defaults
/// with [`Keyframes::curve`] and [`Keyframes::interpolation`].
#[derive(Clone)]
pub struct Keyframes<T: StateContract> {
    state: Box<dyn AnyState<T=T>>,
    initial: T,
    default_curve: fn(f64) -> f64,
    default_interpolation: fn(&T, &T, f64) -> T,
    stops: Vec<KeyframeStop<T>>,
    /// cumulative[i] == absolute end-time of stops[i]; cumulative.last()
    /// (if any) == duration(). Maintained incrementally as stops are
    /// appended so duration()/seek() never need an O(n) recomputation.
    cumulative: Vec<Duration>,
}

impl<T: StateContract + Animatable<T>> Keyframes<T> {
    /// Requires `T: Animatable<T>` so `.to()`/`.to_with_curve()` can default
    /// to `T::interpolate` for any stop that doesn't override interpolation
    /// — override that default for the whole leaf with `.interpolation(...)`,
    /// or use [`Keyframes::new_custom`] for a `T` with no `Animatable` impl
    /// at all. Defaults to `linear` for the curve — override with
    /// `.curve(...)`.
    pub fn new<S: IntoState<T>>(state: S, initial: T) -> Self {
        Keyframes {
            state: state.into_state().as_dyn(),
            initial,
            default_curve: linear,
            default_interpolation: T::interpolate,
            stops: Vec::new(),
            cumulative: Vec::new(),
        }
    }
}

impl<T: StateContract> Keyframes<T> {
    /// No `Animatable` bound — for types with no natural default blend (or
    /// no `Animatable` impl at all). `default_interpolation` must be
    /// supplied explicitly since there's no `T::interpolate` to fall back
    /// to; it can still be changed later with `.interpolation(...)`.
    /// Defaults to `linear` for the curve — override with `.curve(...)`.
    pub fn new_custom<S: IntoState<T>>(
        state: S,
        initial: T,
        default_interpolation: fn(&T, &T, f64) -> T,
    ) -> Self {
        Keyframes {
            state: state.into_state().as_dyn(),
            initial,
            default_curve: linear,
            default_interpolation,
            stops: Vec::new(),
            cumulative: Vec::new(),
        }
    }

    /// Sets the default curve used by `.to()`/`.to_with_interpolation()` for
    /// any stop appended after this call — a per-leaf default that
    /// individual stops can still override via `.to_with_curve()`/
    /// `.to_custom()`.
    pub fn curve(mut self, default_curve: fn(f64) -> f64) -> Self {
        self.default_curve = default_curve;
        self
    }

    /// Sets the default interpolation used by `.to()`/`.to_with_curve()` for
    /// any stop appended after this call — mirrors [`Keyframes::curve`]: a
    /// per-leaf default that individual stops can still override via
    /// `.to_with_interpolation()`/`.to_custom()`.
    pub fn interpolation(mut self, default_interpolation: fn(&T, &T, f64) -> T) -> Self {
        self.default_interpolation = default_interpolation;
        self
    }

    /// Append a stop: default curve, default interpolation.
    pub fn to(self, value: T, duration: Duration) -> Self {
        let curve = self.default_curve;
        let interpolation = self.default_interpolation;
        self.to_custom(value, duration, curve, interpolation)
    }

    /// Append a stop: custom curve, default interpolation.
    pub fn to_with_curve(self, value: T, duration: Duration, curve: fn(f64) -> f64) -> Self {
        let interpolation = self.default_interpolation;
        self.to_custom(value, duration, curve, interpolation)
    }

    /// Append a stop: default curve, custom interpolation.
    pub fn to_with_interpolation(
        self,
        value: T,
        duration: Duration,
        interpolation: fn(&T, &T, f64) -> T,
    ) -> Self {
        let curve = self.default_curve;
        self.to_custom(value, duration, curve, interpolation)
    }

    /// Append a stop: both curve and interpolation explicit.
    pub fn to_custom(
        mut self,
        value: T,
        duration: Duration,
        curve: fn(f64) -> f64,
        interpolation: fn(&T, &T, f64) -> T,
    ) -> Self {
        let prev_end = self.cumulative.last().copied().unwrap_or(Duration::ZERO);
        self.cumulative.push(prev_end + duration);
        self.stops.push(KeyframeStop { value, curve, interpolation });
        self
    }

    fn total(&self) -> Duration {
        self.cumulative.last().copied().unwrap_or(Duration::ZERO)
    }
}

impl<T: StateContract> Timeline for Keyframes<T> {
    fn duration(&self) -> Option<Duration> {
        Some(self.total())
    }

    fn seek_duration(&self) -> Duration {
        self.total()
    }

    // `mode` is unused here by design, not by oversight: a leaf has no
    // children and can never be unbounded, so `duration()` and
    // `seek_duration()` always agree (see the invariant test below) — there
    // is nothing for Playback-vs-Seek to disambiguate. The distinction only
    // matters where it originates, at the composite levels (Chain/Tracks/
    // Repeat), which use it to pick between duration() (can be None) and
    // seek_duration() (always finite) when sizing a child's slot.
    fn seek(&mut self, local_time: Duration, _mode: ResolveMode) -> TimelineStatus {
        let total = self.total();

        if self.stops.is_empty() {
            self.state.set_value(self.initial.clone());
            return TimelineStatus::new(1.0, true, local_time);
        }

        let clamped = local_time.min(total);

        // First stop whose cumulative end-time is >= clamped; `.min(len-1)`
        // is a defensive fallback for the exact-boundary case.
        let idx = self.cumulative.partition_point(|&end| end < clamped)
            .min(self.stops.len() - 1);

        let segment_start = if idx == 0 { Duration::ZERO } else { self.cumulative[idx - 1] };
        let segment_len = self.cumulative[idx].saturating_sub(segment_start);

        let from = if idx == 0 { self.initial.clone() } else { self.stops[idx - 1].value.clone() };
        let to = self.stops[idx].value.clone();
        let curve = self.stops[idx].curve;
        let interpolation = self.stops[idx].interpolation;

        let raw_pct = if segment_len.is_zero() {
            1.0
        } else {
            ((clamped - segment_start).as_secs_f64() / segment_len.as_secs_f64()).clamp(0.0, 1.0)
        };

        let eased = curve(raw_pct);
        let value = interpolation(&from, &to, eased);
        self.state.set_value(value);

        TimelineStatus::new(
            if total.is_zero() { 1.0 } else { clamped.as_secs_f64() / total.as_secs_f64() },
            local_time >= total,
            local_time.saturating_sub(total),
        )
    }
}

impl<T: StateContract> Debug for Keyframes<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Keyframes").field("stops", &self.stops.len()).finish()
    }
}

impl<T> Debug for KeyframeStop<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyframeStop").finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use crate::draw::Color;
    use crate::draw::color::{RED, BLUE};
    use crate::state::{LocalState, ReadState};
    use crate::animation::animation_curve::{ease_in_out, linear};

    use super::*;

    /// A type with no `Animatable` impl, to prove `new_custom` genuinely
    /// supports it (unlike `new`, which requires `Animatable<T>`).
    #[derive(Clone, Debug, PartialEq)]
    struct Meters(f64);

    fn interpolate_meters(from: &Meters, to: &Meters, pct: f64) -> Meters {
        Meters(from.0 * (1.0 - pct) + to.0 * pct)
    }

    #[test]
    fn new_custom_supports_types_without_animatable() {
        let state = LocalState::new(Meters(0.0));
        let mut kf = Keyframes::new_custom(state.clone(), Meters(0.0), interpolate_meters)
            .to(Meters(10.0), Duration::from_secs(1));

        kf.seek(Duration::from_millis(500), ResolveMode::Playback);
        assert_eq!(*state.value(), Meters(5.0));
    }

    #[test]
    fn new_custom_default_interpolation_is_overridable_via_builder() {
        fn always_zero(_from: &Meters, _to: &Meters, _pct: f64) -> Meters { Meters(0.0) }

        let state = LocalState::new(Meters(0.0));
        let mut kf = Keyframes::new_custom(state.clone(), Meters(0.0), interpolate_meters)
            .interpolation(always_zero)
            .to(Meters(10.0), Duration::from_secs(1));

        kf.seek(Duration::from_millis(500), ResolveMode::Playback);
        assert_eq!(*state.value(), Meters(0.0));
    }

    #[test]
    fn single_stop_reads_back_initial_and_final_and_midpoint() {
        let state = LocalState::new(0.0f64);
        let mut kf = Keyframes::new(state.clone(), 0.0).to(10.0, Duration::from_secs(1));

        kf.seek(Duration::ZERO, ResolveMode::Playback);
        assert_eq!(*state.value(), 0.0);

        kf.seek(Duration::from_millis(500), ResolveMode::Playback);
        assert_eq!(*state.value(), 5.0);

        kf.seek(Duration::from_secs(1), ResolveMode::Playback);
        assert_eq!(*state.value(), 10.0);
    }

    #[test]
    fn multi_stop_lands_exactly_on_boundaries_and_uses_correct_segment() {
        let state = LocalState::new(0.0f64);
        let mut kf = Keyframes::new(state.clone(), 0.0)
            .to(10.0, Duration::from_secs(1))
            .to(20.0, Duration::from_secs(1))
            .to(0.0, Duration::from_secs(1));

        kf.seek(Duration::from_secs(1), ResolveMode::Playback);
        assert_eq!(*state.value(), 10.0);

        kf.seek(Duration::from_secs(2), ResolveMode::Playback);
        assert_eq!(*state.value(), 20.0);

        // strictly inside segment 2 (10 -> 20): halfway = 15.0
        kf.seek(Duration::from_millis(1500), ResolveMode::Playback);
        assert_eq!(*state.value(), 15.0);

        // strictly inside segment 3 (20 -> 0): halfway = 10.0
        kf.seek(Duration::from_millis(2500), ResolveMode::Playback);
        assert_eq!(*state.value(), 10.0);
    }

    #[test]
    fn default_curve_is_used_by_to_and_overridable_per_segment() {
        let state = LocalState::new(0.0f64);
        fn snap_to_end(_: f64) -> f64 { 1.0 }

        let mut kf = Keyframes::new(state.clone(), 0.0)
            .curve(snap_to_end) // set the default before appending any stops
            .to(10.0, Duration::from_secs(1)) // uses the default (snap_to_end)
            .to_with_curve(20.0, Duration::from_secs(1), linear); // overrides for this segment only

        kf.seek(Duration::from_millis(500), ResolveMode::Playback);
        assert_eq!(*state.value(), 10.0); // snap_to_end, not linear's 5.0

        kf.seek(Duration::from_millis(1500), ResolveMode::Playback);
        assert_eq!(*state.value(), 15.0); // linear override, not snap_to_end's 20.0
    }

    #[test]
    fn per_segment_curve_is_scoped_to_its_own_segment() {
        let state = LocalState::new(0.0f64);
        // segment 1: linear, segment 2: a curve that snaps to the end immediately
        fn snap_to_end(_: f64) -> f64 { 1.0 }
        let mut kf = Keyframes::new(state.clone(), 0.0)
            .to_with_curve(10.0, Duration::from_secs(1), linear)
            .to_with_curve(20.0, Duration::from_secs(1), snap_to_end);

        kf.seek(Duration::from_millis(500), ResolveMode::Playback);
        assert_eq!(*state.value(), 5.0); // linear midpoint

        kf.seek(Duration::from_millis(1500), ResolveMode::Playback);
        assert_eq!(*state.value(), 20.0); // snap_to_end curve, not linear's 15.0
    }

    #[test]
    fn per_segment_interpolation_override_is_actually_used() {
        let state = LocalState::new(RED);
        let mut hsla_leaf = Keyframes::new(state.clone(), RED)
            .to_with_interpolation(BLUE, Duration::from_secs(1), Color::hsla_blend);
        hsla_leaf.seek(Duration::from_millis(500), ResolveMode::Playback);
        let hsla_result = *state.value();

        let rgba_result = Color::rgba_blend(&RED, &BLUE, 0.5);

        assert_ne!(hsla_result, rgba_result);
    }

    #[test]
    fn zero_duration_segment_snaps_immediately_without_panicking() {
        let state = LocalState::new(0.0f64);
        let mut kf = Keyframes::new(state.clone(), 0.0)
            .to(10.0, Duration::ZERO)
            .to(20.0, Duration::from_secs(1));

        kf.seek(Duration::ZERO, ResolveMode::Playback);
        assert_eq!(*state.value(), 10.0);
    }

    #[test]
    fn beyond_the_end_clamps_and_reports_overshoot() {
        let state = LocalState::new(0.0f64);
        let mut kf = Keyframes::new(state.clone(), 0.0).to(10.0, Duration::from_secs(1));

        let status = kf.seek(Duration::from_secs(3), ResolveMode::Playback);
        assert_eq!(*state.value(), 10.0);
        assert!(status.finished);
        assert_eq!(status.overshoot, Duration::from_secs(2));
    }

    #[test]
    fn duration_and_seek_duration_always_agree_for_a_bare_leaf() {
        let state = LocalState::new(0.0f64);
        let kf = Keyframes::new(state.clone(), 0.0)
            .to(10.0, Duration::from_secs(1))
            .to_with_curve(20.0, Duration::from_secs(2), ease_in_out);

        assert_eq!(kf.duration(), Some(kf.seek_duration()));
        assert_eq!(kf.duration(), Some(Duration::from_secs(3)));
    }
}
