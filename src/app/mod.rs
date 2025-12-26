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
use ratatui::{DefaultTerminal, Frame, layout::Rect};

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

            //2k = 227
            let ratio = if area.width > 200 {
                7
            } else if area.width > 100 {
                5
            } else if area.width > 50 {
                3
            } else {
                1
            };

            frame.render_widget(
                popup.clone(),
                if ratio == 1 {
                    Rect {
                        x: 0,
                        y: 0,
                        width: area.width,
                        height: area.height,
                    }
                } else {
                    Rect {
                        //magic 🧙
                        x: (area.width - (area.width / ratio)) / 2,
                        y: (area.height - (area.height / ratio)) / 2,
                        width: area.width / ratio,
                        height: (area.height / ratio).max(5),
                    }
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
        let is_exit = matches!(
            ev.code,
            KeyCode::Char('c')
                if ev
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL)

        ) || matches!(ev.code, KeyCode::Char('q') | KeyCode::Esc);

        if is_exit {
            self.handle_exit();
            return;
        }

        let popup_result = self
            .popup
            .as_mut()
            .map_or(None, |popup| popup.handle_key(ev));

        let screen_result = match popup_result {
            Some(popup_result) => self.screen.on_popup(popup_result).await,
            None => self.screen.on_key(ev).await,
        };

        if let Some(screen_ev) = screen_result {
            match screen_ev {
                screens::ScreenEvent::Logged(config) => self.screen = Box::new(Main::new(config)),
                screens::ScreenEvent::OpenPopup(popup) => self.popup = Some(popup),
                screens::ScreenEvent::ClosePopup => self.popup = None,
            }
        }
    }

    fn handle_exit(&mut self) {
        self.exited = true;
    }
}
