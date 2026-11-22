use std::env;
use std::path::{Path, PathBuf};

/// How closely a rendered image must match its golden image. Pixels are compared one by one by
/// colour distance (see `color_delta`): a pixel fails when its distance is above `threshold`, and
/// the snapshot fails when more than `max_failed_pixels` pixels fail. A small change anywhere
/// fails; slight rendering noise, such as anti-aliasing, stays below the threshold.
pub struct SnapshotOptions {
    /// The colour distance a pixel may differ by, from 0 (identical) to 1 (the most distant
    /// colours): a pixel fails when its distance is above `MAX_DELTA * threshold²`.
    pub threshold: f32,
    /// How many pixels may fail before the snapshot does. Keep it small, as a few dozen pixels
    /// can be a real change.
    pub max_failed_pixels: usize,
}

impl Default for SnapshotOptions {
    fn default() -> Self {
        SnapshotOptions {
            threshold: 0.1,
            max_failed_pixels: 0,
        }
    }
}

/// Anchored at the manifest dir of the crate whose tests are running, independent of
/// `cargo test`'s cwd: cargo sets `CARGO_MANIFEST_DIR` in the environment of every test binary it
/// runs, so a crate that uses this harness from its own `tests/` (e.g. `carbide_chart`) keeps its
/// golden files in its own `tests/snapshots/`. Falls back to this crate's dir, captured at compile
/// time, when the binary is run outside of cargo.
/// `scope` (typically the current test's name — see `harness::current_test_scope`) nests golden
/// files under a per-test subdirectory, so two different tests can each use a snapshot named
/// `"idle"` without colliding; `None` falls back to the flat, unscoped layout.
fn snapshot_dir(scope: Option<&str>) -> PathBuf {
    let manifest_dir = env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")));
    let base = manifest_dir.join("tests/snapshots");
    match scope {
        Some(scope) => base.join(scope),
        None => base,
    }
}

fn update_requested() -> bool {
    env::var("BLESS").is_ok()
}

/// The largest distance `color_delta` gives between any two colours.
const MAX_DELTA: f32 = 35215.0;

/// The thresholds a failure message counts failing pixels at, to help choose a threshold.
const THRESHOLD_SWEEP: [f32; 6] = [0.0, 0.02, 0.05, 0.1, 0.2, 0.4];

/// The perceived distance between two pixels: both are blended over white by their alpha, and
/// their difference is weighted in YIQ colour space, as in Y. Kotsarenko and F. Ramos, "Measuring
/// perceived color difference using YIQ NTSC transmission color space in mobile applications",
/// Programación Matemática y Software 2(2), 2010,
/// <https://doaj.org/article/b2e3b5088ba943eebd9af2927fef08ad>. 0 for equal colours, at most
/// `MAX_DELTA`.
fn color_delta(a: &image::Rgba<u8>, b: &image::Rgba<u8>) -> f32 {
    if a == b {
        return 0.0;
    }

    let blend = |pixel: &image::Rgba<u8>| {
        let alpha = pixel[3] as f32 / 255.0;
        let channel = |c: u8| 255.0 + (c as f32 - 255.0) * alpha;
        (channel(pixel[0]), channel(pixel[1]), channel(pixel[2]))
    };

    let ((r1, g1, b1), (r2, g2, b2)) = (blend(a), blend(b));
    let y = |r: f32, g: f32, b: f32| r * 0.29889531 + g * 0.58662247 + b * 0.11448223;
    let i = |r: f32, g: f32, b: f32| r * 0.59597799 - g * 0.27417610 - b * 0.32180189;
    let q = |r: f32, g: f32, b: f32| r * 0.21147017 - g * 0.52261711 + b * 0.31114694;

    let dy = y(r1, g1, b1) - y(r2, g2, b2);
    let di = i(r1, g1, b1) - i(r2, g2, b2);
    let dq = q(r1, g1, b1) - q(r2, g2, b2);

    0.5053 * dy * dy + 0.299 * di * di + 0.1957 * dq * dq
}

/// Each pixel's colour distance between two images of the same size.
struct Comparison {
    deltas: Vec<f32>,
}

impl Comparison {
    fn new(golden: &image::RgbaImage, actual: &image::RgbaImage) -> Comparison {
        Comparison { deltas: golden.pixels().zip(actual.pixels()).map(|(g, a)| color_delta(g, a)).collect() }
    }

    /// How many pixels differ by more than `threshold` allows.
    fn failing(&self, threshold: f32) -> usize {
        let limit = MAX_DELTA * threshold * threshold;
        self.deltas.iter().filter(|delta| **delta > limit).count()
    }

    /// The failing pixels at each of `THRESHOLD_SWEEP`, for the failure message.
    fn sweep(&self) -> String {
        THRESHOLD_SWEEP.iter()
            .map(|threshold| format!("{threshold}: {}", self.failing(*threshold)))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// The failing pixels in magenta, over a dimmed greyscale copy of the golden image.
fn diff_image(golden: &image::RgbaImage, comparison: &Comparison, threshold: f32) -> image::RgbaImage {
    let limit = MAX_DELTA * threshold * threshold;

    image::RgbaImage::from_fn(golden.width(), golden.height(), |x, y| {
        let index = (y * golden.width() + x) as usize;

        if comparison.deltas[index] > limit {
            image::Rgba([255, 0, 255, 255])
        } else {
            let g = golden.get_pixel(x, y);
            let gray = (0.299 * g[0] as f32 + 0.587 * g[1] as f32 + 0.114 * g[2] as f32) as u8;
            let dimmed = ((gray as u16) * 40 / 100) as u8;
            image::Rgba([dimmed, dimmed, dimmed, 255])
        }
    })
}

/// Compares `actual` with its golden image (see `compare_and_report_in`). `scope` nests the
/// golden under a per-test subdirectory (see `snapshot_dir`).
pub(crate) fn compare_and_report(name: &str, actual: &image::RgbaImage, options: &SnapshotOptions, scope: Option<&str>) -> Result<(), String> {
    compare_and_report_in(&snapshot_dir(scope), name, actual, options, update_requested())
}

/// Removes the `.new.png` and `.diff.png` an earlier failing run may have left, once the golden
/// matches again, so they don't look like an unresolved mismatch.
fn remove_stale_png_artifacts(dir: &Path, name: &str) {
    let _ = std::fs::remove_file(dir.join(format!("{name}.new.png")));
    let _ = std::fs::remove_file(dir.join(format!("{name}.diff.png")));
}

fn write_golden(dir: &Path, name: &str, actual: &image::RgbaImage) -> Result<(), String> {
    let golden_path = dir.join(format!("{name}.png"));
    std::fs::create_dir_all(dir).map_err(|e| format!("snapshot {name:?}: could not create {}: {e}", dir.display()))?;
    actual.save(&golden_path).map_err(|e| format!("snapshot {name:?}: could not write {}: {e}", golden_path.display()))?;
    remove_stale_png_artifacts(dir, name);
    Ok(())
}

/// The comparison, with `dir` and `update` passed in rather than read from `CARGO_MANIFEST_DIR`
/// and `BLESS`, so unit tests can use a scratch directory without racing over an env var.
///
/// - **`update` (`BLESS`):** the golden is replaced by `actual`, even when its size changed.
/// - **Within the tolerance:** passes, and nothing is written; goldens only change with `BLESS`.
/// - **Beyond the tolerance, or a different size:** fails, and writes `{name}.new.png` and, for
///   the same size, `{name}.diff.png` next to the golden. The message counts the failing pixels
///   at several thresholds.
fn compare_and_report_in(dir: &Path, name: &str, actual: &image::RgbaImage, options: &SnapshotOptions, update: bool) -> Result<(), String> {
    let golden_path = dir.join(format!("{name}.png"));

    if update {
        return write_golden(dir, name, actual);
    }

    let golden = image::open(&golden_path)
        .map_err(|_| format!(
            "snapshot {name:?}: no golden image at {} — run with BLESS=true to create it",
            golden_path.display()
        ))?
        .into_rgba8();

    if golden.dimensions() != actual.dimensions() {
        // No diff image, as pixels can only be compared between images of one size.
        let new_path = dir.join(format!("{name}.new.png"));
        let _ = actual.save(&new_path);
        return Err(format!(
            "snapshot {name:?}: dimensions differ (golden {:?} vs actual {:?}); actual output written to {}",
            golden.dimensions(), actual.dimensions(), new_path.display()
        ));
    }

    let comparison = Comparison::new(&golden, actual);
    let failing = comparison.failing(options.threshold);

    if failing > options.max_failed_pixels {
        let new_path = dir.join(format!("{name}.new.png"));
        let diff_path = dir.join(format!("{name}.diff.png"));
        let _ = actual.save(&new_path);
        let _ = diff_image(&golden, &comparison, options.threshold).save(&diff_path);
        return Err(format!(
            "snapshot {name:?}: {failing} pixels differ by more than the threshold {} (at most {} may); actual output written to {}, diff written to {}. Failing pixels by threshold: {}",
            options.threshold, options.max_failed_pixels, new_path.display(), diff_path.display(), comparison.sweep()
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
            "snapshot {name:?}: no golden file at {} — run with BLESS=true to create it",
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

    /// The behavior this test exists to lock in: running with `BLESS` set (`update:
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

        // Seed the initial golden, as if from an earlier `BLESS=true` run.
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
    /// `.diff.png`), and then — with no `BLESS` involved at all — the *same* test
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
    /// text/JSON path: fail once, then pass on a later run with no `BLESS` involved —
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

    /// A solid 600 by 500 image, the size of a typical snapshot.
    fn chart_sized(pixel: [u8; 4]) -> image::RgbaImage {
        solid(600, 500, pixel)
    }

    /// A small, strong change, such as a short text changing in a 40 by 15 pixel block, fails,
    /// writes the new image and the diff, and leaves the golden as it was.
    #[test]
    fn a_small_strong_change_fails() {
        let dir = scratch_dir("a_small_strong_change_fails");
        let options = SnapshotOptions::default();
        let golden = chart_sized([30, 30, 30, 255]);

        compare_and_report_in(&dir, "chart", &golden, &options, true).expect("seeding the golden");

        let mut changed = golden.clone();
        for x in 20..60 {
            for y in 480..495 {
                if (x + y) % 3 == 0 {
                    changed.put_pixel(x, y, image::Rgba([200, 200, 200, 255]));
                }
            }
        }

        let err = compare_and_report_in(&dir, "chart", &changed, &options, false).expect_err("a small, strong change should fail");
        assert!(err.contains("pixels differ"), "{err}");
        assert!(dir.join("chart.new.png").exists() && dir.join("chart.diff.png").exists());
        assert_eq!(image::open(dir.join("chart.png")).unwrap().into_rgba8(), golden, "a failing comparison leaves the golden as it was");

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Slight differences in many pixels, below the threshold, pass and leave the golden as it was.
    #[test]
    fn a_change_within_the_tolerance_passes_without_updating_the_golden() {
        let dir = scratch_dir("a_change_within_the_tolerance_passes_without_updating_the_golden");
        let options = SnapshotOptions::default();

        compare_and_report_in(&dir, "chart", &chart_sized([30, 30, 30, 255]), &options, true).expect("seeding the golden");

        let mut noisy = chart_sized([30, 30, 30, 255]);
        for (index, pixel) in noisy.pixels_mut().enumerate() {
            if index % 7 == 0 {
                *pixel = image::Rgba([32, 31, 30, 255]);
            }
        }

        compare_and_report_in(&dir, "chart", &noisy, &options, false).expect("noise below the threshold should pass");
        assert_eq!(image::open(dir.join("chart.png")).unwrap().into_rgba8(), chart_sized([30, 30, 30, 255]), "the golden is left as it was");
        assert!(!dir.join("chart.new.png").exists() && !dir.join("chart.diff.png").exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// An identical image writes nothing.
    #[test]
    fn an_identical_image_writes_nothing() {
        let dir = scratch_dir("an_identical_image_writes_nothing");
        let options = SnapshotOptions::default();
        let golden = chart_sized([30, 30, 30, 255]);

        compare_and_report_in(&dir, "chart", &golden, &options, true).expect("seeding the golden");
        let written = std::fs::metadata(dir.join("chart.png")).unwrap().modified().unwrap();

        std::thread::sleep(std::time::Duration::from_millis(20));
        compare_and_report_in(&dir, "chart", &golden, &options, false).expect("an identical image should pass");
        assert_eq!(std::fs::metadata(dir.join("chart.png")).unwrap().modified().unwrap(), written);

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Equal colours are 0 apart, black and white nearly `MAX_DELTA`, and near colours little.
    #[test]
    fn color_delta_is_zero_for_equal_colours_and_bounded() {
        let black = image::Rgba([0, 0, 0, 255]);
        let white = image::Rgba([255, 255, 255, 255]);

        assert_eq!(color_delta(&black, &black), 0.0);
        assert!((color_delta(&black, &white) - 32_857.0).abs() < 1.0, "{}", color_delta(&black, &white));
        assert!(color_delta(&black, &white) <= MAX_DELTA);
        assert!(color_delta(&image::Rgba([30, 30, 30, 255]), &image::Rgba([32, 31, 30, 255])) < MAX_DELTA * 0.02 * 0.02);
        assert_eq!(color_delta(&image::Rgba([0, 0, 0, 0]), &image::Rgba([255, 255, 255, 0])), 0.0, "fully transparent pixels are both white");
    }
}
