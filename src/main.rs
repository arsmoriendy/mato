mod cyclic_list;
use cyclic_list::CyclicList;

use crossterm::event::{self, Event, KeyCode, poll};
use ratatui::{DefaultTerminal, Frame};
use std::time::{Duration, SystemTime};

fn main() {
    let terminal = ratatui::init();
    run(terminal);
    ratatui::restore();
}

fn run(mut terminal: DefaultTerminal) {
    let mut timers = CyclicList::<Timer>::default();

    timers.push_back(Timer::new("work", Duration::from_secs(3)));
    timers.push_back(Timer::new("play", Duration::from_secs(2)));

    let mut app = App {
        timers,
        render_interval: Duration::from_millis(50),
        state: Default::default(),
    };

    loop {
        terminal.draw(|frame| app.render(frame)).unwrap();

        let current_timer_ref = app.timers.current().unwrap();
        let current_timer = &current_timer_ref.borrow().data;

        app.count();
        if app.state.elapsed >= current_timer.duration {
            app.state.reset();
            app.timers.advance();
        }

        if let Some(act) = app.handle_events() {
            match act {
                Action::Quit => break,
            }
        }
    }
}

enum Action {
    Quit,
}

struct App<'a> {
    timers: CyclicList<Timer<'a>>,
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

impl Timer<'_> {
    fn new(name: &str, duration: Duration) -> Timer<'_> {
        Timer { name, duration }
    }
}

impl App<'_> {
    fn render(&self, frame: &mut Frame) {
        let current_timer_ref = self.timers.current().unwrap();
        let current_timer = &current_timer_ref.borrow().data;
        frame.render_widget(
            format!(
                "{}\t: {:?}/{:?}",
                current_timer.name, self.state.elapsed, current_timer.duration,
            ),
            frame.area(),
        )
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
