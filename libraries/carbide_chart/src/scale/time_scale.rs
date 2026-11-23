use std::fmt::{Debug, Formatter};
use std::ops::Range;
use std::sync::Arc;

use carbide::draw::Scalar;
use carbide::environment::Environment;
use carbide::state::{AnyReadState, IntoReadState, ReadState, ReadStateExt};
use carbide_derive::StateValue;
use chrono::{DateTime, Datelike, FixedOffset, Local, Months, NaiveDate, NaiveDateTime, TimeDelta, TimeZone, Timelike};

use crate::{DataValue, Value};
use crate::scale::{Axis, AxisOptions, AxisPosition, PolarRole, Scale, MIN_ZOOM_SPAN};

/// The time zone a `TimeScale` places its ticks in and shows its labels in: whole days start at
/// midnight in it, and months on its first of the month.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default, StateValue)]
pub enum TimeScaleZone {
    /// The machine's local time zone, with its daylight saving changes. The default.
    #[default]
    Local,
    /// A fixed offset from UTC, such as UTC itself (`FixedOffset::east_opt(0)`), the same on every
    /// machine.
    Fixed(FixedOffset),
}

impl TimeScaleZone {
    /// The wall-clock time at `timestamp`, in seconds since the Unix epoch, in this zone.
    fn wall_time(&self, timestamp: Scalar) -> Option<NaiveDateTime> {
        Some(self.date_time(timestamp)?.naive_local())
    }

    /// The moment at `timestamp`, with this zone's offset at that moment.
    fn date_time(&self, timestamp: Scalar) -> Option<DateTime<FixedOffset>> {
        let seconds = timestamp.floor();
        let nanos = ((timestamp - seconds) * 1e9).round().min(999_999_999.0) as u32;
        let utc = DateTime::from_timestamp(seconds as i64, nanos)?;

        Some(match self {
            TimeScaleZone::Local => utc.with_timezone(&Local).fixed_offset(),
            TimeScaleZone::Fixed(offset) => utc.with_timezone(offset),
        })
    }

    /// The timestamp of the wall-clock time `wall` in this zone. A time that occurs twice, as
    /// clocks are turned back, is its first; a time skipped as clocks are turned forward is the
    /// moment after the gap.
    fn timestamp(&self, wall: NaiveDateTime) -> Option<Scalar> {
        fn first<Tz: TimeZone>(zone: &Tz, wall: NaiveDateTime) -> Option<Scalar> {
            let moment = zone.from_local_datetime(&wall).earliest()
                .or_else(|| zone.from_local_datetime(&(wall + TimeDelta::hours(1))).earliest())?;
            Some(moment.timestamp() as Scalar + moment.timestamp_subsec_nanos() as Scalar / 1e9)
        }

        match self {
            TimeScaleZone::Local => first(&Local, wall),
            TimeScaleZone::Fixed(offset) => first(offset, wall),
        }
    }
}

/// The unit a `TimeScale`'s ticks are spaced in, chosen to fit the range (see `TimeScale`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum TimeUnit {
    Second,
    Minute,
    Hour,
    Day,
    Week,
    Month,
    Year,
}

impl TimeUnit {
    /// About how long one of the unit is, in seconds, for choosing a unit: a month and a year are
    /// their average lengths in the Gregorian calendar.
    fn seconds(&self) -> Scalar {
        match self {
            TimeUnit::Second => 1.0,
            TimeUnit::Minute => 60.0,
            TimeUnit::Hour => 3_600.0,
            TimeUnit::Day => 86_400.0,
            TimeUnit::Week => 604_800.0,
            TimeUnit::Month => 2_629_746.0,
            TimeUnit::Year => 31_556_952.0,
        }
    }

    /// The steps ticks can be apart in this unit, smallest first, each dividing the next larger
    /// unit evenly where it can, so ticks fall on round times: 15 minutes, 6 hours, a quarter.
    /// Years continue with 1, 2 and 5 times powers of ten (see `candidate_steps`).
    fn steps(&self) -> &'static [u32] {
        match self {
            TimeUnit::Second | TimeUnit::Minute => &[1, 2, 5, 10, 15, 30],
            TimeUnit::Hour => &[1, 2, 3, 6, 12],
            TimeUnit::Day => &[1, 2],
            TimeUnit::Week => &[1, 2],
            TimeUnit::Month => &[1, 2, 3, 6],
            TimeUnit::Year => &[1, 2, 5],
        }
    }

    /// The start of the step of `step` units that `wall` falls in: seconds, minutes and hours
    /// counted from the start of the next larger unit, weeks from Monday, months from January, and
    /// years from year 0, so steps line up the same way whatever the range.
    fn floor(&self, step: u32, wall: NaiveDateTime) -> NaiveDateTime {
        let date = wall.date();
        let midnight = |date: NaiveDate| date.and_hms_opt(0, 0, 0).expect("midnight exists");

        match self {
            TimeUnit::Second => date.and_hms_opt(wall.hour(), wall.minute(), wall.second() - wall.second() % step).expect("a second of the day"),
            TimeUnit::Minute => date.and_hms_opt(wall.hour(), wall.minute() - wall.minute() % step, 0).expect("a minute of the day"),
            TimeUnit::Hour => date.and_hms_opt(wall.hour() - wall.hour() % step, 0, 0).expect("an hour of the day"),
            TimeUnit::Day => midnight(date),
            TimeUnit::Week => midnight(date - TimeDelta::days(date.weekday().num_days_from_monday() as i64)),
            TimeUnit::Month => {
                let month0 = date.month0() - date.month0() % step;
                midnight(NaiveDate::from_ymd_opt(date.year(), month0 + 1, 1).expect("the first of a month"))
            }
            TimeUnit::Year => {
                let year = date.year() - date.year().rem_euclid(step as i32);
                midnight(NaiveDate::from_ymd_opt(year, 1, 1).unwrap_or(date))
            }
        }
    }

    /// `wall` moved on by `step` units, on the calendar: a month later is the same day of the next
    /// month.
    fn add(&self, step: u32, wall: NaiveDateTime) -> Option<NaiveDateTime> {
        match self {
            TimeUnit::Second => wall.checked_add_signed(TimeDelta::seconds(step as i64)),
            TimeUnit::Minute => wall.checked_add_signed(TimeDelta::minutes(step as i64)),
            TimeUnit::Hour => wall.checked_add_signed(TimeDelta::hours(step as i64)),
            TimeUnit::Day => wall.checked_add_signed(TimeDelta::days(step as i64)),
            TimeUnit::Week => wall.checked_add_signed(TimeDelta::weeks(step as i64)),
            TimeUnit::Month => wall.checked_add_months(Months::new(step)),
            TimeUnit::Year => wall.checked_add_months(Months::new(step.checked_mul(12)?)),
        }
    }
}

/// Every unit and step ticks can be spaced in, smallest first: the steps of each unit (see
/// `TimeUnit::steps`), then years in 1, 2 and 5 times powers of ten.
fn candidate_steps() -> impl Iterator<Item=(TimeUnit, u32)> {
    let units = [TimeUnit::Second, TimeUnit::Minute, TimeUnit::Hour, TimeUnit::Day, TimeUnit::Week, TimeUnit::Month];
    let years = (0..9).flat_map(|power| [1, 2, 5].map(|factor| (TimeUnit::Year, factor * 10u32.pow(power))));

    units.into_iter()
        .flat_map(|unit| unit.steps().iter().map(move |step| (unit, *step)))
        .chain(years)
}

/// A label for the date at a tick, given the date in the scale's time zone and the unit the ticks
/// are spaced in (see `TimeScale::format`).
type DateFormat = Arc<dyn Fn(DateTime<FixedOffset>, TimeUnit) -> String>;

/// A scale of dates and times. Values are seconds since the Unix epoch, which is what dates
/// stand for as coordinates (`DataValue` for chrono's `DateTime`); a date type of your own works
/// by giving its timestamp.
///
/// - Ticks fall on round times in the scale's time zone (`time_zone`, local by default): it picks
///   the smallest step, in seconds, minutes, hours, days, weeks, months or years, that fits the
///   range in at most 6 steps along a horizontal axis or 10 along a vertical one, and rounds the
///   range out to whole steps. Days start at midnight, weeks on Monday, and months on the first,
///   so ticks a month apart are a month apart on the calendar, not 30 days.
/// - Labels show the date for day, week, month and year steps, and the time for second, minute
///   and hour steps, formatted for the environment's locale, unless `format` gives your own.
/// - With an offset (see `Scale::set_offset`), half a step is added at both ends, so bars on the
///   first and last tick aren't cut in half.
///
/// It is the chart's default x or y scale when the data placed on it are dates (`Value::Time`,
/// as chrono's `DateTime` is): the controllers report their datasets' values (see
/// `ScaleBounds::report_value`). Set it on the chart (`Chart::scale`) to give it options, such
/// as its time zone, or for a scale other than the default x and y.
#[derive(Clone)]
pub struct TimeScale {
    axis: Axis,
    axis_options: AxisOptions,
    offset: bool,
    zone: Box<dyn AnyReadState<T=TimeScaleZone>>,
    format: Option<DateFormat>,
    range: Range<Scalar>,
    nice_range: Range<Scalar>,
    ticks: Vec<Scalar>,
    /// The ticks in the zoom window, when the scale shows only part of its range (see `zoom`).
    visible_ticks: Option<Vec<Scalar>>,
    unit: TimeUnit,
}

impl Debug for TimeScale {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TimeScale")
            .field("axis", &self.axis)
            .field("axis_options", &self.axis_options)
            .field("offset", &self.offset)
            .field("zone", &self.zone)
            .field("format", &self.format.as_ref().map(|_| "custom"))
            .field("nice_range", &self.nice_range)
            .field("ticks", &self.ticks)
            .field("unit", &self.unit)
            .finish()
    }
}

impl TimeScale {
    /// The smallest unit and step whose ticks, rounded out to whole steps, span `start` to `end`
    /// in at most `max_spaces` steps, with those ticks.
    fn fitting_ticks(&self, start: Scalar, end: Scalar) -> Option<(TimeUnit, Vec<Scalar>)> {
        let max_spaces = self.max_spaces() as usize;

        candidate_steps()
            .filter(|(unit, step)| (end - start) / (unit.seconds() * *step as Scalar) <= max_spaces as Scalar)
            .find_map(|(unit, step)| {
                let ticks = self.ticks_for(unit, step, start, end)?;
                (ticks.len() >= 2 && ticks.len() - 1 <= max_spaces).then_some((unit, ticks))
            })
    }

    /// Decides the window the scale shows from the zoom asked for, and, when it shows only part
    /// of the range, the ticks for that part and the unit they are labelled in.
    fn zoom_ticks(&mut self) {
        let window = self.axis_options.zoom().limited(MIN_ZOOM_SPAN);
        self.axis_options.set_window(window);
        self.visible_ticks = None;

        if window.is_whole() {
            return;
        }

        let (start, end) = self.mapped_range();
        let (low, high) = (start + window.start * (end - start), start + window.end * (end - start));

        if let Some((unit, ticks)) = self.fitting_ticks(low, high) {
            self.unit = unit;
            let tolerance = (high - low) * 1e-9;
            self.visible_ticks = Some(ticks.into_iter().filter(|tick| *tick >= low - tolerance && *tick <= high + tolerance).collect());
        }
    }

    /// The times at the start and end of the chart area: the nice range, padded by half a step at
    /// each end with an offset (see `Scale::set_offset`).
    fn mapped_range(&self) -> (Scalar, Scalar) {
        let (mut start, mut end) = (self.nice_range.start, self.nice_range.end);

        if self.offset && !self.ticks.is_empty() {
            let half_step = (end - start) / (self.ticks.len() - 1).max(1) as Scalar / 2.0;
            start -= half_step;
            end += half_step;
        }

        (start, end)
    }

    pub fn new(axis: Axis) -> TimeScale {
        TimeScale {
            axis,
            axis_options: AxisOptions::new(axis),
            offset: false,
            zone: TimeScaleZone::Local.as_dyn_read(),
            format: None,
            range: 0.0..86_400.0,
            nice_range: 0.0..86_400.0,
            ticks: vec![],
            visible_ticks: None,
            unit: TimeUnit::Hour,
        }
    }

    /// The time zone ticks are placed in and labels shown in, a value or a state of one. Defaults
    /// to the machine's local time zone (`TimeScaleZone::Local`).
    pub fn time_zone(mut self, zone: impl IntoReadState<TimeScaleZone>) -> TimeScale {
        self.zone = zone.into_read_state().as_dyn_read();
        self
    }

    /// Labels the ticks with your own text for the date at each tick, in the scale's time zone,
    /// given the unit the ticks are spaced in, instead of the locale's date or time.
    pub fn format(mut self, format: impl Fn(DateTime<FixedOffset>, TimeUnit) -> String + 'static) -> TimeScale {
        self.format = Some(Arc::new(format));
        self
    }

    /// Draws the scale on the given side of the chart area, a side or a state of one (see
    /// `LinearScale::position`).
    pub fn position(mut self, position: impl IntoReadState<AxisPosition>) -> TimeScale {
        self.axis_options = self.axis_options.with_position(position);
        self
    }

    /// Orders the scale among the scales on the same side (see `AxisOptions::weight`).
    pub fn weight(mut self, weight: impl IntoReadState<i32>) -> TimeScale {
        self.axis_options = self.axis_options.with_weight(weight);
        self
    }

    /// Whether the scale draws grid lines across the chart area. Defaults to `true`.
    pub fn grid_lines(mut self, grid_lines: impl IntoReadState<bool>) -> TimeScale {
        self.axis_options = self.axis_options.with_grid_lines(grid_lines);
        self
    }

    /// Whether the scale draws tick marks in its band. Defaults to `true`.
    pub fn tick_marks(mut self, tick_marks: impl IntoReadState<bool>) -> TimeScale {
        self.axis_options = self.axis_options.with_tick_marks(tick_marks);
        self
    }

    /// Gives the scale's axis a title, a text or a state of one (see `LinearScale::title`).
    pub fn title(mut self, title: impl IntoReadState<String>) -> TimeScale {
        self.axis_options = self.axis_options.with_title(title);
        self
    }

    crate::scale::axis_options::polar_options!(TimeScale);
    crate::scale::axis_options::zoom_options!(TimeScale);

    /// The unit the ticks are spaced in, as last fitted.
    pub fn unit(&self) -> TimeUnit {
        self.unit
    }

    /// At most how many steps fit along the axis: fewer along a horizontal axis or around a
    /// circle, whose date labels sit side by side.
    fn max_spaces(&self) -> u32 {
        match (self.axis, self.axis_options.polar_role()) {
            (Axis::Horizontal, _) | (Axis::Radial, Some(PolarRole::Angle)) => 6,
            _ => 10,
        }
    }

    /// The ticks for the range, `step` units apart, from the step the range starts in to the first
    /// at or after its end, or `None` when the dates can't be placed.
    fn ticks_for(&self, unit: TimeUnit, step: u32, start: Scalar, end: Scalar) -> Option<Vec<Scalar>> {
        let zone = *self.zone.value();
        let mut ticks = vec![];
        let mut wall = Some(unit.floor(step, zone.wall_time(start)?));

        while let Some(current) = wall {
            let tick = zone.timestamp(current)?;
            if ticks.last().is_some_and(|last| tick <= *last) {
                return None;
            }
            ticks.push(tick);
            if tick >= end {
                break;
            }
            wall = unit.add(step, current);
        }

        Some(ticks)
    }
}

impl Scale for TimeScale {
    fn axis(&self) -> Axis {
        self.axis
    }

    fn axis_options(&self) -> &AxisOptions {
        &self.axis_options
    }

    fn axis_options_mut(&mut self) -> Option<&mut AxisOptions> {
        Some(&mut self.axis_options)
    }

    fn set_polar_role(&mut self, role: Option<crate::scale::PolarRole>) {
        self.axis_options.set_polar_role(role);
    }

    fn sync(&mut self, env: &mut Environment) -> bool {
        let zone = self.zone.sync(env);
        self.axis_options.sync(env) | zone
    }

    /// With an offset, the mapped range is widened by half a step at both ends, the average step
    /// where steps differ, as months do.
    fn normalize(&self, value: Scalar) -> Scalar {
        let (start, end) = self.mapped_range();
        self.axis_options.window().zoom((value - start) / (end - start))
    }

    fn denormalize(&self, normalized: Scalar) -> Scalar {
        let (start, end) = self.mapped_range();
        start + self.axis_options.window().unzoom(normalized) * (end - start)
    }

    fn set_offset(&mut self, offset: bool) {
        self.offset = offset;
    }

    /// The date at `value` in the scale's time zone: the time for second, minute and hour steps,
    /// the date for longer ones, unless `format` gives the text.
    fn label(&self, value: Scalar, env: &mut Environment) -> Option<String> {
        let date = self.zone.value().date_time(value)?;

        if let Some(format) = &self.format {
            return Some(format(date, self.unit));
        }

        Some(format_date(date, self.unit, env))
    }

    fn min(&self) -> Scalar {
        self.nice_range.start
    }

    fn max(&self) -> Scalar {
        self.nice_range.end
    }

    /// Fits the scale to `min` and `max`, in seconds since the Unix epoch. A range without width
    /// is widened to the day around it, and one that isn't finite is the first day of 1970.
    fn set_range(&mut self, min: Scalar, max: Scalar) {
        self.range = if min.is_finite() && max.is_finite() { min..max } else { 0.0..86_400.0 };

        if self.range.start == self.range.end {
            self.range = self.range.start - 43_200.0..self.range.end + 43_200.0;
        }

        self.recalculate_ticks();
    }

    fn range(&self) -> Range<Scalar> {
        self.nice_range.clone()
    }

    /// Picks the unit and step (see `TimeScale`): the smallest whose ticks, rounded out to whole
    /// steps, span the range in at most `max_spaces` steps. Zoomed, the ticks shown, and the
    /// unit they are labelled in, are picked the same way for the part of the range in the
    /// window (see `zoom`).
    fn recalculate_ticks(&mut self) {
        let (start, end) = (self.range.start, self.range.end);

        match self.fitting_ticks(start, end) {
            Some((unit, ticks)) => {
                self.unit = unit;
                self.nice_range = ticks[0]..ticks[ticks.len() - 1];
                self.ticks = ticks;
            }
            None => {
                self.nice_range = start..end;
                self.ticks = vec![start, end];
            }
        }

        self.zoom_ticks();
    }

    fn ticks(&self) -> &[Scalar] {
        self.visible_ticks.as_deref().unwrap_or(&self.ticks)
    }
}

/// `date` for a tick spaced in `unit`: the locale's short date for days and longer, its short
/// time for minutes and hours, and its medium time, with seconds, for seconds.
#[cfg(feature = "carbide_fluent")]
fn format_date(date: DateTime<FixedOffset>, unit: TimeUnit, env: &mut Environment) -> String {
    use carbide::state::{StateSync, ValueState};
    use carbide_fluent::{DateStyle, LocalizedDateTime, TimeStyle};

    let (date_style, time_style) = match unit {
        TimeUnit::Second => (DateStyle::Hidden, TimeStyle::Medium),
        TimeUnit::Minute | TimeUnit::Hour => (DateStyle::Hidden, TimeStyle::Short),
        TimeUnit::Day | TimeUnit::Week | TimeUnit::Month | TimeUnit::Year => (DateStyle::Short, TimeStyle::Hidden),
    };

    let mut text = LocalizedDateTime::new(ValueState::new(date)).date_style(ValueState::new(date_style)).time_style(ValueState::new(time_style));
    text.sync(env);
    text.value().to_string()
}

#[cfg(not(feature = "carbide_fluent"))]
fn format_date(date: DateTime<FixedOffset>, unit: TimeUnit, _env: &mut Environment) -> String {
    let pattern = match unit {
        TimeUnit::Second => "%H:%M:%S",
        TimeUnit::Minute | TimeUnit::Hour => "%H:%M",
        TimeUnit::Day | TimeUnit::Week | TimeUnit::Month | TimeUnit::Year => "%Y-%m-%d",
    };

    date.format(pattern).to_string()
}

/// A moment in time is a coordinate: the seconds since the Unix epoch, with fractions, which a
/// `TimeScale` places and labels as a date.
impl<Tz: TimeZone + 'static> DataValue for DateTime<Tz> where Tz::Offset: 'static {
    fn value(&self) -> Option<Value<'_>> {
        Some(Value::Time(self.timestamp() as Scalar + self.timestamp_subsec_nanos() as Scalar / 1e9))
    }
}

#[cfg(test)]
mod tests {
    use carbide::draw::Scalar;
    use carbide::environment::Environment;
    use chrono::{DateTime, FixedOffset, TimeZone};

    use crate::{DataValue, Value};
    use crate::scale::{Axis, Scale};
    use super::{TimeScale, TimeScaleZone, TimeUnit};

    fn utc() -> FixedOffset {
        FixedOffset::east_opt(0).unwrap()
    }

    fn at(zone: FixedOffset, year: i32, month: u32, day: u32, hour: u32, minute: u32) -> Scalar {
        zone.with_ymd_and_hms(year, month, day, hour, minute, 0).unwrap().value().and_then(Value::number).unwrap()
    }

    fn fitted(axis: Axis, zone: FixedOffset, min: Scalar, max: Scalar) -> TimeScale {
        let mut scale = TimeScale::new(axis).time_zone(TimeScaleZone::Fixed(zone));
        scale.set_range(min, max);
        scale
    }

    /// A date is the seconds since the Unix epoch, whatever its zone, with fractions of a second.
    #[test]
    fn dates_are_timestamps() {
        let noon: DateTime<FixedOffset> = utc().with_ymd_and_hms(1970, 1, 2, 12, 0, 0).unwrap();
        assert_eq!(noon.value(), Some(Value::Time(129_600.0)));

        let east = FixedOffset::east_opt(2 * 3_600).unwrap();
        assert_eq!(east.with_ymd_and_hms(1970, 1, 2, 14, 0, 0).unwrap().value(), Some(Value::Time(129_600.0)), "the same moment in another zone");

        let later = noon + chrono::TimeDelta::milliseconds(250);
        assert_eq!(later.value(), Some(Value::Time(129_600.25)));
    }

    /// The step grows with the range, from seconds to years, and ticks fall on round times: whole
    /// hours, midnights, the first of the month, quarters, round years.
    #[test]
    fn steps_follow_the_range() {
        let zone = utc();

        let hours = fitted(Axis::Horizontal, zone, at(zone, 2026, 3, 4, 9, 20), at(zone, 2026, 3, 4, 17, 40));
        assert_eq!((hours.unit(), hours.ticks()[1] - hours.ticks()[0]), (TimeUnit::Hour, 2.0 * 3_600.0));
        assert_eq!(hours.ticks()[0], at(zone, 2026, 3, 4, 8, 0), "rounded down to an even hour");
        assert_eq!(*hours.ticks().last().unwrap(), at(zone, 2026, 3, 4, 18, 0), "and up past the end");

        let days = fitted(Axis::Horizontal, zone, at(zone, 2026, 3, 4, 9, 0), at(zone, 2026, 3, 8, 15, 0));
        assert_eq!(days.unit(), TimeUnit::Day);
        assert_eq!(days.ticks()[0], at(zone, 2026, 3, 4, 0, 0), "midnight");

        let rounded_out = fitted(Axis::Horizontal, zone, at(zone, 2026, 3, 2, 12, 0), at(zone, 2026, 3, 8, 12, 0));
        assert_eq!(rounded_out.ticks()[1] - rounded_out.ticks()[0], 2.0 * 86_400.0, "six days fit in six daily steps, but rounded out to midnights they take seven, so two days a step");
        assert!(rounded_out.ticks().len() <= 7);

        let months = fitted(Axis::Horizontal, zone, at(zone, 2026, 1, 15, 0, 0), at(zone, 2026, 6, 10, 0, 0));
        assert_eq!(months.unit(), TimeUnit::Month);
        assert_eq!(months.ticks()[0], at(zone, 2026, 1, 1, 0, 0));
        assert_eq!(months.ticks()[1], at(zone, 2026, 2, 1, 0, 0), "a month later on the calendar, 31 days");
        assert_eq!(months.ticks()[2], at(zone, 2026, 3, 1, 0, 0), "and then 28");

        let quarters = fitted(Axis::Horizontal, zone, at(zone, 2025, 2, 1, 0, 0), at(zone, 2026, 5, 1, 0, 0));
        assert_eq!(quarters.unit(), TimeUnit::Month);
        assert_eq!(quarters.ticks()[0], at(zone, 2025, 1, 1, 0, 0), "steps of three months start on a quarter");
        assert_eq!(quarters.ticks()[1], at(zone, 2025, 4, 1, 0, 0));

        let decades = fitted(Axis::Horizontal, zone, at(zone, 1971, 6, 1, 0, 0), at(zone, 2024, 1, 1, 0, 0));
        assert_eq!(decades.unit(), TimeUnit::Year);
        assert_eq!(decades.ticks()[0], at(zone, 1970, 1, 1, 0, 0), "a round year");
        assert!(decades.ticks().len() <= 7, "at most six steps along a horizontal axis");
    }

    /// Ticks fall on round times in the scale's time zone: midnight there, not in UTC.
    #[test]
    fn ticks_follow_the_time_zone() {
        let east = FixedOffset::east_opt(5 * 3_600).unwrap();
        let days = fitted(Axis::Horizontal, east, at(east, 2026, 3, 4, 9, 0), at(east, 2026, 3, 8, 15, 0));

        assert_eq!(days.ticks()[0], at(east, 2026, 3, 4, 0, 0));
        assert_eq!(days.ticks()[0], at(utc(), 2026, 3, 3, 19, 0), "midnight five hours east is 19:00 UTC");
    }

    /// A single moment is widened to the day around it; a range that isn't finite is the first day
    /// of 1970.
    #[test]
    fn empty_ranges_are_widened() {
        let zone = utc();
        let noon = at(zone, 2026, 3, 4, 12, 0);
        let single = fitted(Axis::Horizontal, zone, noon, noon);
        assert!(single.min() <= noon - 43_200.0 && single.max() >= noon + 43_200.0);
        assert_eq!(single.unit(), TimeUnit::Hour);

        let none = fitted(Axis::Horizontal, zone, f64::INFINITY, f64::NEG_INFINITY);
        assert_eq!((none.min(), none.max()), (0.0, 86_400.0));
    }

    /// Labels come from `format` when it is set, given the date in the scale's zone and the unit.
    #[test]
    fn labels_can_be_your_own() {
        let zone = utc();
        let mut scale = TimeScale::new(Axis::Horizontal)
            .time_zone(TimeScaleZone::Fixed(zone))
            .format(|date, unit| format!("{} {:?}", date.format("%b %Y"), unit));
        scale.set_range(at(zone, 2026, 1, 15, 0, 0), at(zone, 2026, 6, 10, 0, 0));

        assert_eq!(scale.label(scale.ticks()[1], &mut Environment::new()).as_deref(), Some("Feb 2026 Month"));
    }

    /// `denormalize` undoes `normalize`, in seconds, with and without an offset.
    #[test]
    fn denormalize_is_the_inverse_of_normalize() {
        let mut scale = TimeScale::new(Axis::Horizontal);
        scale.set_range(1_700_000_000.0, 1_700_086_400.0);

        for value in [1_699_990_000.0, 1_700_000_000.0, 1_700_043_210.0, 1_700_100_000.0] {
            assert!((scale.denormalize(scale.normalize(value)) - value).abs() < 1e-3, "{value}");
        }

        scale.set_offset(true);
        for value in [1_700_000_000.0, 1_700_043_210.0] {
            assert!((scale.denormalize(scale.normalize(value)) - value).abs() < 1e-3, "{value} with an offset");
        }
    }

    /// A zoomed time scale picks its ticks, and the unit they are labelled in, for the part shown:
    /// days inside a year ticked in months.
    #[test]
    fn zoom_picks_a_unit_for_the_part_shown() {
        let utc = utc();
        let mut scale = TimeScale::new(Axis::Horizontal).time_zone(TimeScaleZone::Fixed(utc));
        scale.set_range(at(utc, 2026, 1, 1, 0, 0), at(utc, 2027, 1, 1, 0, 0));
        assert_eq!(scale.unit(), TimeUnit::Month);

        let mut zoomed = TimeScale::new(Axis::Horizontal).time_zone(TimeScaleZone::Fixed(utc)).zoom(crate::ZoomWindow::new(0.5, 0.53));
        zoomed.set_range(at(utc, 2026, 1, 1, 0, 0), at(utc, 2027, 1, 1, 0, 0));
        assert_eq!(zoomed.unit(), TimeUnit::Day);

        let (low, high) = (zoomed.denormalize(0.0), zoomed.denormalize(1.0));
        assert!(zoomed.ticks().len() >= 2);
        assert!(zoomed.ticks().iter().all(|tick| *tick >= low && *tick <= high), "every tick is inside the window");
    }
}
