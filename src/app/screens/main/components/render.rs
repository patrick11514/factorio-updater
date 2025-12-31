use ratatui::{
    Frame,
    layout::{self, Rect},
    widgets::{Block, ListState, Scrollbar, ScrollbarState, Wrap},
};

use crate::app::{
    api::structs::Version,
    components::log::Log,
    config::{Config, InstalledVersion},
    screens::main::screen::{Main, SelectedList},
    utils::{ORANGE, border_with_title, style_list, style_scrollbar},
};

use ratatui::{
    layout::Layout,
    style::{Color, Style},
    symbols::merge::MergeStrategy,
    text::Line,
    widgets::{BorderType, List, ListDirection, Paragraph},
};

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

    render_installed_versions(
        frame,
        main_layout[0],
        installed_versions,
        &mut main.version_list_state,
        &mut main.version_scrollbar_state,
        matches!(main.selected_list, SelectedList::Versions),
    );
    render_more_info(
        frame,
        main_layout[1],
        main.selected_version
            .and_then(|idx| installed_versions.get(idx)),
    );

    render_logs(
        frame,
        layout[2],
        &mut main.logs,
        &mut main.logs_list_state,
        &mut main.logs_scrollbar_state,
        matches!(main.selected_list, SelectedList::Logs),
    );
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

fn render_installed_versions(
    frame: &mut Frame,
    area: Rect,
    installed: &Vec<InstalledVersion>,
    list_state: &mut ListState,
    scrollbar_state: &mut ScrollbarState,
    selected: bool,
) {
    let title = if area.width > 50 {
        "[I] Installed Versions - [A] to install a new version"
    } else {
        "[I] Installed [A] New"
    };

    let area = border_with_title(
        frame,
        Line::from(title).style(
            Style::default()
                .fg(Color::Indexed(112) /* Light Green */)
                .bold(),
        ),
        area,
        if selected {
            Style::default().fg(ORANGE).bold()
        } else {
            Style::default()
        },
    );

    if installed.is_empty() {
        let paragraph = Paragraph::new("No installed versions found")
            .style(Style::default().fg(Color::Red).bold())
            .wrap(Wrap { trim: false })
            .centered();

        frame.render_widget(paragraph, area);
        return;
    }

    let items = installed.iter().map(|version| {
        format!(
            "Factorio{} - {} for {}",
            match version.version {
                Version::Vanilla => "",
                Version::SpaceAge => " Space Age",
                Version::Headless => " Headless",
            },
            version.current_version,
            version.platform
        )
    });

    let list = style_list(List::new(items));
    frame.render_stateful_widget(list, area, list_state);

    let scrollbar = style_scrollbar(Scrollbar::default());
    frame.render_stateful_widget(scrollbar, area, scrollbar_state);
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
        Style::default(),
    );

    if let Some(version) = version {
        //TODO
    } else {
        let paragraph = Paragraph::new("No version selected")
            .style(Style::default().fg(Color::Red).bold())
            .wrap(Wrap { trim: false })
            .centered();

        frame.render_widget(paragraph, area);
    }
}

fn render_logs(
    frame: &mut Frame,
    area: Rect,
    logs: &mut Vec<Box<Log>>,
    list_state: &mut ListState,
    scrollbar_state: &mut ScrollbarState,
    selected: bool,
) {
    let logs_container = border_with_title(
        frame,
        Line::from("[L] Logs").style(Style::default().fg(Color::Indexed(196) /* Red */).bold()),
        area,
        if selected {
            Style::default().fg(ORANGE).bold()
        } else {
            Style::default()
        },
    );

    let items = logs.iter_mut().rev().map(|log| log.render(&logs_container));

    let logs = style_list(List::new(items).direction(ListDirection::BottomToTop));
    frame.render_stateful_widget(logs, logs_container, list_state);

    let scrollbar = style_scrollbar(Scrollbar::default());
    frame.render_stateful_widget(scrollbar, logs_container, scrollbar_state);
}
