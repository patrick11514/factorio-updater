use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    symbols::merge::MergeStrategy,
    text::Line,
    widgets::{Block, List, Scrollbar, ScrollbarOrientation},
};

use crate::app::api::structs::{Arch, Item, Platform, Updates, Version};

pub(crate) mod installation;

pub fn with_title(frame: &mut Frame, title: Line, area: Rect) -> Rect {
    let layout = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints(vec![Constraint::Length(1), Constraint::Min(1)])
        .split(area);

    frame.render_widget(title, layout[0]);

    layout[1]
}

pub fn border_with_title(frame: &mut Frame, title: Line, area: Rect, border_style: Style) -> Rect {
    let outer = Block::bordered()
        .border_style(border_style)
        .merge_borders(MergeStrategy::Exact);
    let inner = outer.inner(area);

    frame.render_widget(outer, area);
    with_title(frame, title, inner)
}

pub fn get_sorted_updates(updates: &Updates, version: &Version, platform: &Platform) -> Vec<Item> {
    let arch: Arch = (version, platform).into();
    let mut updates = updates.get(&arch).unwrap().clone();
    updates.sort();
    updates.reverse();
    updates
}

pub static ORANGE: Color = Color::Indexed(202);

pub fn style_list(list: List) -> List {
    list.block(Block::default())
        .highlight_style(Style::default().fg(ORANGE).bold())
        .highlight_symbol(">> ")
}

pub fn style_scrollbar(scrollbar: Scrollbar) -> Scrollbar {
    scrollbar
        .orientation(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("▴"))
        .end_symbol(Some("▾"))
        .track_symbol(Some("│"))
        .thumb_symbol("█")
}
