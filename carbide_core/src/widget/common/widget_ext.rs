use cgmath::Matrix4;

use crate::color::RED;
use crate::draw::{Color, Rect};
use crate::draw::Dimension;
use crate::environment::{Environment, EnvironmentColor};
use crate::event::{KeyboardEventContext, ModifierKey};
use crate::event::Key;
use crate::flags::WidgetFlag;
use crate::focus::Focus;
use crate::render::Style;
use crate::state::{IntoReadState, RMap1};
use crate::state::{IntoState, ReadState, StateContract};
use crate::widget::{Absolute, Action, AnyWidget, AspectRatio, Background, Border, Changed, Clip, ClipShape, ContentMode, CornerRadii, EdgeInsets, EnvUpdating, Flagged, Flexibility, Frame, GeometryReader, Hidden, HueRotation, Mask, MouseArea, Offset, OnKey, OnKeyAction, Padding, Rotation3DEffect, RoundedRectangle, Saturation, Scroll, Shadow, Shape, Transform};
use crate::widget::luminance::Luminance;
use crate::widget::OnChange;
use crate::widget::Widget;

type AccentColor<C, T, S> = EnvUpdating<C, T, S>;
type ForegroundColor<C, T, S> = EnvUpdating<C, T, S>;


pub trait WidgetExt: AnyWidget + Clone + Sized {
    /// Surround the widget with a frame. The frame is a widget that has fixed width, height or both.
    /// The frame takes two parameters. Both parameters take f64 state. This means you can pass
    /// constant values like 10, 100.2, varying values like LocalState and AnimationState.
    fn frame<W: IntoState<f64>, H: IntoState<f64>>(self, width: W, height: H) -> Frame<W::Output, H::Output, Self> {
        Frame::new(width, height, self)
    }

    /// Changes the flexibility of the widget to a custom value. This can be useful when the
    /// default value does not provide the expected layout for example within a VStack.
    fn custom_flexibility(self, flexibility: u32) -> Flexibility<Self> {
        Flexibility::new(self, flexibility)
    }

    /// Change the flags of a given widget. This can for example be used to make any widget take
    /// Flags::USEMAXCROSSAXIS to make it use the max cross axis instead of expanding infinitely
    /// within a VStack or HStack.
    fn custom_flags(self, flags: WidgetFlag) -> Flagged<Self> {
        Flagged::new(self, flags)
    }

    /// Add a widget to the background of this widget. The proposed size for the widget in the
    /// background will be size chosen of the widget in the foreground. This can be really useful
    /// when trying to add color behind text.
    fn background<B: Widget>(self, background: B) -> Background<Self, B> {
        Background::new(self, background)
    }

    /// This rotates the widget visually around the x and y axis. Notice it will not change the
    /// areas for event handling. The widget will still take up the same space as if the effect
    /// wasn't applies. This only changes the visual. The function takes anything that can be
    /// converted into a state of f64.
    fn rotation_3d_effect<R1: ReadState<T = f64>, R2: ReadState<T = f64>>(
        self,
        x: R1,
        y: R2,
    ) -> Rotation3DEffect<R1, R2, Self> {
        Rotation3DEffect::new(self, x, y)
    }

    /// Rotates the widget around the z axis. The z axis is the axis that goes through you screen.
    /// This is only a visual change and the widget will still take up the same space as if the
    /// effect isn't applied.
    fn rotation_effect<R: ReadState<T = f64>>(self, rotation: R) -> Transform<Self, RMap1<fn(&f64) -> Matrix4<f32>, f64, Matrix4<f32>, R>> {
        Transform::rotation(self, rotation)
    }

    /// Scales the widget visually in a uniform way. It takes a scale factor which is a f64 state.
    /// A scale below 1.0 will make the widget smaller and a scale larger than 1.0 will result in
    /// a larger widget. This is only visual and will not change the size taken up by the actual
    /// widget.
    fn scale_effect<R: ReadState<T = f64>>(self, scale: R) -> Transform<Self, RMap1<fn(&f64) -> Matrix4<f32>, f64, Matrix4<f32>, R>> {
        Transform::scale(self, scale)
    }

    /// Scale the widget in a non uniform way. This takes a dimension and will scale the x axis
    /// with the width value and the y axis with the height value. A value of less than 1.0 will
    /// make the given scale smaller and a value larger than 1.0 will result in a larger widget.
    /// The effect is only graphical and will not change the actual scale of the widget.
    fn scale_effect_non_uniform<R: ReadState<T = Dimension>>(self, scale: R) -> Transform<Self, RMap1<fn(&Dimension) -> Matrix4<f32>, Dimension, Matrix4<f32>, R>> {
        Transform::scale_non_uniform(self, scale)
    }

    /// This can be used to apply a custom transformation matrix to the given widget. This will
    /// only result in visual changes and not affect the actual size of the widget.
    fn transform<R: ReadState<T = Matrix4<f32>>>(self, matrix: R) -> Transform<Self, R> {
        Transform::new(self, matrix)
    }

    fn frame_fixed_width<W: IntoState<f64>>(self, width: W) -> Frame<W::Output, f64, Self> {
        Frame::new(width, 10.0, self)
            .expand_height()
    }

    fn frame_fixed_height<H: IntoState<f64>>(self, height: H) -> Frame<f64, H::Output, Self> {
        Frame::new(10.0, height, self)
            .expand_width()
    }

    /// Set a padding around a widget. This will take any value that can be converted into EdgeInsets
    /// This includes values like 10.0 which will apply a padding of 10.0 at all sides of the widget.
    fn padding<E: IntoReadState<EdgeInsets>>(self, edge_insets: E) -> Padding<Self, E::Output> {
        Padding::new(edge_insets, self)
    }

    fn scroll(self) -> Scroll<Self> {
        Scroll::new(self)
    }

    /// Clip the content of the widget. The clip area will be the requested area for the widget. It
    /// will clip all children graphics within that area. This currently does not change whether an
    /// item is clickable outside the clip area.
    fn clip(self) -> Clip<Self> {
        Clip::new(self)
    }

    fn clip_shape<S: Shape + Clone>(self, shape: S) -> ClipShape<Self, S> {
        ClipShape::new(self, shape)
    }

    fn mask<M: Widget>(self, mask: M) -> Mask<M, Self> {
        Mask::new(mask, self)
    }

    fn corner_radius(self, radius: impl Into<CornerRadii>) -> ClipShape<Self, RoundedRectangle<Style, Style>> {
        ClipShape::new(self, RoundedRectangle::new(radius).fill(Style::Color(RED)).stroke(Style::Color(RED)))
    }

    fn hidden(self) -> Hidden<Self> {
        Hidden::new(self)
    }

    fn offset<X: IntoReadState<f64>, Y: IntoReadState<f64>>(self, offset_x: X, offset_y: Y) -> Offset<X::Output, Y::Output, Self> {
        Offset::new(offset_x, offset_y, self)
    }

    fn absolute<X: IntoReadState<f64>, Y: IntoReadState<f64>>(self, x: X, y: Y) -> Absolute<X::Output, Y::Output, Self> {
        Absolute::new(x, y, self)
    }

    fn aspect_ratio<D: IntoReadState<Dimension>>(self, ratio: D) -> AspectRatio<D::Output, ContentMode, Self> {
        AspectRatio::new(ratio, self)
    }

    fn geometry<G: IntoState<Rect>>(self, geometry: G) -> GeometryReader<Self, G::Output> {
        GeometryReader::new(geometry, self)
    }

    fn on_change<T: StateContract + PartialEq, S: ReadState<T=T>, F: Changed<T>>(self, state: S, f: F) -> OnChange<Self, T, S, F> {
        OnChange::new(self, state, f)
    }

    fn border(self) -> Border<Self, Color> {
        Border::new(self)
    }

    fn foreground_color<C: IntoReadState<Color>>(self, color: C) -> ForegroundColor<Self, Color, C::Output> {
        EnvUpdating::new(EnvironmentColor::Label, color.into_read_state(), self)
    }

    fn accent_color<C: IntoReadState<Color>>(self, color: C) -> AccentColor<Self, Color, C::Output> {
        EnvUpdating::new(EnvironmentColor::Accent, color.into_read_state(), self)
    }

    /// Example: .on_click(move |env: &mut Environment, modifier: ModifierKey| {})
    fn on_click<A: Action + Clone>(self, action: A) -> MouseArea<A, fn(&mut Environment, ModifierKey), Focus, Self, bool, bool> {
        MouseArea::new(self).on_click(action)
    }

    fn hovered<T: IntoState<bool>>(self, hovered: T) -> MouseArea<fn(&mut Environment, ModifierKey), fn(&mut Environment, ModifierKey), Focus, Self, T::Output, bool> {
        MouseArea::new(self).hovered(hovered)
    }

    fn boxed(self) -> Box<dyn AnyWidget> {
        Box::new(self)
    }

    fn on_key_pressed<A2: OnKeyAction>(self, action: A2) -> OnKey<A2, fn(&Key, ModifierKey, &mut KeyboardEventContext), Self> {
        OnKey::new(self)
            .on_key_pressed(action)
    }

    fn on_key_released<A2: OnKeyAction>(self, action: A2) -> OnKey<fn(&Key, ModifierKey, &mut KeyboardEventContext), A2, Self> {
        OnKey::new(self)
            .on_key_released(action)
    }

    fn shadow<S: IntoReadState<f64>, C: IntoReadState<Color>, X: IntoReadState<i32>, Y: IntoReadState<i32>>(self, sigma: S, color: C, x: X, y: Y) -> Shadow<Self, C::Output, S::Output, X::Output, Y::Output> {
        Shadow::new(sigma, self)
            .shadow_color(color)
            .shadow_offset(x, y)
    }

    fn hue_rotation<R: IntoReadState<f64>>(self, rotation: R) -> HueRotation<Self, R::Output> {
        HueRotation::new(self, rotation)
    }

    fn saturation<R: IntoReadState<f64>>(self, shift: R) -> Saturation<Self, R::Output> {
        Saturation::new(self, shift)
    }

    fn luminance<R: IntoReadState<f64>>(self, shift: R) -> Luminance<Self, R::Output> {
        Luminance::new(self, shift)
    }
}
