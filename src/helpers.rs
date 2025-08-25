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
        let show_hour = rhs.dur >= &Duration::from_secs(60 * 60);
        let show_min = rhs.dur >= &Duration::from_secs(60);
        let show_sec = !show_hour;

        let hour = rhs.dur.as_hours();
        let subhour_min = rhs.dur.subhour_min();
        let submin_sec = rhs.dur.submin_sec();

        if show_hour {
            self += format!("{:02}", hour).yellow();
            self += "h".dark_gray();
        }
        if show_min {
            self += format!("{:02}", subhour_min).cyan();
            self += "m".dark_gray();
        }
        if show_sec {
            self += format!("{:02}", submin_sec).blue();
            self += "s".dark_gray();
        }

        self
    }
}
