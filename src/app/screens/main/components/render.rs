use ratatui::{
    Frame,
    layout::{self, Rect},
    widgets::{Block, Wrap},
};

use crate::app::{
    components::log::Log,
    config::{Config, InstalledVersion},
    screens::main::screen::Main,
};

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

    render_title(&main.api.config, frame, layout[0]);

    let main_layout = Layout::default()
            .spacing(layout::Spacing::Overlap(1))
            .direction(layout::Direction::Horizontal)
            .constraints(vec![
                layout::Constraint::Percentage(50),
                layout::Constraint::Percentage(51), /* because of overlap -> one col at the end was empty */
            ])
            .split(layout[1]);

    let installed_versions = &main.api.config.installed_versions;

    render_installed_versions(frame, main_layout[0], installed_versions);
    render_more_info(
        frame,
        main_layout[1],
        main.selected_version
            .and_then(|idx| installed_versions.get(idx)),
    );

    render_logs(frame, layout[2], &mut main.logs);
}

fn render_title(config: &Config, frame: &mut Frame, area: Rect) {
    let title = Paragraph::new(
        Line::from(format!("Factorio Updater - {}", config.username))
            .style(Style::default().fg(Color::Indexed(208) /* Orange */).bold()),
    )
    .block(
        Block::bordered()
            .border_type(BorderType::Plain)
            .merge_borders(MergeStrategy::Exact),
    );

    frame.render_widget(title, area);
}

fn render_installed_versions(frame: &mut Frame, area: Rect, installed: &Vec<InstalledVersion>) {
    let title = if area.width > 50 {
        "Installed Versions - [A] to install a new version"
    } else {
        "[A] Installed"
    };

    let area = border_with_title(
        frame,
        Line::from(title).style(
            Style::default()
                .fg(Color::Indexed(112) /* Light Green */)
                .bold(),
        ),
        area,
    );

    if installed.is_empty() {
        let paragraph = Paragraph::new("No installed versions found")
            .style(Style::default().fg(Color::Red).bold())
            .wrap(Wrap { trim: false })
            .centered();

        frame.render_widget(paragraph, area);
        return;
    }
}

fn render_more_info(frame: &mut Frame, area: Rect, version: Option<&InstalledVersion>) {
    let area = border_with_title(
        frame,
        Line::from("More Info").style(
            Style::default()
                .fg(Color::Indexed(45) /* Light Blue */)
                .bold(),
        ),
        area,
    );

    if let Some(version) = version {
        todo!("Render more info about selected version");
    } else {
        let paragraph = Paragraph::new("No version selected")
            .style(Style::default().fg(Color::Red).bold())
            .wrap(Wrap { trim: false })
            .centered();

        frame.render_widget(paragraph, area);
    }
}

fn render_logs(frame: &mut Frame, area: Rect, logs: &mut Vec<Box<Log>>) {
    let logs_container = border_with_title(
        frame,
        Line::from("Logs").style(Style::default().fg(Color::Indexed(196) /* Red */).bold()),
        area,
    );

    let items = logs.iter_mut().rev().map(|log| log.render(&logs_container));

    let logs = List::new(items).direction(ListDirection::BottomToTop);

    frame.render_widget(logs, logs_container);
}
