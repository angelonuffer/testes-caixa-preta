mod execucao;
mod markdown;
mod modelos;

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
                let pid = self.0.id().to_string();
                let _ = std::process::Command::new("kill")
                    .arg("-TERM")
                    .arg(&pid)
                    .status();
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
            let _ = self.0.kill();
        }
    }

    let mut _servidor_guard = None;
    if let Some(cfg) = &config
        && let Some(cmd_str) = &cfg.servidor
    {
        println!("\x1b[1;36m🚀 Iniciando servidor: {}\x1b[0m", cmd_str);
        let _ = std::fs::write("testes/servidor.log", "");
        let log_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("testes/servidor.log")
            .expect("Falha ao abrir testes/servidor.log");
        let log_file_err = log_file
            .try_clone()
            .expect("Falha ao clonar descritor de testes/servidor.log");

        if let Ok(child) = std::process::Command::new("sh")
            .arg("-c")
            .arg(format!("exec {}", cmd_str))
            .current_dir(testes_dir)
            .stdout(std::process::Stdio::from(log_file))
            .stderr(std::process::Stdio::from(log_file_err))
            .spawn()
        {
            _servidor_guard = Some(KillOnDrop(child));

            if let Some(url) = &cfg.url_base {
                let host_port = url
                    .trim_start_matches("http://")
                    .trim_start_matches("https://")
                    .split('/')
                    .next()
                    .unwrap_or(url);

                let mut ready = false;
                for _ in 0..50 {
                    // wait up to 5 seconds
                    if std::net::TcpStream::connect(host_port).is_ok() {
                        ready = true;
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                if !ready {
                    println!(
                        "\x1b[1;33m⚠️ Aviso: Servidor não parece estar pronto em {}\x1b[0m",
                        url
                    );
                }
            } else {
                std::thread::sleep(std::time::Duration::from_millis(1500));
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
