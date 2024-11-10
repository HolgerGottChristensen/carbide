use carbide::draw::Alignment;
use carbide_core::color::TRANSPARENT;
use carbide_core::draw::{Dimension, Position, Rect};
use carbide_core::environment::{EnvironmentColor, IntoColorReadState};
use carbide_core::focus::Focus;
use carbide_core::state::{AnyReadState, AnyState, LocalState, Map1, Map2, Map3, ReadStateExtNew};
use carbide_core::widget::*;

use crate::{Calendar, PlainDatePicker};
use crate::plain_calendar::DateSelection;

pub struct DatePicker;

impl DatePicker {
    pub fn new(selection: impl Into<DateSelection>) -> PlainDatePicker<LocalState<Focus>, bool> {
        PlainDatePicker::new(selection)
            .delegate(Self::delegate)
            .popup_delegate(Self::popup)
    }

    fn delegate(
        selection: DateSelection,
        focused: Box<dyn AnyState<T=Focus>>,
        enabled: Box<dyn AnyReadState<T=bool>>,
        //text_delegate: TextDelegateGenerator,
    ) -> Box<dyn AnyWidget> {
        let label_color = Map1::read_map(enabled.clone(), |enabled| {
            if *enabled {
                EnvironmentColor::Label
            } else {
                EnvironmentColor::TertiaryLabel
            }
        });

        let stroke_color = match selection.clone() {
            DateSelection::Single(s) => {
                Map1::read_map(s, |a| {
                    if a.is_none() {
                        EnvironmentColor::Red
                    } else {
                        EnvironmentColor::OpaqueSeparator
                    }
                }).as_dyn_read()
            }
            DateSelection::Multi(s) => {
                Map1::read_map(s, |a| {
                    if a.is_empty() {
                        EnvironmentColor::Red
                    } else {
                        EnvironmentColor::OpaqueSeparator
                    }
                }).as_dyn_read()
            }
            DateSelection::Range(s) => {
                Map1::read_map(s, |a| {
                    if let Some(r) = a {
                        if r.start() == r.end() {
                            EnvironmentColor::Red
                        } else {
                            EnvironmentColor::OpaqueSeparator
                        }
                    } else {
                        EnvironmentColor::Red
                    }
                }).as_dyn_read()
            }
        };

        let outline_color = Map2::read_map(EnvironmentColor::Accent.color(), focused, |color, focused| {
            if *focused == Focus::Focused {
                *color
            } else {
                TRANSPARENT
            }
        });

        let background_color = Map1::read_map(enabled.clone(), |enabled| {
            if *enabled {
                EnvironmentColor::SecondarySystemBackground
            } else {
                EnvironmentColor::TertiarySystemBackground
            }
        });

        let text = match selection {
            DateSelection::Single(s) => {
                Map1::read_map(s, |s| {
                    s.map_or("".to_string(), |d| d.format("%d/%m/%Y").to_string())
                }).as_dyn_read()
            }
            DateSelection::Multi(s) => {
                Map1::read_map(s, |s| {
                    let mut list = s.iter().collect::<Vec<_>>();

                    list.sort();

                    list.into_iter().map(|d| {
                        d.format("%d/%m/%Y").to_string()
                    }).collect::<Vec<_>>().join(", ")
                }).as_dyn_read()
            }
            DateSelection::Range(s) => {
                Map1::read_map(s, |s| {
                    s.as_ref().map_or("".to_string(), |r| {
                        if r.start() == r.end() {
                            format!("{}", r.start().format("%d/%m/%Y").to_string())
                        } else {
                            format!("{} — {}", r.start().format("%d/%m/%Y").to_string(), r.end().format("%d/%m/%Y").to_string())
                        }
                    })
                }).as_dyn_read()
            }
        };

        ZStack::new((
            RoundedRectangle::new(CornerRadii::all(3.0))
                .fill(background_color)
                .stroke(stroke_color)
                .stroke_style(1.0),
            HStack::new((
                Text::new(text)
                    .wrap_mode(Wrap::None)
                    .foreground_color(label_color),
                Spacer::new(),
            )).clip()
            .padding(EdgeInsets::vertical_horizontal(0.0, 5.0)),
        )).with_alignment(Alignment::Leading)
        .background(
            RoundedRectangle::new(CornerRadii::all(3.0))
                .stroke(outline_color)
                .stroke_style(1.0)
                .padding(-1.0)
        )
            .frame_fixed_height(22.0)
            .boxed()
    }

    fn popup(
        selection: DateSelection,
        _focused: Box<dyn AnyState<T=Focus>>,
        _enabled: Box<dyn AnyReadState<T=bool>>,
        parent_position: Box<dyn AnyReadState<T=Position>>,
        parent_dimension: Box<dyn AnyReadState<T=Dimension>>,
    ) -> Box<dyn AnyWidget> {
        let geometry = LocalState::new(Rect::default());

        let x = Map3::read_map_env(parent_position.clone(), parent_dimension.clone(), geometry.clone(), |env, position, dimension, geometry| {
            //(position.x + dimension.width / 2.0 - geometry.dimension.width / 2.0).min(env.current_window_width() - geometry.dimension.width).max(0.0)
            todo!()
        });

        let y = Map3::read_map_env(parent_position, parent_dimension, geometry.clone(), |env, position, dimension, geometry| {
            /*// The bottom of the calendar popup is below the bottom of the window when placed below the input.
            let oob_below = position.y + dimension.height + 1.0 + geometry.dimension.height > env.current_window_height();
            // The top of the calendar popup is above the top of the window when placed below the input.
            let oob_above = position.y - 1.0 - geometry.dimension.height < 0.0;

            if oob_below && oob_above {
                (position.y + dimension.height + 1.0).min(env.current_window_height() - geometry.dimension.height).max(0.0)
            } else if !oob_below {
                (position.y + dimension.height + 1.0).max(0.0)
            } else {
                (position.y - 1.0 - geometry.dimension.height).min(env.current_window_height() - geometry.dimension.height)
            }*/
            todo!()
        });

        /*Calendar::new(selection)
            .padding(10.0)
            .background(
                RoundedRectangle::new(5.0)
                    .fill(EnvironmentColor::SecondarySystemBackground)
                    .stroke(EnvironmentColor::OpaqueSeparator)
                    .stroke_style(1.0)
            )
            .on_click(|ctx: MouseAreaActionContext| {})
            .on_click_outside(|ctx: MouseAreaActionContext| {
                ctx.env.transfer_widget(Some("controls_popup_layer".to_string()), WidgetTransferAction::Pop);
            })
            .geometry(geometry)
            .absolute(x, y)
            .boxed()*/

        todo!()
    }
}
