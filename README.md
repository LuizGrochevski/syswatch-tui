# 🖥️ Syswatch-TUI

![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)
![Platform](https://img.shields.io/badge/Platform-Linux%20%7C%20Android%20(Termux)-green?style=for-the-badge)
![Tests](https://img.shields.io/badge/Tests-27%20passing-brightgreen?style=for-the-badge)

Dashboard de monitoramento de sistema em tempo real no
terminal, construído em **Rust** com **Ratatui**. Funciona nativamente em **Linux** (lendo `/proc/stat` e `/proc/net/dev`) e em **Android via Termux** (lendo `/sys` e `/proc`), com detecção automática de ambiente em runtime e sem dependências externas obrigatórias.

---

## 🚀 Funcionalidades

- 🖥️ **CPU** — uso real por core em tempo real com gráfico de barras + histórico de 60s
- 🧠 **Memória** — RAM e SWAP com barras de progresso e valores exatos
- ⚙️ **Processos** — processos ativos com PID, CPU% e memória
- 🌐 **Rede** — interfaces com IP, máscara, MTU e flags
- ℹ️ **Sistema** — temperatura, governor, faixa de frequências, uptime
- 🎨 **Cores dinâmicas RGB** — verde/amarelo/vermelho por nível de uso
- 📈 **Gráfico de histórico** — linha do tempo com Braille markers
- ⌨️ **Navegação por teclado** — Tab, setas, teclas 1-5
- 🔄 **Atualização em tempo real** — refresh a cada segundo
- 🐧 **Multiplataforma** — backend dedicado para Linux nativo e Android/Termux, selecionado automaticamente

---

## 🛠️ Tecnologias

| Tecnologia | Uso |
|---|---|
| Rust | Linguagem principal |
| Ratatui | Renderização TUI |
| Crossterm | Eventos de teclado e terminal |
| /proc/stat | Uso real de CPU (Linux) |
| /proc/net/dev | Métricas de rede (Linux) |
| /proc + /sys | Métricas nativas (Android/Termux) |
| ip addr / ifconfig | Dados de interfaces de rede (fallback) |

---

## 🏗️ Arquitetura

```
main.rs (event loop + layout)
    │
    ├── metrics/
    │   ├── collector.rs   — seleciona backend (Linux x Termux) e agrega métricas
    │   ├── linux.rs       — backend Linux: /proc/stat, /proc/net/dev, thermal/hwmon
    │   ├── android.rs     — backend Android/Termux: leitura via /proc e /sys
    │   └── network.rs     — parsing de ifconfig (fallback para Termux)
    │
    └── ui/
        ├── cpu.rs         — gauge global + barchart por core + histórico
        ├── memory.rs      — gauges RAM e SWAP
        ├── processes.rs   — tabela de processos
        ├── network.rs     — tabela de interfaces
        └── sysinfo.rs     — painel de informações do sistema
```

A detecção de ambiente acontece uma única vez, na inicialização do `MetricsCollector`, via `is_termux()`. A partir daí cada coleta (CPU, processos, rede) escolhe o backend correto sem overhead de checagem repetida.

---

## 📦 Instalação

### Linux
```bash
git clone https://github.com/LuizGrochevski/syswatch-tui
cd syswatch-tui
cargo build --release
cargo run --release
```

### Android (Termux)
```bash
pkg update && pkg install rust git
git clone https://github.com/LuizGrochevski/syswatch-tui
cd syswatch-tui
cargo build --release
cargo run --release
```

---

## ⌨️ Controles

| Tecla | Ação |
|---|---|
| `1` | Painel CPU |
| `2` | Painel Memória |
| `3` | Painel Processos |
| `4` | Painel Rede |
| `5` | Painel Sistema |
| `Tab` / `→` | Próximo painel |
| `←` | Painel anterior |
| `q` | Sair |
| `Ctrl+C` | Forçar saída |

---

## 📊 Painéis

### CPU
```
CPU Global ──────────────────────────────
                    6.2%
┌ CPU — 4 cores | 3900 MHz | 6.2% ───────┐
│  ████  ████  ████  ████                │
│  C0    C1    C2    C3                  │
│  5     9     9     5                   │
├ 📈 Histórico CPU (60s) ─────────────────┤
│ 100%                                   │
│  50%        ▪▪▪                        │
│   0%  ▪▪▪▪▪▪▪▪▪▪▪▪▪▪▪▪▪▪▪▪▪▪▪▪▪▪▪▪▪▪   │
└─────────────────────────────────────────┘
```

### Memória
```
┌ 🧠 RAM — 9.0 GB / 15.1 GB ─────────────┐
│ ████████████████████░░░░░░░░░  59%     │
└──────────────────────────────────────────┘
┌ 💾 SWAP — 0 B / 0 B ────────────────────┐
│              sem swap                  │
└──────────────────────────────────────────┘
```

### Sistema
```
┌ ℹ️  Informações do Sistema ─────────────┐
│ Sistema       Linux Mint 22.3           │
│ Hostname      luizfelipe-Positivo...    │
│ Uptime        7h 33m 29s                │
│ Temperatura   38.0°C                    │
│ CPU Governor  powersave                 │
│ CPU Cores     4 cores                   │
│ CPU Freq      3900 MHz                  │
│ RAM Total     15.1 GB                   │
│ RAM Usado     9.1 GB (61%)              │
│ SWAP Total    sem swap                  │
├ 🔧 Detalhes CPU ─────────────────────────┤
│ Frequências disponíveis: 800 - 3900 MHz │
│ Uso por core: C0:0% C1:10% C2:25% C3:0% │
└──────────────────────────────────────────┘
```

---

## 💡 Detalhes técnicos

### Backend Linux

No Linux, o uso de CPU é calculado a partir de **dois snapshots de `/proc/stat`** separados por uma janela de 200ms — exatamente como o `htop` calcula. A fórmula é:

```
uso% = (delta_ocupado / delta_total) × 100
```

onde `ocupado = total - (idle + iowait)`. Isso dá o uso real do processador, em vez de uma estimativa.

A rede é lida direto de `/proc/net/dev` (contadores de rx/tx por interface), sem depender do `ifconfig` — que não vem mais instalado por padrão em distros Linux modernas. Os IPs das interfaces são obtidos via `ip addr show` (iproute2, presente por padrão).

| Métrica | Fonte (Linux) |
|---|---|
| CPU por core | delta de `/proc/stat` |
| Frequência | `/sys/devices/system/cpu/cpuN/cpufreq/scaling_cur_freq` |
| Faixa de frequência | `scaling_available_frequencies` → fallback `cpuinfo_min/max_freq` |
| Temperatura | `/sys/class/thermal/thermal_zoneN/temp` → fallback `hwmon` |
| Rede | `/proc/net/dev` + `ip addr show` |
| Memória | `/proc/meminfo` |
| Processos | `/proc/PID/stat` + `/proc/PID/statm` |

### Backend Android/Termux

O Android restringe acesso direto a `/proc/stat` e `/proc/net/dev` em builds não-root, então o backend Termux usa uma aproximação:

| Métrica | Fonte (Termux) |
|---|---|
| CPU por core | proporção `freq_atual / freq_max` (DVFS) |
| Temperatura | `/sys/class/thermal/thermal_zoneN/temp` |
| Governor | `/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor` |
| Memória | `/proc/meminfo` |
| Processos | `/proc/PID/stat` + `/proc/PID/statm` |
| Interfaces | `ifconfig` |

A detecção automática (`is_termux()`) verifica a variável de ambiente `TERMUX_VERSION`, o prefixo `PREFIX` e a existência de `/data/data/com.termux`, garantindo que o binário escolha o backend certo sem flags manuais.

---

## 🧪 Testes

```bash
cargo test
```

```
test metrics::android::tests::test_ler_memoria              ok
test metrics::android::tests::test_pct_ram                   ok
test metrics::android::tests::test_ler_cpus                  ok
test metrics::android::tests::test_cpu_uso_range              ok
test metrics::android::tests::test_mem_usado                  ok
test metrics::linux::tests::test_ler_stat_retorna_cores        ok
test metrics::linux::tests::test_calcular_uso_range            ok
test metrics::linux::tests::test_calcular_uso_idle             ok
test metrics::linux::tests::test_calcular_uso_100              ok
test metrics::linux::tests::test_ler_net_dev_retorna_algo       ok
test metrics::linux::tests::test_net_dev_tem_loopback           ok
test metrics::linux::tests::test_ler_interfaces_linux_retorna_algo ok
test metrics::linux::tests::test_detectar_os_nao_vazio          ok
test metrics::linux::tests::test_ler_cpus_linux_uso_range        ok
test metrics::network::tests::test_parsear_ifconfig_basico       ok
test metrics::network::tests::test_parsear_ifconfig_flags         ok
test metrics::network::tests::test_parsear_ifconfig_mtu            ok
test metrics::network::tests::test_ler_interfaces_nao_panics        ok
test metrics::collector::tests::test_formatar_bytes_gb               ok
test metrics::collector::tests::test_formatar_bytes_mb                ok
test metrics::collector::tests::test_formatar_bytes_kb                 ok
test metrics::collector::tests::test_formatar_bytes_b                   ok
test metrics::collector::tests::test_formatar_uptime_dias                 ok
test metrics::collector::tests::test_formatar_uptime_horas                 ok
test metrics::collector::tests::test_collector_coletar                      ok
test metrics::collector::tests::test_is_termux_detectavel                    ok
test metrics::collector::tests::test_detectar_os_nao_vazio                    ok
────────────────────────────────────────────────────
Total: 27 passed
```

Testado em **Linux Mint 22.3** (4 cores, 15.1 GB RAM) e originalmente desenvolvido em **Termux** (Android, ARM).

---

## 👨‍💻 Autor

**Luiz Felipe Grochevski** —
[LinkedIn](https://www.linkedin.com/in/luiz-felipe-grochevski) | [GitHub](https://github.com/LuizGrochevski)

