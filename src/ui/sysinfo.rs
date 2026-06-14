use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};
use crate::metrics::collector::{SystemMetrics, formatar_bytes, formatar_uptime};

pub fn ler_temperatura() -> Option<f32> {
    for i in 0..10 {
        let path = format!("/sys/class/thermal/thermal_zone{}/temp", i);
        if let Ok(conteudo) = std::fs::read_to_string(&path) {
            if let Ok(temp) = conteudo.trim().parse::<i32>() {
                let celsius = temp as f32 / 1000.0;
                if celsius > 0.0 && celsius < 150.0 {
                    return Some(celsius);
                }
            }
        }
    }
    None
}

pub fn ler_governor() -> String {
    std::fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor")
        .unwrap_or_default()
        .trim()
        .to_string()
}

pub fn ler_frequencias_disponiveis() -> Vec<u64> {
    std::fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_available_frequencies")
        .unwrap_or_default()
        .split_whitespace()
        .filter_map(|s| s.parse::<u64>().ok())
        .map(|f| f / 1000)
        .collect()
}

pub fn render(f: &mut Frame, area: Rect, metrics: &SystemMetrics) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(12), Constraint::Min(0)])
        .split(area);

    render_info(f, chunks[0], metrics);
    render_cpu_detail(f, chunks[1], metrics);
}

fn render_info(f: &mut Frame, area: Rect, metrics: &SystemMetrics) {
    let temperatura = ler_temperatura();
    let governor = ler_governor();

    let cor_temp = temperatura.map(|t| {
        if t >= 70.0 { Color::Rgb(220, 50, 50) }
        else if t >= 55.0 { Color::Rgb(220, 180, 0) }
        else { Color::Rgb(0, 200, 80) }
    }).unwrap_or(Color::DarkGray);

    let temp_str = temperatura
        .map(|t| format!("{:.1}°C", t))
        .unwrap_or_else(|| "N/A".to_string());

    let pct_ram = if metrics.memoria.total > 0 {
        metrics.memoria.usado as f32 / metrics.memoria.total as f32 * 100.0
    } else { 0.0 };

    let rows = vec![
        Row::new(vec![
            Cell::from("Sistema").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Cell::from(metrics.os_nome.clone()).style(Style::default().fg(Color::White)),
        ]),
        Row::new(vec![
            Cell::from("Hostname").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Cell::from(metrics.hostname.clone()).style(Style::default().fg(Color::Yellow)),
        ]),
        Row::new(vec![
            Cell::from("Uptime").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Cell::from(formatar_uptime(metrics.uptime_secs)).style(Style::default().fg(Color::Green)),
        ]),
        Row::new(vec![
            Cell::from("Temperatura").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Cell::from(temp_str).style(Style::default().fg(cor_temp)),
        ]),
        Row::new(vec![
            Cell::from("CPU Governor").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Cell::from(governor).style(Style::default().fg(Color::White)),
        ]),
        Row::new(vec![
            Cell::from("CPU Cores").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Cell::from(format!("{} cores", metrics.cpu.usage_por_core.len())).style(Style::default().fg(Color::White)),
        ]),
        Row::new(vec![
            Cell::from("CPU Freq").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Cell::from(format!("{} MHz", metrics.cpu.frequencia_mhz)).style(Style::default().fg(Color::White)),
        ]),
        Row::new(vec![
            Cell::from("RAM Total").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Cell::from(formatar_bytes(metrics.memoria.total)).style(Style::default().fg(Color::White)),
        ]),
        Row::new(vec![
            Cell::from("RAM Usado").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Cell::from(format!("{} ({:.0}%)", formatar_bytes(metrics.memoria.usado), pct_ram))
                .style(Style::default().fg(Color::White)),
        ]),
        Row::new(vec![
            Cell::from("SWAP Total").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Cell::from(formatar_bytes(metrics.memoria.swap_total)).style(Style::default().fg(Color::White)),
        ]),
    ];

    let tabela = Table::new(rows, [Constraint::Length(14), Constraint::Min(20)])
        .block(Block::default().title(" ℹ️  Informações do Sistema ").borders(Borders::ALL));

    f.render_widget(tabela, area);
}

fn render_cpu_detail(f: &mut Frame, area: Rect, metrics: &SystemMetrics) {
    let freqs = ler_frequencias_disponiveis();
    let freq_str = if freqs.is_empty() {
        "N/A".to_string()
    } else {
        format!("{} - {} MHz ({} níveis)",
            freqs.iter().min().unwrap_or(&0),
            freqs.iter().max().unwrap_or(&0),
            freqs.len()
        )
    };

    let linhas = vec![
        Line::from(vec![
            Span::styled(" Frequências disponíveis: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(freq_str, Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled(" Uso por core: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(
                metrics.cpu.usage_por_core.iter().enumerate()
                    .map(|(i, u)| format!("C{}:{:.0}%", i, u))
                    .collect::<Vec<_>>()
                    .join("  "),
                Style::default().fg(Color::White)
            ),
        ]),
    ];

    let paragrafo = Paragraph::new(linhas)
        .block(Block::default().title(" 🔧 Detalhes CPU ").borders(Borders::ALL));

    f.render_widget(paragrafo, area);
}
