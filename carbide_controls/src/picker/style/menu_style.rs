use crate::identifiable::{AnyIdentifiableWidget, AnySelectableWidget};
use crate::picker::picker_action::PickerAction;
use crate::picker::picker_selection::PickerSelectionType;
use crate::picker::style::{PickerStyle};
use crate::toggle::{CheckboxStyle, ToggleValue};
use crate::{ControlsOverlayKey, UnfocusAction};
use carbide_core::color::{Color, ColorExt, TRANSPARENT};
use carbide_core::environment::{EnvironmentColor, IntoColorReadState};
use carbide_core::focus::Focus;
use carbide_core::state::{AnyReadState, AnyState, EnvMap1, IntoState, LocalState, Map1, Map2, Map3, RMap1, RMap3, ReadState, ReadStateExtNew};
use carbide_core::widget::{AnySequence, AnyWidget, AspectRatio, Circle, CommonWidget, ContentMode, CornerRadii, CrossAxisAlignment, EdgeInsets, Ellipse, ForEach, Gradient, GradientPosition, HStack, IfElse, MouseArea, MouseAreaAction, MouseAreaActionContext, Overlay, OverlayManager, Padding, Rectangle, RoundedRectangle, Sequence, Spacer, Text, VStack, Widget, WidgetExt, Wrap, ZStack};
use std::fmt::Debug;
use carbide::draw::Rect;
use carbide::{closure, lens};
use carbide::event::EventId;
use carbide::state::StateExtNew;
use carbide::widget::{Background, Frame, WidgetId};
use carbide_core::utils::clone_box;
use carbide_core::draw::{Alignment, Dimension};
use carbide_core::flags::WidgetFlag;
use carbide_core::render::Style;
use carbide_core::widget::canvas::{Canvas, CanvasContext, LineCap};
use crate::picker::style::menu::{MenuStyleBase, MenuStyleItemBase, MenuStylePopupBase};

#[derive(Debug, Clone)]
pub struct MenuStyle;

impl MenuStyle {
    fn generate(&self, focus: Box<dyn AnyState<T=Focus>>, enabled: Box<dyn AnyReadState<T=bool>>, label: Box<dyn AnyReadState<T=String>>, model: Box<dyn AnySequence<dyn AnySelectableWidget>>, picker_selection_type: PickerSelectionType) -> impl Widget {
        let mark = Self::mark(&enabled);

        let content = Self::content(enabled.clone(), model.clone());

        let content_and_mark = HStack::new((
            content
                .clip()
                .alignment(Alignment::Leading)
                .padding(EdgeInsets::single(0.0, 0.0, 6.0, 2.0)),
            mark.custom_flexibility(15)
        ))
            .spacing(0.0)
            .cross_axis_alignment(CrossAxisAlignment::Start);

        let outline_color = Map2::read_map(EnvironmentColor::Accent.color(), focus.clone(), |color, focused| {
            if *focused == Focus::Focused {
                *color
            } else {
                TRANSPARENT
            }
        });

        let background = RoundedRectangle::new(CornerRadii::all(5.0))
            .fill(EnvironmentColor::SecondarySystemBackground)
            .stroke(EnvironmentColor::OpaqueSeparator)
            .stroke_style(1.0);

        let widget = ZStack::new((
            background,
            content_and_mark,
        ))
            .text_wrap(Wrap::None)
            .background(
                RoundedRectangle::new(CornerRadii::all(5.0))
                    .stroke(outline_color)
                    .stroke_style(1.0)
                    .padding(-1.0)
            )
            .frame_fixed_height(22.0);

        let rect = LocalState::new(Rect::default());

        let geometry = widget.geometry(rect.clone());

        let clickable = MenuStyleBase::new(
            geometry,
            focus,
            enabled,
            move |event_id| {
                MenuStylePopupBase {
                    id: WidgetId::new(),
                    position: lens!(rect.position).as_dyn_read(),
                    dimension: lens!(rect.dimension).as_dyn_read(),
                    child: VStack::new(ForEach::custom_widget(model.clone(), move |item: &dyn AnySelectableWidget| {
                        Self::popup_item(item, event_id)
                    })).spacing(1.0).padding(1.0).background(Rectangle::new().fill(EnvironmentColor::OpaqueSeparator)).boxed(),
                }.boxed()
            }
        );

        HStack::new((
            Text::new(label).custom_flexibility(15),
            clickable
        )).spacing(8.0)
    }

    fn popup_item(item: &dyn AnySelectableWidget, event_id: EventId) -> impl Widget {
        let selection = clone_box(item.selection());

        let hovered = LocalState::new(*selection.value()).as_dyn();

        let background_color = Map1::read_map(hovered.clone(), |hovered| {
            if *hovered {
                EnvironmentColor::Accent
            } else {
                EnvironmentColor::SecondarySystemBackground
            }
        });

        let visual = HStack::new((
            clone_box(item.as_widget()).frame_fixed_height(22.0).fit_width().padding(EdgeInsets::single(0.0, 0.0, 5.0, 0.0)),
            Spacer::new()
        )).background(Rectangle::new().fill(background_color))
            .boxed();

        MenuStyleItemBase::new(visual, selection, hovered, event_id)
    }

    fn mark(enabled: &Box<dyn AnyReadState<T=bool>>) -> impl Widget {
        let arrows = Self::arrows(&enabled);

        let mark_color = Map3::read_map(enabled.clone(), EnvironmentColor::Accent.color(), EnvironmentColor::TertiarySystemFill.color(), |enabled, color, disabled_color| {
            if *enabled {
                Style::Gradient(Gradient::linear(
                    vec![color.lightened(0.05), *color],
                    GradientPosition::Alignment(Alignment::Top),
                    GradientPosition::Alignment(Alignment::Bottom)
                ))
            } else {
                Style::Gradient(Gradient::linear(
                    vec![disabled_color.lightened(0.05), *disabled_color],
                    GradientPosition::Alignment(Alignment::Top),
                    GradientPosition::Alignment(Alignment::Bottom)
                ))
            }
        });

        let mark = ZStack::new((
            RoundedRectangle::new(5.0).fill(mark_color),
            arrows
        ))
            .padding(3.0)
            .aspect_ratio(Dimension::new(1.0, 1.0));
        mark
    }

    fn content(enabled: Box<dyn AnyReadState<T=bool>>, model: Box<dyn AnySequence<dyn AnySelectableWidget>>) -> impl Widget {
        let label_color = Map1::read_map(enabled.clone(), |enabled| {
            if *enabled {
                EnvironmentColor::DarkText
            } else {
                EnvironmentColor::TertiaryLabel
            }
        });

        let content = HStack::new(
            ForEach::custom_widget(model, |input: &dyn AnySelectableWidget| {
                clone_box(input.as_widget()).flagged(Map1::read_map(clone_box(input.selection()), |selected| {
                    if *selected {
                        WidgetFlag::empty()
                    } else {
                        WidgetFlag::IGNORE
                    }
                }))
            })
        )
            .foreground_color(label_color)
            .text_wrap(Wrap::None);
        content
    }

    fn arrows(enabled: &Box<dyn AnyReadState<T=bool>>) -> impl Widget {
        let mark_color = Map1::read_map(enabled.clone(), |enabled| {
            if *enabled {
                EnvironmentColor::DarkText
            } else {
                EnvironmentColor::TertiaryLabel
            }
        });

        let arrows = Canvas::new(move |ctx: &mut CanvasContext| {
            let padding = ctx.dimension() * 0.22;
            let arrow_width = ctx.width() * 0.4;
            let arrow_height = ctx.height() * 0.2;

            // Points for upward arrow
            let x = ctx.width() / 2.0;
            let y = ctx.height() / 2.0 - padding.height / 2.0;
            ctx.move_to(x - arrow_width / 2.0, y);
            ctx.line_to(x, y - arrow_height);
            ctx.line_to(x + arrow_width / 2.0, y);

            // Points for downward arrow
            let x = ctx.width() / 2.0;
            let y = ctx.height() / 2.0 + padding.height / 2.0;
            ctx.move_to(x - arrow_width / 2.0, y);
            ctx.line_to(x, y + arrow_height);
            ctx.line_to(x + arrow_width / 2.0, y);

            ctx.set_stroke_style(mark_color.clone());
            ctx.set_line_width(1.5);
            ctx.set_line_cap(LineCap::Round);
            ctx.stroke()
        });

        arrows
    }
}

impl PickerStyle for MenuStyle {
    fn create(&self, focus: Box<dyn AnyState<T=Focus>>, enabled: Box<dyn AnyReadState<T=bool>>, label: Box<dyn AnyReadState<T=String>>, model: Box<dyn AnySequence<dyn AnySelectableWidget>>, picker_selection_type: PickerSelectionType) -> Box<dyn AnyWidget> {
        MenuStyle.generate(focus, enabled, label, model, picker_selection_type).boxed()
    }
}

#[derive(Clone, Debug)]
struct MenuAction {
    popup: Box<dyn AnyWidget>,
    enabled: Box<dyn AnyReadState<T=bool>>
}

impl MouseAreaAction for MenuAction {
    fn call(&mut self, ctx: MouseAreaActionContext) {
        if !*self.enabled.value() {
            return;
        }
        OverlayManager::get::<ControlsOverlayKey>(ctx.env_stack, |manager| {
            manager.insert(self.popup.clone())
        })
    }
}