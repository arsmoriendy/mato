use anyhow::{Context, Result, bail};
use ratatui::{style::Stylize, text::Line};
use std::{fmt::Display, ops::Add, time::Duration};

pub trait ExtendedDuration {
    fn as_hours(&self) -> u64;
    fn subhour_min(&self) -> u64;
    fn submin_sec(&self) -> u64;
    /// Custom ISO 8601 time only format
    fn from_iso_str(value: &str) -> Result<Self>
    where
        Self: Sized;
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

    fn from_iso_str(value: &str) -> Result<Self> {
        let (mut h_idx, mut m_idx, mut s_idx) = (0usize, 0usize, 0usize);

        for (i, c) in value.chars().enumerate() {
            match c {
                'h' | 'H' => h_idx = i,
                'm' | 'M' => m_idx = i,
                's' | 'S' => s_idx = i,
                _ => {}
            }
        }

        if h_idx == 0 && m_idx == 0 && s_idx == 0 {
            bail!("Invalid duration format")
        }

        let (mut h, mut m, mut s) = (0u64, 0u64, 0u64);

        if h_idx != 0 {
            h = value[0..h_idx].parse().context("Failed parsing hour")?;
        }

        let mut start_idx = if h_idx != 0 { h_idx + 1 } else { 0 };
        if m_idx != 0 {
            m = value[start_idx..m_idx]
                .parse()
                .context("Failed parsing minutes")?;
        }

        if s_idx != 0 {
            start_idx = if m_idx != 0 { m_idx + 1 } else { start_idx };
            s = value[start_idx..s_idx]
                .parse()
                .context("Failed parsing seconds")?;
        }

        Ok(Duration::from_secs(h * 3600 + m * 60 + s))
    }
}

/// Wrapper for formatting `Duration` in custom ISO 8601 time format.
pub struct IsoDuration {
    h: Option<u64>,
    m: Option<u64>,
    s: Option<u64>,
}

impl From<&Duration> for IsoDuration {
    fn from(value: &Duration) -> Self {
        let hour = value.as_hours();
        let subhour_min = value.subhour_min();
        let submin_sec = value.submin_sec();

        let show_hour = value >= &Duration::from_secs(60 * 60);
        let show_min = value >= &Duration::from_secs(60) && subhour_min > 0;
        let show_sec = !show_min || submin_sec > 0;

        Self {
            h: if show_hour { Some(hour) } else { None },
            m: if show_min { Some(subhour_min) } else { None },
            s: if show_sec { Some(submin_sec) } else { None },
        }
    }
}

impl Display for IsoDuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(h) = self.h {
            write!(f, "{}h", h)?;
        }
        if let Some(m) = self.m {
            write!(f, "{}m", m)?;
        }
        if let Some(s) = self.s {
            write!(f, "{}s", s)?;
        }

        Ok(())
    }
}

impl<'a> Add<IsoDuration> for Line<'a> {
    type Output = Self;

    fn add(mut self, rhs: IsoDuration) -> Self::Output {
        if let Some(h) = rhs.h {
            self += format!("{}h", h).yellow();
        }
        if let Some(m) = rhs.m {
            self += format!("{}m", m).cyan();
        }
        if let Some(s) = rhs.s {
            self += format!("{}s", s).blue();
        }

        self
    }
}
