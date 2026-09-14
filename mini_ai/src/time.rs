use bevy::prelude::*;
use core::time::Duration;

pub fn plugin(app: &mut App) {
    let ticks = Tick::from_months(1) + Tick::from_days(12) + Tick::from_time(7, 15);
    let timestep = Duration::from_micros(TIMESTEP);

    app.insert_resource(Time::<Fixed>::from_duration(timestep))
        .insert_resource(ticks)
        .add_systems(FixedPreUpdate, update_ticks);
}

pub fn update_ticks(time: Res<Time<Fixed>>, mut tick: ResMut<Tick>) {
    *tick = Tick(tick.0 + Tick::from_duration(time.delta()).0);
}

pub const TIMESTEP: u64 = 15625; // 64 Hz

pub const MINS_PER_HOUR: u32 = 60;
pub const HOURS_PER_DAY: u32 = 24;
pub const DAYS_PER_WEEK: u32 = 7;
pub const DAYS_PER_MONTH: u32 = 28;
pub const DAYS_PER_YEAR: u32 = 112;
pub const WEEKS_PER_MONTH: u32 = 4;
pub const MONTH_PER_YEAR: u32 = 4;

#[derive(Resource, Clone, Copy, PartialEq, Eq, Debug)]
pub struct Tick(u32);

impl From<Tick> for u32 {
    fn from(Tick(ticks): Tick) -> Self {
        ticks
    }
}

impl std::ops::Add<Self> for Tick {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl std::ops::Sub<Self> for Tick {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl std::ops::Mul<u32> for Tick {
    type Output = Self;
    fn mul(self, rhs: u32) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl Tick {
    // 15625 / 1_000_000 *   42 = 0.65625s
    // 15625 / 1_000_000 *   48 = 0.75000s  ~770 real years
    // 15625 / 1_000_000 *   64 = 1.00000s
    // 15625 / 1_000_000 *   96 = 1.50000s
    // 15625 / 1_000_000 *  128 = 2.00000s
    // 15625 / 1_000_000 *  144 = 2.25000s
    pub const MINUTE: Self = Self(48);
    pub const HOUR: Self = Self(Self::MINUTE.0 * MINS_PER_HOUR); // 0.75m
    pub const DAY: Self = Self(Self::HOUR.0 * HOURS_PER_DAY); //      18m  0.3h
    pub const WEEK: Self = Self(Self::DAY.0 * DAYS_PER_WEEK); //     126m  2.1h
    pub const MONTH: Self = Self(Self::DAY.0 * DAYS_PER_MONTH); //   504m  8.4h
    pub const YEAR: Self = Self(Self::DAY.0 * DAYS_PER_YEAR); //    2016m 33.6h

    #[must_use]
    pub const fn from_duration(duration: Duration) -> Self {
        Self((duration.checked_div(TIMESTEP as u32).unwrap()).as_micros() as u32)
    }

    #[must_use]
    pub const fn duration(self) -> Duration {
        Duration::from_micros(TIMESTEP * self.0 as u64)
    }

    #[must_use]
    pub const fn new(ticks: u32) -> Self {
        Self(ticks)
    }

    #[must_use]
    pub const fn from_years(years: u32) -> Self {
        Self(years * Self::YEAR.0)
    }

    #[must_use]
    pub const fn from_months(months: u32) -> Self {
        Self(months * Self::MONTH.0)
    }

    #[must_use]
    pub const fn from_weeks(weeks: u32) -> Self {
        Self(weeks * Self::WEEK.0)
    }

    #[must_use]
    pub const fn from_days(days: u32) -> Self {
        Self(days * Self::DAY.0)
    }

    #[must_use]
    pub const fn from_time(hours: u32, mins: u32) -> Self {
        Self(hours * Self::HOUR.0 + mins * Self::MINUTE.0)
    }

    #[must_use]
    pub const fn year(&self) -> u32 {
        self.0 / Self::YEAR.0
    }

    #[must_use]
    pub const fn month_of_year(&self) -> u32 {
        (self.0 / Self::MONTH.0) % MONTH_PER_YEAR
    }

    #[must_use]
    pub const fn num_day_of_week(&self) -> u32 {
        (self.0 / Self::DAY.0) % DAYS_PER_WEEK + 1
    }

    #[must_use]
    pub const fn num_day_of_month(&self) -> u32 {
        (self.0 / Self::DAY.0) % DAYS_PER_MONTH + 1
    }

    #[must_use]
    pub const fn num_day_of_year(&self) -> u32 {
        (self.0 / Self::DAY.0) % DAYS_PER_YEAR + 1
    }

    #[must_use]
    pub const fn hour_of_day(&self) -> u32 {
        (self.0 / Self::HOUR.0) % HOURS_PER_DAY
    }

    #[must_use]
    pub const fn minute_of_hour(&self) -> u32 {
        (self.0 / Self::MINUTE.0) % MINS_PER_HOUR
    }

    #[must_use]
    pub const fn is_weekend(&self) -> bool {
        matches!(self.num_day_of_week(), 6 | 7)
    }

    #[must_use]
    pub fn format_date(&self) -> String {
        let day = self.num_day_of_month();
        let month = match self.month_of_year() {
            0 => "Spring",
            1 => "Summer",
            2 => "Autumn",
            3 => "Winter",
            _ => unreachable!(),
        };
        format!("{day} {month}")
    }

    #[must_use]
    pub fn format_time(&self) -> String {
        let hours = self.hour_of_day();
        let mins = self.minute_of_hour();
        format!("{hours:02}:{mins:02}")
    }
}

#[test]
fn duration_conversion() {
    assert_eq!(Tick::MINUTE, Tick::from_duration(Tick::MINUTE.duration()));
}
