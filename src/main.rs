use std::time::{Duration, SystemTime};

use crossterm::event::{self, Event, KeyCode, poll};
use ratatui::{DefaultTerminal, Frame};

fn main() {
    let terminal = ratatui::init();
    run(terminal);
    ratatui::restore();
}

fn run(mut terminal: DefaultTerminal) {
    let app = App {
        start: SystemTime::now(),
        duration: Duration::from_secs(5),
        interval: Duration::from_millis(50),
    };

    loop {
        terminal.draw(|frame| app.render(frame)).unwrap();

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

struct App {
    start: SystemTime,
    duration: Duration,
    interval: Duration,
}

impl App {
    fn render(&self, frame: &mut Frame) {
        let elapsed = self.start.elapsed().unwrap();
        if elapsed >= self.duration {
            frame.render_widget("done", frame.area());
            return;
        }

        frame.render_widget(format!("{:?}", elapsed), frame.area())
    }

    fn handle_events(&self) -> Option<Action> {
        if poll(self.interval).unwrap()
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
