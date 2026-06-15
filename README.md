# 🖥️ Syswatch-TUI

![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)
![Platform](https://img.shields.io/badge/Platform-Linux%20%7C%20Android%20(Termux)-green?style=for-the-badge)
![Tests](https://img.shields.io/badge/Tests-16%20passing-brightgreen?style=for-the-badge)

Dashboard de monitoramento de sistema em tempo real no 
terminal, construído em **Rust** com **Ratatui**. Funciona nativamente em **Android via Termux** lendo métricas diretamente de `/proc` e `/sys`, sem dependências externas de sistema.

---

## 🚀 Funcionalidades

- 🖥️ **CPU** — uso por core em tempo real com gráfico de 
barras + histórico de 60s
- 🧠 **Memória** — RAM e SWAP com barras de progresso e 
valores exatos
- ⚙️ **Processos** — processos ativos com PID, CPU% e 
memória
- 🌐 **Rede** — interfaces com IP, máscara de rede, MTU e 
flags
- ℹ️ **Sistema** — temperatura, governor, frequências 
disponíveis, uptime
- 🎨 **Cores dinâmicas RGB** — verde/amarelo/vermelho por 
nível de uso
- 📈 **Gráfico de histórico** — linha do tempo com Braille 
markers
- ⌨️ **Navegação por teclado** — Tab, setas, teclas 1-5
- 🔄 **Atualização em tempo real** — refresh a cada 
segundo
- 📱 **Otimizado para Android** — lê `/proc` e `/sys` 
diretamente sem root

---

## 🛠️ Tecnologias

| Tecnologia | Uso |
|---|---|
| Rust | Linguagem principal |
| Ratatui | Renderização TUI |
| Crossterm | Eventos de teclado e terminal |
| /proc + /sys | Métricas nativas Android/Termux |
| ifconfig | Dados de interfaces de rede |

---

## 🏗️ Arquitetura

```
main.rs (event loop + layout)
    │
    ├── metrics/
    │   ├── collector.rs   — abstração de métricas do 
sistema
    │   ├── android.rs     — leitura nativa /proc e /sys
    │   └── network.rs     — parsing de ifconfig para 
interfaces
    │
    └── ui/
        ├── cpu.rs         — gauge global + barchart por 
core + histórico
        ├── memory.rs      — gauges RAM e SWAP
        ├── processes.rs   — tabela de processos
        ├── network.rs     — tabela de interfaces
        └── sysinfo.rs     — painel de informações do 
sistema
```

---

## 📦 Instalação

### Linux
```bash
git clone https://github.com/LuizGrochevski/syswatch-tui
cd syswatch-tui
cargo build --release
./target/release/syswatch-tui
```

### Android (Termux)
```bash
pkg update && pkg install rust git
git clone https://github.com/LuizGrochevski/syswatch-tui
cd syswatch-tui
cargo build --release
./target/release/syswatch-tui
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
                    68.5%
┌ CPU — 8 cores | 1152 MHz | 68.5% ─────┐
│  ████  ████  ████  ████  ████  ████   │
│  C0    C1    C2    C3    C4    C5     │
│  63    63    63    63    79    79     │
├ 📈 Histórico CPU (60s) ────────────────┤
│ 100%                        ▪▪        │
│  50%  ▪▪ ▪ ▪▪ ▪  ▪▪ ▪ ▪▪  ▪▪▪▪ ▪▪  │
│   0%                                  │
└───────────────────────────────────────┘
```

### Memória
```
┌ 🧠 RAM — 4.2 GB / 5.2 GB ────────────┐
│ ████████████████████░░░░░░░░░  79%    │
└───────────────────────────────────────┘
┌ 💾 SWAP — 2.5 GB / 4.0 GB ───────────┐
│ ████████████████░░░░░░░░░░░░░  63%    │
└───────────────────────────────────────┘
```

### Sistema
```
┌ ℹ️  Informações do Sistema ────────────┐
│ Sistema       Android (Termux)         │
│ Hostname      android                  │
│ Uptime        0h 15m 32s              │
│ Temperatura   42.0°C                  │
│ CPU Governor  schedutil               │
│ CPU Cores     8 cores                 │
│ CPU Freq      1152 MHz                │
│ RAM Total     5.2 GB                  │
│ RAM Usado     4.2 GB (79%)            │
│ SWAP Total    4.0 GB                  │
├ 🔧 Detalhes CPU ──────────────────────┤
│ Frequências: 300 - 1804 MHz (9 níveis)│
│ Uso por core: C0:63% C1:63% C2:63%.. │
└───────────────────────────────────────┘
```

---

## 💡 Detalhes técnicos

### Métricas no Android
O Android restringe acesso a `/proc/stat` e 
`/proc/net/dev`. O Syswatch-TUI contorna isso lendo:

| Métrica | Fonte |
|---|---|
| CPU por core | 
| `/sys/devices/system/cpu/cpuN/cpufreq/scaling_cur_freq` |
| CPU max freq | 
| `/sys/devices/system/cpu/cpuN/cpufreq/cpuinfo_max_freq` |
| Temperatura | `/sys/class/thermal/thermal_zoneN/temp` |
| Governor | 
| `/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor` |
| Memória | `/proc/meminfo` |
| Processos | `/proc/PID/stat` + `/proc/PID/statm` |
| Uptime | `/proc/uptime` |
| Hostname | `/proc/sys/kernel/hostname` |
| Interfaces | `ifconfig` |

O uso de CPU por core é calculado como proporção 
`freq_atual / freq_max` — aproximação precisa em processadores com DVFS (Dynamic Voltage and Frequency Scaling).

---

## 🧪 Testes

```bash
cargo test -- --test-threads=1
```

```
metrics::android::tests::test_ler_memoria           ok
metrics::android::tests::test_pct_ram               ok
metrics::android::tests::test_ler_cpus              ok
metrics::android::tests::test_cpu_uso_range         ok
metrics::android::tests::test_mem_usado             ok
metrics::network::tests::test_parsear_ifconfig_basico    
ok
metrics::network::tests::test_parsear_ifconfig_flags     
ok
metrics::network::tests::test_parsear_ifconfig_mtu       
ok
metrics::network::tests::test_ler_interfaces_retorna_algo 
ok
metrics::collector::tests::test_formatar_bytes_gb   ok
metrics::collector::tests::test_formatar_bytes_mb   ok
metrics::collector::tests::test_formatar_bytes_kb   ok
metrics::collector::tests::test_formatar_bytes_b    ok
metrics::collector::tests::test_formatar_uptime_dias ok
metrics::collector::tests::test_formatar_uptime_horas ok
metrics::collector::tests::test_collector_coletar   ok
────────────────────────────────────────────────────
Total: 16 passed
```

---

## 👨‍💻 Autor

**Luiz Felipe Grochevski** — 
[LinkedIn](https://www.linkedin.com/in/luiz-felipe-grochevski) | [GitHub](https://github.com/LuizGrochevski)

