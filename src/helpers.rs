use ratatui::{style::Stylize, text::Line};
use std::{ops::Add, time::Duration};

pub trait ExtendedDuration {
    fn as_hours(&self) -> u64;
    fn subhour_min(&self) -> u64;
    fn submin_sec(&self) -> u64;
}

impl ExtendedDuration for Duration {
    fn as_hours(&self) -> u64 {
        self.as_secs() / 3600
    }

    fn subhour_min(&self) -> u64 {
        self.as_secs() % 3600 / 60
    }

    fn submin_sec(&self) -> u64 {
        self.as_secs() % 60
    }
}

/// Wrapper for formatting `Duration` in custom ISO 8601 time format.
pub struct IsoDur<'a> {
    dur: &'a Duration,
}

impl<'a> From<&'a Duration> for IsoDur<'a> {
    fn from(value: &'a Duration) -> Self {
        Self { dur: value }
    }
}

impl<'a> Add<IsoDur<'a>> for Line<'a> {
    type Output = Line<'a>;

    fn add(mut self, rhs: IsoDur<'a>) -> Self::Output {
        let hour = rhs.dur.as_hours();
        let subhour_min = rhs.dur.subhour_min();
        let submin_sec = rhs.dur.submin_sec();

        let show_hour = rhs.dur >= &Duration::from_secs(60 * 60);
        let show_min = rhs.dur >= &Duration::from_secs(60) && subhour_min > 0;
        let show_sec = !show_hour && (submin_sec > 0 || !show_min);

        if show_hour {
            self += format!("{}h", hour).yellow();
        }
        if show_min {
            self += format!("{}m", subhour_min).cyan();
        }
        if show_sec {
            self += format!("{}s", submin_sec).blue();
        }

        self
    }
}
