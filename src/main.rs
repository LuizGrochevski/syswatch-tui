mod metrics;
mod ui;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use metrics::{MetricsCollector, SystemMetrics, formatar_uptime};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::{
    io,
    time::{Duration, Instant},
};

const INTERVALO_ATUALIZACAO: u64 = 1000;

struct App {
    collector: MetricsCollector,
    metrics: Option<SystemMetrics>,
    historico_cpu: Vec<f32>,
    tab_ativa: usize,
}

impl App {
    fn new() -> Self {
        Self {
            collector: MetricsCollector::new(),
            metrics: None,
            historico_cpu: Vec::new(),
            tab_ativa: 0,
        }
    }

    fn atualizar(&mut self) {
        let m = self.collector.coletar();
        self.historico_cpu.push(m.cpu.usage_global);
        if self.historico_cpu.len() > 60 {
            self.historico_cpu.remove(0);
        }
        self.metrics = Some(m);
    }
}

fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    app.atualizar();

    let resultado = run(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(e) = resultado {
        eprintln!("Erro: {}", e);
    }

    Ok(())
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    let mut ultimo_tick = Instant::now();

    loop {
        terminal.draw(|f| desenhar(f, app))?;

        let timeout = Duration::from_millis(INTERVALO_ATUALIZACAO)
            .checked_sub(ultimo_tick.elapsed())
            .unwrap_or_default();

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match (key.code, key.modifiers) {
                    (KeyCode::Char('q'), _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                        return Ok(());
                    }
                    (KeyCode::Tab, _) | (KeyCode::Right, _) => {
                        app.tab_ativa = (app.tab_ativa + 1) % 4;
                    }
                    (KeyCode::BackTab, _) | (KeyCode::Left, _) => {
                        app.tab_ativa = app.tab_ativa.saturating_sub(1);
                    }
                    (KeyCode::Char('1'), _) => app.tab_ativa = 0,
                    (KeyCode::Char('2'), _) => app.tab_ativa = 1,
                    (KeyCode::Char('3'), _) => app.tab_ativa = 2,
                    (KeyCode::Char('4'), _) => app.tab_ativa = 3,
                    _ => {}
                }
            }
        }

        if ultimo_tick.elapsed() >= Duration::from_millis(INTERVALO_ATUALIZACAO) {
            app.atualizar();
            ultimo_tick = Instant::now();
        }
    }
}

fn desenhar(f: &mut ratatui::Frame, app: &App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Length(3),  // Tabs
            Constraint::Min(0),     // Conteúdo
            Constraint::Length(1),  // Footer
        ])
        .split(area);

    // Header
    if let Some(m) = &app.metrics {
        let uptime = formatar_uptime(m.uptime_secs);
        let header_text = vec![Line::from(vec![
            Span::styled(" 🛡️  Syswatch-TUI ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("│ "),
            Span::styled(&m.hostname, Style::default().fg(Color::Yellow)),
            Span::raw(" │ "),
            Span::styled(&m.os_nome, Style::default().fg(Color::White)),
            Span::raw(" │ uptime: "),
            Span::styled(uptime, Style::default().fg(Color::Green)),
        ])];
        let header = Paragraph::new(header_text)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(header, chunks[0]);
    }

    // Tabs
    let tabs_labels = vec!["[1] CPU", "[2] Memória", "[3] Processos", "[4] Rede"];
    let tabs_text = tabs_labels.iter().enumerate().map(|(i, label)| {
        if i == app.tab_ativa {
            Span::styled(*label, Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD))
        } else {
            Span::styled(*label, Style::default().fg(Color::White))
        }
    }).flat_map(|s| vec![s, Span::raw("  ")]).collect::<Vec<_>>();

    let tabs = Paragraph::new(Line::from(tabs_text))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(tabs, chunks[1]);

    // Conteúdo
    if let Some(m) = &app.metrics {
        match app.tab_ativa {
            0 => {
                let sub = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(3), Constraint::Min(0)])
                    .split(chunks[2]);
                ui::cpu::render_gauge(f, sub[0], &m.cpu);
                ui::cpu::render(f, sub[1], &m.cpu, &app.historico_cpu);
            }
            1 => ui::memory::render(f, chunks[2], &m.memoria),
            2 => ui::processes::render(f, chunks[2], &m.processos),
            3 => ui::network::render(f, chunks[2], &m.redes),
            _ => {}
        }
    }

    // Footer
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" q", Style::default().fg(Color::Yellow)),
        Span::raw(": sair  "),
        Span::styled("Tab/←→", Style::default().fg(Color::Yellow)),
        Span::raw(": navegar  "),
        Span::styled("1-4", Style::default().fg(Color::Yellow)),
        Span::raw(": painéis  "),
        Span::styled("Ctrl+C", Style::default().fg(Color::Yellow)),
        Span::raw(": forçar saída"),
    ]));
    f.render_widget(footer, chunks[3]);
}
