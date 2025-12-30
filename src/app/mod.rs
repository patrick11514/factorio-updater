use std::time::Duration;

use crate::app::{
    api::Api,
    components::popup::{Popup, PopupSize},
    config::Config,
    screens::{Screen, login::Login, main::screen::Main},
};
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent};
use futures_util::StreamExt;
use ratatui::{DefaultTerminal, Frame, layout::Rect};

mod api;
mod components;
mod config;
mod screens;
mod utils;
mod workflows;

pub struct App<'a> {
    exited: bool,
    screen: Box<dyn Screen>,
    screen_run_task: Option<tokio::task::JoinHandle<()>>,
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
                    Ok(event) => {
                        let _ = tx.send(event).await;
                    }
                    _ => {}
                }
            }
        });

        let mut app = Self {
            exited: false,
            screen: match config {
                Some(config) => Box::new(Main::new(Api::new(config))),
                None => Box::new(Login::default()),
            },
            popup: None,
            event_rx: rx,
            screen_run_task: None,
        };

        app.screen.init();

        app
    }

    pub async fn main_loop(mut self, term: &mut DefaultTerminal) -> anyhow::Result<()> {
        let mut ticker = tokio::time::interval(Duration::from_millis(16) /* ~62FPS */);

        while !self.exited {
            term.draw(|frame| self.draw(frame))?;

            tokio::select! {
                event = self.event_rx.recv()  => {
                    self.handle_event(event.unwrap()).await;
                }
                _ = ticker.tick() => {
                    if let Some(ev) = self.screen.tick() {
                        self.handle_screen_event(ev);
                    }
                }
            }
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        self.screen.render(frame);

        if let Some(popup) = &mut self.popup {
            let area = frame.area();

            //2k = 227
            let ratio = if area.width > 200 {
                match popup.size {
                    PopupSize::Small => 7,
                    PopupSize::Medium => 5,
                    PopupSize::Large => 3,
                }
            } else if area.width > 100 {
                match popup.size {
                    PopupSize::Small => 5,
                    PopupSize::Medium => 3,
                    PopupSize::Large => 2,
                }
            } else if area.width > 50 {
                match popup.size {
                    PopupSize::Small => 3,
                    PopupSize::Medium => 2,
                    PopupSize::Large => 1,
                }
            } else {
                1
            };

            frame.render_widget(
                popup,
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
        let soft_exit = matches!(ev.code, KeyCode::Char('q') | KeyCode::Esc);

        let is_exit = matches!(
            ev.code,
            KeyCode::Char('c')
                if ev
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL)

        );

        if soft_exit {
            match self.popup {
                Some(_) => {
                    self.popup = None;
                }
                None => {
                    self.handle_exit();
                }
            }
        }

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
            self.handle_screen_event(screen_ev);
        }
    }

    fn handle_screen_event(&mut self, event: screens::ScreenEvent) {
        match event {
            screens::ScreenEvent::Logged(config) => self.switch_screen(Main::new(config)),
            screens::ScreenEvent::OpenPopup(popup) => self.popup = Some(popup),
            screens::ScreenEvent::ClosePopup => self.popup = None,
            screens::ScreenEvent::Logout => {
                self.popup = None;
                self.switch_screen(Login::default());
            }
            screens::ScreenEvent::RunInit => {
                self.screen.init();
            }
            screens::ScreenEvent::PopupControl(popup_control) => {
                if let Some(popup) = &mut self.popup {
                    popup.handle_control(popup_control);
                }
            }
        }
    }

    fn handle_exit(&mut self) {
        self.exited = true;
    }

    fn switch_screen<T: Screen + 'static>(&mut self, screen: T) {
        if let Some(handle) = self.screen_run_task.take() {
            handle.abort();
        }

        self.screen = Box::new(screen);
        let handle = self.screen.init();

        self.screen_run_task = handle;
    }
}
