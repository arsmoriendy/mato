mod cli;
mod helpers;

use clap::Parser;
use crossterm::event::{self, Event, KeyCode, poll};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Flex, Layout},
    style::{Color, Stylize},
    text::{Line, Span},
    widgets::{Block, Gauge},
};
use std::{
    collections::HashMap,
    time::{Duration, SystemTime},
};

use crate::{cli::Cli, helpers::IsoDur};

fn main() {
    let args = Cli::parse();

    let mut timers: Vec<Timer> = vec![];
    for (i, name) in args.names.iter().enumerate() {
        timers.push(Timer {
            name,
            duration: Duration::from_secs(args.durations[i] * 60),
        });
    }

    let mut app = App {
        args: &args,
        timers: timers,
        render_interval: Duration::from_millis(args.tick),
        keymaps: Keymaps::default(),
        current_timer_idx: 0,
        paused: false,
        start: SystemTime::now(),
        elapsed: Duration::ZERO,
        cycles: 0,
    };

    let terminal = ratatui::init();
    app.run(terminal);
    ratatui::restore();
}

#[derive(Debug)]
enum Action {
    Quit,
    PlayPause,
    Next,
}

struct Keymaps(HashMap<KeyCode, Action>);

impl Default for Keymaps {
    fn default() -> Self {
        use Action::*;
        use KeyCode::Char;

        Self(HashMap::from([
            (Char('q'), Quit),
            (Char(']'), Next),
            (Char('p'), PlayPause),
            (Char(' '), PlayPause),
        ]))
    }
}

struct App<'a> {
    args: &'a Cli,
    timers: Vec<Timer<'a>>,
    render_interval: Duration,
    keymaps: Keymaps,
    current_timer_idx: usize,
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

impl App<'_> {
    fn run(&mut self, mut terminal: DefaultTerminal) {
        loop {
            if self.args.cycles != 0 && self.cycles >= self.args.cycles {
                break;
            }

            terminal.draw(|frame| self.render(frame)).unwrap();

            self.count();

            let current_timer = self.current_timer();

            if self.elapsed >= current_timer.duration {
                self.advance_timer();
            }

            if let Some(act) = self.handle_events() {
                match act {
                    Action::Quit => break,
                    Action::PlayPause => self.toggle_pause(),
                    Action::Next => self.advance_timer(),
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
            Line::styled("Paused", Color::LightRed)
        } else {
            Line::default()
        };
        let timer_name_line = Line::from(vec![current_timer.name.into()]).centered();
        let cycle_line =
            (Line::raw("Cycles: ") + format!("{}", self.cycles).yellow()).right_aligned();
        let elapsed_line = Line::default()
            + Span::raw("Elapsed: ")
            + IsoDur::from(&self.elapsed)
            + " / ".dark_gray()
            + IsoDur::from(&current_timer.duration);
        let time_left_line =
            (Line::default() + Span::raw("Time Left: ") + IsoDur::from(&time_left)).right_aligned();
        let block = Block::bordered()
            .title(paused_line)
            .title(timer_name_line)
            .title(cycle_line)
            .title_bottom(elapsed_line)
            .title_bottom(time_left_line);
        let gague = Gauge::default()
            .percent(elapsed_percent)
            .use_unicode(true)
            .gauge_style(if self.paused {
                Color::Red
            } else {
                Color::Green
            })
            .block(block);

        // btm_rgt_area
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

        // layouts
        let [main_area, bottom_area] =
            Layout::vertical([Constraint::Percentage(100), Constraint::Length(1)])
                .areas(frame.area());
        let [_btm_lft_area, btm_rgt_area] =
            Layout::horizontal([Constraint::Percentage(100), Constraint::Percentage(100)])
                .flex(Flex::SpaceBetween)
                .areas(bottom_area);
        let [gague_area] = Layout::vertical([Constraint::Length(5)])
            .flex(Flex::Center)
            .areas(main_area);

        frame.render_widget(gague, gague_area);
        frame.render_widget(keymaps_line, btm_rgt_area);
    }

    fn current_timer<'a>(&'a self) -> &'a Timer<'a> {
        &self.timers[self.current_timer_idx]
    }

    fn advance_timer(&mut self) {
        self.current_timer_idx = if self.current_timer_idx >= self.timers.len() - 1 {
            0
        } else {
            self.current_timer_idx + 1
        };

        self.reset_timer();
        if self.current_timer_idx == 0 {
            self.cycles += 1;
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
