use ratatui::text::Line;

use crate::app::components::popup::{PopupBuilder, PopupSize};

pub fn install_popup(builder: &mut PopupBuilder) {
    builder
        .title(Line::from("Version installation").centered())
        .size(PopupSize::Medium);
}
