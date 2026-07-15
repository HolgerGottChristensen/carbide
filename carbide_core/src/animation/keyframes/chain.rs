use crate::animation::keyframes::timeline::{ResolveMode, Timeline, TimelineStatus};
use crate::time::Duration;

/// Sequential composition: children play back to back. Named `Chain` rather
/// than `Sequence` to avoid colliding with the pre-existing
/// `carbide_core::widget::common::widget_sequence::Sequence` trait, which is
/// commonly in scope via `use carbide_core::widget::*;`.
///
/// Exposes two first-class construction paths: the fluent `.then()` builder
/// for small, hand-written, known-at-compile-time groups, and
/// [`Chain::from_vec`] for programmatically-built groups (e.g. N staggered
/// steps generated in a loop) — both produce identical results since
/// `from_vec` is implemented in terms of `.then_boxed()`.
#[derive(Debug, Default)]
pub struct Chain {
    children: Vec<Box<dyn Timeline>>,
    /// Cached sum of children's `duration()`; `None` if any child is `None`
    /// (an unbounded child blocks the cursor — later siblings are
    /// unreachable via live playback, matching the permissive-mid-chain
    /// decision).
    duration: Option<Duration>,
    /// Cached sum of children's `seek_duration()` — always finite, so
    /// `seek()` can still reach children after an unbounded one.
    seek_duration: Duration,
}

impl Chain {
    pub fn new() -> Self {
        Chain { children: Vec::new(), duration: Some(Duration::ZERO), seek_duration: Duration::ZERO }
    }

    pub fn then(self, child: impl Timeline + 'static) -> Self {
        self.then_boxed(Box::new(child))
    }

    /// Escape hatch for appending a child that's already boxed (used by
    /// [`Chain::from_vec`], and available directly for dynamic construction).
    pub fn then_boxed(mut self, child: Box<dyn Timeline>) -> Self {
        self.duration = match (self.duration, child.duration()) {
            (Some(a), Some(b)) => Some(a + b),
            _ => None,
        };
        self.seek_duration += child.seek_duration();
        self.children.push(child);
        self
    }

    /// Construct directly from a `Vec` of already-boxed children — the
    /// natural fit for programmatically-built groups where the number of
    /// children isn't known until runtime.
    pub fn from_vec(children: Vec<Box<dyn Timeline>>) -> Self {
        let mut chain = Chain::new();
        for child in children {
            chain = chain.then_boxed(child);
        }
        chain
    }
}

impl Timeline for Chain {
    fn duration(&self) -> Option<Duration> {
        self.duration
    }

    fn seek_duration(&self) -> Duration {
        self.seek_duration
    }

    fn seek(&mut self, local_time: Duration, mode: ResolveMode) -> TimelineStatus {
        let total = match mode {
            ResolveMode::Playback => self.duration,
            ResolveMode::Seek => Some(self.seek_duration),
        };

        let clamped = match total {
            Some(t) => local_time.min(t),
            None => local_time,
        };

        let mut cursor = Duration::ZERO;
        // Once true, every remaining child is "not yet reached" (either
        // because we've already dispatched into the active child, or
        // because an earlier unbounded child in Playback mode blocks the
        // cursor from ever advancing past it).
        let mut blocked = false;
        let n = self.children.len();
        // Remember which child is active and what local_time we fed it, so
        // it can be re-applied last (see below) — its write must win even
        // if a later "not yet reached" sibling shares its bound target.
        let mut active: Option<(usize, Duration)> = None;

        for (i, child) in self.children.iter_mut().enumerate() {
            if blocked {
                child.seek(Duration::ZERO, mode);
                continue;
            }

            let width = match mode {
                ResolveMode::Playback => child.duration(),
                ResolveMode::Seek => Some(child.seek_duration()),
            };

            match width {
                None => {
                    let local = clamped.saturating_sub(cursor);
                    child.seek(local, mode);
                    active = Some((i, local));
                    blocked = true;
                }
                Some(w) => {
                    let is_last = i == n - 1;
                    let end = cursor + w;
                    // Half-open [cursor, end) for every slot except the
                    // last, which is closed [cursor, end] so the final
                    // resting frame at local_time == total is observable
                    // rather than skipped past.
                    let in_slot = if is_last { clamped <= end } else { clamped < end };

                    if in_slot {
                        let local = clamped.saturating_sub(cursor).min(w);
                        child.seek(local, mode);
                        active = Some((i, local));
                        blocked = true;
                    } else {
                        child.seek(w, mode);
                        cursor = end;
                    }
                }
            }
        }

        // Re-apply the active child's seek last, after every "not yet
        // reached" sibling has had its (harmless-in-isolation) turn. Without
        // this, an active child mid-flight can have its correct value
        // silently overwritten by a later sibling's "reset to my own start"
        // write, whenever the two happen to share the same bound `State`
        // target (a natural pattern — see `Chain` nesting `Tracks` that both
        // animate the same property across sequential steps).
        if let Some((idx, local)) = active {
            self.children[idx].seek(local, mode);
        }

        TimelineStatus::new(
            match total {
                Some(t) if !t.is_zero() => clamped.as_secs_f64() / t.as_secs_f64(),
                Some(_) => 1.0,
                None => 0.0,
            },
            match total {
                Some(t) => local_time >= t,
                None => false,
            },
            match total {
                Some(t) => local_time.saturating_sub(t),
                None => Duration::ZERO,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::animation::keyframes::{Delay, Keyframes, Repeat, Tracks};
    use crate::state::{LocalState, ReadState, RepeatMode};

    use super::*;

    #[test]
    fn chain_can_hold_tracks_as_children() {
        let offset_state = LocalState::new(0.0f64);
        let color_state = LocalState::new(0.0f64); // stand-in for a Color state

        let intro = Tracks::new()
            .and(Keyframes::new(offset_state.clone(), 0.0).to(10.0, Duration::from_secs(1)))
            .and(Keyframes::new(color_state.clone(), 0.0).to(100.0, Duration::from_secs(1)));
        let outro = Tracks::new()
            .and(Keyframes::new(offset_state.clone(), 10.0).to(0.0, Duration::from_secs(1)))
            .and(Keyframes::new(color_state.clone(), 100.0).to(0.0, Duration::from_secs(1)));

        let mut chain = Chain::new().then(intro).then(outro);
        assert_eq!(chain.duration(), Some(Duration::from_secs(2))); // 1s intro + 1s outro

        // Halfway through the intro Tracks.
        chain.seek(Duration::from_millis(500), ResolveMode::Playback);
        assert_eq!(*offset_state.value(), 5.0);
        assert_eq!(*color_state.value(), 50.0);

        // Halfway through the outro Tracks (intro has fully settled by then).
        chain.seek(Duration::from_millis(1500), ResolveMode::Playback);
        assert_eq!(*offset_state.value(), 5.0);
        assert_eq!(*color_state.value(), 50.0);
    }

    #[test]
    fn two_bounded_children_duration_is_sum_and_offset_is_correct() {
        let a_state = LocalState::new(0.0f64);
        let b_state = LocalState::new(0.0f64);
        let a = Keyframes::new(a_state.clone(), 0.0).to(10.0, Duration::from_secs(1));
        let b = Keyframes::new(b_state.clone(), 0.0).to(10.0, Duration::from_secs(1));

        let mut chain = Chain::new().then(a).then(b);
        assert_eq!(chain.duration(), Some(Duration::from_secs(2)));

        // Inside child b's slot, offset by 1s: local_time 1.5s -> child b at 0.5s -> 5.0
        chain.seek(Duration::from_millis(1500), ResolveMode::Playback);
        assert_eq!(*b_state.value(), 5.0);
    }

    #[test]
    fn active_childs_write_wins_over_a_shared_not_yet_reached_sibling() {
        // Regression test for a real bug: a "not yet reached" sibling's
        // "reset to my own start" write was applied AFTER the active
        // child's write (pure iteration order), so whenever both targeted
        // the same shared `State`, the not-yet-reached sibling silently
        // clobbered the active child's correct, in-flight value — visible
        // only when sampled strictly mid-flight (not at an exact boundary,
        // which is why earlier tests didn't catch it).
        let shared = LocalState::new(0.0f64);
        let a = Keyframes::new(shared.clone(), 0.0).to(10.0, Duration::from_millis(250));
        let b = Delay::new(Duration::from_millis(800));
        let c = Keyframes::new(shared.clone(), 10.0).to(0.0, Duration::from_millis(250));

        let mut chain = Chain::new().then(a).then(b).then(c);

        // Strictly mid-`a` (40% through its 250ms span): a is active, c is
        // not yet reached but shares `shared` and would reset it to c's own
        // "at rest" value (10.0, matching a's *final* value) if c's write
        // were allowed to land after a's.
        chain.seek(Duration::from_millis(100), ResolveMode::Playback);
        assert_eq!(*shared.value(), 4.0);
    }

    #[test]
    fn scrubbing_correctness_not_yet_reached_and_already_passed() {
        let a_state = LocalState::new(0.0f64);
        let b_state = LocalState::new(0.0f64);
        let a = Keyframes::new(a_state.clone(), 0.0).to(10.0, Duration::from_secs(1));
        let b = Keyframes::new(b_state.clone(), 0.0).to(20.0, Duration::from_secs(1));

        let mut chain = Chain::new().then(a).then(b);

        // Inside child a: b is not yet reached -> seeked to its start (0.0).
        chain.seek(Duration::from_millis(500), ResolveMode::Playback);
        assert_eq!(*b_state.value(), 0.0);

        // Inside child b: a is already passed -> seeked to its own end (10.0).
        chain.seek(Duration::from_millis(1500), ResolveMode::Playback);
        assert_eq!(*a_state.value(), 10.0);
    }

    #[test]
    fn delay_as_middle_child_holds_previous_value_then_advances() {
        let color_state = LocalState::new(0.0f64);
        let a = Keyframes::new(color_state.clone(), 0.0).to(10.0, Duration::from_millis(250));
        let b = Delay::new(Duration::from_millis(800));
        let c = Keyframes::new(color_state.clone(), 10.0).to(0.0, Duration::from_millis(250));

        let mut chain = Chain::new().then(a).then(b).then(c);

        chain.seek(Duration::from_millis(250), ResolveMode::Playback);
        assert_eq!(*color_state.value(), 10.0);

        // Sample several points inside the delay window: value holds at 10.0.
        chain.seek(Duration::from_millis(400), ResolveMode::Playback);
        assert_eq!(*color_state.value(), 10.0);
        chain.seek(Duration::from_millis(700), ResolveMode::Playback);
        assert_eq!(*color_state.value(), 10.0);
        chain.seek(Duration::from_millis(1000), ResolveMode::Playback);
        assert_eq!(*color_state.value(), 10.0);

        // Right after the delay window ends, child c begins.
        chain.seek(Duration::from_millis(1175), ResolveMode::Playback);
        assert_eq!(*color_state.value(), 5.0);
    }

    #[test]
    fn unbounded_non_last_child_blocks_playback_but_is_reachable_via_seek() {
        let a_state = LocalState::new(0.0f64);
        let b_state = LocalState::new(0.0f64);
        let repeat_a = Repeat::forever(
            Keyframes::new(a_state.clone(), 0.0).to(360.0, Duration::from_secs(1)),
            RepeatMode::FromBeginning,
        );
        let leaf_b = Keyframes::new(b_state.clone(), 0.0).to(999.0, Duration::from_secs(1));

        let mut chain = Chain::from_vec(vec![
            Box::new(repeat_a),
            Box::new(leaf_b),
        ]);

        assert_eq!(chain.duration(), None);
        assert_eq!(chain.seek_duration(), Duration::from_secs(2)); // one representative loop + leaf_b

        // Far beyond leaf_a's single-loop duration: still animating leaf_a's
        // own loop, leaf_b's target is never touched in Playback mode.
        chain.seek(Duration::from_secs(50), ResolveMode::Playback);
        assert_eq!(*b_state.value(), 0.0);

        // Explicit seek past the one-representative-loop boundary DOES reach leaf_b.
        chain.seek(Duration::from_millis(1500), ResolveMode::Seek);
        assert_ne!(*b_state.value(), 0.0);
    }

    #[test]
    fn from_vec_and_builder_agree() {
        let a1 = LocalState::new(0.0f64);
        let b1 = LocalState::new(0.0f64);
        let mut builder_chain = Chain::new()
            .then(Keyframes::new(a1.clone(), 0.0).to(10.0, Duration::from_secs(1)))
            .then(Keyframes::new(b1.clone(), 0.0).to(20.0, Duration::from_secs(1)));

        let a2 = LocalState::new(0.0f64);
        let b2 = LocalState::new(0.0f64);
        let mut vec_chain = Chain::from_vec(vec![
            Box::new(Keyframes::new(a2.clone(), 0.0).to(10.0, Duration::from_secs(1))),
            Box::new(Keyframes::new(b2.clone(), 0.0).to(20.0, Duration::from_secs(1))),
        ]);

        assert_eq!(builder_chain.duration(), vec_chain.duration());
        assert_eq!(builder_chain.seek_duration(), vec_chain.seek_duration());

        for t in [0, 300, 1000, 1300, 2000] {
            builder_chain.seek(Duration::from_millis(t), ResolveMode::Playback);
            vec_chain.seek(Duration::from_millis(t), ResolveMode::Playback);
            assert_eq!(*a1.value(), *a2.value());
            assert_eq!(*b1.value(), *b2.value());
        }
    }
}
