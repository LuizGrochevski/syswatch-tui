use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Gauge},
    Frame,
};
use crate::metrics::collector::{MemoryMetrics, formatar_bytes};

pub fn render(f: &mut Frame, area: Rect, mem: &MemoryMetrics) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // RAM gauge
            Constraint::Length(1),  // espaço
            Constraint::Length(3),  // SWAP gauge
            Constraint::Min(0),     // área vazia restante
        ])
        .split(area);

    // RAM
    let pct_ram = if mem.total > 0 {
        (mem.usado * 100 / mem.total) as u16
    } else { 0 };

    let cor_ram = cor_por_pct(pct_ram);
    let ram_gauge = Gauge::default()
        .block(Block::default()
            .title(format!(" 🧠 RAM — {} / {} ",
                formatar_bytes(mem.usado),
                formatar_bytes(mem.total)))
            .borders(Borders::ALL))
        .gauge_style(Style::default().fg(cor_ram).bg(Color::Black))
        .percent(pct_ram)
        .label(format!("{}%", pct_ram));
    f.render_widget(ram_gauge, chunks[0]);

    // SWAP
    let pct_swap = if mem.swap_total > 0 {
        (mem.swap_usado * 100 / mem.swap_total) as u16
    } else { 0 };

    let cor_swap = cor_por_pct(pct_swap);
    let label_swap = if mem.swap_total == 0 {
        "sem swap".to_string()
    } else {
        format!("{}%", pct_swap)
    };

    let swap_gauge = Gauge::default()
        .block(Block::default()
            .title(format!(" 💾 SWAP — {} / {} ",
                formatar_bytes(mem.swap_usado),
                formatar_bytes(mem.swap_total)))
            .borders(Borders::ALL))
        .gauge_style(Style::default().fg(cor_swap).bg(Color::Black))
        .percent(pct_swap)
        .label(label_swap);
    f.render_widget(swap_gauge, chunks[2]);
}

fn cor_por_pct(pct: u16) -> Color {
    if pct >= 80 { Color::Rgb(220, 50, 50) }
    else if pct >= 50 { Color::Rgb(220, 180, 0) }
    else { Color::Rgb(0, 200, 80) }
}