use std::ascii::escape_default;
use carbide_core::Color;
use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::{Environment, EnvironmentColor};
use carbide_core::event::{MouseEvent, MouseEventHandler};
use carbide_core::flags::Flags;
use carbide_core::focus::{Focus, Focusable};
use carbide_core::focus::Refocus;
use carbide_core::layout::Layout;
use carbide_core::state::{BoolState, FocusState, LocalState, Map2, Map3, Map4, MapOwnedState, ReadState, State, StateKey, StringState, TState, ValueState};
use carbide_core::widget::{Capsule, CommonWidget, CrossAxisAlignment, HSplit, HStack, WidgetId, Rectangle, Spacer, Text, Widget, WidgetExt, WidgetIter, WidgetIterMut, ZStack};
use crate::PlainButton;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Focusable, Layout, MouseEvent)]
pub struct PlainSlider {
    id: WidgetId,
    #[state]
    focus: FocusState,
    child: Vec<Box<dyn Widget>>,
    position: Position,
    dimension: Dimension,
    #[state]
    state: TState<f64>,
    percent: TState<f64>,
    start: f64,
    end: f64,
    cross_axis_alignment: CrossAxisAlignment,
    dragging: bool,
    steps: TState<Option<f64>>,
    thumb: fn() -> Box<dyn Widget>,
    indicator: fn() -> Box<dyn Widget>,
    background: fn() -> Box<dyn Widget>,
}

impl PlainSlider {

    pub fn new(value: impl Into<TState<f64>>, start: f64, end: f64) -> Box<Self> {
        let focus_state = LocalState::new(Focus::Unfocused);

        Self::new_internal(
            focus_state.into(),
            value.into(),
            start,
            end,
            ValueState::new(None),
            Self::default_background,
            Self::default_indicator,
            Self::default_thumb,
        )
    }

    pub fn step(mut self, step: f64) -> Box<Self> {
        Self::new_internal(
            self.focus,
            self.state,
            self.start,
            self.end,
            ValueState::new(Some(step)),
            self.background,
            self.indicator,
            self.thumb
        )
    }

    pub fn background(mut self, background: fn() -> Box<dyn Widget>) -> Box<Self> {
        Self::new_internal(
            self.focus,
            self.state,
            self.start,
            self.end,
            self.steps,
            background,
            self.indicator,
            self.thumb
        )
    }

    pub fn indicator(mut self, indicator: fn() -> Box<dyn Widget>) -> Box<Self> {
        Self::new_internal(
            self.focus,
            self.state,
            self.start,
            self.end,
            self.steps,
            self.background,
            indicator,
            self.thumb
        )
    }

    pub fn thumb(mut self, thumb: fn() -> Box<dyn Widget>) -> Box<Self> {
        Self::new_internal(
            self.focus,
            self.state,
            self.start,
            self.end,
            self.steps,
            self.background,
            self.indicator,
            thumb
        )
    }

    fn default_background() -> Box<dyn Widget> {
        Rectangle::new().fill(EnvironmentColor::Red)
            .frame_fixed_height(10.0)
    }

    fn default_indicator() -> Box<dyn Widget> {
        Rectangle::new().fill(EnvironmentColor::Green)
            .frame_fixed_height(10.0)
    }

    fn default_thumb() -> Box<dyn Widget> {
        Rectangle::new().fill(EnvironmentColor::Blue)
            .frame(26, 26)
    }

    fn new_internal(
        focus: FocusState,
        state: TState<f64>,
        start: f64,
        end: f64,
        steps: TState<Option<f64>>,
        background: fn() -> Box<dyn Widget>,
        indicator: fn() -> Box<dyn Widget>,
        thumb: fn() -> Box<dyn Widget>,
    ) -> Box<Self> {

        let progress_state = Map4::map(state.clone(), start, end, steps.clone(), |state: &f64, start: &f64, end: &f64, steps: &Option<f64>| {
            (*state - *start) / (*end - *start)
        }, |new_value: f64, state: &f64, start: &f64, end: &f64, steps: &Option<f64>| {

            if let Some(steps) = *steps {
                let number_of_steps = ((end - start) / steps).ceil();

                let stepped_percent = (number_of_steps * new_value).round() / number_of_steps;

                (Some(stepped_percent * (*end - *start) + *start), None, None, None)
            } else {
                (Some(new_value * (*end - *start) + *start), None, None, None)
            }

        });

        let children = vec![
            background(),
            indicator(),
            thumb()
        ];

        Box::new(PlainSlider {
            id: WidgetId::new(),
            focus,
            child: children,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(0.0, 0.0),
            state,
            percent: progress_state,
            start,
            end,
            cross_axis_alignment: CrossAxisAlignment::Center,
            dragging: false,
            steps,
            thumb,
            indicator,
            background
        })
    }
}

impl Focusable for PlainSlider {
    fn focus_children(&self) -> bool {
        false
    }
}

impl MouseEventHandler for PlainSlider {
    fn handle_mouse_event(&mut self, event: &MouseEvent, _consumed: &bool, _env: &mut Environment) {

        match event {
            MouseEvent::Press(_, position, _) => {
                if self.child[2].is_inside(*position) || self.child[0].is_inside(*position) {
                    self.dragging = true;

                    let relative_to_position = *position - self.position;
                    let p = (relative_to_position.x() - self.child[2].width() / 2.0) / (self.dimension.width - self.child[2].width());
                    self.percent.set_value(p.max(0.0).min(1.0));
                }
            }
            MouseEvent::Release(_, _, _) => {
                self.dragging = false;
            }
            MouseEvent::Move { to, .. } => {
                if !self.dragging { return; }

                let relative_to_position = *to - self.position;
                let p = (relative_to_position.x() - self.child[2].width() / 2.0) / (self.dimension.width - self.child[2].width());
                self.percent.set_value(p.max(0.0).min(1.0));
            }
            _ => ()
        }
    }
}

impl Layout for PlainSlider {
    fn calculate_size(&mut self, requested_size: Dimension, env: &mut Environment) -> Dimension {
        let percent = *self.percent.value();

        let background = self.child[0].calculate_size(requested_size, env);

        let requested_leading_width = requested_size.width * percent;
        let leading_size = Dimension::new(requested_leading_width, requested_size.height);

        let track = self.child[1].calculate_size(leading_size, env);

        let thumb = self.child[2].calculate_size(requested_size, env);

        let height = background.height.max(track.height).max(thumb.height);

        self.set_dimension(Dimension::new(requested_size.width, height));
        self.dimension
    }

    fn position_children(&mut self) {
        let position = self.position();

        let cross = self.cross_axis_alignment;

        let (child0_height, child1_height, child2_height) = match cross {
            CrossAxisAlignment::Start => {
                (position.y(), position.y(), position.y())
            }
            CrossAxisAlignment::Center => {
                (
                    position.y() + self.height() / 2.0 - self.child[0].height() / 2.0,
                    position.y() + self.height() / 2.0 - self.child[1].height() / 2.0,
                    position.y() + self.height() / 2.0 - self.child[2].height() / 2.0,
                )
            }
            CrossAxisAlignment::End => {
                (
                    position.y() + self.height() - self.child[0].height(),
                    position.y() + self.height() - self.child[1].height(),
                    position.y() + self.height() - self.child[2].height(),
                )
            }
        };

        self.child[0].set_position(Position::new(position.x(),  child0_height));
        self.child[1].set_position(Position::new(position.x(),  child1_height));

        let x = if let Some(steps) = *self.steps.value() {
            let number_of_steps = ((self.end - self.start) / steps).ceil();

            let stepped_percent = (number_of_steps * *self.percent.value()).round() / number_of_steps;

            self.x() + (self.child[0].width() - self.child[2].width()) * stepped_percent

        } else {
            self.x() + (self.child[0].width() - self.child[2].width()) * *self.percent.value()
        };

        self.child[2].set_position(Position::new(x, child2_height));

        self.child[0].position_children();
        self.child[1].position_children();
        self.child[2].position_children();
    }
}

impl CommonWidget for PlainSlider {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn flag(&self) -> Flags {
        Flags::FOCUSABLE
    }


    fn get_focus(&self) -> Focus {
        self.focus.value().clone()
    }

    fn set_focus(&mut self, focus: Focus) {
        *self.focus.value_mut() = focus;
    }

    fn children(&self) -> carbide_core::widget::WidgetIter {
        let contains_proxy_or_ignored = (self.child).iter().fold(false, |a, b| a || (b.flag() == carbide_core::flags::Flags::PROXY || b.flag() == carbide_core::flags::Flags::IGNORE));
        if !contains_proxy_or_ignored {
            carbide_core::widget::WidgetIter::Vec((self.child).iter())
        } else {
            (self.child)
                .iter()
                .filter(|x| x.flag() != carbide_core::flags::Flags::IGNORE)
                .rfold(carbide_core::widget::WidgetIter::Empty, |acc, x| {
                    if x.flag() == carbide_core::flags::Flags::PROXY {
                        carbide_core::widget::WidgetIter::Multi(Box::new(x.children()), Box::new(acc))
                    } else {
                        carbide_core::widget::WidgetIter::Single(x, Box::new(acc))
                    }
                })
        }
    }

    fn children_mut(&mut self) -> carbide_core::widget::WidgetIterMut {
        let contains_proxy_or_ignored = (self.child).iter().fold(false, |a, b| a || (b.flag() == carbide_core::flags::Flags::PROXY || b.flag() == carbide_core::flags::Flags::IGNORE));
        if !contains_proxy_or_ignored {
            carbide_core::widget::WidgetIterMut::Vec((self.child).iter_mut())
        } else {
            (self.child)
                .iter_mut()
                .filter(|x| x.flag() != carbide_core::flags::Flags::IGNORE)
                .rfold(carbide_core::widget::WidgetIterMut::Empty, |acc, x| {
                    if x.flag() == carbide_core::flags::Flags::PROXY {
                        carbide_core::widget::WidgetIterMut::Multi(Box::new(x.children_mut()), Box::new(acc))
                    } else {
                        carbide_core::widget::WidgetIterMut::Single(x, Box::new(acc))
                    }
                })
        }
    }

    fn children_direct(&mut self) -> carbide_core::widget::WidgetIterMut {
        carbide_core::widget::WidgetIterMut::Vec((self.child).iter_mut())
    }

    fn children_direct_rev(&mut self) -> carbide_core::widget::WidgetIterMut {
        carbide_core::widget::WidgetIterMut::VecRev((self.child).iter_mut().rev())
    }

    fn position(&self) -> Position {
        self.position
    }

    fn set_position(&mut self, position: Position) {
        self.position = position;
    }

    fn dimension(&self) -> Dimension {
        self.dimension
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        self.dimension = dimension
    }
}

impl WidgetExt for PlainSlider {}
