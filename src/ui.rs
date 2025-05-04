use ratatui::layout::{Constraint, Direction, Flex, Layout, Rect};
use ratatui::widgets::{Block, Paragraph};

pub fn total_positions(height: u16, area: Rect, data: (f64, f64)) -> Rect {
    let [area] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(height)])
        .flex(Flex::Center)
        .areas(area);
}

pub fn net_greeks(height: u16, area: Rect) -> Rect {
    let [area] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(height)])
        .flex(Flex::Center)
        .areas(area);
    area
}

pub fn top_bar(height: u16, area: Rect) -> Rect {
    
}