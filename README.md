# 🖥️ Syswatch-TUI

![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)
![Platform](https://img.shields.io/badge/Platform-Linux%20%7C%20Android%20(Termux)-green?style=for-the-badge)
![Tests](https://img.shields.io/badge/Tests-12%20passing-brightgreen?style=for-the-badge)

Dashboard de monitoramento de sistema em tempo real no terminal, construído em **Rust** com **Ratatui**. Funciona nativamente em **Android via Termux** lendo métricas diretamente de `/proc` e `/sys`.

---

## 🚀 Funcionalidades

- 🖥️ **CPU** — uso por core em tempo real com gráfico de barras colorido
- 🧠 **Memória** — RAM e SWAP com barras de progresso e valores exatos
- ⚙️ **Processos** — lista dos processos ativos ordenados por memória
- 🌐 **Rede** — painel de interfaces de rede
- 🎨 **Cores dinâmicas** — verde/amarelo/vermelho por nível de uso (RGB)
- ⌨️ **Navegação por teclado** — Tab, setas, teclas 1-4
- 🔄 **Atualização em tempo real** — refresh a cada segundo
- 📱 **Otimizado para Android** — lê `/proc/meminfo` e `/sys/devices/system/cpu` diretamente

---

## 🛠️ Tecnologias

| Tecnologia | Uso |
|---|---|
| Rust | Linguagem principal |
| Ratatui | Renderização TUI |
| Crossterm | Eventos de teclado e terminal |
| sysinfo | Coleta de métricas (Linux desktop) |
| /proc + /sys | Métricas nativas Android/Termux |

---

## 🏗️ Arquitetura

```
main.rs (event loop)
    │
    ├── metrics/
    │   ├── collector.rs   — abstração de métricas
    │   └── android.rs     — leitura nativa /proc e /sys
    │
    └── ui/
        ├── cpu.rs         — painel CPU (gauge + barchart)
        ├── memory.rs      — painel memória (gauges)
        ├── processes.rs   — painel processos (tabela)
        └── network.rs     — painel rede (tabela)
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
| `Tab` / `→` | Próximo painel |
| `←` | Painel anterior |
| `q` | Sair |
| `Ctrl+C` | Forçar saída |

---

## 📊 Screenshots

**CPU — 8 cores com uso por frequência relativa**
```
CPU Global ──────────────────────────
              61.2%
┌ CPU — 8 cores | 806 MHz | 61.2% ──┐
│  ████  ████  ████  ████            │
│  ████  ████  ████  ████            │
│  ████  ████  ████  ████            │
│  44    44    44    44   79  79  79 │
│  C0    C1    C2    C3   C4  C5  C6 │
└────────────────────────────────────┘
```

**Memória — RAM e SWAP em tempo real**
```
┌ 🧠 RAM — 4.3 GB / 5.2 GB ─────────┐
│ ████████████████████░░░░░░░  81%   │
└────────────────────────────────────┘
┌ 💾 SWAP — 2.6 GB / 4.0 GB ────────┐
│ ████████████████░░░░░░░░░░░  64%   │
└────────────────────────────────────┘
```

---

## 🧪 Testes

```bash
cargo test
```

```
metrics::android::tests::test_ler_memoria      ok
metrics::android::tests::test_pct_ram          ok
metrics::android::tests::test_ler_cpus         ok
metrics::android::tests::test_cpu_uso_range    ok
metrics::android::tests::test_mem_usado        ok
metrics::collector::tests::test_formatar_bytes_gb   ok
metrics::collector::tests::test_formatar_bytes_mb   ok
metrics::collector::tests::test_formatar_bytes_kb   ok
metrics::collector::tests::test_formatar_bytes_b    ok
metrics::collector::tests::test_formatar_uptime_dias ok
metrics::collector::tests::test_formatar_uptime_horas ok
metrics::collector::tests::test_collector_coletar   ok
────────────────────────────────────────────────────
Total: 12 passed
```

---

## 💡 Detalhes técnicos

### Métricas no Android
O Android restringe acesso a `/proc/stat` e `/proc/net/dev`. O Syswatch-TUI contorna isso lendo:
- **CPU**: `/sys/devices/system/cpu/cpuN/cpufreq/scaling_cur_freq` e `cpuinfo_max_freq`
- **Memória**: `/proc/meminfo`
- **Uptime**: `/proc/uptime`
- **Hostname**: `/proc/sys/kernel/hostname`

O uso de CPU por core é calculado como proporção da frequência atual vs máxima — uma aproximação precisa em processadores modernos com DVFS.

---

## 👨‍💻 Autor

**Luiz Felipe Grochevski** — [LinkedIn](https://www.linkedin.com/in/luiz-felipe-grochevski) | [GitHub](https://github.com/LuizGrochevski)

