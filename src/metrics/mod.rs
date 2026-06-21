pub mod android;
pub mod linux;
pub mod network;
pub mod collector;

// Re-exports para conveniência de quem importa via `metrics::Tipo`
// em vez de `metrics::collector::Tipo`. Alguns desses tipos só são
// usados via o caminho completo em src/ui/*, por isso o compilador
// (corretamente) não vê uso direto deste re-export — mas removê-lo
// quebraria a API pública do módulo para quem depende dela.
#[allow(unused_imports)]
pub use collector::{
    MetricsCollector,
    SystemMetrics,
    CpuMetrics,
    MemoryMetrics,
    ProcessoInfo,
    NetworkMetrics,
    formatar_bytes,
    formatar_uptime,
};
