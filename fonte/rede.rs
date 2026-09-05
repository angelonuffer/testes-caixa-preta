use std::collections::HashSet;
use std::fs;
use std::net::TcpListener;
use std::path::Path;
use std::time::{Duration, Instant};

/// Substitui $PORTA e ${PORTA} por uma porta específica.
pub fn substituir_porta(texto: &str, porta: u16) -> String {
    let s = porta.to_string();
    texto.replace("${PORTA}", &s).replace("$PORTA", &s)
}

/// Aloca uma porta livre temporariamente no sistema ligando em 127.0.0.1:0.
pub fn alocar_porta_livre() -> Result<u16, std::io::Error> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    drop(listener);
    Ok(port)
}

/// Obtém todos os PIDs descendentes de um PID raiz através de /proc.
pub fn obter_arvore_de_processos(pid_raiz: u32) -> HashSet<u32> {
    let mut pids = HashSet::new();
    pids.insert(pid_raiz);

    #[cfg(target_os = "linux")]
    {
        // Tenta ler /proc/<pid>/task/<tid>/children
        let mut fila = vec![pid_raiz];
        while let Some(pid) = fila.pop() {
            let task_dir = format!("/proc/{}/task", pid);
            if let Ok(entries) = fs::read_dir(&task_dir) {
                for entry in entries.flatten() {
                    let children_file = entry.path().join("children");
                    if let Ok(content) = fs::read_to_string(&children_file) {
                        for token in content.split_whitespace() {
                            if let Ok(child_pid) = token.parse::<u32>()
                                && pids.insert(child_pid)
                            {
                                fila.push(child_pid);
                            }
                        }
                    }
                }
            }
        }

        // Também faz varredura geral de PPID para cobrir processos que possam não ter sido
        // registrados em children ou caso task/<tid>/children não esteja habilitado no kernel
        let mut mudou = true;
        while mudou {
            mudou = false;
            if let Ok(entries) = fs::read_dir("/proc") {
                for entry in entries.flatten() {
                    let file_name = entry.file_name();
                    let name_str = file_name.to_string_lossy();
                    if let Ok(pid) = name_str.parse::<u32>() {
                        if pids.contains(&pid) {
                            continue;
                        }
                        if let Ok(stat) = fs::read_to_string(format!("/proc/{}/stat", pid))
                            && let Some(pos) = stat.rfind(')')
                        {
                            let rest = &stat[pos + 1..];
                            let parts: Vec<&str> = rest.split_whitespace().collect();
                            if parts.len() >= 2
                                && let Ok(ppid) = parts[1].parse::<u32>()
                                && pids.contains(&ppid)
                            {
                                pids.insert(pid);
                                mudou = true;
                            }
                        }
                    }
                }
            }
        }
    }

    pids
}

/// Identifica a primeira porta TCP em escuta aberta por qualquer PID do conjunto fornecido.
pub fn detectar_primeira_porta_em_escuta(pids: &HashSet<u32>) -> Option<u16> {
    #[cfg(target_os = "linux")]
    {
        let mut socket_inodes = HashSet::new();

        for &pid in pids {
            let fd_dir = format!("/proc/{}/fd", pid);
            if let Ok(entries) = fs::read_dir(&fd_dir) {
                for entry in entries.flatten() {
                    if let Ok(target) = fs::read_link(entry.path()) {
                        let s = target.to_string_lossy();
                        if let Some(rest) = s.strip_prefix("socket:[")
                            && let Some(inode_str) = rest.strip_suffix(']')
                            && let Ok(inode) = inode_str.parse::<u64>()
                        {
                            socket_inodes.insert(inode);
                        }
                    }
                }
            }
        }

        if socket_inodes.is_empty() {
            return None;
        }

        // Inspeciona /proc/net/tcp e /proc/net/tcp6 procurando sockets em estado 0A (TCP_LISTEN)
        for path_str in &["/proc/net/tcp", "/proc/net/tcp6"] {
            if let Ok(content) = fs::read_to_string(Path::new(path_str)) {
                for line in content.lines().skip(1) {
                    let tokens: Vec<&str> = line.split_whitespace().collect();
                    if tokens.len() >= 10 {
                        let local_address = tokens[1];
                        let state = tokens[3];
                        let inode_str = tokens[9];

                        if state == "0A"
                            && let Ok(inode) = inode_str.parse::<u64>()
                            && socket_inodes.contains(&inode)
                            && let Some(hex_port) = local_address.split(':').nth(1)
                            && let Ok(port) = u16::from_str_radix(hex_port, 16)
                            && port > 0
                        {
                            return Some(port);
                        }
                    }
                }
            }
        }
    }

    None
}

/// Aguarda continuamente até que o processo ou seus descendentes abram a primeira porta TCP em escuta.
pub fn aguardar_primeira_porta(pid_raiz: u32, tempo_limite: Duration) -> Result<u16, String> {
    let inicio = Instant::now();

    while inicio.elapsed() < tempo_limite {
        let pids = obter_arvore_de_processos(pid_raiz);
        if let Some(porta) = detectar_primeira_porta_em_escuta(&pids) {
            return Ok(porta);
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    Err(format!(
        "Tempo limite de {}s excedido aguardando porta do processo {}",
        tempo_limite.as_secs(),
        pid_raiz
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_substituir_porta() {
        assert_eq!(
            substituir_porta("http://localhost:$PORTA", 3000),
            "http://localhost:3000"
        );
        assert_eq!(
            substituir_porta("http://localhost:${PORTA}/teste", 8080),
            "http://localhost:8080/teste"
        );
        assert_eq!(
            substituir_porta("echo $PORTA ${PORTA}", 5000),
            "echo 5000 5000"
        );
    }

    #[test]
    fn test_alocar_porta_livre() {
        let porta = alocar_porta_livre().expect("Deveria alocar porta livre");
        assert!(porta > 0);
    }
}
