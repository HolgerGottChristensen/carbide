//! Drawing on a `Canvas`: the transform `fill`, `stroke` and `fill_text` apply, and how it is
//! saved and restored.

use carbide_core::color::{BLUE, GREEN, RED, WHITE, YELLOW};
use carbide_core::draw::{Alignment, Angle, Dimension};
use carbide_core::math::Vector3;
use carbide_core::text::{FontStyle, FontWeight, TextDecoration};
use carbide_core::widget::canvas::{Canvas, CanvasContext};
use carbide_core::widget::{HStack, Rectangle, WidgetExt};
use carbide_testing::Harness;

use support::{rotate_around, square};

/// Four 40×40 squares: an outline where the square is drawn without a transform, then the same
/// square translated 100 to the right (filled), rotated 30° around its own centre (filled), and
/// scaled 2× along x around the canvas's origin (stroked). Pins down that `fill` and `stroke`
/// apply the current transform, in the canvas's own coordinates, and that a scale also widens the
/// stroke along x.
#[test]
fn canvas_transforms() {
    let canvas = Canvas::new(|ctx: &mut CanvasContext| {
        ctx.set_line_width(2.0);

        // Where the square is without a transform.
        square(ctx, 20.0, 20.0, 40.0);
        ctx.set_stroke_style(WHITE);
        ctx.stroke();

        ctx.save();
        ctx.translate(Vector3::new(100.0, 0.0, 0.0));
        square(ctx, 20.0, 20.0, 40.0);
        ctx.set_fill_style(GREEN);
        ctx.fill();
        ctx.restore();

        ctx.save();
        rotate_around(ctx, 40.0, 140.0, 30.0);
        square(ctx, 20.0, 120.0, 40.0);
        ctx.set_fill_style(BLUE);
        ctx.fill();
        ctx.restore();

        ctx.save();
        ctx.scale(Vector3::new(2.0, 1.0, 1.0));
        square(ctx, 20.0, 220.0, 40.0);
        ctx.set_stroke_style(YELLOW);
        ctx.stroke();
        ctx.restore();

    })
        .frame(300.0, 300.0);

    let mut harness = Harness::new(canvas, Dimension::new(300.0, 300.0));

    harness.snapshot("canvas_transforms");
}

/// A square rotated 45° around its centre between `save` and `restore`, then the same square drawn
/// again after `restore`, unrotated; and a square drawn after `set_transform` replaced a rotation
/// with a translation. Pins down that the transform is part of the saved state, and that
/// `set_transform` replaces rather than adds to the current transform.
#[test]
fn canvas_transform_save_restore() {
    let canvas = Canvas::new(|ctx: &mut CanvasContext| {
        ctx.save();
        rotate_around(ctx, 70.0, 70.0, 45.0);
        square(ctx, 40.0, 40.0, 60.0);
        ctx.set_fill_style(RED);
        ctx.fill();
        ctx.restore();

        // Back to no transform: drawn over the rotated one, unrotated.
        square(ctx, 50.0, 50.0, 40.0);
        ctx.set_fill_style(BLUE);
        ctx.fill();

        // A rotation replaced by a translation: the square is only moved.
        ctx.rotate(Angle::Degrees(45.0));
        ctx.set_transform(carbide_core::math::Matrix4::from_translation(Vector3::new(150.0, 150.0, 0.0)));
        square(ctx, 20.0, 20.0, 60.0);
        ctx.set_fill_style(GREEN);
        ctx.fill();

    })
        .frame(300.0, 300.0);

    let mut harness = Harness::new(canvas, Dimension::new(300.0, 300.0));

    harness.snapshot("canvas_transform_save_restore");
}

/// The same square, translated by (40, 40) and scaled 2× in both orders. Pins down that each added
/// transform is multiplied on the right of the current one, so it applies to the path first:
/// translating then scaling scales the square and then moves it by (40, 40); scaling then
/// translating moves it by (40, 40) and then scales the whole, so it lands at (80, 80).
#[test]
fn canvas_transform_order() {
    let canvas = Canvas::new(|ctx: &mut CanvasContext| {
        ctx.save();
        ctx.translate(Vector3::new(40.0, 40.0, 0.0));
        ctx.scale(Vector3::new(2.0, 2.0, 1.0));
        square(ctx, 0.0, 0.0, 30.0);
        ctx.set_fill_style(GREEN);
        ctx.fill();
        ctx.restore();

        ctx.save();
        ctx.scale(Vector3::new(2.0, 2.0, 1.0));
        ctx.translate(Vector3::new(40.0, 40.0, 0.0));
        square(ctx, 0.0, 0.0, 30.0);
        ctx.set_fill_style(BLUE);
        ctx.fill();
        ctx.restore();

    })
        .frame(300.0, 300.0);

    let mut harness = Harness::new(canvas, Dimension::new(300.0, 300.0));

    harness.snapshot("canvas_transform_order");
}

/// Text drawn without a transform, translated, rotated 90° and -45° around where it is drawn, and
/// scaled 2×. Pins down that `fill_text` applies the current transform like `fill` and `stroke`:
/// rotated text reads along the rotated direction (down the canvas at 90°), and scaled text is
/// drawn larger.
#[test]
fn canvas_text_transforms() {
    let canvas = Canvas::new(|ctx: &mut CanvasContext| {
        ctx.set_fill_style(WHITE);
        ctx.set_text_align(Alignment::TopLeading);
        ctx.fill_text("Plain", 20.0, 20.0);

        ctx.save();
        ctx.translate(Vector3::new(140.0, 0.0, 0.0));
        ctx.set_fill_style(GREEN);
        ctx.fill_text("Translated", 20.0, 20.0);
        ctx.restore();

        ctx.save();
        rotate_around(ctx, 40.0, 80.0, 90.0);
        ctx.set_fill_style(BLUE);
        ctx.fill_text("Rotated 90°", 40.0, 80.0);
        ctx.restore();

        ctx.save();
        rotate_around(ctx, 120.0, 200.0, -45.0);
        ctx.set_fill_style(RED);
        ctx.fill_text("Rotated -45°", 120.0, 200.0);
        ctx.restore();

        ctx.save();
        ctx.scale(Vector3::new(2.0, 2.0, 1.0));
        ctx.set_fill_style(YELLOW);
        ctx.fill_text("Scaled", 60.0, 120.0);
        ctx.restore();

    })
        .frame(300.0, 300.0);

    let mut harness = Harness::new(canvas, Dimension::new(300.0, 300.0));

    harness.snapshot("canvas_text_transforms");
}

/// Text as the text style starts out, then larger, bold, and in a whole replaced style. Pins down
/// that `fill_text` draws with the current text style, and that `measure_text` measures with it,
/// so a 28 high line takes about twice the room of a 14 high one. Italic and underlined text are
/// left out, as the text renderer draws neither yet.
#[test]
fn canvas_text_styles() {
    let canvas = Canvas::new(|ctx: &mut CanvasContext| {
        ctx.set_fill_style(WHITE);
        ctx.set_text_align(Alignment::TopLeading);
        ctx.fill_text("Plain", 20.0, 20.0);
        let plain = ctx.measure_text("Plain");

        ctx.set_font_size(28);
        ctx.fill_text("Large", 20.0, 50.0);
        let large = ctx.measure_text("Plain");
        assert!(large.height > 1.8 * plain.height, "28 is about twice as high as 14: {large:?} against {plain:?}");

        ctx.set_font_size(14);
        ctx.set_font_weight(FontWeight::Bold);
        ctx.fill_text("Bold", 20.0, 100.0);

        let mut replaced = ctx.get_text_style().clone();
        replaced.font_size = 20;
        replaced.font_weight = FontWeight::Normal;
        ctx.set_text_style(replaced);
        ctx.set_fill_style(GREEN);
        ctx.fill_text("Replaced", 20.0, 140.0);
    })
        .frame(300.0, 300.0);

    let mut harness = Harness::new(canvas, Dimension::new(300.0, 300.0));

    harness.snapshot("canvas_text_styles");
}

/// Text drawn large and bold between `save` and `restore`, then again after `restore`. Pins down
/// that the text style is part of the saved state: after `restore` every part of it, the family,
/// size, weight, font style and decoration, is back as it was before `save`, and the text is
/// drawn in it.
#[test]
fn canvas_text_style_save_restore() {
    let canvas = Canvas::new(|ctx: &mut CanvasContext| {
        ctx.set_fill_style(WHITE);
        ctx.set_text_align(Alignment::TopLeading);
        let before = ctx.get_text_style().clone();

        ctx.save();
        ctx.set_font_size(32);
        ctx.set_font_weight(FontWeight::Bold);
        ctx.set_font_style(FontStyle::Italic);
        ctx.set_text_decoration(TextDecoration::Underline);
        ctx.set_font_family("DejaVu Sans");
        ctx.set_fill_style(RED);
        ctx.fill_text("Saved", 20.0, 20.0);
        ctx.restore();

        assert_eq!(ctx.get_text_style(), &before, "restore brings back the style from before save");
        ctx.fill_text("Restored", 20.0, 80.0);
    })
        .frame(300.0, 300.0);

    let mut harness = Harness::new(canvas, Dimension::new(300.0, 300.0));

    harness.snapshot("canvas_text_style_save_restore");
}

/// A 100×100 square rotated 30° around x and 40° around y about its centre, twice side by side:
/// on the left by the `Rotation3DEffect` widget, on the right by a canvas with `rotate_3d`. Pins
/// down that `rotate_3d` rotates exactly like the widget, perspective included, so the two shapes
/// are the same.
#[test]
fn canvas_rotate_3d() {
    const ROTATION_Y: f64 = 40.0;
    const ROTATION_X: f64 = 30.0;
    let widget = Rectangle::new()
        .fill(GREEN)
        .frame(100.0, 100.0)
        .rotation_3d_effect(Angle::Degrees(ROTATION_X), Angle::Degrees(ROTATION_Y));

    let canvas = Canvas::new(|ctx: &mut CanvasContext| {
        ctx.translate(Vector3::new(50.0, 50.0, 0.0));
        ctx.rotate_3d(Angle::Degrees(ROTATION_X), Angle::Degrees(ROTATION_Y));
        ctx.translate(Vector3::new(-50.0, -50.0, 0.0));

        square(ctx, 0.0, 0.0, 100.0);
        ctx.set_fill_style(GREEN);
        ctx.fill();
    })
        .frame(100.0, 100.0);

    let side_by_side = HStack::new((widget, canvas)).spacing(40.0);

    let mut harness = Harness::new(side_by_side, Dimension::new(300.0, 160.0));

    harness.snapshot("canvas_rotate_3d");
}

// ===============================
//  Utility
// ===============================

mod support {
    use carbide_core::draw::{Angle, Scalar};
    use carbide_core::math::Vector3;
    use carbide_core::widget::canvas::CanvasContext;
    /// Adds a `size` square with its top left corner at (`x`, `y`) to a new path.
    pub fn square(ctx: &mut CanvasContext, x: Scalar, y: Scalar, size: Scalar) {
        ctx.begin_path();
        ctx.move_to(x, y);
        ctx.line_to(x + size, y);
        ctx.line_to(x + size, y + size);
        ctx.line_to(x, y + size);
        ctx.close_path();
    }

    /// Rotates by `degrees` around (`x`, `y`) rather than around the origin.
    pub fn rotate_around(ctx: &mut CanvasContext, x: Scalar, y: Scalar, degrees: f64) {
        ctx.translate(Vector3::new(x, y, 0.0));
        ctx.rotate(Angle::Degrees(degrees));
        ctx.translate(Vector3::new(-x, -y, 0.0));
    }
}
