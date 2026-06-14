use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};
use crate::metrics::collector::{NetworkMetrics, formatar_bytes};

pub fn render(f: &mut Frame, area: Rect, redes: &[NetworkMetrics]) {
    let header = Row::new(vec![
        Cell::from("Interface").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Cell::from("↑ Enviado").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Cell::from("↓ Recebido").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Cell::from("Pkts TX").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Cell::from("Pkts RX").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
    ]).height(1).bottom_margin(1);

    let rows: Vec<Row> = redes.iter().map(|r| {
        Row::new(vec![
            Cell::from(r.interface.clone()).style(Style::default().fg(Color::Yellow)),
            Cell::from(formatar_bytes(r.bytes_enviados)).style(Style::default().fg(Color::Green)),
            Cell::from(formatar_bytes(r.bytes_recebidos)).style(Style::default().fg(Color::Blue)),
            Cell::from(r.pacotes_enviados.to_string()),
            Cell::from(r.pacotes_recebidos.to_string()),
        ])
    }).collect();

    let tabela = Table::new(rows, [
        Constraint::Min(12),
        Constraint::Length(12),
        Constraint::Length(12),
        Constraint::Length(10),
        Constraint::Length(10),
    ])
    .header(header)
    .block(Block::default()
        .title(" 🌐 Rede ")
        .borders(Borders::ALL));

    f.render_widget(tabela, area);
}

use ratatui::layout::Constraint;
