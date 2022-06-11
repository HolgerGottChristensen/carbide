use cgmath::Matrix4;

use crate::draw::Dimension;
use crate::prelude::*;
use crate::widget::window_menu::MenuBar;

pub trait WidgetExt: Widget + Sized + 'static {

    /// Surround the widget with a frame. The frame is a widget that has fixed width, height or both.
    /// The frame takes two parameters. Both parameters take f64 state. This means you can pass
    /// constant values like 10, 100.2, varying values like LocalState and AnimationState.
    fn frame(self, width: impl Into<TState<f64>>, height: impl Into<TState<f64>>) -> Box<Frame> {
        Frame::init(width, height, Box::new(self))
    }

    /// Changes the flexibility of the widget to a custom value. This can be useful when the
    /// default value does not provide the expected layout for example within a VStack.
    fn custom_flexibility(self, flexibility: u32) -> Box<dyn Widget> {
        Flexibility::new(Box::new(self), flexibility)
    }

    /// Change the flags of a given widget. This can for example be used to make any widget take
    /// Flags::USEMAXCROSSAXIS to make it use the max cross axis instead of expanding infinitely
    /// within a VStack or HStack.
    fn custom_flags(self, flags: Flags) -> Box<dyn Widget> {
        Flagged::new(Box::new(self), flags)
    }

    /// Add a widget to the background of this widget. The proposed size for the widget in the
    /// background will be size chosen of the widget in the foreground. This can be really useful
    /// when trying to add color behind text.
    fn background(self, background: Box<dyn Widget>) -> Box<Background> {
        Background::new(Box::new(self), background)
    }

    /// This rotates the widget visually around the x and y axis. Notice it will not change the
    /// areas for event handling. The widget will still take up the same space as if the effect
    /// wasn't applies. This only changes the visual. The function takes anything that can be
    /// converted into a state of f64.
    fn rotation_3d_effect(self, x: impl Into<TState<f64>>, y: impl  Into<TState<f64>>) -> Box<Rotation3DEffect> {
        Rotation3DEffect::new(Box::new(self), x.into(), y.into())
    }

    /// Rotates the widget around the z axis. The z axis is the axis that goes through you screen.
    /// This is only a visual change and the widget will still take up the same space as if the
    /// effect isn't applied.
    fn rotation_effect(self, rotation: impl Into<TState<f64>>) -> Box<Transform> {
        Transform::rotation(Box::new(self), rotation)
    }

    /// Scales the widget visually in a uniform way. It takes a scale factor which is a f64 state.
    /// A scale below 1.0 will make the widget smaller and a scale larger than 1.0 will result in
    /// a larger widget. This is only visual and will not change the size taken up by the actual
    /// widget.
    fn scale_effect(self, scale: impl Into<TState<f64>>) -> Box<Transform> {
        Transform::scale(Box::new(self), scale)
    }

    /// Scale the widget in a non uniform way. This takes a dimension and will scale the x axis
    /// with the width value and the y axis with the height value. A value of less than 1.0 will
    /// make the given scale smaller and a value larger than 1.0 will result in a larger widget.
    /// The effect is only graphical and will not change the actual scale of the widget.
    fn scale_effect_non_uniform<K1: Into<TState<Dimension>>>(self, scale: K1) -> Box<Transform> {
        Transform::scale_non_uniform(Box::new(self), scale)
    }

    /// This can be used to apply a custom transformation matrix to the given widget. This will
    /// only result in visual changes and not affect the actual size of the widget.
    fn transform<K1: Into<TState<Matrix4<f32>>>>(self, matrix: K1) -> Box<Transform> {
        Transform::new(Box::new(self), matrix)
    }

    fn frame_fixed_width(self, width: impl Into<TState<f64>>) -> Box<Frame> {
        Frame::init_width(width.into(), Box::new(self))
    }

    fn frame_fixed_height(self, height: impl Into<TState<f64>>) -> Box<Frame> {
        Frame::init_height(height.into(), Box::new(self))
    }

    fn menu(self, menus: Vec<Menu>) -> Box<dyn Widget> {
        MenuBar::new(menus, Box::new(self))
    }

    /// Set a padding around a widget. This will take any value that can be converted into EdgeInsets
    /// This includes values like 10.0 which will apply a padding of 10.0 at all sides of the widget.
    fn padding(self, edge_insets: impl Into<EdgeInsets>) -> Box<Padding> {
        Padding::init(edge_insets.into(), Box::new(self))
    }

    /// Clip the content of the widget. The clip area will be the requested area for the widget. It
    /// will clip all children graphics within that area. This currently does not change whether an
    /// item is clickable outside the clip area.
    fn clip(self) -> Box<Clip> {
        Clip::new(Box::new(self))
    }

    fn clip_shape(self, shape: Box<dyn Shape>) -> Box<ClipShape> {
        ClipShape::new(Box::new(self), shape)
    }

    fn corner_radius<R: Into<CornerRadii>>(self, radius: R) -> Box<ClipShape> {
        ClipShape::new(Box::new(self), RoundedRectangle::new(radius.into()))
    }

    fn hidden(self) -> Box<Hidden> {
        Hidden::new(Box::new(self))
    }

    fn offset<K1: Into<F64State>, K2: Into<F64State>>(
        self,
        offset_x: K1,
        offset_y: K2,
    ) -> Box<Offset> {
        Offset::new(offset_x.into(), offset_y.into(), Box::new(self))
    }

    fn border(self) -> Box<Border> {
        Border::initialize(Box::new(self))
    }

    fn foreground_color<C: Into<ColorState>>(self, color: C) -> Box<EnvUpdating> {
        let mut e = EnvUpdating::new(Box::new(self));
        e.add(EnvironmentStateContainer::Color { key: EnvironmentColor::Label, value: color.into() });

        e
    }

    fn accent_color<C: Into<ColorState>>(self, color: C) -> Box<EnvUpdating> {
        let mut e = EnvUpdating::new(Box::new(self));
        e.add(EnvironmentStateContainer::Color { key: EnvironmentColor::Accent, value: color.into() });

        e
    }
}
