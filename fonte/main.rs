use serde::Deserialize;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Deserialize, Debug)]
struct CasoDeTeste {
    comando: String,
    #[serde(rename = "saída_esperada")]
    saida_esperada: serde_yaml::Value,
}

fn value_to_string(v: &serde_yaml::Value) -> String {
    match v {
        serde_yaml::Value::String(s) => s.clone(),
        serde_yaml::Value::Number(n) => n.to_string(),
        serde_yaml::Value::Bool(b) => b.to_string(),
        serde_yaml::Value::Null => "".to_string(),
        _ => serde_yaml::to_string(v).unwrap().trim().to_string(),
    }
}

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
        .filter(|p| p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("yaml"))
        .collect();
    
    files.sort(); // Run tests in a deterministic order

    for path in files {
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(err) => {
                eprintln!("Falha ao ler o arquivo {}: {}", path.display(), err);
                continue;
            }
        };

        let casos: Vec<CasoDeTeste> = match serde_yaml::from_str(&content) {
            Ok(c) => c,
            Err(err) => {
                eprintln!("Erro ao fazer parse do arquivo {}: {}", path.display(), err);
                continue;
            }
        };

        println!("Executando testes do arquivo: {}", path.display());

        for caso in casos {
            total += 1;
            print!("Testando comando: `{}` ... ", caso.comando);

            let output = match Command::new("sh").arg("-c").arg(&caso.comando).output() {
                Ok(o) => o,
                Err(err) => {
                    println!("FALHOU (erro de execução: {})", err);
                    continue;
                }
            };

            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let expected = value_to_string(&caso.saida_esperada).trim().to_string();

            if stdout == expected {
                println!("PASSOU");
                passed += 1;
            } else {
                println!("FALHOU");
                println!("  Esperado: {}", expected);
                println!("  Obtido:   {}", stdout);
            }
        }
    }

    println!("\nRelatório: {}/{} testes passaram.", passed, total);
    if passed < total {
        std::process::exit(1);
    }
}
