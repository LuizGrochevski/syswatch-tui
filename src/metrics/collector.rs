use crate::metrics::android::{ler_memoria, ler_cpus, ler_processos_self};

#[derive(Clone, Debug)]
pub struct CpuMetrics {
    pub usage_global: f32,
    pub usage_por_core: Vec<f32>,
    pub frequencia_mhz: u64,
    pub nome: String,
}

#[derive(Clone, Debug)]
pub struct MemoryMetrics {
    pub total: u64,
    pub usado: u64,
    pub livre: u64,
    pub swap_total: u64,
    pub swap_usado: u64,
}

#[derive(Clone, Debug)]
pub struct ProcessoInfo {
    pub pid: u32,
    pub nome: String,
    pub cpu: f32,
    pub memoria_mb: f64,
    pub status: String,
}

#[derive(Clone, Debug)]
pub struct NetworkMetrics {
    pub interface: String,
    pub bytes_enviados: u64,
    pub bytes_recebidos: u64,
    pub pacotes_enviados: u64,
    pub pacotes_recebidos: u64,
}

#[derive(Clone, Debug)]
pub struct SystemMetrics {
    pub cpu: CpuMetrics,
    pub memoria: MemoryMetrics,
    pub processos: Vec<ProcessoInfo>,
    pub redes: Vec<NetworkMetrics>,
    pub uptime_secs: u64,
    pub hostname: String,
    pub os_nome: String,
}

pub struct MetricsCollector;

impl MetricsCollector {
    pub fn new() -> Self { Self }

    pub fn coletar(&mut self) -> SystemMetrics {
        let cpu_raw = ler_cpus();
        let mem_raw = ler_memoria();
        let procs_raw = ler_processos_self();

        let cpu = CpuMetrics {
            usage_global: cpu_raw.uso_global,
            usage_por_core: cpu_raw.cores.iter().map(|c| c.uso).collect(),
            frequencia_mhz: cpu_raw.cores.first().map(|c| c.freq_mhz).unwrap_or(0),
            nome: format!("{} cores", cpu_raw.cores.len()),
        };

        let memoria = MemoryMetrics {
            total: mem_raw.total_kb * 1024,
            usado: mem_raw.usado_kb() * 1024,
            livre: mem_raw.disponivel_kb * 1024,
            swap_total: mem_raw.swap_total_kb * 1024,
            swap_usado: mem_raw.swap_usado_kb() * 1024,
        };

        let processos = procs_raw.iter().map(|(pid, nome, cpu, mem)| ProcessoInfo {
            pid: *pid,
            nome: nome.clone(),
            cpu: *cpu,
            memoria_mb: *mem,
            status: "".to_string(),
        }).collect();

        let hostname = std::fs::read_to_string("/proc/sys/kernel/hostname")
            .unwrap_or_else(|_| "android".to_string())
            .trim()
            .to_string();

        let uptime_secs = std::fs::read_to_string("/proc/uptime")
            .unwrap_or_default()
            .split_whitespace()
            .next()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0) as u64;

        SystemMetrics {
            cpu,
            memoria,
            processos,
            redes: vec![],
            uptime_secs,
            hostname,
            os_nome: "Android (Termux)".to_string(),
        }
    }
}

pub fn formatar_bytes(bytes: u64) -> String {
    const GB: u64 = 1024 * 1024 * 1024;
    const MB: u64 = 1024 * 1024;
    const KB: u64 = 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

pub fn formatar_uptime(secs: u64) -> String {
    let dias = secs / 86400;
    let horas = (secs % 86400) / 3600;
    let minutos = (secs % 3600) / 60;
    let segundos = secs % 60;
    if dias > 0 {
        format!("{}d {}h {}m {}s", dias, horas, minutos, segundos)
    } else {
        format!("{}h {}m {}s", horas, minutos, segundos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_formatar_bytes_gb() {
        assert_eq!(formatar_bytes(1024 * 1024 * 1024), "1.0 GB");
    }

    #[test]
    fn test_formatar_bytes_mb() {
        assert_eq!(formatar_bytes(1024 * 1024), "1.0 MB");
    }

    #[test]
    fn test_formatar_bytes_kb() {
        assert_eq!(formatar_bytes(1024), "1.0 KB");
    }

    #[test]
    fn test_formatar_bytes_b() {
        assert_eq!(formatar_bytes(512), "512 B");
    }

    #[test]
    fn test_formatar_uptime_dias() {
        assert_eq!(formatar_uptime(90061), "1d 1h 1m 1s");
    }

    #[test]
    fn test_formatar_uptime_horas() {
        assert_eq!(formatar_uptime(3661), "1h 1m 1s");
    }

    #[test]
    fn test_collector_coletar() {
        let mut c = MetricsCollector::new();
        let m = c.coletar();
        assert!(m.memoria.total > 0);
        assert!(!m.cpu.usage_por_core.is_empty());
    }
}
