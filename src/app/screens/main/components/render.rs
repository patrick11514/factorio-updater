use ratatui::{layout, widgets::Block};

use crate::app::screens::main::screen::Main;

use std::sync::atomic;

use async_trait::async_trait;
use crossterm::event::KeyEvent;
use derive_builder::Builder;
use ratatui::{
    layout::Layout,
    style::{Color, Style},
    symbols::merge::MergeStrategy,
    text::Line,
    widgets::{BorderType, List, ListDirection, Paragraph},
};

use crate::utils::border_with_title;

pub fn render(main: &mut Main, frame: &mut ratatui::Frame) {
    let outer = Block::default();

    let inner = outer.inner(frame.area());

    let layout = Layout::default()
        .spacing(layout::Spacing::Overlap(1))
        .direction(layout::Direction::Vertical)
        .constraints(vec![
            layout::Constraint::Length(3),
            layout::Constraint::Ratio(2, 3),
            layout::Constraint::Min(3),
        ])
        .split(inner);

    frame.render_widget(outer, frame.area());

    let title = Paragraph::new(
        Line::from(format!("Factorio Updater - {}", main.api.config.username))
            .style(Style::default().fg(Color::Indexed(208) /* Orange */).bold()),
    )
    .block(
        Block::bordered()
            .border_type(BorderType::Plain)
            .merge_borders(MergeStrategy::Exact),
    );

    frame.render_widget(title, layout[0]);

    let main_layout = Layout::default()
            .spacing(layout::Spacing::Overlap(1))
            .direction(layout::Direction::Horizontal)
            .constraints(vec![
                layout::Constraint::Percentage(50),
                layout::Constraint::Percentage(51), /* because of overlap -> one col at the end was empty */
            ])
            .split(layout[1]);

    let installed_versions = border_with_title(
        frame,
        Line::from("Installed Versions").style(
            Style::default()
                .fg(Color::Indexed(112) /* Light Green */)
                .bold(),
        ),
        main_layout[0],
    );

    let more_info = border_with_title(
        frame,
        Line::from("More Info").style(
            Style::default()
                .fg(Color::Indexed(45) /* Light Blue */)
                .bold(),
        ),
        main_layout[1],
    );

    let logs_container = border_with_title(
        frame,
        Line::from("Logs").style(Style::default().fg(Color::Indexed(196) /* Red */).bold()),
        layout[2],
    );

    let items = main
        .logs
        .iter_mut()
        .rev()
        .map(|log| log.render(&logs_container));

    let logs = List::new(items).direction(ListDirection::BottomToTop);

    frame.render_widget(logs, logs_container);
}
