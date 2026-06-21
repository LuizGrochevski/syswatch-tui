use std::process::Command;

#[derive(Clone, Debug)]
pub struct InterfaceInfo {
    pub nome: String,
    pub ip: Option<String>,
    pub mascara: Option<String>,
    pub flags: Vec<String>,
    pub mtu: Option<u32>,
}

/// Tenta chamar ifconfig. Se não estiver disponível (Linux moderno sem net-tools),
/// retorna vec vazio silenciosamente — o collector.rs usará /proc/net/dev nesses casos.
pub fn ler_interfaces() -> Vec<InterfaceInfo> {
    let output = match Command::new("ifconfig").output() {
        Ok(o) if o.status.success() => o,
        _ => return vec![],
    };

    let texto = String::from_utf8_lossy(&output.stdout).to_string();
    parsear_ifconfig(&texto)
}

pub fn parsear_ifconfig(texto: &str) -> Vec<InterfaceInfo> {
    let mut interfaces = Vec::new();
    let mut atual: Option<InterfaceInfo> = None;

    for linha in texto.lines() {
        if !linha.starts_with(' ') && !linha.starts_with('\t') && !linha.is_empty() {
            if let Some(iface) = atual.take() {
                interfaces.push(iface);
            }
            let nome = linha.split(':').next().unwrap_or("").trim().to_string();
            let flags: Vec<String> = if let Some(flags_str) = linha.split('<').nth(1) {
                flags_str.split('>').next().unwrap_or("")
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect()
            } else {
                vec![]
            };
            let mtu = linha.split("mtu ").nth(1)
                .and_then(|s| s.split_whitespace().next())
                .and_then(|s| s.parse().ok());

            atual = Some(InterfaceInfo { nome, ip: None, mascara: None, flags, mtu });
        } else if let Some(ref mut iface) = atual {
            let linha_trim = linha.trim();
            if linha_trim.starts_with("inet ") {
                let partes: Vec<&str> = linha_trim.split_whitespace().collect();
                for (i, p) in partes.iter().enumerate() {
                    if *p == "inet"    { iface.ip      = partes.get(i + 1).map(|s| s.to_string()); }
                    if *p == "netmask" { iface.mascara = partes.get(i + 1).map(|s| s.to_string()); }
                }
            }
        }
    }

    if let Some(iface) = atual {
        interfaces.push(iface);
    }

    interfaces.into_iter().filter(|i| !i.nome.is_empty()).collect()
}

#[allow(dead_code)] // API de conveniência, disponível para uso futuro
pub fn obter_ip_local() -> Option<String> {
    ler_interfaces()
        .into_iter()
        .find(|i| !i.flags.contains(&"LOOPBACK".to_string()) && i.ip.is_some())
        .and_then(|i| i.ip)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsear_ifconfig_basico() {
        let texto = "lo: flags=73<UP,LOOPBACK,RUNNING>  mtu 65536\n        inet 127.0.0.1  netmask 255.0.0.0\n\nwlan0: flags=4163<UP,BROADCAST,RUNNING,MULTICAST>  mtu 1500\n        inet 192.168.0.16  netmask 255.255.255.0\n";
        let ifaces = parsear_ifconfig(texto);
        assert_eq!(ifaces.len(), 2);
        assert_eq!(ifaces[0].nome, "lo");
        assert_eq!(ifaces[0].ip, Some("127.0.0.1".to_string()));
        assert_eq!(ifaces[1].nome, "wlan0");
        assert_eq!(ifaces[1].ip, Some("192.168.0.16".to_string()));
    }

    #[test]
    fn test_parsear_ifconfig_flags() {
        let texto = "wlan0: flags=4163<UP,BROADCAST,RUNNING,MULTICAST>  mtu 1500\n        inet 192.168.0.1  netmask 255.255.255.0\n";
        let ifaces = parsear_ifconfig(texto);
        assert!(ifaces[0].flags.contains(&"UP".to_string()));
        assert!(ifaces[0].flags.contains(&"RUNNING".to_string()));
    }

    #[test]
    fn test_parsear_ifconfig_mtu() {
        let texto = "lo: flags=73<UP,LOOPBACK>  mtu 65536\n        inet 127.0.0.1  netmask 255.0.0.0\n";
        let ifaces = parsear_ifconfig(texto);
        assert_eq!(ifaces[0].mtu, Some(65536));
    }

    #[test]
    fn test_ler_interfaces_nao_panics() {
        // Apenas verifica que não entra em panic — ifconfig pode não existir
        let _ = ler_interfaces();
    }
}

