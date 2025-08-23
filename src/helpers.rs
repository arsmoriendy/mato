use ratatui::{style::Stylize, text::Line};
use std::{ops::Add, time::Duration};

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

        let sec = rhs.dur.as_secs();
        let hour = sec / 3600;

        // magic numbers
        let subhour_min = sec % 3600 / 60;
        let submin_sec = sec % 60;

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
