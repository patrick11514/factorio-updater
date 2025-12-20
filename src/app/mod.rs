use crate::{
    app::{
        api::Api,
        screens::{Screen, login::Login, main::Main},
    },
    config::Config,
};
use crossterm::event::{EventStream, KeyCode, KeyEvent};
use futures_util::StreamExt;
use ratatui::{DefaultTerminal, Frame};

mod api;
mod components;
mod screens;

pub struct App {
    exited: bool,
    events: EventStream,
    screen: Box<dyn Screen>,
}

impl App {
    pub async fn new() -> Self {
        let config = Config::load().await.unwrap();
        Self {
            exited: false,
            events: EventStream::default(),
            screen: match config {
                Some(config) => Box::new(Main::new(Api::new(config))),
                None => Box::new(Login::default()),
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
        match &event {
            Ok(event) => match event {
                crossterm::event::Event::Key(key_event) => self.handle_key(key_event).await,
                _ => {}
            },
            _ => {}
        };
    }

    async fn handle_key(&mut self, ev: &KeyEvent) {
        let res = match &ev.code {
            KeyCode::Char('q') | KeyCode::Esc => return self.handle_exit(),
            _ => self.screen.on_key(ev.clone()).await,
        };

        if let Some(ev) = res {
            match ev {
                screens::ScreenEvent::Logged(config) => self.screen = Box::new(Main::new(config)),
            }
        }
    }

    fn handle_exit(&mut self) {
        self.exited = true;
    }
}
