use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Bar, BarChart, BarGroup, Block, Borders, Gauge, Paragraph},
    Frame,
};
use crate::metrics::collector::CpuMetrics;

pub fn render(f: &mut Frame, area: Rect, cpu: &CpuMetrics, historico: &[f32]) {
    let cor = cor_por_uso(cpu.usage_global);

    let titulo = format!(
        " 🖥️  CPU — {} | {:.0} MHz | {:.1}% ",
        cpu.nome, cpu.frequencia_mhz, cpu.usage_global
    );

    let bloco = Block::default()
        .title(titulo)
        .borders(Borders::ALL)
        .style(Style::default().fg(cor));

    // Gráfico de barras por core
    let barras: Vec<Bar> = cpu.usage_por_core
        .iter()
        .enumerate()
        .map(|(i, uso)| {
            Bar::default()
                .value(*uso as u64)
                .label(Line::from(format!("C{}", i)))
                .style(Style::default().fg(cor_por_uso(*uso)))
                .value_style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
        })
        .collect();

    let grupo = BarGroup::default().bars(&barras);

    let chart = BarChart::default()
        .block(bloco)
        .max(100)
        .bar_width(3)
        .bar_gap(1)
        .data(grupo);

    f.render_widget(chart, area);
}

pub fn render_gauge(f: &mut Frame, area: Rect, cpu: &CpuMetrics) {
    let cor = cor_por_uso(cpu.usage_global);
    let gauge = Gauge::default()
        .block(Block::default().title(" CPU Global ").borders(Borders::ALL))
        .gauge_style(Style::default().fg(cor).bg(Color::Black))
        .percent(cpu.usage_global as u16)
        .label(format!("{:.1}%", cpu.usage_global));
    f.render_widget(gauge, area);
}

fn cor_por_uso(uso: f32) -> Color {
    if uso >= 80.0 { Color::Rgb(220, 50, 50) }
    else if uso >= 50.0 { Color::Rgb(220, 180, 0) }
    else { Color::Rgb(0, 200, 80) }
}
