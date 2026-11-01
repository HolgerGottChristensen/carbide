use std::env;
use std::path::{Path, PathBuf};

/// Structural pixel comparison, via the `image_compare` crate's `rgba_hybrid_compare` (SSIM on
/// luma, RMS on chroma/alpha). Every mature prior-art system researched for this fixture
/// (Servo/WebRender's `wrench`, egui_kittest, wgpu's own CTS harness) avoids exact byte matches —
/// AA/font-hinting/driver differences make that flaky even on one machine. A raw per-channel delta
/// tolerance still flags every one of those pixels as "different," just below a cutoff; SSIM
/// instead judges a small neighborhood at once, so a region that's merely shifted by AA/hinting
/// noise scores as structurally the same while a genuinely wrong pixel block still scores low.
pub struct SnapshotOptions {
    /// Minimum acceptable `image_compare::Similarity::score` — 1.0 is pixel-for-pixel identical,
    /// 0.0 is completely dissimilar.
    pub min_similarity: f64,
}

impl Default for SnapshotOptions {
    fn default() -> Self {
        SnapshotOptions {
            min_similarity: 0.98,
        }
    }
}

/// Anchored at compile time via `CARGO_MANIFEST_DIR`, independent of `cargo test`'s cwd.
/// `scope` (typically the current test's name — see `harness::current_test_scope`) nests golden
/// files under a per-test subdirectory, so two different tests can each use a snapshot named
/// `"idle"` without colliding; `None` falls back to the flat, unscoped layout.
fn snapshot_dir(scope: Option<&str>) -> PathBuf {
    let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots");
    match scope {
        Some(scope) => base.join(scope),
        None => base,
    }
}

fn update_requested() -> bool {
    env::var("UPDATE_SNAPSHOTS").is_ok()
}

/// Below this, on any of the deviation map's R/G/B channels (structure/chroma-U/chroma-V,
/// 0 = identical, 255 = maximally different — see `image_compare`'s `to_color_map`), a pixel is
/// still considered visually matching for highlighting purposes; this is purely about what the
/// `.diff.png` visualization highlights, independent of `SnapshotOptions::min_similarity`, which
/// gates pass/fail on the aggregate score instead.
const DIFF_HIGHLIGHT_THRESHOLD: u8 = 8;

/// For each pixel, if the structural deviation map (from `comparison.image.to_color_map()`) shows
/// any channel above `DIFF_HIGHLIGHT_THRESHOLD`, output a solid alert color (magenta) at full
/// opacity; otherwise output a dimmed/desaturated version of the golden pixel (~40% brightness
/// grayscale). This makes mismatched regions pop against a muted, still-recognizable background
/// rather than requiring a reviewer to flip between two full-color images and spot the difference
/// by eye — the same technique most reftest tooling (e.g. `wrench`) uses, simplified to a
/// threshold-based highlight instead of a continuous heatmap.
fn diff_image(golden: &image::RgbaImage, comparison: &image_compare::Similarity) -> image::RgbaImage {
    let deviation = comparison.image.to_color_map().into_rgba8();

    image::RgbaImage::from_fn(golden.width(), golden.height(), |x, y| {
        let d = deviation.get_pixel(x, y);
        let differs = d[0] > DIFF_HIGHLIGHT_THRESHOLD || d[1] > DIFF_HIGHLIGHT_THRESHOLD || d[2] > DIFF_HIGHLIGHT_THRESHOLD;

        if differs {
            image::Rgba([255, 0, 255, 255])
        } else {
            let g = golden.get_pixel(x, y);
            let gray = (0.299 * g[0] as f32 + 0.587 * g[1] as f32 + 0.114 * g[2] as f32) as u8;
            let dimmed = ((gray as u16) * 40 / 100) as u8;
            image::Rgba([dimmed, dimmed, dimmed, 255])
        }
    })
}

/// Pixel-snapshot comparison. On mismatch, writes `{name}.new.png` (the actual output) and
/// `{name}.diff.png` (a visualization, see `diff_image`) next to the golden file. `scope` nests
/// the golden under a per-test subdirectory — see `snapshot_dir`.
pub(crate) fn compare_and_report(name: &str, actual: &image::RgbaImage, options: &SnapshotOptions, scope: Option<&str>) -> Result<(), String> {
    compare_and_report_in(&snapshot_dir(scope), name, actual, options, update_requested())
}

/// The actual comparison logic, taking `dir` and `update` explicitly rather than deriving them
/// from `CARGO_MANIFEST_DIR` and the `UPDATE_SNAPSHOTS` env var internally (`compare_and_report`
/// does that). Split out so unit tests (see `tests` below) can exercise both the "update" and
/// "compare" branches deterministically against a scratch directory — reading/writing a
/// process-wide env var from a test would race against every other test in the same binary
/// running concurrently on another thread, since `UPDATE_SNAPSHOTS` isn't thread-local.
///
/// UPDATE_SNAPSHOTS's overwrite (`update == true`) intentionally never reads the old golden's
/// dimensions before overwriting — a widget under test is allowed to legitimately change size
/// (a layout tweak, a new frame size in the test itself) between one snapshot run and the next,
/// and running with `UPDATE_SNAPSHOTS=true` after such a change should just accept the new size,
/// the same as it accepts any other pixel difference, rather than requiring the old golden to be
/// deleted by hand first.
/// A prior failing run may have left `.new.png`/`.diff.png` behind. Called both when `update`
/// just brought the golden in line with `actual`, and when a plain (non-update) comparison
/// simply passes on its own (e.g. the underlying rendering bug that caused the earlier failure
/// got fixed by an unrelated code change) — in both cases the leftover files are now stale and,
/// left in place, would look like an unresolved mismatch even though there isn't one anymore.
fn remove_stale_png_artifacts(dir: &Path, name: &str) {
    let _ = std::fs::remove_file(dir.join(format!("{name}.new.png")));
    let _ = std::fs::remove_file(dir.join(format!("{name}.diff.png")));
}

fn compare_and_report_in(dir: &Path, name: &str, actual: &image::RgbaImage, options: &SnapshotOptions, update: bool) -> Result<(), String> {
    let golden_path = dir.join(format!("{name}.png"));

    if update {
        std::fs::create_dir_all(dir).map_err(|e| format!("snapshot {name:?}: could not create {}: {e}", dir.display()))?;
        actual.save(&golden_path).map_err(|e| format!("snapshot {name:?}: could not write {}: {e}", golden_path.display()))?;
        remove_stale_png_artifacts(dir, name);
        return Ok(());
    }

    let golden = image::open(&golden_path)
        .map_err(|_| format!(
            "snapshot {name:?}: no golden image at {} — run with UPDATE_SNAPSHOTS=true to create it",
            golden_path.display()
        ))?
        .into_rgba8();

    if golden.dimensions() != actual.dimensions() {
        // No `diff_image` here — `image_compare::rgba_hybrid_compare` (and any other per-pixel
        // comparison) requires matching dimensions, so there's no meaningful magenta/dimmed
        // overlay to produce. Still write `.new.png` so a reviewer can look at the resized output
        // directly and decide whether to accept it with `UPDATE_SNAPSHOTS=true`.
        let new_path = dir.join(format!("{name}.new.png"));
        let _ = actual.save(&new_path);
        return Err(format!(
            "snapshot {name:?}: dimensions differ (golden {:?} vs actual {:?}); actual output written to {}",
            golden.dimensions(), actual.dimensions(), new_path.display()
        ));
    }

    let comparison = image_compare::rgba_hybrid_compare(&golden, actual)
        .expect("dimensions were already checked to match above");

    if comparison.score < options.min_similarity {
        let new_path = dir.join(format!("{name}.new.png"));
        let diff_path = dir.join(format!("{name}.diff.png"));
        let _ = actual.save(&new_path);
        let _ = diff_image(&golden, &comparison).save(&diff_path);
        return Err(format!(
            "snapshot {name:?}: structural similarity {:.4} is below the minimum {:.4}; actual output written to {}, diff visualization written to {}",
            comparison.score, options.min_similarity, new_path.display(), diff_path.display()
        ));
    }

    remove_stale_png_artifacts(dir, name);
    Ok(())
}

/// Exact-match JSON comparison, shared by the accessibility-tree snapshot and the
/// render-instruction snapshot — one comparison mechanism, two content producers. Unlike pixels,
/// this content has no anti-aliasing analog, so exact match (not tolerance) is the right bar.
///
/// `kind` distinguishes the two (`"a11y"`, `"render"`) so the same `name` can be used for both a
/// pixel, an accessibility, and a render-instruction snapshot without colliding: golden files are
/// `{name}.{kind}.json`. On mismatch, writes `{name}.{kind}.new.json` — `.new` immediately before
/// the extension, matching the pixel snapshot's `{name}.new.png` convention, rather than
/// `{name}.{kind}.json.new`. `scope` nests the golden under a per-test subdirectory — see
/// `snapshot_dir`.
pub(crate) fn compare_text_and_report(name: &str, actual: &str, kind: &str, scope: Option<&str>) -> Result<(), String> {
    compare_text_and_report_in(&snapshot_dir(scope), name, actual, kind, update_requested())
}

/// See `remove_stale_png_artifacts` for the reasoning — same idea, one file instead of two.
fn remove_stale_json_artifact(dir: &Path, name: &str, kind: &str) {
    let _ = std::fs::remove_file(dir.join(format!("{name}.{kind}.new.json")));
}

/// See `compare_and_report_in` for why `dir`/`update` are taken explicitly instead of derived
/// internally — same reasoning, testability without touching the process-wide env var.
fn compare_text_and_report_in(dir: &Path, name: &str, actual: &str, kind: &str, update: bool) -> Result<(), String> {
    let golden_path = dir.join(format!("{name}.{kind}.json"));

    if update {
        std::fs::create_dir_all(dir).map_err(|e| format!("snapshot {name:?}: could not create {}: {e}", dir.display()))?;
        std::fs::write(&golden_path, actual).map_err(|e| format!("snapshot {name:?}: could not write {}: {e}", golden_path.display()))?;
        remove_stale_json_artifact(dir, name, kind);
        return Ok(());
    }

    let golden = std::fs::read_to_string(&golden_path)
        .map_err(|_| format!(
            "snapshot {name:?}: no golden file at {} — run with UPDATE_SNAPSHOTS=true to create it",
            golden_path.display()
        ))?;

    if golden != actual {
        let new_path = dir.join(format!("{name}.{kind}.new.json"));
        let _ = std::fs::write(&new_path, actual);
        return Err(format!(
            "snapshot {name:?}: content differs from golden; actual output written to {}",
            new_path.display()
        ));
    }

    remove_stale_json_artifact(dir, name, kind);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each test gets its own directory under `std::env::temp_dir()`, keyed by the calling test's
    /// own name — distinct from `snapshot_dir`'s real `tests/snapshots/` (anchored on
    /// `CARGO_MANIFEST_DIR`), so these tests never touch real golden files. Removed both before
    /// (in case a previous run of the same test panicked mid-way and left something behind) and
    /// after use.
    fn scratch_dir(test_name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("carbide_wgpu_snapshot_test_{test_name}"));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    fn solid(width: u32, height: u32, pixel: [u8; 4]) -> image::RgbaImage {
        image::RgbaImage::from_pixel(width, height, image::Rgba(pixel))
    }

    /// The behavior this test exists to lock in: running with `UPDATE_SNAPSHOTS` set (`update:
    /// true`) after a widget's rendered size has legitimately changed should just replace the
    /// golden with the new, differently-sized output — not fail, and not require deleting the old
    /// golden by hand first. Without `update`, the same size change is (correctly) still reported
    /// as a mismatch, since that path is for detecting unintentional differences.
    #[test]
    fn update_overwrites_golden_even_when_dimensions_change() {
        let dir = scratch_dir("update_overwrites_golden_even_when_dimensions_change");
        let options = SnapshotOptions::default();

        let small = solid(4, 4, [10, 20, 30, 255]);
        let big = solid(8, 6, [40, 50, 60, 255]);

        // Seed the initial golden, as if from an earlier `UPDATE_SNAPSHOTS=true` run.
        compare_and_report_in(&dir, "widget", &small, &options, true)
            .expect("seeding the initial golden should succeed");

        let golden_path = dir.join("widget.png");
        assert_eq!(
            image::open(&golden_path).unwrap().into_rgba8().dimensions(),
            (4, 4),
            "sanity check: the golden should have been written at the seeded size"
        );

        // Without `update`, a real size change is a real mismatch — this should fail, not
        // silently pass or silently adopt the new size.
        let err = compare_and_report_in(&dir, "widget", &big, &options, false)
            .expect_err("a dimension change should be reported as a mismatch when not updating");
        assert!(
            err.contains("dimensions differ"),
            "expected a dimensions-differ message, got: {err}"
        );

        // That failed comparison should have left a `.new.png` behind — sanity check before the
        // real assertion below, which is that `update` cleans this up.
        assert!(dir.join("widget.new.png").exists());

        // With `update`, the same dimension change should be accepted and applied.
        compare_and_report_in(&dir, "widget", &big, &options, true)
            .expect("updating should accept a legitimately resized snapshot");

        let updated = image::open(&golden_path).unwrap().into_rgba8();
        assert_eq!(updated.dimensions(), (8, 6), "the golden should now reflect the new size");
        assert_eq!(*updated.get_pixel(0, 0), image::Rgba([40, 50, 60, 255]), "and the new content");

        // The stale `.new.png` from the failed comparison above is now identical to the golden
        // and would be misleading left in place — `update` should have removed it.
        assert!(!dir.join("widget.new.png").exists(), "update should clean up the now-stale .new.png");

        // And a subsequent non-update comparison against the now-updated golden, with a
        // matching-size, matching-content image, should simply pass.
        compare_and_report_in(&dir, "widget", &big, &options, false)
            .expect("comparing the same image the golden was just updated to should pass");

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Same cleanup guarantee as above, but for the ordinary same-size pixel-mismatch path, which
    /// writes both `.new.png` *and* `.diff.png` (dimension mismatches, tested above, only ever
    /// write `.new.png` — there's no diff visualization possible between differently-sized
    /// images).
    #[test]
    fn update_cleans_up_stale_new_and_diff_artifacts() {
        let dir = scratch_dir("update_cleans_up_stale_new_and_diff_artifacts");
        let options = SnapshotOptions::default();

        let golden_image = solid(4, 4, [0, 0, 0, 255]);
        let changed = solid(4, 4, [255, 255, 255, 255]);

        compare_and_report_in(&dir, "widget", &golden_image, &options, true)
            .expect("seeding the initial golden should succeed");

        compare_and_report_in(&dir, "widget", &changed, &options, false)
            .expect_err("a large same-size pixel change should be reported as a mismatch");

        assert!(dir.join("widget.new.png").exists());
        assert!(dir.join("widget.diff.png").exists());

        compare_and_report_in(&dir, "widget", &changed, &options, true)
            .expect("updating should accept the changed content");

        assert!(!dir.join("widget.new.png").exists(), "update should clean up the now-stale .new.png");
        assert!(!dir.join("widget.diff.png").exists(), "update should clean up the now-stale .diff.png");

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The scenario this test exists to lock in: a test fails once (writing `.new.png`/
    /// `.diff.png`), and then — with no `UPDATE_SNAPSHOTS` involved at all — the *same* test
    /// passes on a later run (e.g. a flaky render, or an unrelated code change that happened to
    /// fix the earlier mismatch). The leftover `.new.png`/`.diff.png` from the failing run should
    /// not still be sitting there afterward looking like an unresolved mismatch.
    #[test]
    fn passing_comparison_cleans_up_stale_new_and_diff_artifacts() {
        let dir = scratch_dir("passing_comparison_cleans_up_stale_new_and_diff_artifacts");
        let options = SnapshotOptions::default();

        let golden_image = solid(4, 4, [0, 0, 0, 255]);
        let changed = solid(4, 4, [255, 255, 255, 255]);

        compare_and_report_in(&dir, "widget", &golden_image, &options, true)
            .expect("seeding the initial golden should succeed");

        compare_and_report_in(&dir, "widget", &changed, &options, false)
            .expect_err("a large same-size pixel change should be reported as a mismatch");

        assert!(dir.join("widget.new.png").exists());
        assert!(dir.join("widget.diff.png").exists());

        // No update — just comparing against the golden again, but this time with output that
        // matches it, as if the earlier mismatch had been transient or since fixed elsewhere.
        compare_and_report_in(&dir, "widget", &golden_image, &options, false)
            .expect("comparing matching content against the golden should pass");

        assert!(!dir.join("widget.new.png").exists(), "a passing comparison should clean up the now-stale .new.png");
        assert!(!dir.join("widget.diff.png").exists(), "a passing comparison should clean up the now-stale .diff.png");

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A dimension mismatch can't produce the usual magenta/dimmed diff visualization (that
    /// requires equal dimensions to compare pixel-for-pixel), but it should still write
    /// `{name}.new.png` — a reviewer deciding whether to accept a resize wants to see what the
    /// new output actually looks like, the same as for any other kind of mismatch.
    #[test]
    fn dimension_mismatch_without_update_still_writes_new_png() {
        let dir = scratch_dir("dimension_mismatch_without_update_still_writes_new_png");
        let options = SnapshotOptions::default();

        compare_and_report_in(&dir, "widget", &solid(4, 4, [1, 2, 3, 255]), &options, true)
            .expect("seeding the initial golden should succeed");

        let big = solid(8, 6, [4, 5, 6, 255]);
        let err = compare_and_report_in(&dir, "widget", &big, &options, false).expect_err("should fail");
        assert!(err.contains(&dir.join("widget.new.png").display().to_string()), "error should point at the written new.png, got: {err}");

        let new_png = image::open(dir.join("widget.new.png")).unwrap().into_rgba8();
        assert_eq!(new_png.dimensions(), (8, 6));

        // No diff.png — there's nothing meaningful to overlay between two different-sized images.
        assert!(!dir.join("widget.diff.png").exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Same cleanup guarantee as `update_cleans_up_stale_new_and_diff_artifacts`, for the
    /// exact-match text/JSON comparison used by `snapshot_accessibility_tree`/
    /// `snapshot_render_instructions`.
    #[test]
    fn update_cleans_up_stale_new_json_artifact() {
        let dir = scratch_dir("update_cleans_up_stale_new_json_artifact");

        compare_text_and_report_in(&dir, "widget", "{\"a\":1}", "a11y", true)
            .expect("seeding the initial golden should succeed");

        compare_text_and_report_in(&dir, "widget", "{\"a\":2}", "a11y", false)
            .expect_err("differing content should be reported as a mismatch");

        let new_path = dir.join("widget.a11y.new.json");
        assert!(new_path.exists());

        compare_text_and_report_in(&dir, "widget", "{\"a\":2}", "a11y", true)
            .expect("updating should accept the changed content");

        assert!(!new_path.exists(), "update should clean up the now-stale .new.json");
        assert_eq!(std::fs::read_to_string(dir.join("widget.a11y.json")).unwrap(), "{\"a\":2}");

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Same scenario as `passing_comparison_cleans_up_stale_new_and_diff_artifacts`, for the
    /// text/JSON path: fail once, then pass on a later run with no `UPDATE_SNAPSHOTS` involved —
    /// the stale `.new.json` should be gone afterward.
    #[test]
    fn passing_text_comparison_cleans_up_stale_new_json_artifact() {
        let dir = scratch_dir("passing_text_comparison_cleans_up_stale_new_json_artifact");

        compare_text_and_report_in(&dir, "widget", "{\"a\":1}", "a11y", true)
            .expect("seeding the initial golden should succeed");

        compare_text_and_report_in(&dir, "widget", "{\"a\":2}", "a11y", false)
            .expect_err("differing content should be reported as a mismatch");

        let new_path = dir.join("widget.a11y.new.json");
        assert!(new_path.exists());

        compare_text_and_report_in(&dir, "widget", "{\"a\":1}", "a11y", false)
            .expect("comparing matching content against the golden should pass");

        assert!(!new_path.exists(), "a passing comparison should clean up the now-stale .new.json");

        let _ = std::fs::remove_dir_all(&dir);
    }
}
