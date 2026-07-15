use crate::animation::keyframes::timeline::{ResolveMode, Timeline, TimelineStatus};
use crate::state::RepeatMode;
use crate::time::Duration;

/// Wraps one child [`Timeline`] and adds repeat/alternate/count semantics to
/// it — a wrapper node, not a built-in field on every timeline type. This is
/// what makes repeat compose to arbitrary depth for free: wrap **one
/// child** before pushing it into a [`Chain`](crate::animation::keyframes::Chain)/
/// [`Tracks`](crate::animation::keyframes::Tracks) for *per-child* looping,
/// or wrap an **entire** `Chain`/`Tracks` for *whole-group* looping — zero
/// changes needed to `Chain`/`Tracks`/`Keyframes` either way, and both can
/// coexist independently in the same tree, because a `Repeat` only ever asks
/// its child two black-box questions (`duration()`/`seek_duration()`, and
/// `seek(t, mode)`) and never introspects what's inside.
#[derive(Debug)]
pub struct Repeat {
    child: Box<dyn Timeline>,
    mode: RepeatMode,
    /// `None` = unbounded; `Some(n)` = n directional passes (e.g. under
    /// `Alternate`, `count: Some(3)` means forward, back, forward).
    count: Option<u32>,
}

impl Repeat {
    pub fn forever(child: impl Timeline + 'static, mode: RepeatMode) -> Self {
        Repeat { child: Box::new(child), mode, count: None }
    }

    pub fn counted(child: impl Timeline + 'static, mode: RepeatMode, count: u32) -> Self {
        Repeat { child: Box::new(child), mode, count: Some(count) }
    }
}

impl Timeline for Repeat {
    fn duration(&self) -> Option<Duration> {
        match self.count {
            None => None,
            Some(count) => self.child.duration().map(|d| d.mul_f64(count as f64)),
        }
    }

    fn seek_duration(&self) -> Duration {
        self.child.seek_duration().mul_f64(self.count.unwrap_or(1) as f64)
    }

    fn seek(&mut self, local_time: Duration, mode: ResolveMode) -> TimelineStatus {
        // The length of one pass through the child. In Playback mode this is
        // the child's own duration() (None => the child itself never
        // settles, so we forward local_time straight through — an
        // unboundedly-repeating wrapper around an already-unbounded child is
        // still unbounded, trivially, and there's no single pass to fold
        // against). In Seek mode this is always the child's seek_duration()
        // (finite), matching every other composite's mode-dispatch.
        let width = match mode {
            ResolveMode::Playback => self.child.duration(),
            ResolveMode::Seek => Some(self.child.seek_duration()),
        };

        let width = match width {
            Some(w) => w,
            None => return self.child.seek(local_time, mode),
        };

        if width.is_zero() {
            self.child.seek(Duration::ZERO, mode);
            return TimelineStatus::new(1.0, true, local_time);
        }

        // Degenerate: RepeatMode::None on a Repeat wrapper is a contradiction
        // in terms (None means "don't repeat"). Treat it as exactly one
        // forward pass regardless of `count`, matching an unwrapped child
        // exactly, so the combination fails safely rather than doing
        // something undocumented.
        if matches!(self.mode, RepeatMode::None) {
            let clamped = local_time.min(width);
            let status = self.child.seek(clamped, mode);
            return TimelineStatus::new(status.progress, local_time >= width, local_time.saturating_sub(width));
        }

        // Degenerate: zero repeats requested — instantly finished at rest.
        if self.count == Some(0) {
            self.child.seek(Duration::ZERO, mode);
            return TimelineStatus::new(1.0, true, local_time);
        }

        let raw = local_time.as_secs_f64() / width.as_secs_f64();

        let (iteration, frac, exhausted) = match self.count {
            None => (raw.floor() as u32, raw - raw.floor(), false),
            Some(count) if raw >= count as f64 => (count - 1, 1.0, true),
            Some(_) => (raw.floor() as u32, raw - raw.floor(), false),
        };

        let forward = match self.mode {
            RepeatMode::Alternate => iteration % 2 == 0,
            _ => true, // FromBeginning is always forward (hard jump-cut wrap)
        };

        let local = if exhausted {
            if forward { width } else { Duration::ZERO }
        } else if forward {
            width.mul_f64(frac)
        } else {
            width.mul_f64(1.0 - frac)
        };

        let status = self.child.seek(local, mode);

        let overall_progress = match self.count {
            Some(count) => ((iteration as f64 + frac) / count as f64).clamp(0.0, 1.0),
            None => status.progress, // unbounded: report the child's own within-pass progress
        };

        let overshoot = match self.count {
            Some(count) if exhausted => local_time.saturating_sub(width.mul_f64(count as f64)),
            _ => Duration::ZERO,
        };

        TimelineStatus::new(overall_progress, exhausted, overshoot)
    }
}

#[cfg(test)]
mod tests {
    use crate::animation::keyframes::{Chain, Keyframes, Tracks};
    use crate::state::{LocalState, ReadState};

    use super::*;

    fn kf(state: &LocalState<f64>, initial: f64, to: f64, dur: Duration) -> Keyframes<f64> {
        Keyframes::new(state.clone(), initial).to(to, dur)
    }

    #[test]
    fn counted_alternate_matches_hand_validated_table() {
        let state = LocalState::new(0.0f64);
        let mut repeat = Repeat::counted(kf(&state, 0.0, 1.0, Duration::from_secs(1)), RepeatMode::Alternate, 3);

        repeat.seek(Duration::ZERO, ResolveMode::Playback);
        assert_eq!(*state.value(), 0.0); // t=0: from

        repeat.seek(Duration::from_millis(500), ResolveMode::Playback);
        assert_eq!(*state.value(), 0.5); // mid first pass (forward)

        repeat.seek(Duration::from_millis(1250), ResolveMode::Playback);
        assert_eq!(*state.value(), 0.75); // mid second pass (mirrored: 1 - 0.25)

        let status = repeat.seek(Duration::from_secs(3), ResolveMode::Playback);
        assert_eq!(*state.value(), 1.0); // final rest value: 3 passes end forward, at "to"
        assert!(status.finished);
    }

    #[test]
    fn forever_from_beginning_is_periodic() {
        let state = LocalState::new(0.0f64);
        let mut repeat = Repeat::forever(kf(&state, 0.0, 1.0, Duration::from_secs(1)), RepeatMode::FromBeginning);

        assert_eq!(repeat.duration(), None);
        assert_eq!(repeat.seek_duration(), Duration::from_secs(1));

        repeat.seek(Duration::from_millis(300), ResolveMode::Playback);
        let a = *state.value();
        repeat.seek(Duration::from_millis(1300), ResolveMode::Playback);
        let b = *state.value();
        assert_eq!(a, b);
    }

    #[test]
    fn forever_alternate_ping_pongs_over_a_full_cycle() {
        let state = LocalState::new(0.0f64);
        let mut repeat = Repeat::forever(kf(&state, 0.0, 1.0, Duration::from_secs(1)), RepeatMode::Alternate);

        repeat.seek(Duration::from_millis(300), ResolveMode::Playback);
        let forward_val = *state.value();

        // 2 * leaf_duration - t should mirror back to the same value within one period.
        repeat.seek(Duration::from_millis(1700), ResolveMode::Playback);
        let mirrored_val = *state.value();

        assert!((forward_val - mirrored_val).abs() < 1e-9);
    }

    #[test]
    fn nested_repeat_composes_without_special_casing() {
        let inner_state = LocalState::new(0.0f64);
        let outer_state = LocalState::new(0.0f64);

        let inner_repeat = Repeat::forever(
            kf(&inner_state, 0.0, 1.0, Duration::from_millis(500)),
            RepeatMode::FromBeginning,
        );
        let chain = Chain::from_vec(vec![
            Box::new(inner_repeat),
            Box::new(kf(&outer_state, 0.0, 1.0, Duration::from_secs(1))),
        ]);
        // Wrap the (unbounded) chain in an outer Repeat as well — since the
        // chain itself never settles (the inner repeat blocks it), the outer
        // Repeat's own duration() is also None, and its seek algorithm just
        // forwards local_time straight through (the `width == None` early
        // return), delegating entirely to the chain/inner-repeat beneath it.
        let mut outer = Repeat::forever(chain, RepeatMode::Alternate);

        // The inner leaf's own periodic cycle should still be observable
        // through both levels of wrapping.
        outer.seek(Duration::from_millis(100), ResolveMode::Playback);
        let a = *inner_state.value();
        outer.seek(Duration::from_millis(600), ResolveMode::Playback);
        let b = *inner_state.value();
        assert_eq!(a, b); // one inner period later, same value
    }

    #[test]
    fn worked_example_tracks_with_counted_and_plain_leaf_seek_to_50_percent() {
        let x_state = LocalState::new(0.0f64);
        let y_state = LocalState::new(0.0f64);

        let x = Repeat::counted(kf(&x_state, 0.0, 1.0, Duration::from_secs(1)), RepeatMode::FromBeginning, 4);
        let y = kf(&y_state, 0.0, 1.0, Duration::from_secs(10));

        let mut tracks = Tracks::new().and(x).and(y);
        assert_eq!(tracks.seek_duration(), Duration::from_secs(10));

        // seek(0.5) => target = 5s absolute.
        tracks.seek(Duration::from_secs(5), ResolveMode::Seek);

        // X (4 * 1s = 4s total) is already exhausted by 5s, resting at "to".
        assert_eq!(*x_state.value(), 1.0);
        // Y (10s, no repeat) is at 50%.
        assert_eq!(*y_state.value(), 0.5);
    }

    #[test]
    fn degenerate_none_mode_behaves_as_single_pass() {
        let wrapped_state = LocalState::new(0.0f64);
        let bare_state = LocalState::new(0.0f64);

        let mut wrapped = Repeat::counted(kf(&wrapped_state, 0.0, 1.0, Duration::from_secs(1)), RepeatMode::None, 5);
        let mut bare = kf(&bare_state, 0.0, 1.0, Duration::from_secs(1));

        for t in [0, 250, 500, 999, 1000, 1500] {
            wrapped.seek(Duration::from_millis(t), ResolveMode::Playback);
            bare.seek(Duration::from_millis(t), ResolveMode::Playback);
            assert_eq!(*wrapped_state.value(), *bare_state.value());
        }
    }

    #[test]
    fn count_zero_is_instantly_finished_at_rest() {
        let state = LocalState::new(0.0f64);
        let mut repeat = Repeat::counted(kf(&state, 0.0, 1.0, Duration::from_secs(1)), RepeatMode::FromBeginning, 0);

        let status = repeat.seek(Duration::ZERO, ResolveMode::Playback);
        assert!(status.finished);
    }
}
