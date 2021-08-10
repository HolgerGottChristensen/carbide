use crate::prelude::*;

pub trait WidgetExt: Widget + Sized + 'static {
    fn frame<K1: Into<F64State>, K2: Into<F64State>>(self, width: K1, height: K2) -> Box<Frame> {
        Frame::init(width.into(), height.into(), Box::new(self))
    }

    fn frame_width(self, width: F64State) -> Box<Frame> {
        Frame::init_width(width, Box::new(self))
    }

    fn padding<E: Into<EdgeInsets>>(self, edge_insets: E) -> Box<Padding> {
        Padding::init(edge_insets.into(), Box::new(self))
    }
    fn clip(self) -> Box<Clip> {
        Clip::new(Box::new(self))
    }

    fn clip_shape(self, shape: Box<dyn Shape>) -> Box<ClipShape> {
        ClipShape::new(Box::new(self), shape)
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

    /*fn foreground_color<C: Into<ColorState>>(self, color: C) -> Box<EnvUpdating<GS>> {
        let mut e = EnvUpdating::new(Box::new(self));
        e.add(EnvironmentStateContainer::Color { key: EnvironmentColor::Label, value: color.into() });

        e
    }

    fn accent_color<C: Into<ColorState>>(self, color: C) -> Box<EnvUpdating<GS>> {
        let mut e = EnvUpdating::new(Box::new(self));
        e.add(EnvironmentStateContainer::Color { key: EnvironmentColor::Accent, value: color.into() });

        e
    }*/
}
