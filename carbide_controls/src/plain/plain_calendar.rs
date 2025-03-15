use std::collections::HashSet;
use std::fmt::{Debug, Formatter};
use std::iter;
use std::ops::RangeInclusive;

use chrono::{Datelike, Local, Month, NaiveDate, Weekday};
use carbide::color::RED;
use carbide::math::num_traits::FromPrimitive;
use carbide::widget::MouseAreaActionContext;
use carbide_core::closure;
use carbide_core::CommonWidgetImpl;
use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::EnvironmentColor;
use carbide_core::flags::WidgetFlag;
use carbide_core::state::{AnyReadState, AnyState, LocalState, Map1, Map2, Map3, ReadState, ReadStateExtNew, State, StateExtNew};
use carbide_core::widget::{AnyWidget, CommonWidget, ForEach, HStack, Image, Rectangle, Spacer, Text, VGrid, VGridColumn, VStack, Widget, WidgetExt, WidgetId, ZStack};
use crate::button::{Button, PlainStyle};
use crate::ControlsExt;

pub trait PlainCalendarHeaderDelegate: Clone + 'static {
    fn call(&self, weekday: impl ReadState<T=Weekday>) -> Box<dyn AnyWidget>;
}

impl<K> PlainCalendarHeaderDelegate for K where K: Fn(Box<dyn AnyReadState<T=Weekday>>) -> Box<dyn AnyWidget> + Clone + 'static {
    fn call(&self, weekday: impl ReadState<T=Weekday>) -> Box<dyn AnyWidget> {
        self(weekday.as_dyn_read())
    }
}

type DefaultPlainCalendarHeaderDelegate = fn(Box<dyn AnyReadState<T=Weekday>>) -> Box<dyn AnyWidget>;


pub trait PlainCalendarTitleDelegate: Clone + 'static {
    fn call(&self, month: impl State<T=Month>, year: impl State<T=i32>, selection: DateSelection) -> Box<dyn AnyWidget>;
}

impl<K> PlainCalendarTitleDelegate for K where K: Fn(Box<dyn AnyState<T=Month>>, Box<dyn AnyState<T=i32>>, DateSelection) -> Box<dyn AnyWidget> + Clone + 'static {
    fn call(&self, month: impl State<T=Month>, year: impl State<T=i32>, selection: DateSelection) -> Box<dyn AnyWidget> {
        self(month.as_dyn(), year.as_dyn(), selection)
    }
}

type DefaultPlainCalendarTitleDelegate = fn(Box<dyn AnyState<T=Month>>, Box<dyn AnyState<T=i32>>, DateSelection) -> Box<dyn AnyWidget>;


pub trait PlainCalendarItemDelegate: Clone + 'static {
    fn call(&self, selected: impl ReadState<T=Selected>, hovered: impl ReadState<T=bool>, pressed: impl ReadState<T=bool>, date: impl ReadState<T=NaiveDate>) -> Box<dyn AnyWidget>;
}

impl<K> PlainCalendarItemDelegate for K where K: Fn(Box<dyn AnyReadState<T=Selected>>, Box<dyn AnyReadState<T=bool>>, Box<dyn AnyReadState<T=bool>>, Box<dyn AnyReadState<T=NaiveDate>>) -> Box<dyn AnyWidget> + Clone + 'static {
    fn call(&self, selected: impl ReadState<T=Selected>, hovered: impl ReadState<T=bool>, pressed: impl ReadState<T=bool>, date: impl ReadState<T=NaiveDate>) -> Box<dyn AnyWidget> {
        self(selected.as_dyn_read(), hovered.as_dyn_read(), pressed.as_dyn_read(), date.as_dyn_read())
    }
}

type DefaultPlainCalendarItemDelegate = fn(Box<dyn AnyReadState<T=Selected>>, Box<dyn AnyReadState<T=bool>>, Box<dyn AnyReadState<T=bool>>, Box<dyn AnyReadState<T=NaiveDate>>) -> Box<dyn AnyWidget>;


pub trait PlainCalendarHiddenDelegate: Clone + 'static {
    fn call(&self) -> Box<dyn AnyWidget>;
}

impl<K> PlainCalendarHiddenDelegate for K where K: Fn() -> Box<dyn AnyWidget> + Clone + 'static {
    fn call(&self) -> Box<dyn AnyWidget> {
        self()
    }
}

type DefaultPlainCalendarHiddenDelegate = fn() -> Box<dyn AnyWidget>;

#[derive(Clone, Debug)]
pub enum Selected {
    Start,
    End,
    In,
    None,
}

#[derive(Clone, Debug)]
pub enum DateSelection {
    Single(Box<dyn AnyState<T=Option<NaiveDate>>>),
    Multi(Box<dyn AnyState<T=HashSet<NaiveDate>>>),
    Range(Box<dyn AnyState<T=Option<RangeInclusive<NaiveDate>>>>),
}

impl Into<DateSelection> for LocalState<Option<NaiveDate>> {
    fn into(self) -> DateSelection {
        DateSelection::Single(self.as_dyn())
    }
}

impl Into<DateSelection> for LocalState<HashSet<NaiveDate>> {
    fn into(self) -> DateSelection {
        DateSelection::Multi(self.as_dyn())
    }
}

impl Into<DateSelection> for LocalState<Option<RangeInclusive<NaiveDate>>> {
    fn into(self) -> DateSelection {
        DateSelection::Range(self.as_dyn())
    }
}

#[derive(Clone, Widget)]
pub struct PlainCalendar<H, I, D, T>
where
    H: PlainCalendarHeaderDelegate,
    I: PlainCalendarItemDelegate,
    D: PlainCalendarHiddenDelegate,
    T: PlainCalendarTitleDelegate,
{
    #[id] id: WidgetId,
    position: Position,
    dimension: Dimension,
    child: Box<dyn AnyWidget>,
    month: Box<dyn AnyState<T=Month>>,
    year: Box<dyn AnyState<T=i32>>,
    selection: DateSelection,
    spacing: Dimension,

    header_delegate: H,
    item_delegate: I,
    hidden_delegate: D,
    title_delegate: T,
    column: VGridColumn,
}

impl PlainCalendar<DefaultPlainCalendarHeaderDelegate, DefaultPlainCalendarItemDelegate, DefaultPlainCalendarHiddenDelegate, DefaultPlainCalendarTitleDelegate> {
    pub fn new<S: Into<DateSelection>>(selection: S) -> PlainCalendar<DefaultPlainCalendarHeaderDelegate, DefaultPlainCalendarItemDelegate, DefaultPlainCalendarHiddenDelegate, DefaultPlainCalendarTitleDelegate> {
        let month = LocalState::new(Month::from_u32(Local::now().month()).unwrap());
        let year = LocalState::new(Local::now().year());
        let spacing = Dimension::new(10.0, 10.0);
        let column = VGridColumn::Flexible { minimum: 0.0, maximum: f64::MAX };

        Self::new_internal(month.as_dyn(), year.as_dyn(), selection.into(), spacing, Self::default_header_delegate, Self::default_item_delegate, Self::default_hidden_delegate, column, Self::default_title_delegate)
    }

    fn default_header_delegate(weekday: impl ReadState<T=Weekday>) -> Box<dyn AnyWidget> {
        ZStack::new((
            Rectangle::new().fill(EnvironmentColor::Orange),
            Text::new(Map1::read_map(weekday, |d| format!("{}", d))),
        )).boxed()
    }

    fn default_item_delegate(selected: impl ReadState<T=Selected>, _hovered: impl ReadState<T=bool>, _pressed: impl ReadState<T=bool>, date: impl ReadState<T=NaiveDate>) -> Box<dyn AnyWidget> {
        let color = Map1::read_map(selected, |selected| {
            match selected {
                Selected::Start => EnvironmentColor::Green,
                Selected::End => EnvironmentColor::Green,
                Selected::In => EnvironmentColor::Pink,
                Selected::None => EnvironmentColor::Accent,
            }
        });

        ZStack::new((
            Rectangle::new().fill(color),
            Text::new(Map1::read_map(date.clone(), |d| d.day())),
        )).boxed()
    }

    fn default_hidden_delegate() -> Box<dyn AnyWidget> {
        Rectangle::new().fill(RED).boxed()
    }

    fn default_title_delegate(month: impl State<T=Month>, year: impl State<T=i32>, _selection: DateSelection) -> Box<dyn AnyWidget> {
        HStack::new((
            Text::new(Map1::read_map(month.clone(), |m| format!("{:?}", m))),
            Text::new(year.clone()),
            Spacer::new(),
            Button::new(Image::new("icons/arrow-left-s-line.png"), closure!(|ctx: MouseAreaActionContext| {
                if *$month == Month::January {
                    *$year -= 1;
                }
                *$month = month.pred();
            })),
            Button::new(Image::new("icons/arrow-right-s-line.png"), closure!(|ctx: MouseAreaActionContext| {
                if *$month == Month::December {
                    *$year += 1;
                }
                *$month = month.succ();
            })),
        )).boxed()
    }
}

impl<H: PlainCalendarHeaderDelegate, I: PlainCalendarItemDelegate, D: PlainCalendarHiddenDelegate, T: PlainCalendarTitleDelegate> PlainCalendar<H, I, D, T> {

    pub fn month(self, month: Box<dyn AnyState<T=Month>>) -> PlainCalendar<H, I, D, T> {
        Self::new_internal(month, self.year, self.selection, self.spacing, self.header_delegate, self.item_delegate, self.hidden_delegate, self.column, self.title_delegate)
    }

    pub fn year(self, year: Box<dyn AnyState<T=i32>>) -> PlainCalendar<H, I, D, T> {
        Self::new_internal(self.month, year, self.selection, self.spacing, self.header_delegate, self.item_delegate, self.hidden_delegate, self.column, self.title_delegate)
    }

    pub fn spacing(self, spacing: Dimension) -> PlainCalendar<H, I, D, T> {
        Self::new_internal(self.month, self.year, self.selection, spacing, self.header_delegate, self.item_delegate, self.hidden_delegate, self.column, self.title_delegate)
    }

    pub fn header_delegate<H2: PlainCalendarHeaderDelegate>(self, header_delegate: H2) -> PlainCalendar<H2, I, D, T> {
        Self::new_internal(self.month, self.year, self.selection, self.spacing, header_delegate, self.item_delegate, self.hidden_delegate, self.column, self.title_delegate)
    }

    pub fn item_delegate<I2: PlainCalendarItemDelegate>(self, item_delegate: I2) -> PlainCalendar<H, I2, D, T> {
        Self::new_internal(self.month, self.year, self.selection, self.spacing, self.header_delegate, item_delegate, self.hidden_delegate, self.column, self.title_delegate)
    }

    pub fn hidden_delegate<D2: PlainCalendarHiddenDelegate>(self, hidden_delegate: D2) -> PlainCalendar<H, I, D2, T> {
        Self::new_internal(self.month, self.year, self.selection, self.spacing, self.header_delegate, self.item_delegate, hidden_delegate, self.column, self.title_delegate)
    }

    pub fn title_delegate<T2: PlainCalendarTitleDelegate>(self, title_delegate: T2) -> PlainCalendar<H, I, D, T2> {
        Self::new_internal(self.month, self.year, self.selection, self.spacing, self.header_delegate, self.item_delegate, self.hidden_delegate, self.column, title_delegate)
    }

    pub fn column(self, column: VGridColumn) -> PlainCalendar<H, I, D, T> {
        Self::new_internal(self.month, self.year, self.selection, self.spacing, self.header_delegate, self.item_delegate, self.hidden_delegate, column, self.title_delegate)
    }

    fn new_internal<H2: PlainCalendarHeaderDelegate, I2: PlainCalendarItemDelegate, D2: PlainCalendarHiddenDelegate, T2: PlainCalendarTitleDelegate>(month: Box<dyn AnyState<T=Month>>, year: Box<dyn AnyState<T=i32>>, selection: DateSelection, spacing: Dimension, header_delegate: H2, item_delegate: I2, hidden_delegate: D2, column: VGridColumn, title_delegate: T2) -> PlainCalendar<H2, I2, D2, T2> {
        let first_weekday = Map2::read_map(month.clone(), year.clone(), |m, y| {
            NaiveDate::from_ymd_opt(*y, m.number_from_month(), 1).unwrap().weekday().num_days_from_monday()
        });

        let days_in_month = Map2::read_map(month.clone(), year.clone(), |m, y| {
            if m.number_from_month() == 12 {
                NaiveDate::from_ymd_opt(*y + 1, 1, 1)
            } else {
                NaiveDate::from_ymd_opt(*y, m.number_from_month() + 1, 1)
            }.unwrap().signed_duration_since(NaiveDate::from_ymd_opt(*y, m.number_from_month(), 1).unwrap())
                .num_days() as u32
        });

        let month2 = month.clone();
        let year2 = year.clone();
        let selection2 = selection.clone();

        let header_delegate_foreach = header_delegate.clone();
        let item_delegate_foreach = item_delegate.clone();
        let hidden_delegate_foreach = hidden_delegate.clone();

        let headers = ForEach::new(vec![Weekday::Mon, Weekday::Tue, Weekday::Wed, Weekday::Thu, Weekday::Fri, Weekday::Sat, Weekday::Sun], move |day, _| {
            header_delegate_foreach.call(day)
        });

        let hidden_offsets = ForEach::new_read(first_weekday, move |_, _| {
            hidden_delegate_foreach.call()
        });

        let dates = ForEach::new_read(days_in_month, move |day: Box<dyn AnyState<T=u32>>, _| {
            let item_delegate_foreach = item_delegate_foreach.clone();

            let date = Map3::read_map(year2.clone(), month2.clone(), day.clone(), |y, m, d| {
                NaiveDate::from_ymd_opt(*y, m.number_from_month(), *d + 1).unwrap()
            });

            let selected = match selection2.clone() {
                DateSelection::Single(s) => Map2::read_map(date.clone(), s, |d, s| {
                    if Some(d) == s.as_ref() {
                        Selected::Start
                    } else {
                        Selected::None
                    }
                }).as_dyn_read(),
                DateSelection::Multi(m) => Map2::read_map(date.clone(), m, |d, s| {
                    if s.contains(d) {
                        Selected::Start
                    } else {
                        Selected::None
                    }
                }).as_dyn_read(),
                DateSelection::Range(range) => {
                    Map2::read_map(date.clone(), range, |d, s| {
                        if let Some(range) = s.as_ref() {
                            if range.start() == d {
                                Selected::Start
                            } else if range.end() == d {
                                Selected::End
                            } else if range.contains(d) {
                                Selected::In
                            } else {
                                Selected::None
                            }
                        } else {
                            Selected::None
                        }
                    }).as_dyn_read()
                }
            };

            let s2 = selection2.clone();

            let hovered = LocalState::new(false);
            let pressed = LocalState::new(false);

            let hovered_for_delegate = hovered.clone();
            let pressed_for_delegate = pressed.clone();
            let date_for_delegate = date.clone();

            let label = item_delegate_foreach.call(selected.clone(), hovered_for_delegate.clone(), pressed_for_delegate.clone(), date_for_delegate.clone());

            Button::new(label, move |ctx: MouseAreaActionContext| {
                let date = date.value().clone();

                match s2.clone() {
                    DateSelection::Single(mut s) => {
                        *s.value_mut() = Some(date);
                    }
                    DateSelection::Multi(mut m) => {
                        let value = &mut *m.value_mut();

                        if !value.contains(&date) {
                            value.insert(date);
                        } else {
                            value.remove(&date);
                        }
                    }
                    DateSelection::Range(mut r) => {
                        let value = &mut *r.value_mut();

                        if let Some(val) = value {
                            if &date < val.start() {
                                *val = date..=date;
                            } else if val.start() == val.end() {
                                *val = *val.start()..=date;
                            } else {
                                *val = date..=date;
                            }
                        } else {
                            *value = Some(date..=date);
                        }
                    }
                }
            }).hovered(hovered)
                .pressed(pressed)
        });

        let columns = iter::repeat(column.clone()).take(7).collect::<Vec<_>>();

        let grid = VGrid::new((
            headers,
            hidden_offsets,
            dates
        ), columns)
            .spacing(spacing.clone());

        PlainCalendar {
            id: WidgetId::new(),
            position: Default::default(),
            dimension: Default::default(),
            child: VStack::new((
                title_delegate.call(month.clone(), year.clone(), selection.clone()).flagged(WidgetFlag::USEMAXCROSSAXIS),
                grid
            )).spacing(0.0).button_style(PlainStyle).boxed(),
            month,
            year,
            selection,
            spacing,
            header_delegate,
            item_delegate,
            hidden_delegate,
            title_delegate,
            column,
        }
    }
}

impl<H: PlainCalendarHeaderDelegate, I: PlainCalendarItemDelegate, D: PlainCalendarHiddenDelegate, T: PlainCalendarTitleDelegate> CommonWidget for PlainCalendar<H, I, D, T> {
    CommonWidgetImpl!(self, child: self.child, position: self.position, dimension: self.dimension, flexibility: 10);
}

impl<H: PlainCalendarHeaderDelegate, I: PlainCalendarItemDelegate, D: PlainCalendarHiddenDelegate, T: PlainCalendarTitleDelegate> Debug for PlainCalendar<H, I, D, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlainCalendar")
            .field("id", &self.id)
            .field("position", &self.position)
            .field("dimension", &self.dimension)
            .field("selection", &self.selection)
            .finish()
    }
}
