pub(crate) mod login;
pub(crate) mod main;

use ratatui::Frame;

use crate::app::screens::{login::Login, main::Main};

pub enum Screen {
    Login(Login),
    Main(Main),
}

trait Renderable {
    fn render(&mut self, frame: &mut Frame);
}

impl Screen {
    pub fn render(&mut self, frame: &mut Frame) {
        match self {
            Screen::Login(login) => login.render(frame),
            Screen::Main(main) => main.render(frame),
        }
    }
}

impl Default for Screen {
    fn default() -> Self {
        Screen::Login(Login {})
    }
}
