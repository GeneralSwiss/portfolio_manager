use crate::{Portfolio, Position};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Flex, Layout, Rect};
use ratatui::widgets::{Paragraph, Row, Table};

pub fn net_greeks(height: u16, area: Rect) -> Rect {
    let [area] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(height)])
        .flex(Flex::Center)
        .areas(area);
    area
}

pub fn display_portfolio_total(amount: f64) -> Paragraph<'static> {
    Paragraph::new(format!("Portfolio Total: ${amount}"))
}

pub fn main_page(frame: &mut Frame, portfolio: &Portfolio) {
    let [summary_bar, main] = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(4), Constraint::Percentage(100)])
        .areas(frame.area());

    let [_, total_area] = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .areas(summary_bar);

    frame.render_widget(display_portfolio_total(100.0), total_area);
    frame.render_widget(create_position_table(&portfolio.positions), main);
}

impl<'a> From<&'a Position> for Row<'a> {
    fn from(value: &'a Position) -> Self {
        Row::new(vec![
            value.id.clone(),
            value.underlying.clone(),
            value.market_value().to_string(),
        ])
    }
}

pub fn create_position_table(positions: &[Position]) -> Table {
    Table::default()
        .header(Row::new(vec!["ID", "Underlying", "Market Value"]))
        .rows(positions.iter().map(|p| p.into()))
}
