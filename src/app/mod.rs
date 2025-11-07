use crate::{
    app::screens::{Screen, login::Login, main::Main},
    config::Config,
};
use crossterm::event::{EventStream, KeyCode, KeyEvent};
use futures_util::StreamExt;
use ratatui::{DefaultTerminal, Frame};

mod screens;

#[derive(Default)]
pub struct App {
    exited: bool,
    events: EventStream,
    screen: Screen,
}

impl App {
    pub async fn new() -> Self {
        let config = Config::load().await.unwrap();
        Self {
            exited: false,
            events: EventStream::default(),
            screen: match config {
                Some(config) => Screen::Main(Main::new(config)),
                None => Screen::Login(Login {}),
            },
        }
    }

    pub async fn main_loop(mut self, term: &mut DefaultTerminal) -> anyhow::Result<()> {
        while !self.exited {
            term.draw(|frame| self.draw(frame))?;
            self.handle_events().await;
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        self.screen.render(frame);
    }

    async fn handle_events(&mut self) {
        let event = self.events.next().await.unwrap();
        match event {
            Ok(event) => match event {
                crossterm::event::Event::Key(key_event) => self.handle_key(key_event),
                _ => {}
            },
            _ => {}
        }
    }

    fn handle_key(&mut self, ev: KeyEvent) {
        match ev.code {
            KeyCode::Char('q') | KeyCode::Esc => self.handle_exit(),
            _ => {}
        }
    }

    fn handle_exit(&mut self) {
        self.exited = true;
    }
}
