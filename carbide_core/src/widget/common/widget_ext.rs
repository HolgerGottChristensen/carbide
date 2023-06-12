use cgmath::Matrix4;
use carbide_core::state::{IntoReadState, RMap1};

use crate::Color;
use crate::draw::Dimension;
use crate::environment::{EnvironmentColor, EnvironmentColorState, EnvironmentStateContainer};
use crate::flags::Flags;
use crate::state::{ReadState, TState};
use crate::widget::{Background, Border, Clip, ClipShape, CornerRadii, EdgeInsets, EnvUpdating, Flagged, Flexibility, Frame, Hidden, Offset, Padding, Rotation3DEffect, RoundedRectangle, Shape, Transform, Widget};
use crate::state::ReadStateExtNew;

pub trait WidgetExt: Widget + Sized + Clone + 'static {
    /// Surround the widget with a frame. The frame is a widget that has fixed width, height or both.
    /// The frame takes two parameters. Both parameters take f64 state. This means you can pass
    /// constant values like 10, 100.2, varying values like LocalState and AnimationState.
    fn frame<W: IntoReadState<f64>, H: IntoReadState<f64>>(self, width: W, height: H) -> Box<Frame<f64, f64, W::Output, H::Output, Self>> {
        Frame::new(width, height, self)
    }

    /// Changes the flexibility of the widget to a custom value. This can be useful when the
    /// default value does not provide the expected layout for example within a VStack.
    fn custom_flexibility(self, flexibility: u32) -> Box<Flexibility<Self>> {
        Flexibility::new(self, flexibility)
    }

    /// Change the flags of a given widget. This can for example be used to make any widget take
    /// Flags::USEMAXCROSSAXIS to make it use the max cross axis instead of expanding infinitely
    /// within a VStack or HStack.
    fn custom_flags(self, flags: Flags) -> Box<Flagged<Self>> {
        Flagged::new(self, flags)
    }

    /// Add a widget to the background of this widget. The proposed size for the widget in the
    /// background will be size chosen of the widget in the foreground. This can be really useful
    /// when trying to add color behind text.
    fn background<B: Widget + Clone>(self, background: B) -> Box<Background<Self, B>> {
        Background::new(self, background)
    }

    /// This rotates the widget visually around the x and y axis. Notice it will not change the
    /// areas for event handling. The widget will still take up the same space as if the effect
    /// wasn't applies. This only changes the visual. The function takes anything that can be
    /// converted into a state of f64.
    fn rotation_3d_effect<R1: ReadState<T = f64> + Clone, R2: ReadState<T = f64> + Clone>(
        self,
        x: R1,
        y: R2,
    ) -> Box<Rotation3DEffect<R1, R2, Self>> {
        Rotation3DEffect::new(self, x, y)
    }

    /// Rotates the widget around the z axis. The z axis is the axis that goes through you screen.
    /// This is only a visual change and the widget will still take up the same space as if the
    /// effect isn't applied.
    fn rotation_effect<R: ReadState<T = f64> + Clone>(self, rotation: R) -> Box<Transform<Self, RMap1<fn(&f64) -> Matrix4<f32>, f64, Matrix4<f32>, R>>> {
        Transform::rotation(self, rotation)
    }

    /// Scales the widget visually in a uniform way. It takes a scale factor which is a f64 state.
    /// A scale below 1.0 will make the widget smaller and a scale larger than 1.0 will result in
    /// a larger widget. This is only visual and will not change the size taken up by the actual
    /// widget.
    fn scale_effect<R: ReadState<T = f64> + Clone>(self, scale: R) -> Box<Transform<Self, RMap1<fn(&f64) -> Matrix4<f32>, f64, Matrix4<f32>, R>>> {
        Transform::scale(self, scale)
    }

    /// Scale the widget in a non uniform way. This takes a dimension and will scale the x axis
    /// with the width value and the y axis with the height value. A value of less than 1.0 will
    /// make the given scale smaller and a value larger than 1.0 will result in a larger widget.
    /// The effect is only graphical and will not change the actual scale of the widget.
    fn scale_effect_non_uniform<R: ReadState<T = Dimension> + Clone>(self, scale: R) -> Box<Transform<Self, RMap1<fn(&Dimension) -> Matrix4<f32>, Dimension, Matrix4<f32>, R>>> {
        Transform::scale_non_uniform(self, scale)
    }

    /// This can be used to apply a custom transformation matrix to the given widget. This will
    /// only result in visual changes and not affect the actual size of the widget.
    fn transform<R: ReadState<T = Matrix4<f32>> + Clone>(self, matrix: R) -> Box<Transform<Self, R>> {
        Transform::new(self, matrix)
    }

    fn frame_fixed_width<W: IntoReadState<f64>>(self, width: W) -> Box<Frame<f64, f64, W::Output, f64, Self>> {
        Frame::new(width, 10.0, self)
            .expand_height()
    }

    fn frame_fixed_height<H: IntoReadState<f64>>(self, height: H) -> Box<Frame<f64, f64, f64, H::Output, Self>> {
        Frame::new(10.0, height, self)
            .expand_width()
    }

    /// Set a padding around a widget. This will take any value that can be converted into EdgeInsets
    /// This includes values like 10.0 which will apply a padding of 10.0 at all sides of the widget.
    fn padding(self, edge_insets: impl Into<EdgeInsets>) -> Box<Padding> {
        Padding::new(edge_insets.into(), Box::new(self))
    }

    /// Clip the content of the widget. The clip area will be the requested area for the widget. It
    /// will clip all children graphics within that area. This currently does not change whether an
    /// item is clickable outside the clip area.
    fn clip(self) -> Box<Clip<Self>> {
        Clip::new(self)
    }

    fn clip_shape<S: Shape + Clone>(self, shape: S) -> Box<ClipShape<Self, S>> {
        ClipShape::new(self, shape)
    }

    fn corner_radius(self, radius: impl Into<CornerRadii>) -> Box<ClipShape<Self, RoundedRectangle<EnvironmentColorState, EnvironmentColorState>>> {
        ClipShape::new(self, *RoundedRectangle::new(radius.into()))
    }

    fn hidden(self) -> Box<Hidden> {
        Hidden::new(Box::new(self))
    }

    /// Offset a widget. It will only change the locating of the rendered widget, but will not
    /// change its position for event handling.
    fn offset<X: IntoReadState<f64>, Y: IntoReadState<f64>>(self, offset_x: X, offset_y: Y) -> Box<Offset<X::Output, Y::Output, Self>> {
        Offset::new(offset_x, offset_y, self)
    }

    fn border(self) -> Box<Border<Self, Color>> {
        Border::new(self)
    }

    fn foreground_color(self, color: impl Into<TState<Color>>) -> Box<EnvUpdating<Self>> {
        let mut e = EnvUpdating::new(self);
        e.add(EnvironmentStateContainer::Color {
            key: EnvironmentColor::Label,
            value: color.into(),
        });

        e
    }

    fn accent_color<C: IntoReadState<Color>>(self, color: C) -> Box<EnvUpdating<Self>> {
        let mut e = EnvUpdating::new(self);
        e.add(EnvironmentStateContainer::Color {
            key: EnvironmentColor::Accent,
            value: TState::new(Box::new(color.into_read_state().ignore_writes())),
        });

        e
    }
}
