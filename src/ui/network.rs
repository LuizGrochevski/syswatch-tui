use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};
use crate::metrics::collector::{NetworkMetrics, formatar_bytes};

/// Renderiza a tabela de interfaces de rede a partir das métricas já coletadas
/// (SystemMetrics.redes), em vez de chamar ler_interfaces() direto.
/// Isso garante que os contadores rx/tx do /proc/net/dev (Linux) apareçam na tela.
pub fn render(f: &mut Frame, area: Rect, redes: &[NetworkMetrics]) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0)])
        .split(area);

    let header = Row::new(vec![
        Cell::from("Interface").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Cell::from("IP").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Cell::from("MTU").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Cell::from("RX").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Cell::from("TX").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Cell::from("Flags").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
    ]).height(1).bottom_margin(1);

    let rows: Vec<Row> = redes.iter().map(|iface| {
        let ativo = iface.flags.contains(&"UP".to_string());
        let cor = if ativo { Color::Rgb(0, 200, 80) } else { Color::DarkGray };
        let loopback = iface.flags.contains(&"LOOPBACK".to_string());
        let cor_nome = if loopback { Color::DarkGray } else { Color::Yellow };

        let rx = if iface.bytes_recebidos > 0 {
            formatar_bytes(iface.bytes_recebidos)
        } else {
            "-".to_string()
        };
        let tx = if iface.bytes_enviados > 0 {
            formatar_bytes(iface.bytes_enviados)
        } else {
            "-".to_string()
        };

        Row::new(vec![
            Cell::from(iface.interface.clone()).style(Style::default().fg(cor_nome).add_modifier(Modifier::BOLD)),
            Cell::from(iface.ip.clone().unwrap_or_else(|| "-".to_string())).style(Style::default().fg(cor)),
            Cell::from(iface.mtu.map(|m| m.to_string()).unwrap_or_else(|| "-".to_string())).style(Style::default().fg(Color::White)),
            Cell::from(rx).style(Style::default().fg(Color::Green)),
            Cell::from(tx).style(Style::default().fg(Color::Magenta)),
            Cell::from(iface.flags.join(", ")).style(Style::default().fg(Color::DarkGray)),
        ])
    }).collect();

    let tabela = Table::new(rows, [
        Constraint::Length(15),
        Constraint::Length(16),
        Constraint::Length(7),
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Min(15),
    ])
    .header(header)
    .block(Block::default()
        .title(" 🌐 Interfaces de Rede ")
        .borders(Borders::ALL));

    f.render_widget(tabela, chunks[0]);
}
