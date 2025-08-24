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
    iter::{Cycle, Peekable},
    slice::Iter,
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
        timers: timers.iter().cycle().peekable(),
        render_interval: Duration::from_millis(args.tick),
        state: Default::default(),
    };

    let terminal = ratatui::init();
    app.run(terminal);
    ratatui::restore();
}

enum Action {
    Quit,
    PlayPause,
    Next,
}

struct App<'a> {
    args: &'a Cli,
    timers: Peekable<Cycle<Iter<'a, Timer<'a>>>>,
    render_interval: Duration,
    state: AppState,
}

struct AppState {
    paused: bool,
    start: SystemTime,
    elapsed: Duration,
    cycles: u64,
}

impl AppState {
    fn reset_timer(&mut self) {
        self.start = SystemTime::now();
        self.elapsed = Duration::ZERO;
    }

    fn toggle_pause(&mut self) {
        self.paused = !self.paused
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            paused: false,
            start: SystemTime::now(),
            elapsed: Duration::ZERO,
            cycles: 0,
        }
    }
}

#[derive(Debug)]
struct Timer<'a> {
    name: &'a str,
    duration: Duration,
}

impl App<'_> {
    fn run(&mut self, mut terminal: DefaultTerminal) {
        // used to anchor cycles
        let first_timer_ref = *self.timers.peek().unwrap();

        loop {
            terminal.draw(|frame| self.render(frame)).unwrap();

            self.count();

            let current_timer = self.timers.peek().unwrap();

            if self.state.elapsed >= current_timer.duration {
                self.timers.next();
                self.state.reset_timer();
                if std::ptr::eq(*self.timers.peek().unwrap(), first_timer_ref) {
                    self.state.cycles += 1;
                    // break on cycle limit (if specified)
                    if self.args.cycles != 0 && self.state.cycles >= self.args.cycles {
                        break;
                    }
                }
            }

            if let Some(act) = self.handle_events() {
                match act {
                    Action::Quit => break,
                    Action::PlayPause => self.state.toggle_pause(),
                    Action::Next => {
                        // TODO: this is the same as line 105-113
                        self.timers.next();
                        self.state.reset_timer();
                        if std::ptr::eq(*self.timers.peek().unwrap(), first_timer_ref) {
                            self.state.cycles += 1;
                            // break on cycle limit (if specified)
                            if self.args.cycles != 0 && self.state.cycles >= self.args.cycles {
                                break;
                            }
                        }
                    }
                };
            }
        }
    }

    fn render(&mut self, frame: &mut Frame) {
        let current_timer = self.timers.peek().unwrap();
        let time_left = current_timer.duration - self.state.elapsed;
        let elapsed_percent = u16::try_from(
            self.state.elapsed.as_millis() * 100 / current_timer.duration.as_millis(),
        )
        .unwrap();

        // main
        let paused_line = if self.state.paused {
            Line::styled("Paused", Color::LightRed)
        } else {
            Line::default()
        };
        let timer_name_line = Line::from(vec![current_timer.name.into()]).centered();
        let cycle_line =
            (Line::raw("Cycles: ") + format!("{}", self.state.cycles).yellow()).right_aligned();
        let elapsed_line = Line::default()
            + Span::raw("Elapsed: ")
            + IsoDur::from(&self.state.elapsed)
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
            .gauge_style(if self.state.paused {
                Color::Red
            } else {
                Color::Green
            })
            .block(block);

        // btm_rgt_area
        let legend =
            Line::from(vec!["q: quit p/<Space>: play/pause ]: next".into()]).right_aligned();

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
        frame.render_widget(legend, btm_rgt_area);
    }

    fn count(&mut self) {
        if self.state.paused {
            self.state.start = SystemTime::now() - self.state.elapsed;
        } else {
            self.state.elapsed = self.state.start.elapsed().unwrap();
        }
    }

    fn handle_events(&self) -> Option<Action> {
        if poll(self.render_interval).unwrap()
            && let Event::Key(key) = event::read().unwrap()
        {
            return match key.code {
                KeyCode::Char('q') => Some(Action::Quit),
                KeyCode::Char(' ') | KeyCode::Char('p') => Some(Action::PlayPause),
                KeyCode::Char(']') => Some(Action::Next),
                _ => None,
            };
        }
        None
    }
}
