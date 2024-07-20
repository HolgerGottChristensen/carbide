use chrono::{Local, Month, NaiveDate, Weekday};
use chrono::Datelike;
use carbide::color::ColorExt;

use carbide_core::a;
use carbide_core::color::TRANSPARENT;
use carbide_core::draw::Dimension;
use carbide_core::environment::{EnvironmentColor, IntoColorReadState};
use carbide_core::state::{LocalState, Map1, Map2, Map4, ReadState, State, ValueState};
use carbide_core::widget::{AnyWidget, Circle, HStack, Image, Rectangle, Spacer, Text, VGridColumn, WidgetExt, ZStack};

use crate::{PlainButton, PlainCalendar, PlainCalendarHeaderDelegate, PlainCalendarHiddenDelegate, PlainCalendarItemDelegate, PlainCalendarTitleDelegate, Selected};
use crate::plain::plain_calendar::DateSelection;

pub struct Calendar;

impl Calendar {
    pub fn new<S: Into<DateSelection>>(selection: S) -> PlainCalendar<CalendarHeaderDelegate, CalendarItemDelegate, CalendarHiddenDelegate, CalendarTitleDelegate> {
        PlainCalendar::new(selection)
            .header_delegate(CalendarHeaderDelegate)
            .item_delegate(CalendarItemDelegate)
            .hidden_delegate(CalendarHiddenDelegate)
            .title_delegate(CalendarTitleDelegate)
            .column(VGridColumn::Fixed(30.0))
            .spacing(Dimension::new(7.0, 5.0))
    }
}

#[derive(Clone, Debug)]
pub struct CalendarHeaderDelegate;

impl PlainCalendarHeaderDelegate for CalendarHeaderDelegate {
    fn call(&self, weekday: impl ReadState<T=Weekday>) -> Box<dyn AnyWidget> {
        ZStack::new((
            Rectangle::new().fill(TRANSPARENT),
            Text::new(Map1::read_map(weekday, |d| format!("{}", d))).foreground_color(EnvironmentColor::TertiaryLabel),
        )).frame(30.0, 30.0).boxed()
    }
}

#[derive(Clone, Debug)]
pub struct CalendarItemDelegate;

impl PlainCalendarItemDelegate for CalendarItemDelegate {
    fn call(&self, selected: impl ReadState<T=Selected>, _hovered: impl ReadState<T=bool>, _pressed: impl ReadState<T=bool>, date: impl ReadState<T=NaiveDate>) -> Box<dyn AnyWidget> {
        let today = ValueState::new(Local::now().naive_local().date());

        let is_today = Map2::read_map(today, date.clone(), |today, date| today == date);

        let color = Map4::read_map(selected, is_today, EnvironmentColor::Accent.color(), EnvironmentColor::Label.color(), |selected, is_today, accent, label| {
            match selected {
                Selected::Start => accent.with_opacity(0.1),
                Selected::End => accent.with_opacity(0.1),
                Selected::In => accent.with_opacity(0.1),
                Selected::None => {
                    if *is_today {
                        label.with_opacity(0.02)
                    } else {
                        TRANSPARENT
                    }
                },
            }
        });

        ZStack::new((
            Circle::new().fill(color),
            Text::new(Map1::read_map(date.clone(), |d| d.day())),
        )).frame(30.0, 30.0).boxed()
    }
}

#[derive(Clone, Debug)]
pub struct CalendarHiddenDelegate;

impl PlainCalendarHiddenDelegate for CalendarHiddenDelegate {
    fn call(&self) -> Box<dyn AnyWidget> {
        Rectangle::new().fill(TRANSPARENT).frame(30.0,30.0).boxed()
    }
}

#[derive(Clone, Debug)]
pub struct CalendarTitleDelegate;

impl PlainCalendarTitleDelegate for CalendarTitleDelegate {
    fn call(&self, month: impl State<T=Month>, year: impl State<T=i32>, _selection: DateSelection) -> Box<dyn AnyWidget> {
        let hover1 = LocalState::new(false);
        let hover2 = LocalState::new(false);

        let color1 = Map2::read_map(hover1.clone(), EnvironmentColor::Accent.color(), |hover, accent| {
            if *hover {
                accent.lightened(0.1)
            } else {
                *accent
            }
        });

        let color2 = Map2::read_map(hover2.clone(), EnvironmentColor::Accent.color(), |hover, accent| {
            if *hover {
                accent.lightened(0.1)
            } else {
                *accent
            }
        });

        HStack::new((
            Text::new(Map1::read_map(month.clone(), |m| format!("{:?}", m))),
            Text::new(year.clone()),
            Spacer::new(),
            PlainButton::new(a!(|_,_| {
                if *$month == Month::January {
                    *$year -= 1;
                }
                *$month = month.pred();
            })).delegate(move |_, _, _, _| {
                Image::new_icon("icons/arrow-left-s-line.png").resizeable().foreground_color(color1.clone()).frame(30.0, 30.0).boxed()
            }).hovered(hover1),
            PlainButton::new(a!(|_,_| {
                if *$month == Month::December {
                    *$year += 1;
                }
                *$month = month.succ();
            })).delegate(move |_, _, _, _| {
                Image::new_icon("icons/arrow-right-s-line.png").resizeable().foreground_color(color2.clone()).frame(30.0, 30.0).boxed()
            }).hovered(hover2),
        )).boxed()
    }
}