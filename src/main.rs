mod cli;
mod helpers;

use clap::Parser;
use crossterm::event::{self, Event, KeyCode, poll};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Flex, Layout},
    style::Stylize,
    text::{Line, Span},
    widgets::{Block, Gauge},
};
use std::{
    iter::{Cycle, Peekable},
    slice::Iter,
    time::{Duration, SystemTime},
};

use crate::helpers::IsoDur;

fn main() {
    let args = cli::Cli::parse();

    let mut timers: Vec<Timer> = vec![];
    for (i, name) in args.names.iter().enumerate() {
        timers.push(Timer {
            name,
            duration: Duration::from_millis(args.durations[i]),
        });
    }

    let mut app = App {
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
}

struct App<'a> {
    timers: Peekable<Cycle<Iter<'a, Timer<'a>>>>,
    render_interval: Duration,
    state: AppState,
}

struct AppState {
    start: SystemTime,
    elapsed: Duration,
}

impl AppState {
    fn reset(&mut self) {
        self.start = SystemTime::now();
        self.elapsed = Duration::ZERO;
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            start: SystemTime::now(),
            elapsed: Duration::ZERO,
        }
    }
}

struct Timer<'a> {
    name: &'a str,
    duration: Duration,
}

impl App<'_> {
    fn run(&mut self, mut terminal: DefaultTerminal) {
        loop {
            terminal.draw(|frame| self.render(frame)).unwrap();

            self.count();

            let current_timer = self.timers.peek().unwrap();

            if self.state.elapsed >= current_timer.duration {
                self.timers.next();
                self.state.reset();
            }

            if let Some(act) = self.handle_events() {
                match act {
                    Action::Quit => break,
                }
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
        let timer_name_line = Line::from(vec![current_timer.name.into()]).centered();
        let elapsed_line = Line::default()
            + Span::raw("Elapsed: ")
            + IsoDur::from(&self.state.elapsed)
            + " / ".dark_gray()
            + IsoDur::from(&current_timer.duration);
        let time_left_line =
            (Line::default() + Span::raw("Time Left: ") + IsoDur::from(&time_left)).right_aligned();
        let block = Block::bordered()
            .title(timer_name_line)
            .title_bottom(elapsed_line)
            .title_bottom(time_left_line);
        let gague = Gauge::default().percent(elapsed_percent).block(block);

        // btm_rgt_area
        let legend = Line::from(vec!["q: quit".into()]).right_aligned();

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
        self.state.elapsed = self.state.start.elapsed().unwrap();
    }

    fn handle_events(&self) -> Option<Action> {
        if poll(self.render_interval).unwrap()
            && let Event::Key(key) = event::read().unwrap()
        {
            return match key.code {
                KeyCode::Char('q') => Some(Action::Quit),
                _ => None,
            };
        }
        None
    }
}
