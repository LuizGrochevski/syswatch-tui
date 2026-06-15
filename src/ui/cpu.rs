use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    symbols,
    widgets::{Axis, Bar, BarChart, BarGroup, Block, Borders, Chart, Dataset, Gauge, GraphType},
    Frame,
};
use crate::metrics::collector::CpuMetrics;

pub fn render(f: &mut Frame, area: Rect, cpu: &CpuMetrics, historico: &[f32]) {
    let n_cores = cpu.usage_por_core.len().max(1);
    let barchart_height = (area.height / 2).max(12).min(area.height.saturating_sub(8));

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(barchart_height),
            Constraint::Min(8),
        ])
        .split(area);

    render_barchart(f, chunks[0], cpu, n_cores);
    render_historico(f, chunks[1], historico);
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

fn render_barchart(f: &mut Frame, area: Rect, cpu: &CpuMetrics, _n_cores: usize) {
    let titulo = format!(
        " 🖥️  CPU — {} | {} MHz | {:.1}% ",
        cpu.nome, cpu.frequencia_mhz, cpu.usage_global
    );

    let barras: Vec<Bar> = cpu.usage_por_core
        .iter()
        .enumerate()
        .map(|(i, uso)| {
            Bar::default()
                .value(*uso as u64)
                .label(ratatui::text::Line::from(format!("C{}", i)))
                .style(Style::default().fg(cor_por_uso(*uso)))
                .value_style(Style::default().fg(Color::White).bg(cor_por_uso(*uso)))
        })
        .collect();

    let grupo = BarGroup::default().bars(&barras);

    let bar_w = {
        let disponivel = area.width.saturating_sub(4); // bordas
        let por_core = disponivel / (_n_cores.max(1) as u16);
        por_core.saturating_sub(1).clamp(3, 8)
    };

    let chart = BarChart::default()
        .block(Block::default().title(titulo).borders(Borders::ALL))
        .max(100)
        .bar_width(bar_w)
        .bar_gap(1)
        .data(grupo);

    f.render_widget(chart, area);
}

fn render_historico(f: &mut Frame, area: Rect, historico: &[f32]) {
    if historico.is_empty() {
        return;
    }

    let dados: Vec<(f64, f64)> = historico
        .iter()
        .enumerate()
        .map(|(i, v)| (i as f64, *v as f64))
        .collect();

    let max_x = dados.len().saturating_sub(1) as f64;
    let cor = cor_por_uso(*historico.last().unwrap_or(&0.0));

    let dataset = Dataset::default()
        .name("CPU%")
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(cor))
        .data(&dados);

    let chart = Chart::new(vec![dataset])
        .block(Block::default()
            .title(" 📈 Histórico CPU (60s) ")
            .borders(Borders::ALL))
        .x_axis(Axis::default()
            .bounds([0.0, max_x.max(60.0)])
            .style(Style::default().fg(Color::DarkGray)))
        .y_axis(Axis::default()
            .bounds([0.0, 100.0])
            .labels(vec![
                ratatui::text::Span::raw("0%"),
                ratatui::text::Span::raw("50%"),
                ratatui::text::Span::raw("100%"),
            ])
            .style(Style::default().fg(Color::DarkGray)));

    f.render_widget(chart, area);
}

pub fn cor_por_uso(uso: f32) -> Color {
    if uso >= 80.0 { Color::Rgb(220, 50, 50) }
    else if uso >= 50.0 { Color::Rgb(220, 180, 0) }
    else { Color::Rgb(0, 200, 80) }
}