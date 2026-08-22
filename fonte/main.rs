use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
struct ResultadoComando {
    saida_padrao: String,
    erro_padrao: String,
    codigo_saida: i32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(untagged)]
enum ResultadoTeste {
    Simples(ResultadoComando),
    Tubo(Vec<ResultadoComando>),
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum Teste {
    Simples(CasoDeTeste),
    Tubo { tubo: Vec<PassoTubo> },
}

#[derive(Deserialize, Debug)]
struct CasoDeTeste {
    comando: String,
    entrada: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum PassoTubo {
    Entrada { entrada: String },
    Comando { comando: String },
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

        let casos: Vec<Teste> = match serde_yaml::from_str(&content) {
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
        let mut expected_results: Option<Vec<ResultadoTeste>> = None;

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

        let mut actual_results: Vec<ResultadoTeste> = Vec::new();

        println!("Executando testes do arquivo: {}", path.display());

        for (idx, caso) in casos.iter().enumerate() {
            match caso {
                Teste::Simples(caso_simples) => {
                    total += 1;
                    print!("Testando comando: `{}` ... ", caso_simples.comando);

                    let mut cmd = Command::new("sh");
                    cmd.arg("-c").arg(&caso_simples.comando);

                    if caso_simples.entrada.is_some() {
                        cmd.stdin(Stdio::piped());
                    }
                    cmd.stdout(Stdio::piped());
                    cmd.stderr(Stdio::piped());

                    let mut child = match cmd.spawn() {
                        Ok(c) => c,
                        Err(err) => {
                            println!("FALHOU (erro ao iniciar processo: {})", err);
                            continue;
                        }
                    };

                    if let Some(ref entrada_str) = caso_simples.entrada
                        && let Some(mut stdin) = child.stdin.take()
                    {
                        let _ = stdin.write_all(entrada_str.as_bytes());
                    }

                    let output = match child.wait_with_output() {
                        Ok(o) => o,
                        Err(err) => {
                            println!("FALHOU (erro ao aguardar processo: {})", err);
                            continue;
                        }
                    };

                    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                    let code = output.status.code().unwrap_or(-1);

                    let res = ResultadoComando {
                        saida_padrao: stdout.clone(),
                        erro_padrao: stderr.clone(),
                        codigo_saida: code,
                    };

                    actual_results.push(ResultadoTeste::Simples(res));

                    if let Some(ref esperados) = expected_results {
                        if idx < esperados.len() {
                            if let ResultadoTeste::Simples(ref esperado) = esperados[idx] {
                                let mut fail = false;
                                if esperado.saida_padrao != stdout {
                                    if !fail {
                                        println!("FALHOU");
                                        fail = true;
                                    }
                                    println!("  saida_padrao esperada: {}", esperado.saida_padrao);
                                    println!("  saida_padrao obtida:   {}", stdout);
                                }
                                if esperado.erro_padrao != stderr {
                                    if !fail {
                                        println!("FALHOU");
                                        fail = true;
                                    }
                                    println!("  erro_padrao esperado: {}", esperado.erro_padrao);
                                    println!("  erro_padrao obtido:   {}", stderr);
                                }
                                if esperado.codigo_saida != code {
                                    if !fail {
                                        println!("FALHOU");
                                        fail = true;
                                    }
                                    println!("  codigo_saida esperado: {}", esperado.codigo_saida);
                                    println!("  codigo_saida obtido:   {}", code);
                                }
                                if !fail {
                                    println!("PASSOU");
                                    passed += 1;
                                }
                            } else {
                                println!(
                                    "FALHOU (tipo incompatível no snapshot, esperado Simples)"
                                );
                            }
                        } else {
                            println!("FALHOU (não há saída correspondente no arquivo de snapshot)");
                        }
                    } else {
                        println!("GERADO");
                        passed += 1;
                    }
                }
                Teste::Tubo { tubo } => {
                    total += 1;
                    print!("Testando tubo ... ");

                    let mut current_input = String::new();
                    let mut tubo_falhou = false;
                    let mut tubo_results = Vec::new();

                    for (i, passo) in tubo.iter().enumerate() {
                        match passo {
                            PassoTubo::Entrada { entrada } => {
                                current_input = entrada.clone();
                            }
                            PassoTubo::Comando { comando } => {
                                let mut cmd = Command::new("sh");
                                cmd.arg("-c").arg(comando);

                                cmd.stdin(Stdio::piped());
                                cmd.stdout(Stdio::piped());
                                cmd.stderr(Stdio::piped());

                                let mut child = match cmd.spawn() {
                                    Ok(c) => c,
                                    Err(err) => {
                                        println!(
                                            "FALHOU (passo {} erro ao iniciar processo: {})",
                                            i + 1,
                                            err
                                        );
                                        tubo_falhou = true;
                                        break;
                                    }
                                };

                                if let Some(mut stdin) = child.stdin.take() {
                                    let _ = stdin.write_all(current_input.as_bytes());
                                }

                                let output = match child.wait_with_output() {
                                    Ok(o) => o,
                                    Err(err) => {
                                        println!(
                                            "FALHOU (passo {} erro ao aguardar processo: {})",
                                            i + 1,
                                            err
                                        );
                                        tubo_falhou = true;
                                        break;
                                    }
                                };

                                let stdout =
                                    String::from_utf8_lossy(&output.stdout).trim().to_string();
                                let stderr =
                                    String::from_utf8_lossy(&output.stderr).trim().to_string();
                                let code = output.status.code().unwrap_or(-1);

                                current_input = stdout.clone();

                                tubo_results.push(ResultadoComando {
                                    saida_padrao: stdout,
                                    erro_padrao: stderr,
                                    codigo_saida: code,
                                });
                            }
                        }
                    }

                    if !tubo_falhou {
                        actual_results.push(ResultadoTeste::Tubo(tubo_results.clone()));

                        if let Some(ref esperados) = expected_results {
                            if idx < esperados.len() {
                                if let ResultadoTeste::Tubo(ref esperado) = esperados[idx] {
                                    let mut fail = false;
                                    if esperado.len() != tubo_results.len() {
                                        println!(
                                            "FALHOU (quantidade de comandos no tubo não corresponde ao snapshot)"
                                        );
                                        fail = true;
                                    } else {
                                        for k in 0..esperado.len() {
                                            if esperado[k].saida_padrao
                                                != tubo_results[k].saida_padrao
                                                || esperado[k].erro_padrao
                                                    != tubo_results[k].erro_padrao
                                                || esperado[k].codigo_saida
                                                    != tubo_results[k].codigo_saida
                                            {
                                                if !fail {
                                                    println!("FALHOU");
                                                    fail = true;
                                                }
                                                println!("  Passo do tubo: {}", k + 1);
                                                if esperado[k].saida_padrao
                                                    != tubo_results[k].saida_padrao
                                                {
                                                    println!(
                                                        "    saida_padrao esperada: {}",
                                                        esperado[k].saida_padrao
                                                    );
                                                    println!(
                                                        "    saida_padrao obtida:   {}",
                                                        tubo_results[k].saida_padrao
                                                    );
                                                }
                                                if esperado[k].erro_padrao
                                                    != tubo_results[k].erro_padrao
                                                {
                                                    println!(
                                                        "    erro_padrao esperado: {}",
                                                        esperado[k].erro_padrao
                                                    );
                                                    println!(
                                                        "    erro_padrao obtido:   {}",
                                                        tubo_results[k].erro_padrao
                                                    );
                                                }
                                                if esperado[k].codigo_saida
                                                    != tubo_results[k].codigo_saida
                                                {
                                                    println!(
                                                        "    codigo_saida esperado: {}",
                                                        esperado[k].codigo_saida
                                                    );
                                                    println!(
                                                        "    codigo_saida obtido:   {}",
                                                        tubo_results[k].codigo_saida
                                                    );
                                                }
                                            }
                                        }
                                    }
                                    if !fail {
                                        println!("PASSOU");
                                        passed += 1;
                                    }
                                } else {
                                    println!(
                                        "FALHOU (tipo incompatível no snapshot, esperado Tubo)"
                                    );
                                }
                            } else {
                                println!(
                                    "FALHOU (não há saída correspondente no arquivo de snapshot para o tubo)"
                                );
                            }
                        } else {
                            println!("GERADO");
                            passed += 1;
                        }
                    }
                }
            }
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
