use std::time::Duration;

use crate::{
    app::{
        api::Api,
        components::popup::Popup,
        screens::{Screen, login::Login, main::Main},
    },
    config::Config,
};
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent};
use futures_util::StreamExt;
use ratatui::{DefaultTerminal, Frame, layout::Rect, widgets::Widget};

mod api;
mod components;
mod screens;

pub struct App<'a> {
    exited: bool,
    screen: Box<dyn Screen>,
    popup: Option<Popup<'a>>,
    event_rx: tokio::sync::mpsc::Receiver<Event>,
}

impl App<'_> {
    pub async fn new() -> Self {
        let config = Config::load().await.unwrap();

        let (tx, rx) = tokio::sync::mpsc::channel(1024);

        tokio::spawn(async move {
            let mut stream = EventStream::default();
            while let Some(event) = stream.next().await {
                match event {
                    Ok(event) => tx.send(event).await.unwrap(),
                    _ => {}
                }
            }
        });

        Self {
            exited: false,
            screen: match config {
                Some(config) => Box::new(Main::new(Api::new(config))),
                None => Box::new(Login::default()),
            },
            popup: None,
            event_rx: rx,
        }
    }

    pub async fn main_loop(mut self, term: &mut DefaultTerminal) -> anyhow::Result<()> {
        let mut ticker = tokio::time::interval(Duration::from_millis(16) /* ~62FPS */);

        while !self.exited {
            term.draw(|frame| self.draw(frame))?;

            tokio::select! {
                event = self.event_rx.recv()  => {
                    self.handle_event(event.unwrap()).await;
                }
                _ = ticker.tick() => {}
            }
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        self.screen.render(frame);
        if let Some(popup) = &self.popup {
            let area = frame.area();

            frame.render_widget(
                popup.clone(),
                Rect {
                    x: area.x / 2,
                    y: area.y / 2,
                    width: area.width / 2,
                    height: area.height / 2,
                },
            );
        }
    }

    async fn handle_event(&mut self, event: Event) {
        match &event {
            crossterm::event::Event::Key(key_event) => self.handle_key(key_event).await,
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
