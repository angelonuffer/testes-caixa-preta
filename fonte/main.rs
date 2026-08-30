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
                    || p.extension().and_then(|s| s.to_str()) == Some("md"))
                && !p
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .ends_with("-saídas.yaml")
        })
        .collect();

    files.sort();

    for path in files {
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

        let casos: Vec<Cenario> = if path.extension().and_then(|s| s.to_str()) == Some("md") {
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

        let is_md = path.extension().and_then(|s| s.to_str()) == Some("md");

        if is_md {
            let mut expected_md = Vec::new();
            for caso in &casos {
                match caso {
                    Cenario::Navegador(cn) => {
                        let mut arquivos = std::collections::BTreeMap::new();
                        for passo in &cn.navegação {
                            if let Some(hash) = &passo.hash_esperado {
                                arquivos.insert(passo.arquivo.clone(), hash.clone());
                            }
                        }
                        if !arquivos.is_empty() {
                            expected_md.push(ResultadoCenario::Navegador(
                                modelos::ResultadoNavegador { arquivos },
                            ));
                        }
                    }
                    _ => {}
                }
            }
            if !expected_md.is_empty() {
                expected_results = Some(expected_md);
            }
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
            );
        }

        if !is_md && !has_saidas {
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
