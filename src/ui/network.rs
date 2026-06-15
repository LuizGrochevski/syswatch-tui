use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    text::{Line, Span},
    Frame,
};
use crate::metrics::network::ler_interfaces;

pub fn render(f: &mut Frame, area: Rect) {
    let interfaces = ler_interfaces();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0)])
        .split(area);

    let header = Row::new(vec![
        Cell::from("Interface").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Cell::from("IP").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Cell::from("Máscara").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Cell::from("MTU").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Cell::from("Flags").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
    ]).height(1).bottom_margin(1);

    let rows: Vec<Row> = interfaces.iter().map(|iface| {
        let ativo = iface.flags.contains(&"UP".to_string()) && iface.flags.contains(&"RUNNING".to_string());
        let cor = if ativo { Color::Rgb(0, 200, 80) } else { Color::DarkGray };
        let loopback = iface.flags.contains(&"LOOPBACK".to_string());
        let cor_nome = if loopback { Color::DarkGray } else { Color::Yellow };

        Row::new(vec![
            Cell::from(iface.nome.clone()).style(Style::default().fg(cor_nome).add_modifier(Modifier::BOLD)),
            Cell::from(iface.ip.clone().unwrap_or_else(|| "-".to_string())).style(Style::default().fg(cor)),
            Cell::from(iface.mascara.clone().unwrap_or_else(|| "-".to_string())).style(Style::default().fg(Color::White)),
            Cell::from(iface.mtu.map(|m| m.to_string()).unwrap_or_else(|| "-".to_string())).style(Style::default().fg(Color::White)),
            Cell::from(iface.flags.join(", ")).style(Style::default().fg(Color::DarkGray)),
        ])
    }).collect();

    let tabela = Table::new(rows, [
        Constraint::Length(15),
        Constraint::Length(16),
        Constraint::Length(16),
        Constraint::Length(7),
        Constraint::Min(20),
    ])
    .header(header)
    .block(Block::default()
        .title(" 🌐 Interfaces de Rede ")
        .borders(Borders::ALL));

    f.render_widget(tabela, chunks[0]);
}
