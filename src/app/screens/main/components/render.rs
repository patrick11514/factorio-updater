use std::collections::HashMap;

use ratatui::{
    Frame,
    layout::{self, Rect},
    style::Modifier,
    text::Span,
    widgets::{Block, ListState, Scrollbar, ScrollbarState, Wrap},
};

use crate::app::{
    api::structs::Version,
    components::log::Log,
    config::{Config, InstalledVersion},
    screens::main::{
        components::run::{InstalledVersionDetails, InstalledVersionState, UpdateType},
        screen::{Main, SelectedList},
    },
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
        &Main::get_installed_versions(&main.api.config.installed_versions),
        &main.installed_version_details,
        &mut main.version_list_state,
        &mut main.version_scrollbar_state,
        matches!(main.selected_list, SelectedList::Versions),
    );
    render_more_info(
        frame,
        main_layout[1],
        main.selected_version
            .and_then(|idx| installed_versions.get(&idx)),
        main.selected_version
            .and_then(|idx| main.installed_version_details.get(&idx)),
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
    installed: &Vec<(&uuid::Uuid, &InstalledVersion)>,
    details: &HashMap<uuid::Uuid, InstalledVersionDetails>,
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

    let items = installed.iter().map(|(uuid, version)| {
        let detail = details.get(uuid);

        let state = detail.map(|d| &d.state);

        Line::from(vec![
            match state {
                None => Span::styled(" ? ", Style::default().fg(Color::Yellow)),
                Some(InstalledVersionState::UpToDate) => {
                    Span::styled(" ✔ ", Style::default().fg(Color::Green))
                }
                Some(InstalledVersionState::UpdateAvailable(_)) => {
                    Span::styled(" ▲ ", Style::default().fg(Color::Blue))
                }
                Some(InstalledVersionState::Updating) => {
                    Span::styled(" ⟳ ", Style::default().fg(Color::Magenta))
                }
            },
            Span::styled("Factorio ", Style::default().fg(ORANGE).bold()), // Standard Orange
            match version.version {
                Version::Vanilla => Span::raw(""), // Vanilla usually doesn't need a tag
                Version::SpaceAge => Span::styled(
                    "Space Age ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::ITALIC),
                ),
                Version::Headless => Span::styled("Headless ", Style::default().fg(Color::Magenta)),
            },
            Span::styled(
                &version.current_version,
                Style::default().fg(Color::White).bold(),
            ),
            Span::styled(" • ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}", version.platform),
                Style::default().fg(Color::DarkGray),
            ),
        ])
    });

    let list = style_list(List::new(items));
    frame.render_stateful_widget(list, area, list_state);

    let scrollbar = style_scrollbar(Scrollbar::default());
    frame.render_stateful_widget(scrollbar, area, scrollbar_state);
}

fn render_more_info(
    frame: &mut Frame,
    area: Rect,
    version: Option<&InstalledVersion>,
    details: Option<&InstalledVersionDetails>,
) {
    let area = border_with_title(
        frame,
        Line::from(if let Some(details) = details {
            if matches!(details.state, InstalledVersionState::UpdateAvailable(_)) {
                "More Info - [U] to update"
            } else {
                "More Info"
            }
        } else {
            "More Info -  ?"
        })
        .style(
            Style::default()
                .fg(Color::Indexed(45) /* Light Blue */)
                .bold(),
        ),
        area,
        Style::default(),
    );

    if let Some(info) = version {
        let label_style = Style::default().fg(Color::Cyan);
        let value_style = Style::default().fg(Color::White);
        let path_style = Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::ITALIC);

        let mut lines = vec![];

        lines.push(Line::from(vec![
            Span::styled(
                match info.version {
                    Version::Vanilla => "Factorio Vanilla",
                    Version::SpaceAge => "Factorio Space Age",
                    Version::Headless => "Factorio Headless",
                },
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(
                format!("({})", info.platform),
                Style::default().fg(Color::Gray),
            ),
        ]));

        lines.push(Line::from(""));

        lines.push(Line::from(vec![
            Span::styled("Current Version: ", label_style),
            Span::styled(&info.current_version, value_style),
        ]));

        lines.push(Line::from(vec![
            Span::styled("Install Path:    ", label_style),
            Span::styled(info.path.display().to_string(), path_style),
        ]));

        lines.push(Line::from(""));

        if let Some(detail) = details {
            match &detail.state {
                InstalledVersionState::UpToDate => {
                    lines.push(Line::from(vec![
                        Span::styled("Status: ", label_style),
                        Span::styled("✔ Up to date", Style::default().fg(Color::Green)),
                    ]));
                }
                InstalledVersionState::UpdateAvailable(update_type) => {
                    lines.push(Line::from(vec![
                        Span::styled("Status: ", label_style),
                        Span::styled("▲ Update Available", Style::default().fg(Color::Yellow)),
                    ]));

                    match update_type {
                        UpdateType::FullGame(target_ver) => {
                            lines.push(Line::from(vec![
                                Span::raw("  • Method: "),
                                Span::styled("Full Game Download", Style::default().fg(Color::Red)),
                            ]));
                            lines.push(Line::from(vec![
                                Span::raw("  • Target: "),
                                Span::styled(target_ver, Style::default().fg(Color::Green).bold()),
                            ]));
                        }
                        UpdateType::Patch(diffs) => {
                            let target_ver =
                                diffs.last().map(|d| &d.to).unwrap_or(&info.current_version);
                            let count = diffs.len();

                            lines.push(Line::from(vec![
                                Span::raw("  • Method: "),
                                Span::styled(
                                    format!("Incremental Patches ({})", count),
                                    Style::default().fg(Color::Blue),
                                ),
                            ]));
                            lines.push(Line::from(vec![
                                Span::raw("  • Target: "),
                                Span::styled(target_ver, Style::default().fg(Color::Green).bold()),
                            ]));

                            lines.push(Line::from(""));
                            lines.push(Line::from(Span::styled(
                                "Patch Chain:",
                                Style::default().fg(Color::Gray),
                            )));

                            if count <= 3 {
                                for diff in diffs {
                                    lines.push(Line::from(format!(
                                        "    {} -> {}",
                                        diff.from, diff.to
                                    )));
                                }
                            } else {
                                let first = &diffs[0];
                                let last = diffs.last().unwrap();
                                lines.push(Line::from(format!(
                                    "    {} -> ... ({} steps) ... -> {}",
                                    first.from, count, last.to
                                )));
                            }
                        }
                    }
                }
                InstalledVersionState::Updating => {
                    lines.push(Line::from(vec![
                        Span::styled("Status: ", label_style),
                        Span::styled("⟳ Updating", Style::default().fg(Color::Magenta)),
                    ]));
                }
            }
        }

        let paragraph = Paragraph::new(lines)
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
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
    logs: &mut Vec<Log>,
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

    let items = logs
        .iter_mut()
        .rev()
        .map(|log| log.render(&logs_container, selected));

    let logs = style_list(List::new(items).direction(ListDirection::BottomToTop));
    frame.render_stateful_widget(logs, logs_container, list_state);

    let scrollbar = style_scrollbar(Scrollbar::default());
    frame.render_stateful_widget(scrollbar, logs_container, scrollbar_state);
}
