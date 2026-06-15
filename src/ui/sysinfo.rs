use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};
use crate::metrics::collector::{SystemMetrics, formatar_bytes, formatar_uptime};

/// Tenta ler a faixa de frequência disponível do core 0.
/// No Linux desktop scaling_available_frequencies pode não existir —
/// nesse caso mostra min e max separadamente via cpuinfo_{min,max}_freq.
fn ler_faixa_frequencia() -> String {
    let base = "/sys/devices/system/cpu/cpu0/cpufreq";

    // Opção 1: scaling_available_frequencies (Termux / alguns kernels)
    let disponivel = std::fs::read_to_string(format!("{}/scaling_available_frequencies", base))
        .unwrap_or_default();
    let freqs: Vec<u64> = disponivel
        .split_whitespace()
        .filter_map(|s| s.parse::<u64>().ok())
        .map(|f| f / 1000)
        .collect();

    if !freqs.is_empty() {
        let min = freqs.iter().min().unwrap_or(&0);
        let max = freqs.iter().max().unwrap_or(&0);
        return format!("{} - {} MHz ({} níveis)", min, max, freqs.len());
    }

    // Opção 2: cpuinfo_min_freq + cpuinfo_max_freq (Linux desktop padrão)
    let min_khz = std::fs::read_to_string(format!("{}/cpuinfo_min_freq", base))
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .map(|v| v / 1000);

    let max_khz = std::fs::read_to_string(format!("{}/cpuinfo_max_freq", base))
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .map(|v| v / 1000);

    match (min_khz, max_khz) {
        (Some(min), Some(max)) => format!("{} - {} MHz", min, max),
        (None, Some(max))      => format!("até {} MHz", max),
        (Some(min), None)      => format!("a partir de {} MHz", min),
        (None, None)           => "não disponível".to_string(),
    }
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
    // Usa temperatura e governor já coletados pelo collector (linux.rs)
    // evitando leituras duplicadas de /sys a cada frame
    let cor_temp = metrics.temperatura.map(|t| {
        if t >= 70.0      { Color::Rgb(220, 50, 50) }
        else if t >= 55.0 { Color::Rgb(220, 180, 0) }
        else              { Color::Rgb(0, 200, 80) }
    }).unwrap_or(Color::DarkGray);

    let temp_str = metrics.temperatura
        .map(|t| format!("{:.1}°C", t))
        .unwrap_or_else(|| "N/A".to_string());

    let governor_str = metrics.governor
        .clone()
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
            Cell::from(governor_str).style(Style::default().fg(Color::White)),
        ]),
        Row::new(vec![
            Cell::from("CPU Cores").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Cell::from(format!("{} cores", metrics.cpu.usage_por_core.len()))
                .style(Style::default().fg(Color::White)),
        ]),
        Row::new(vec![
            Cell::from("CPU Freq").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Cell::from(format!("{} MHz", metrics.cpu.frequencia_mhz))
                .style(Style::default().fg(Color::White)),
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
            Cell::from(
                if metrics.memoria.swap_total == 0 {
                    "sem swap".to_string()
                } else {
                    formatar_bytes(metrics.memoria.swap_total)
                }
            ).style(Style::default().fg(Color::White)),
        ]),
    ];

    let tabela = Table::new(rows, [Constraint::Length(14), Constraint::Min(20)])
        .block(Block::default().title(" ℹ️  Informações do Sistema ").borders(Borders::ALL));
    f.render_widget(tabela, area);
}

fn render_cpu_detail(f: &mut Frame, area: Rect, metrics: &SystemMetrics) {
    let freq_str = ler_faixa_frequencia();

    let uso_cores = metrics.cpu.usage_por_core.iter().enumerate()
        .map(|(i, u)| format!("C{}:{:.0}%", i, u))
        .collect::<Vec<_>>()
        .join("  ");

    let linhas = vec![
        Line::from(vec![
            Span::styled(" Frequências disponíveis: ",
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(freq_str, Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled(" Uso por core: ",
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(uso_cores, Style::default().fg(Color::White)),
        ]),
    ];

    let paragrafo = Paragraph::new(linhas)
        .block(Block::default().title(" 🔧 Detalhes CPU ").borders(Borders::ALL));
    f.render_widget(paragrafo, area);
}