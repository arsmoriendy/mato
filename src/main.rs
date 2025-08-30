mod cli;
mod helpers;

use anyhow::{Context, Result, bail};
use clap::Parser;
use notify_rust::Notification;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, poll},
    layout::{Constraint, Flex, Layout},
    style::{Color::*, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Gauge, Tabs},
};
use std::{
    cmp::min,
    collections::HashMap,
    time::{Duration, SystemTime},
};

use crate::{
    cli::Cli,
    helpers::{ExtendedDuration, IsoDuration},
};

fn main() -> Result<()> {
    let args = Cli::parse();
    let mut app = App::from_args(&args)?;

    let terminal = ratatui::init();
    app.run(terminal);
    ratatui::restore();

    Ok(())
}

#[derive(Debug)]
enum Action {
    Quit,
    PlayPause,
    Next,
    Prev,
}

struct Keymaps(HashMap<KeyCode, Action>);

impl Default for Keymaps {
    fn default() -> Self {
        use Action::*;
        use KeyCode::Char;

        Self(HashMap::from([
            (Char('q'), Quit),
            (Char('['), Prev),
            (Char(']'), Next),
            (Char('p'), PlayPause),
            (Char(' '), PlayPause),
        ]))
    }
}

struct App<'a> {
    args: &'a Cli,
    timers: Vec<Timer<'a>>,
    current_timer_idx: usize,
    render_interval: Duration,
    keymaps: Keymaps,
    // states
    paused: bool,
    start: SystemTime,
    elapsed: Duration,
    cycles: u64,
}

#[derive(Debug)]
struct Timer<'a> {
    name: &'a str,
    duration: Duration,
}

impl<'a> App<'a> {
    fn from_args(args: &'a Cli) -> Result<Self> {
        let mut timers: Vec<Timer> = vec![];
        for i in 0..=min(args.durations.len(), args.names.len()) - 1 {
            let name = &args.names[i];
            let iso_dur_str = &args.durations[i];

            let duration = Duration::from_iso_str(iso_dur_str).context(format!(
                "Failed parsing duration \"{}\" for timer \"{}\"",
                iso_dur_str, name
            ))?;

            if duration == Duration::ZERO {
                bail!("Duration for timer \"{}\" cannot be zero", name)
            }

            timers.push(Timer { name, duration });
        }

        Ok(App {
            args,
            timers,
            render_interval: Duration::from_millis(args.tick),
            keymaps: Keymaps::default(),
            current_timer_idx: 0,
            paused: false,
            start: SystemTime::now(),
            elapsed: Duration::ZERO,
            cycles: 0,
        })
    }

    fn run(&mut self, mut terminal: DefaultTerminal) {
        loop {
            if self.args.cycles != 0 && self.cycles >= self.args.cycles {
                break;
            }

            terminal.draw(|frame| self.render(frame)).unwrap();

            self.count();

            if self.elapsed >= self.current_timer().duration {
                self.next_timer();

                if !self.args.no_notify {
                    let prev_timer = &self.timers[if self.current_timer_idx == 0 {
                        self.timers.len() - 1
                    } else {
                        self.current_timer_idx - 1
                    }];

                    let ntf_sum = format!("\"{}\" timer has ended.", prev_timer.name);
                    let ntf_body = format!(
                        "Started \"{}\" timer for {}",
                        self.current_timer().name,
                        IsoDuration::from(&self.current_timer().duration)
                    );

                    Notification::new()
                        .summary(&ntf_sum)
                        .body(&ntf_body)
                        .show()
                        .ok();
                }
            }

            if let Some(act) = self.handle_events() {
                match act {
                    Action::Quit => break,
                    Action::PlayPause => self.toggle_pause(),
                    Action::Next => self.next_timer(),
                    Action::Prev => self.prev_timer(),
                };
            }
        }
    }

    fn render(&mut self, frame: &mut Frame) {
        let current_timer = self.current_timer();
        let time_left = current_timer.duration - self.elapsed;
        let elapsed_percent =
            u16::try_from(self.elapsed.as_millis() * 100 / current_timer.duration.as_millis())
                .unwrap();

        // main
        let paused_line = if self.paused {
            Line::styled("Paused", LightRed).centered()
        } else {
            Line::default()
        };
        let cycle_line =
            (Line::raw("Cycles: ") + format!("{}", self.cycles).yellow()).right_aligned();
        let elapsed_line =
            Line::default() + Span::raw("Elapsed: ") + IsoDuration::from(&self.elapsed);
        let time_left_line =
            (Line::default() + Span::raw("Time Left: ") + IsoDuration::from(&time_left))
                .right_aligned();
        let block = Block::bordered()
            .title(paused_line)
            .title(cycle_line)
            .title_bottom(elapsed_line)
            .title_bottom(time_left_line);

        let mut gague = Gauge::default()
            .percent(elapsed_percent)
            .use_unicode(true)
            .block(block);
        if self.paused {
            gague = gague.gauge_style(Red);
        }

        let tabs = Tabs::new(
            // get timer names
            self.timers.iter().enumerate().map(|(i, t)| {
                Line::default()
                    + Span::raw(format!(" {} ", t.name)).style(if i == self.current_timer_idx {
                        Style::default().reversed()
                    } else {
                        Style::default()
                    })
                    + Span::raw(" (")
                    + IsoDuration::from(&t.duration)
                    + Span::raw(")")
            }),
        )
        .style(Style::new().dim())
        .highlight_style(Style::new().not_dim()) // HACK: override default Style::reversed()
        .select(self.current_timer_idx);

        let keymaps_line = Line::from(
            self.keymaps
                .0
                .iter()
                .map(|(kc, act)| {
                    [
                        kc.to_string().cyan(),
                        ": ".into(),
                        format!("{:?} ", act).into(),
                    ]
                })
                .flatten()
                .collect::<Vec<Span>>(),
        )
        .right_aligned();

        let meta_line = Line::from(format!(
            "{} {}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        ));

        // layouts
        let [main_area, bottom_area] =
            Layout::vertical([Constraint::Percentage(100), Constraint::Length(1)])
                .areas(frame.area());
        let [btm_lft_area, btm_rgt_area] = Layout::horizontal([
            Constraint::Percentage(100),
            Constraint::Length(keymaps_line.width().try_into().unwrap()),
        ])
        .flex(Flex::SpaceBetween)
        .areas(bottom_area);
        let [tabs_area, gague_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Length(5)])
                .flex(Flex::Center)
                .areas(main_area);

        frame.render_widget(tabs, tabs_area);
        frame.render_widget(gague, gague_area);
        frame.render_widget(meta_line, btm_lft_area);
        frame.render_widget(keymaps_line, btm_rgt_area);
    }

    fn current_timer(&'a self) -> &'a Timer<'a> {
        &self.timers[self.current_timer_idx]
    }

    fn next_timer(&mut self) {
        self.current_timer_idx = if self.current_timer_idx >= self.timers.len() - 1 {
            0
        } else {
            self.current_timer_idx + 1
        };

        self.reset_timer();

        // when reaching first timer, increment cycle
        if self.current_timer_idx == 0 {
            self.cycles += 1;
        }
    }

    fn prev_timer(&mut self) {
        self.current_timer_idx = if self.current_timer_idx <= 0 {
            self.timers.len() - 1
        } else {
            self.current_timer_idx - 1
        };

        self.reset_timer();

        // when reaching last timer, decrement cycle, with a limit of 0
        if self.current_timer_idx == self.timers.len() - 1 && self.cycles > 0 {
            self.cycles -= 1;
        }
    }

    fn count(&mut self) {
        if self.paused {
            self.start = SystemTime::now() - self.elapsed;
        } else {
            self.elapsed = self.start.elapsed().unwrap();
        }
    }

    fn handle_events(&self) -> Option<&Action> {
        if poll(self.render_interval).unwrap()
            && let Event::Key(key) = event::read().unwrap()
        {
            return self.keymaps.0.get(&key.code);
        }
        None
    }

    fn reset_timer(&mut self) {
        self.start = SystemTime::now();
        self.elapsed = Duration::ZERO;
    }

    fn toggle_pause(&mut self) {
        self.paused = !self.paused
    }
}
