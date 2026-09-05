mod execucao;
mod markdown;
mod modelos;
mod rede;

use execucao::executar_cenario;
use modelos::{Cenario, ResultadoCenario};
use std::fs;
use std::path::Path;

fn main() {
    let testes_dir = Path::new("./testes");
    if !testes_dir.exists() {
        eprintln!("\x1b[1;31m❌ Diretório ./testes não encontrado.\x1b[0m");
        std::process::exit(1);
    }

    let mut config: Option<modelos::Configuracao> = None;
    let config_path = testes_dir.join("caixa-preta.yaml");
    if config_path.exists()
        && let Ok(content) = fs::read_to_string(&config_path)
    {
        config = serde_yaml::from_str(&content).ok();
    }

    struct KillOnDrop(std::process::Child);
    impl Drop for KillOnDrop {
        fn drop(&mut self) {
            #[cfg(unix)]
            {
                let pid = self.0.id();
                let pids = rede::obter_arvore_de_processos(pid);

                let _ = std::process::Command::new("kill")
                    .arg("-TERM")
                    .arg(format!("-{}", pid))
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .status();
                for p in &pids {
                    let _ = std::process::Command::new("kill")
                        .arg("-TERM")
                        .arg(p.to_string())
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .status();
                }

                std::thread::sleep(std::time::Duration::from_millis(300));

                let _ = std::process::Command::new("kill")
                    .arg("-KILL")
                    .arg(format!("-{}", pid))
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .status();
                for p in &pids {
                    let _ = std::process::Command::new("kill")
                        .arg("-KILL")
                        .arg(p.to_string())
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .status();
                }
            }
            let _ = self.0.kill();
        }
    }

    let mut _servidor_guard = None;
    let mut porta_servidor: Option<u16> = None;

    if let Some(cfg) = &mut config
        && let Some(cmd_str) = &cfg.servidor
    {
        let cmd_para_executar = if cmd_str.contains("$PORTA") || cmd_str.contains("${PORTA}") {
            match rede::alocar_porta_livre() {
                Ok(p) => {
                    porta_servidor = Some(p);
                    rede::substituir_porta(cmd_str, p)
                }
                Err(err) => {
                    eprintln!("\x1b[1;31m❌ Falha ao alocar porta livre: {}\x1b[0m", err);
                    cmd_str.clone()
                }
            }
        } else {
            cmd_str.clone()
        };

        println!(
            "\x1b[1;36m🚀 Iniciando servidor: {}\x1b[0m",
            cmd_para_executar
        );
        let _ = std::fs::write("testes/servidor.log", "");
        let log_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("testes/servidor.log")
            .expect("Falha ao abrir testes/servidor.log");
        let log_file_err = log_file
            .try_clone()
            .expect("Falha ao clonar descritor de testes/servidor.log");

        let mut cmd = std::process::Command::new("sh");
        cmd.arg("-c")
            .arg(format!("exec {}", cmd_para_executar))
            .current_dir(testes_dir)
            .stdout(std::process::Stdio::from(log_file))
            .stderr(std::process::Stdio::from(log_file_err));

        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            cmd.process_group(0);
        }

        if let Ok(child) = cmd.spawn() {
            let pid_servidor = child.id();
            _servidor_guard = Some(KillOnDrop(child));

            let tempo_limite_segundos = cfg.tempo_espera.unwrap_or(30);
            let tempo_limite = std::time::Duration::from_secs(tempo_limite_segundos);

            if porta_servidor.is_none() {
                match rede::aguardar_primeira_porta(pid_servidor, tempo_limite) {
                    Ok(p) => {
                        porta_servidor = Some(p);
                    }
                    Err(err) => {
                        eprintln!("\x1b[1;33m⚠️ Aviso na detecção de porta: {}\x1b[0m", err);
                    }
                }
            }

            if let Some(porta) = porta_servidor {
                println!("\x1b[1;32m🔌 Servidor escutando na porta: {}\x1b[0m", porta);
                // SAFETY: Executado de forma síncrona na inicialização do runner antes de threads adicionais
                unsafe {
                    std::env::set_var("PORTA", porta.to_string());
                }

                if let Some(base) = &cfg.url_base {
                    cfg.url_base = Some(rede::substituir_porta(base, porta));
                }
            }

            if let Some(url) = &cfg.url_base {
                // SAFETY: Executado de forma síncrona na inicialização do runner antes de threads adicionais
                unsafe {
                    std::env::set_var("URL_BASE", url);
                }

                let host_port = url
                    .trim_start_matches("http://")
                    .trim_start_matches("https://")
                    .split('/')
                    .next()
                    .unwrap_or(url);

                let addr_str = if host_port.contains(':') {
                    host_port.to_string()
                } else if url.starts_with("https://") {
                    format!("{}:443", host_port)
                } else {
                    format!("{}:80", host_port)
                };

                let total_tentativas = tempo_limite_segundos * 10;
                let mut ready = false;

                for _ in 0..total_tentativas {
                    if let Some(guard) = &mut _servidor_guard
                        && let Ok(Some(status)) = guard.0.try_wait()
                    {
                        eprintln!(
                            "\x1b[1;31m❌ Processo do servidor encerrou prematuramente com status: {}\x1b[0m",
                            status
                        );
                        break;
                    }

                    if std::net::TcpStream::connect(&addr_str).is_ok() {
                        ready = true;
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }

                if !ready {
                    eprintln!(
                        "\x1b[1;33m⚠️ Aviso: Servidor não parece estar pronto em {} após {} segundos.\x1b[0m",
                        url, tempo_limite_segundos
                    );
                    if let Ok(log_content) = fs::read_to_string("testes/servidor.log") {
                        let trimmed = log_content.trim();
                        if !trimmed.is_empty() {
                            eprintln!(
                                "\x1b[1;33mÚltimas linhas de testes/servidor.log:\x1b[0m\n{}",
                                trimmed
                            );
                        }
                    }
                }
            } else {
                let tempo_ms = cfg.tempo_espera.map(|s| s * 1000).unwrap_or(1500);
                std::thread::sleep(std::time::Duration::from_millis(tempo_ms));
            }
        }
    }

    let mut total = 0;
    let mut passed = 0;

    let entries = match fs::read_dir(testes_dir) {
        Ok(e) => e,
        Err(err) => {
            eprintln!(
                "\x1b[1;31m❌ Falha ao ler o diretório ./testes: {}\x1b[0m",
                err
            );
            std::process::exit(1);
        }
    };

    let mut files: Vec<_> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.is_file()
                && (p.extension().and_then(|s| s.to_str()) == Some("yaml")
                    || p.extension().and_then(|s| s.to_str()) == Some("md")
                    || p.extension().and_then(|s| s.to_str()) == Some("nix"))
                && !p
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .ends_with("-saídas.yaml")
                && p.file_name().unwrap() != "caixa-preta.yaml"
        })
        .collect();

    files.sort();

    for path in files {
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        let is_md = ext == "md";
        let is_nix = ext == "nix";

        let casos: Vec<Cenario> = if is_nix {
            let output = match std::process::Command::new("nix")
                .arg("eval")
                .arg("--json")
                .arg("-f")
                .arg(&path)
                .output()
            {
                Ok(o) => o,
                Err(err) => {
                    eprintln!(
                        "\x1b[1;31m❌ Erro ao executar nix eval para {}: {}\x1b[0m",
                        path.display(),
                        err
                    );
                    continue;
                }
            };

            if !output.status.success() {
                eprintln!(
                    "\x1b[1;31m❌ Falha ao avaliar {}: {}\x1b[0m",
                    path.display(),
                    String::from_utf8_lossy(&output.stderr)
                );
                continue;
            }

            match serde_json::from_slice(&output.stdout) {
                Ok(c) => c,
                Err(err) => {
                    eprintln!(
                        "\x1b[1;31m❌ Erro ao fazer parse do JSON do Nix para {}: {}\x1b[0m",
                        path.display(),
                        err
                    );
                    continue;
                }
            }
        } else {
            let content = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(err) => {
                    eprintln!(
                        "\x1b[1;31m❌ Falha ao ler o arquivo {}: {}\x1b[0m",
                        path.display(),
                        err
                    );
                    continue;
                }
            };

            if is_md {
                match markdown::parse_markdown(&content) {
                    Ok(c) => c,
                    Err(err) => {
                        eprintln!(
                            "\x1b[1;31m❌ Erro ao fazer parse do arquivo {}: {}\x1b[0m",
                            path.display(),
                            err
                        );
                        continue;
                    }
                }
            } else {
                match serde_yaml::from_str(&content) {
                    Ok(c) => c,
                    Err(err) => {
                        eprintln!(
                            "\x1b[1;31m❌ Erro ao fazer parse do arquivo {}: {}\x1b[0m",
                            path.display(),
                            err
                        );
                        continue;
                    }
                }
            }
        };

        let mut saidas_arquivo = path.clone();
        let stem = saidas_arquivo
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string();
        saidas_arquivo.set_file_name(format!("{}-saídas.yaml", stem));

        let has_saidas = saidas_arquivo.exists();
        let mut expected_results: Option<Vec<ResultadoCenario>> = None;

        let mut expected_embedded = Vec::new();
        for caso in &casos {
            if let Cenario::Navegador(cn) = caso {
                let mut telas = std::collections::BTreeMap::new();
                for passo in &cn.navegação {
                    if let (Some(tela), Some(hash)) = (&passo.capturar_tela, &passo.hash_esperado) {
                        telas.insert(tela.clone(), hash.clone());
                    }
                }
                if !telas.is_empty() {
                    expected_embedded.push(ResultadoCenario::Navegador(
                        modelos::ResultadoNavegador { telas },
                    ));
                }
            }
        }

        if !expected_embedded.is_empty() {
            expected_results = Some(expected_embedded);
        } else if has_saidas {
            let saidas_content = match fs::read_to_string(&saidas_arquivo) {
                Ok(c) => c,
                Err(err) => {
                    eprintln!(
                        "\x1b[1;31m❌ Falha ao ler o arquivo de saídas {}: {}\x1b[0m",
                        saidas_arquivo.display(),
                        err
                    );
                    continue;
                }
            };
            expected_results = match serde_yaml::from_str(&saidas_content) {
                Ok(c) => Some(c),
                Err(err) => {
                    eprintln!(
                        "\x1b[1;31m❌ Erro ao fazer parse do arquivo de saídas {}: {}\x1b[0m",
                        saidas_arquivo.display(),
                        err
                    );
                    continue;
                }
            };
        }

        let mut actual_results: Vec<ResultadoCenario> = Vec::new();

        println!("\x1b[1;34m📄 {}\x1b[0m", path.display());

        for (idx, caso) in casos.iter().enumerate() {
            executar_cenario(
                caso,
                idx,
                &expected_results,
                &mut actual_results,
                &mut passed,
                &mut total,
                &config,
            );
        }

        if !is_md && !is_nix && !has_saidas {
            let serialized = serde_yaml::to_string(&actual_results).unwrap();
            if let Err(err) = fs::write(&saidas_arquivo, serialized) {
                eprintln!(
                    "\x1b[1;31m❌ Falha ao salvar o arquivo de saídas {}: {}\x1b[0m",
                    saidas_arquivo.display(),
                    err
                );
            } else {
                println!(
                    "\x1b[1;32m💾 Arquivo de saídas {} gerado com sucesso.\x1b[0m",
                    saidas_arquivo.display()
                );
            }
        }
    }

    let cor_relatorio = if passed == total {
        "\x1b[1;32m"
    } else {
        "\x1b[1;31m"
    };
    let emoji_relatorio = if passed == total { "✅" } else { "❌" };
    println!(
        "\n{}{} {}/{} testes passaram.\x1b[0m",
        cor_relatorio, emoji_relatorio, passed, total
    );

    if passed < total {
        std::process::exit(1);
    }
}
