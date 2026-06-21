use std::fs;
use std::path::Path;

#[derive(Clone, Debug)]
pub struct CpuAndroid {
    pub cores: Vec<CoreInfo>,
    pub uso_global: f32,
}

#[derive(Clone, Debug)]
#[allow(dead_code)] // id e max_freq_mhz reservados para uso futuro na UI
pub struct CoreInfo {
    pub id: usize,
    pub freq_mhz: u64,
    pub max_freq_mhz: u64,
    pub online: bool,
    pub uso: f32,
}

#[derive(Clone, Debug)]
pub struct MemAndroid {
    pub total_kb: u64,
    pub livre_kb: u64,
    pub disponivel_kb: u64,
    pub buffers_kb: u64,
    pub cached_kb: u64,
    pub swap_total_kb: u64,
    pub swap_livre_kb: u64,
}

impl MemAndroid {
    pub fn usado_kb(&self) -> u64 {
        self.total_kb.saturating_sub(self.disponivel_kb)
    }
    pub fn swap_usado_kb(&self) -> u64 {
        self.swap_total_kb.saturating_sub(self.swap_livre_kb)
    }
    #[allow(dead_code)] // API de conveniência, usada em testes e disponível para a UI
    pub fn pct_ram(&self) -> f32 {
        if self.total_kb == 0 { return 0.0; }
        (self.usado_kb() as f32 / self.total_kb as f32) * 100.0
    }
    #[allow(dead_code)] // API de conveniência, usada em testes e disponível para a UI
    pub fn pct_swap(&self) -> f32 {
        if self.swap_total_kb == 0 { return 0.0; }
        (self.swap_usado_kb() as f32 / self.swap_total_kb as f32) * 100.0
    }
}

pub fn ler_memoria() -> MemAndroid {
    let mut mem = MemAndroid {
        total_kb: 0, livre_kb: 0, disponivel_kb: 0,
        buffers_kb: 0, cached_kb: 0,
        swap_total_kb: 0, swap_livre_kb: 0,
    };

    if let Ok(conteudo) = fs::read_to_string("/proc/meminfo") {
        for linha in conteudo.lines() {
            let partes: Vec<&str> = linha.split_whitespace().collect();
            if partes.len() < 2 { continue; }
            let valor: u64 = partes[1].parse().unwrap_or(0);
            match partes[0] {
                "MemTotal:" => mem.total_kb = valor,
                "MemFree:" => mem.livre_kb = valor,
                "MemAvailable:" => mem.disponivel_kb = valor,
                "Buffers:" => mem.buffers_kb = valor,
                "Cached:" => mem.cached_kb = valor,
                "SwapTotal:" => mem.swap_total_kb = valor,
                "SwapFree:" => mem.swap_livre_kb = valor,
                _ => {}
            }
        }
    }
    mem
}

pub fn ler_cpus() -> CpuAndroid {
    let mut cores = Vec::new();
    let mut i = 0;

    loop {
        let base = format!("/sys/devices/system/cpu/cpu{}", i);
        if !Path::new(&base).exists() { break; }

        let online = fs::read_to_string(format!("{}/online", base))
            .unwrap_or_else(|_| "1".to_string())
            .trim()
            .parse::<u8>()
            .unwrap_or(1) == 1;

        let freq_khz = fs::read_to_string(format!("{}/cpufreq/scaling_cur_freq", base))
            .unwrap_or_else(|_| "0".to_string())
            .trim()
            .parse::<u64>()
            .unwrap_or(0);

        let max_freq_khz = fs::read_to_string(format!("{}/cpufreq/cpuinfo_max_freq", base))
            .unwrap_or_else(|_| "0".to_string())
            .trim()
            .parse::<u64>()
            .unwrap_or(0);

        let uso = if max_freq_khz > 0 && online {
            (freq_khz as f32 / max_freq_khz as f32 * 100.0).clamp(0.0, 100.0)
        } else { 0.0 };

        cores.push(CoreInfo {
            id: i,
            freq_mhz: freq_khz / 1000,
            max_freq_mhz: max_freq_khz / 1000,
            online,
            uso,
        });

        i += 1;
    }

    let uso_global = if cores.is_empty() { 0.0 } else {
        let online: Vec<f32> = cores.iter().filter(|c| c.online).map(|c| c.uso).collect();
        if online.is_empty() { 0.0 } else { online.iter().sum::<f32>() / online.len() as f32 }
    };

    CpuAndroid { cores, uso_global }
}

fn ler_ticks_proc(pid: u32) -> Option<u64> {
    let stat = fs::read_to_string(format!("/proc/{}/stat", pid)).ok()?;
    let partes: Vec<&str> = stat.split_whitespace().collect();
    let utime: u64 = partes.get(13)?.parse().ok()?;
    let stime: u64 = partes.get(14)?.parse().ok()?;
    Some(utime + stime)
}

fn ler_uptime_ticks() -> u64 {
    fs::read_to_string("/proc/uptime")
        .unwrap_or_default()
        .split_whitespace()
        .next()
        .and_then(|s| s.parse::<f64>().ok())
        .map(|s| (s * 100.0) as u64)
        .unwrap_or(0)
}

pub fn ler_processos_self() -> Vec<(u32, String, f32, f64)> {
    use std::collections::HashMap;

    // Primeira leitura
    let mut ticks1: HashMap<u32, u64> = HashMap::new();
    let uptime1 = ler_uptime_ticks();

    let mut procs_info = Vec::new();

    if let Ok(dirs) = fs::read_dir("/proc") {
        for entry in dirs.flatten() {
            let nome = entry.file_name();
            let nome_str = nome.to_string_lossy();
            if let Ok(pid) = nome_str.parse::<u32>() {
                let comm = fs::read_to_string(format!("/proc/{}/comm", pid))
                    .unwrap_or_default()
                    .trim()
                    .to_string();
                let statm = fs::read_to_string(format!("/proc/{}/statm", pid))
                    .unwrap_or_default();
                let paginas: u64 = statm.split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                let mem_mb = paginas as f64 * 4096.0 / 1024.0 / 1024.0;
                if !comm.is_empty() {
                    if let Some(ticks) = ler_ticks_proc(pid) {
                        ticks1.insert(pid, ticks);
                    }
                    procs_info.push((pid, comm, mem_mb));
                }
            }
        }
    }

    // Pequena pausa para calcular diferença
    std::thread::sleep(std::time::Duration::from_millis(100));
    let uptime2 = ler_uptime_ticks();
    let delta_uptime = (uptime2.saturating_sub(uptime1)) as f32;

    let mut procs: Vec<(u32, String, f32, f64)> = procs_info.into_iter().map(|(pid, comm, mem)| {
        let cpu = if delta_uptime > 0.0 {
            if let Some(t1) = ticks1.get(&pid) {
                if let Some(t2) = ler_ticks_proc(pid) {
                    let delta = t2.saturating_sub(*t1) as f32;
                    (delta / delta_uptime * 100.0).min(100.0)
                } else { 0.0 }
            } else { 0.0 }
        } else { 0.0 };
        (pid, comm, cpu, mem)
    }).collect();

    procs.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
    procs.truncate(15);
    procs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ler_memoria() {
        let mem = ler_memoria();
        assert!(mem.total_kb > 0);
        assert!(mem.disponivel_kb <= mem.total_kb);
    }

    #[test]
    fn test_pct_ram() {
        let mem = ler_memoria();
        let pct = mem.pct_ram();
        assert!(pct >= 0.0 && pct <= 100.0);
    }

    #[test]
    fn test_ler_cpus() {
        let cpu = ler_cpus();
        assert!(!cpu.cores.is_empty());
    }

    #[test]
    fn test_cpu_uso_range() {
        let cpu = ler_cpus();
        for core in &cpu.cores {
            assert!(core.uso >= 0.0 && core.uso <= 100.0);
        }
    }

    #[test]
    fn test_mem_usado() {
        let mem = ler_memoria();
        assert!(mem.usado_kb() <= mem.total_kb);
    }
}

