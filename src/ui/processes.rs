use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame,
};
use crate::metrics::collector::ProcessoInfo;

pub fn render(f: &mut Frame, area: Rect, processos: &[ProcessoInfo]) {
    let header = Row::new(vec![
        Cell::from("PID").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Cell::from("Nome").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Cell::from("CPU%").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Cell::from("Mem MB").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Cell::from("Status").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
    ]).height(1).bottom_margin(1);

    let rows: Vec<Row> = processos.iter().map(|p| {
        let cor_cpu = if p.cpu >= 50.0 { Color::Red }
            else if p.cpu >= 20.0 { Color::Yellow }
            else { Color::White };

        Row::new(vec![
            Cell::from(p.pid.to_string()),
            Cell::from(p.nome.chars().take(20).collect::<String>()),
            Cell::from(format!("{:.1}", p.cpu)).style(Style::default().fg(cor_cpu)),
            Cell::from(format!("{:.1}", p.memoria_mb)),
            Cell::from(p.status.clone()).style(Style::default().fg(Color::DarkGray)),
        ])
    }).collect();

    let tabela = Table::new(rows, [
        Constraint::Length(7),
        Constraint::Min(20),
        Constraint::Length(6),
        Constraint::Length(8),
        Constraint::Length(10),
    ])
    .header(header)
    .block(Block::default()
        .title(" ⚙️  Processos (Top por CPU) ")
        .borders(Borders::ALL))
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    let mut state = TableState::default();
    f.render_stateful_widget(tabela, area, &mut state);
}
