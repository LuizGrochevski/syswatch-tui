/// Backend de métricas para Linux nativo.
///
/// Diferenças em relação ao android.rs:
/// - CPU: delta de /proc/stat entre duas leituras (uso REAL, não proxy de frequência)
/// - Rede: /proc/net/dev (sem dependência de ifconfig)
/// - IP das interfaces: /proc/net/fib_trie (IPv4) — fallback para campo vazio
/// - Temperatura: /sys/class/thermal/thermal_zone*/temp (igual ao Android, funciona no Linux)
/// - Processos: /proc/PID/stat (idêntico ao android.rs, reutilizamos as mesmas structs)

use std::fs;
use std::path::Path;
use std::collections::HashMap;

// ─── CPU ─────────────────────────────────────────────────────────────────────

/// Snapshot cru de contadores do /proc/stat para UM core (ou "cpu" agregado).
#[derive(Clone, Debug, Default)]
pub struct StatSnapshot {
    pub user:    u64,
    pub nice:    u64,
    pub system:  u64,
    pub idle:    u64,
    pub iowait:  u64,
    pub irq:     u64,
    pub softirq: u64,
    pub steal:   u64,
}

impl StatSnapshot {
    pub fn total(&self) -> u64 {
        self.user + self.nice + self.system + self.idle
            + self.iowait + self.irq + self.softirq + self.steal
    }
    pub fn ocupado(&self) -> u64 {
        self.total().saturating_sub(self.idle + self.iowait)
    }
}

/// Lê /proc/stat e devolve (snapshot_global, Vec<snapshot_por_core>).
pub fn ler_stat() -> (StatSnapshot, Vec<StatSnapshot>) {
    let conteudo = match fs::read_to_string("/proc/stat") {
        Ok(c) => c,
        Err(_) => return (StatSnapshot::default(), vec![]),
    };

    let mut global = StatSnapshot::default();
    let mut cores: Vec<StatSnapshot> = Vec::new();

    for linha in conteudo.lines() {
        let partes: Vec<&str> = linha.split_whitespace().collect();
        if partes.is_empty() { continue; }

        let nome = partes[0];

        // "cpu" agregado
        if nome == "cpu" {
            global = parse_stat_linha(&partes);
            continue;
        }

        // "cpu0", "cpu1", ...
        if nome.starts_with("cpu") && nome.len() > 3 {
            if nome[3..].parse::<usize>().is_ok() {
                cores.push(parse_stat_linha(&partes));
            }
        }
    }

    (global, cores)
}

fn parse_stat_linha(partes: &[&str]) -> StatSnapshot {
    let n = |i: usize| -> u64 { partes.get(i).and_then(|s| s.parse().ok()).unwrap_or(0) };
    StatSnapshot {
        user:    n(1),
        nice:    n(2),
        system:  n(3),
        idle:    n(4),
        iowait:  n(5),
        irq:     n(6),
        softirq: n(7),
        steal:   n(8),
    }
}

/// Calcula porcentagem de uso entre dois snapshots (0.0–100.0).
pub fn calcular_uso(antes: &StatSnapshot, depois: &StatSnapshot) -> f32 {
    let delta_total = depois.total().saturating_sub(antes.total()) as f32;
    let delta_ocup  = depois.ocupado().saturating_sub(antes.ocupado()) as f32;
    if delta_total == 0.0 { return 0.0; }
    (delta_ocup / delta_total * 100.0).clamp(0.0, 100.0)
}

// ─── STRUCT CPU (mesma interface que android.rs) ─────────────────────────────

#[derive(Clone, Debug)]
pub struct CpuLinux {
    pub cores: Vec<CoreInfoLinux>,
    pub uso_global: f32,
}

#[derive(Clone, Debug)]
#[allow(dead_code)] // id, max_freq_mhz e online reservados para uso futuro na UI
pub struct CoreInfoLinux {
    pub id: usize,
    pub freq_mhz: u64,
    pub max_freq_mhz: u64,
    pub online: bool,
    pub uso: f32,
}

/// Lê frequência atual de um core via /sys (igual ao Android — funciona no Linux também).
fn freq_mhz(core_id: usize) -> (u64, u64) {
    let base = format!("/sys/devices/system/cpu/cpu{}", core_id);
    let cur = fs::read_to_string(format!("{}/cpufreq/scaling_cur_freq", base))
        .ok().and_then(|s| s.trim().parse::<u64>().ok()).unwrap_or(0) / 1000;
    let max = fs::read_to_string(format!("{}/cpufreq/cpuinfo_max_freq", base))
        .ok().and_then(|s| s.trim().parse::<u64>().ok()).unwrap_or(0) / 1000;
    (cur, max)
}

/// Coleta CPU fazendo DOIS snapshots de /proc/stat separados por `pausa_ms`.
/// Isso dá uso REAL de CPU (não proxy de frequência).
pub fn ler_cpus_linux(pausa_ms: u64) -> CpuLinux {
    let (g1, cores1) = ler_stat();
    std::thread::sleep(std::time::Duration::from_millis(pausa_ms));
    let (g2, cores2) = ler_stat();

    let n_cores = cores1.len().min(cores2.len());
    let mut cores: Vec<CoreInfoLinux> = (0..n_cores).map(|i| {
        let uso = calcular_uso(&cores1[i], &cores2[i]);
        let (freq, max_freq) = freq_mhz(i);

        let online = Path::new(&format!("/sys/devices/system/cpu/cpu{}/online", i))
            .exists()
            .then(|| fs::read_to_string(format!("/sys/devices/system/cpu/cpu{}/online", i))
                .ok()
                .and_then(|s| s.trim().parse::<u8>().ok())
                .map(|v| v == 1)
                .unwrap_or(true))
            .unwrap_or(true); // cpu0 não tem arquivo "online" — sempre está ligado

        CoreInfoLinux { id: i, freq_mhz: freq, max_freq_mhz: max_freq, online, uso }
    }).collect();

    // Se /proc/stat não teve linhas de core (raro), cria pelo menos 1 entry
    if cores.is_empty() {
        let uso_global = calcular_uso(&g1, &g2);
        let (freq, max_freq) = freq_mhz(0);
        cores.push(CoreInfoLinux { id: 0, freq_mhz: freq, max_freq_mhz: max_freq, online: true, uso: uso_global });
    }

    let uso_global = calcular_uso(&g1, &g2);
    CpuLinux { cores, uso_global }
}

// ─── REDE via /proc/net/dev ───────────────────────────────────────────────────

#[derive(Clone, Debug)]
#[allow(dead_code)] // mascara reservado (proc/net não expõe direto; calculável a partir do prefixo CIDR se necessário)
pub struct InterfaceLinux {
    pub nome: String,
    pub ip: Option<String>,
    pub mascara: Option<String>,
    pub flags: Vec<String>,
    pub mtu: Option<u32>,
    // Bytes e pacotes do /proc/net/dev
    pub rx_bytes:   u64,
    pub tx_bytes:   u64,
    pub rx_packets: u64,
    pub tx_packets: u64,
}

/// Lê /proc/net/dev e devolve counters básicos por interface.
/// Formato das colunas:
///   Interface | rx_bytes rx_packets rx_errs rx_drop ... | tx_bytes tx_packets ...
pub fn ler_net_dev() -> HashMap<String, (u64, u64, u64, u64)> {
    let mut mapa: HashMap<String, (u64, u64, u64, u64)> = HashMap::new();

    let conteudo = match fs::read_to_string("/proc/net/dev") {
        Ok(c) => c,
        Err(_) => return mapa,
    };

    // Pula as 2 primeiras linhas (cabeçalho)
    for linha in conteudo.lines().skip(2) {
        let linha = linha.trim();
        if linha.is_empty() { continue; }

        let (nome_parte, dados_parte) = match linha.split_once(':') {
            Some(v) => v,
            None => continue,
        };

        let nome = nome_parte.trim().to_string();
        let cols: Vec<u64> = dados_parte
            .split_whitespace()
            .map(|s| s.parse().unwrap_or(0))
            .collect();

        // Índices: 0=rx_bytes, 1=rx_packets, 8=tx_bytes, 9=tx_packets
        let rx_bytes   = cols.get(0).copied().unwrap_or(0);
        let rx_packets = cols.get(1).copied().unwrap_or(0);
        let tx_bytes   = cols.get(8).copied().unwrap_or(0);
        let tx_packets = cols.get(9).copied().unwrap_or(0);

        mapa.insert(nome, (rx_bytes, tx_bytes, rx_packets, tx_packets));
    }

    mapa
}

/// Lê IPs das interfaces via /proc/net/fib_trie (Linux).
/// Retorna HashMap<interface_name, ip_string>.
pub fn ler_ips_linux() -> HashMap<String, String> {
    let mut ips: HashMap<String, String> = HashMap::new();

    // Abordagem simples: /proc/net/if_inet6 para IPv6, e para IPv4 usamos
    // /sys/class/net/<iface>/address indiretamente — mas o mais confiável
    // sem ifconfig é ler /proc/net/fib_trie.
    // Alternativa ainda mais simples e portável: tentar 'ip addr' com fallback silencioso.

    // Tentamos `ip addr show` (iproute2 — presente em 99% dos Linux modernos)
    if let Ok(output) = std::process::Command::new("ip")
        .args(["addr", "show"])
        .output()
    {
        let texto = String::from_utf8_lossy(&output.stdout);
        let mut iface_atual: Option<String> = None;

        for linha in texto.lines() {
            // Linha de interface: "2: eth0: <FLAGS> ..."
            if !linha.starts_with(' ') && !linha.starts_with('\t') {
                let partes: Vec<&str> = linha.split_whitespace().collect();
                if partes.len() >= 2 {
                    // Remove o índice numérico e os dois pontos
                    let nome = partes[1].trim_end_matches(':').to_string();
                    iface_atual = Some(nome);
                }
            } else if let Some(ref nome) = iface_atual {
                let linha_trim = linha.trim();
                // Linha de IP: "    inet 192.168.1.10/24 ..."
                if linha_trim.starts_with("inet ") && !linha_trim.starts_with("inet6") {
                    let partes: Vec<&str> = linha_trim.split_whitespace().collect();
                    if let Some(cidr) = partes.get(1) {
                        // Remove o prefixo CIDR (/24) para ficar só o IP
                        let ip = cidr.split('/').next().unwrap_or("").to_string();
                        if !ip.is_empty() {
                            ips.entry(nome.clone()).or_insert(ip);
                        }
                    }
                }
            }
        }
    }

    ips
}

/// Lê MTU e flags de /sys/class/net/<iface>/
fn ler_flags_e_mtu(nome: &str) -> (Vec<String>, Option<u32>) {
    let base = format!("/sys/class/net/{}", nome);

    let mtu = fs::read_to_string(format!("{}/mtu", base))
        .ok()
        .and_then(|s| s.trim().parse().ok());

    let mut flags = Vec::new();

    // operstate: up/down
    if let Ok(state) = fs::read_to_string(format!("{}/operstate", base)) {
        if state.trim() == "up" { flags.push("UP".to_string()); }
    }

    // type: 772 = loopback
    if let Ok(tipo) = fs::read_to_string(format!("{}/type", base)) {
        if tipo.trim() == "772" { flags.push("LOOPBACK".to_string()); }
    }

    (flags, mtu)
}

/// Ponto de entrada principal: devolve interfaces com IP, MTU, flags e counters.
pub fn ler_interfaces_linux() -> Vec<InterfaceLinux> {
    let net_dev = ler_net_dev();
    let ips = ler_ips_linux();

    net_dev.into_iter().map(|(nome, (rx_b, tx_b, rx_p, tx_p))| {
        let ip = ips.get(&nome).cloned();
        let (flags, mtu) = ler_flags_e_mtu(&nome);
        InterfaceLinux {
            nome,
            ip,
            mascara: None, // /proc/net não expõe máscara diretamente; aceitável para display
            flags,
            mtu,
            rx_bytes:   rx_b,
            tx_bytes:   tx_b,
            rx_packets: rx_p,
            tx_packets: tx_p,
        }
    })
    .filter(|i| !i.nome.is_empty())
    .collect()
}

// ─── TEMPERATURA ─────────────────────────────────────────────────────────────

/// Lê temperatura do primeiro thermal_zone disponível (funciona igual no Android e Linux).
pub fn ler_temperatura_linux() -> Option<f32> {
    for i in 0..10 {
        let path = format!("/sys/class/thermal/thermal_zone{}/temp", i);
        if let Ok(val) = fs::read_to_string(&path) {
            if let Ok(raw) = val.trim().parse::<i64>() {
                return Some(raw as f32 / 1000.0);
            }
        }
    }
    // Fallback: hwmon (alguns sistemas Linux desktop usam isso)
    for entry in fs::read_dir("/sys/class/hwmon").into_iter().flatten().flatten() {
        let base = entry.path();
        for i in 1..=5 {
            let path = base.join(format!("temp{}_input", i));
            if let Ok(val) = fs::read_to_string(&path) {
                if let Ok(raw) = val.trim().parse::<i64>() {
                    return Some(raw as f32 / 1000.0);
                }
            }
        }
    }
    None
}

// ─── GOVERNOR ────────────────────────────────────────────────────────────────

pub fn ler_governor_linux() -> Option<String> {
    fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor")
        .ok()
        .map(|s| s.trim().to_string())
}

// ─── PROCESSOS (reutiliza lógica do android.rs — /proc é igual) ──────────────

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

pub fn ler_processos_linux() -> Vec<(u32, String, f32, f64)> {
    let mut ticks1: HashMap<u32, u64> = HashMap::new();
    let uptime1 = ler_uptime_ticks();
    let mut procs_info = Vec::new();

    if let Ok(dirs) = fs::read_dir("/proc") {
        for entry in dirs.flatten() {
            let nome = entry.file_name();
            let nome_str = nome.to_string_lossy();
            if let Ok(pid) = nome_str.parse::<u32>() {
                let comm = fs::read_to_string(format!("/proc/{}/comm", pid))
                    .unwrap_or_default().trim().to_string();
                let statm = fs::read_to_string(format!("/proc/{}/statm", pid))
                    .unwrap_or_default();
                let paginas: u64 = statm.split_whitespace()
                    .nth(1).and_then(|s| s.parse().ok()).unwrap_or(0);
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

    std::thread::sleep(std::time::Duration::from_millis(100));
    let uptime2 = ler_uptime_ticks();
    let delta_uptime = uptime2.saturating_sub(uptime1) as f32;

    let mut procs: Vec<(u32, String, f32, f64)> = procs_info.into_iter().map(|(pid, comm, mem)| {
        let cpu = if delta_uptime > 0.0 {
            if let Some(t1) = ticks1.get(&pid) {
                ler_ticks_proc(pid)
                    .map(|t2| (t2.saturating_sub(*t1) as f32 / delta_uptime * 100.0).min(100.0))
                    .unwrap_or(0.0)
            } else { 0.0 }
        } else { 0.0 };
        (pid, comm, cpu, mem)
    }).collect();

    procs.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
    procs.truncate(15);
    procs
}

// ─── DETECÇÃO DE AMBIENTE ─────────────────────────────────────────────────────

/// Retorna true se estiver rodando em Termux/Android.
pub fn is_termux() -> bool {
    std::env::var("TERMUX_VERSION").is_ok()
        || std::env::var("PREFIX").map(|p| p.contains("com.termux")).unwrap_or(false)
        || std::path::Path::new("/data/data/com.termux").exists()
}

/// Nome legível do OS.
pub fn detectar_os() -> String {
    if is_termux() {
        return "Android (Termux)".to_string();
    }
    // Tenta /etc/os-release
    if let Ok(conteudo) = fs::read_to_string("/etc/os-release") {
        for linha in conteudo.lines() {
            if linha.starts_with("PRETTY_NAME=") {
                return linha
                    .trim_start_matches("PRETTY_NAME=")
                    .trim_matches('"')
                    .to_string();
            }
        }
    }
    "Linux".to_string()
}

// ─── TESTES ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ler_stat_retorna_cores() {
        if is_termux() { return; }
        let (global, cores) = ler_stat();
        // /proc/stat deve existir em qualquer Linux/Android
        assert!(global.total() > 0 || cores.len() > 0);
    }

    #[test]
    fn test_calcular_uso_range() {
        // Simula dois snapshots com diferença conhecida
        let antes  = StatSnapshot { user: 100, idle: 900, ..Default::default() };
        let depois = StatSnapshot { user: 150, idle: 950, ..Default::default() }; // +50 ocup, +100 total
        let uso = calcular_uso(&antes, &depois);
        assert!(uso >= 0.0 && uso <= 100.0);
        // delta_ocup=50, delta_total=100 → 50%
        assert!((uso - 50.0).abs() < 0.1, "esperado ~50%, obtido {}", uso);
    }

    #[test]
    fn test_calcular_uso_idle() {
        let antes  = StatSnapshot { idle: 500, ..Default::default() };
        let depois = StatSnapshot { idle: 600, ..Default::default() };
        let uso = calcular_uso(&antes, &depois);
        // Tudo foi idle → uso = 0%
        assert_eq!(uso, 0.0);
    }

    #[test]
    fn test_calcular_uso_100() {
        let antes  = StatSnapshot { user: 0,   idle: 0, ..Default::default() };
        let depois = StatSnapshot { user: 100, idle: 0, ..Default::default() };
        let uso = calcular_uso(&antes, &depois);
        assert!((uso - 100.0).abs() < 0.1, "esperado 100%, obtido {}", uso);
    }

    #[test]
    fn test_ler_net_dev_retorna_algo() {
        // /proc/net/dev existe em qualquer Linux
        if is_termux() { return; }
        let mapa = ler_net_dev();
        assert!(!mapa.is_empty(), "/proc/net/dev deve ter ao menos 'lo'");
    }

    #[test]
    fn test_net_dev_tem_loopback() {
        if is_termux() { return; }
        let mapa = ler_net_dev();
        assert!(mapa.contains_key("lo"), "loopback deve estar presente");
    }

    #[test]
    fn test_ler_interfaces_linux_retorna_algo() {
        if is_termux() { return; }
        let ifaces = ler_interfaces_linux();
        assert!(!ifaces.is_empty());
    }

    #[test]
    fn test_detectar_os_nao_vazio() {
        let os = detectar_os();
        assert!(!os.is_empty());
    }

    #[test]
    fn test_ler_cpus_linux_uso_range() {
        // Pausa de 200ms para ter um delta razoável
        let cpu = ler_cpus_linux(200);
        assert!(!cpu.cores.is_empty());
        assert!(cpu.uso_global >= 0.0 && cpu.uso_global <= 100.0);
        for core in &cpu.cores {
            assert!(core.uso >= 0.0 && core.uso <= 100.0,
                "core {} uso={}", core.id, core.uso);
        }
    }
}

