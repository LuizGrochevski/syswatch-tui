use sysinfo::{System, Networks, CpuRefreshKind, RefreshKind};

fn main() {
    let mut sys = System::new_with_specifics(
        RefreshKind::everything().with_cpu(CpuRefreshKind::everything())
    );
    std::thread::sleep(std::time::Duration::from_millis(1000));
    sys.refresh_cpu_all();
    println!("=== CPUs ===");
    for cpu in sys.cpus() {
        println!("  {} | {:.1}% | {} MHz", cpu.name(), cpu.cpu_usage(), cpu.frequency());
    }
    println!("\n=== Redes ===");
    let nets = Networks::new_with_refreshed_list();
    for (name, data) in &nets {
        println!("  {} | RX: {} TX: {}", name, data.total_received(), data.total_transmitted());
    }
}
