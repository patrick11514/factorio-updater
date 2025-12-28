use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    symbols::merge::MergeStrategy,
    text::Line,
};

pub fn with_title(frame: &mut Frame, title: Line, area: Rect) -> Rect {
    let layout = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints(vec![Constraint::Length(1), Constraint::Min(1)])
        .split(area);

    frame.render_widget(title, layout[0]);

    layout[1]
}

pub fn border_with_title(frame: &mut Frame, title: Line, area: Rect) -> Rect {
    let outer = ratatui::widgets::Block::bordered().merge_borders(MergeStrategy::Exact);
    let inner = outer.inner(area);

    frame.render_widget(outer, area);
    with_title(frame, title, inner)
}
