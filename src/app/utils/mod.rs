use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    symbols::merge::MergeStrategy,
    text::Line,
    widgets::{Block, List, Scrollbar, ScrollbarOrientation},
};

use crate::app::api::structs::{Arch, Item, Platform, Updates, Version};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::api::structs::Stable;
    use std::collections::HashMap;

    #[test]
    fn test_get_sorted_updates_basic() {
        let mut updates_map = HashMap::new();
        let arch = Arch::CoreLinux64;
        let items = vec![
            Item::Stable(Stable {
                stable: "1.1.0".to_string(),
            }),
            Item::Stable(Stable {
                stable: "1.0.0".to_string(),
            }),
            Item::Stable(Stable {
                stable: "1.2.0".to_string(),
            }),
        ];
        updates_map.insert(arch.clone(), items);

        let sorted = get_sorted_updates(&updates_map, &Version::Vanilla, &Platform::Linux64);

        assert_eq!(sorted.len(), 3);
        assert_eq!(sorted[0].to_string_raw(), "1.2.0");
        assert_eq!(sorted[1].to_string_raw(), "1.1.0");
        assert_eq!(sorted[2].to_string_raw(), "1.0.0");
    }

    #[test]
    #[should_panic]
    fn test_get_sorted_updates_missing_arch() {
        let updates_map = HashMap::new();
        // Should panic because unwrap() is used on the map get
        // And no updates are presented in HashMap
        get_sorted_updates(&updates_map, &Version::Vanilla, &Platform::Linux64);
    }
}
