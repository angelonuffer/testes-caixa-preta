mod execucao;
mod modelos;

use execucao::executar_cenario;
use modelos::{Cenario, ResultadoCenario};
use std::fs;
use std::path::Path;

fn main() {
    let testes_dir = Path::new("./testes");
    if !testes_dir.exists() {
        eprintln!("Diretório ./testes não encontrado.");
        std::process::exit(1);
    }

    let mut total = 0;
    let mut passed = 0;

    let entries = match fs::read_dir(testes_dir) {
        Ok(e) => e,
        Err(err) => {
            eprintln!("Falha ao ler o diretório ./testes: {}", err);
            std::process::exit(1);
        }
    };

    let mut files: Vec<_> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.is_file()
                && p.extension().and_then(|s| s.to_str()) == Some("yaml")
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
                eprintln!("Falha ao ler o arquivo {}: {}", path.display(), err);
                continue;
            }
        };

        let casos: Vec<Cenario> = match serde_yaml::from_str(&content) {
            Ok(c) => c,
            Err(err) => {
                eprintln!("Erro ao fazer parse do arquivo {}: {}", path.display(), err);
                continue;
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

        if has_saidas {
            let saidas_content = match fs::read_to_string(&saidas_arquivo) {
                Ok(c) => c,
                Err(err) => {
                    eprintln!(
                        "Falha ao ler o arquivo de saídas {}: {}",
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
                        "Erro ao fazer parse do arquivo de saídas {}: {}",
                        saidas_arquivo.display(),
                        err
                    );
                    continue;
                }
            };
        }

        let mut actual_results: Vec<ResultadoCenario> = Vec::new();

        println!("Executando testes do arquivo: {}", path.display());

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

        if !has_saidas {
            let serialized = serde_yaml::to_string(&actual_results).unwrap();
            if let Err(err) = fs::write(&saidas_arquivo, serialized) {
                eprintln!(
                    "Falha ao salvar o arquivo de saídas {}: {}",
                    saidas_arquivo.display(),
                    err
                );
            } else {
                println!(
                    "Arquivo de saídas {} gerado com sucesso.",
                    saidas_arquivo.display()
                );
            }
        }
    }

    println!("\nRelatório: {}/{} testes passaram.", passed, total);
    if passed < total {
        std::process::exit(1);
    }
}
