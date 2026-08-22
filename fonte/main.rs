use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
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
struct ResultadoNavegador {
    arquivo_gerado: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(untagged)]
enum ResultadoCenario {
    Comando(ResultadoComando),
    Comandos(Vec<ResultadoComando>),
    Navegador(ResultadoNavegador),
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum Cenario {
    Comando(CenarioComando),
    Comandos(CenarioComandos),
    Navegador(CenarioNavegador),
}

#[derive(Deserialize, Debug)]
struct CenarioComando {
    comando: String,
    entrada: Option<String>,
}

#[derive(Deserialize, Debug)]
struct CenarioComandos {
    comandos: Vec<String>,
    entrada: Option<String>,
}

#[derive(Deserialize, Debug)]
struct CenarioNavegador {
    endereço: String,
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
            match caso {
                Cenario::Comando(caso_comando) => {
                    total += 1;
                    print!("Testando cenário: `{}` ... ", caso_comando.comando);

                    let mut cmd = Command::new("sh");
                    cmd.arg("-c").arg(&caso_comando.comando);

                    if caso_comando.entrada.is_some() {
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

                    if let Some(ref entrada_str) = caso_comando.entrada
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

                    actual_results.push(ResultadoCenario::Comando(res));

                    if let Some(ref esperados) = expected_results {
                        if idx < esperados.len() {
                            if let ResultadoCenario::Comando(ref esperado) = esperados[idx] {
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
                                    "FALHOU (tipo incompatível no snapshot, esperado Comando)"
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
                Cenario::Comandos(cenario_comandos) => {
                    total += 1;
                    print!("Testando cenário ... ");

                    let mut current_input = cenario_comandos.entrada.clone().unwrap_or_default();
                    let mut cenario_falhou = false;
                    let mut cenario_results = Vec::new();

                    for (i, comando) in cenario_comandos.comandos.iter().enumerate() {
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
                                cenario_falhou = true;
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
                                cenario_falhou = true;
                                break;
                            }
                        };

                        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                        let code = output.status.code().unwrap_or(-1);

                        current_input = stdout.clone();

                        cenario_results.push(ResultadoComando {
                            saida_padrao: stdout,
                            erro_padrao: stderr,
                            codigo_saida: code,
                        });
                    }

                    if !cenario_falhou {
                        actual_results.push(ResultadoCenario::Comandos(cenario_results.clone()));

                        if let Some(ref esperados) = expected_results {
                            if idx < esperados.len() {
                                if let ResultadoCenario::Comandos(ref esperado) = esperados[idx] {
                                    let mut fail = false;
                                    if esperado.len() != cenario_results.len() {
                                        println!(
                                            "FALHOU (quantidade de comandos no cenário não corresponde ao snapshot)"
                                        );
                                        fail = true;
                                    } else {
                                        for k in 0..esperado.len() {
                                            if esperado[k].saida_padrao
                                                != cenario_results[k].saida_padrao
                                                || esperado[k].erro_padrao
                                                    != cenario_results[k].erro_padrao
                                                || esperado[k].codigo_saida
                                                    != cenario_results[k].codigo_saida
                                            {
                                                if !fail {
                                                    println!("FALHOU");
                                                    fail = true;
                                                }
                                                println!("  Passo do cenário: {}", k + 1);
                                                if esperado[k].saida_padrao
                                                    != cenario_results[k].saida_padrao
                                                {
                                                    println!(
                                                        "    saida_padrao esperada: {}",
                                                        esperado[k].saida_padrao
                                                    );
                                                    println!(
                                                        "    saida_padrao obtida:   {}",
                                                        cenario_results[k].saida_padrao
                                                    );
                                                }
                                                if esperado[k].erro_padrao
                                                    != cenario_results[k].erro_padrao
                                                {
                                                    println!(
                                                        "    erro_padrao esperado: {}",
                                                        esperado[k].erro_padrao
                                                    );
                                                    println!(
                                                        "    erro_padrao obtido:   {}",
                                                        cenario_results[k].erro_padrao
                                                    );
                                                }
                                                if esperado[k].codigo_saida
                                                    != cenario_results[k].codigo_saida
                                                {
                                                    println!(
                                                        "    codigo_saida esperado: {}",
                                                        esperado[k].codigo_saida
                                                    );
                                                    println!(
                                                        "    codigo_saida obtido:   {}",
                                                        cenario_results[k].codigo_saida
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
                                        "FALHOU (tipo incompatível no snapshot, esperado Comandos)"
                                    );
                                }
                            } else {
                                println!(
                                    "FALHOU (não há saída correspondente no arquivo de snapshot para o cenário)"
                                );
                            }
                        } else {
                            println!("GERADO");
                            passed += 1;
                        }
                    }
                }
                Cenario::Navegador(cenario_navegador) => {
                    total += 1;
                    print!(
                        "Testando cenário de navegador: `{}` ... ",
                        cenario_navegador.endereço
                    );

                    let mut hasher = DefaultHasher::new();
                    cenario_navegador.endereço.hash(&mut hasher);
                    let hash_str = format!("{:x}", hasher.finish());

                    let telas_dir = Path::new("./testes/telas");
                    if !telas_dir.exists() {
                        if let Err(err) = fs::create_dir_all(telas_dir) {
                            println!("FALHOU (erro ao criar diretório telas: {})", err);
                            continue;
                        }
                    }

                    let screenshot_path = telas_dir.join(format!("{}.png", hash_str));

                    let mut cmd = Command::new("chromium-browser");
                    cmd.arg("--headless")
                        .arg("--disable-gpu")
                        .arg("--no-sandbox")
                        .arg(format!("--screenshot={}", screenshot_path.to_str().unwrap()))
                        .arg(&cenario_navegador.endereço);

                    let child = match cmd.spawn() {
                        Ok(c) => c,
                        Err(err) => {
                            println!("FALHOU (erro ao iniciar chromium-browser: {})", err);
                            continue;
                        }
                    };

                    let output = match child.wait_with_output() {
                        Ok(o) => o,
                        Err(err) => {
                            println!("FALHOU (erro ao aguardar processo: {})", err);
                            continue;
                        }
                    };

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                        println!("FALHOU (comando falhou: {})", stderr);
                        continue;
                    }

                    let res = ResultadoNavegador {
                        arquivo_gerado: format!("testes/telas/{}.png", hash_str),
                    };

                    actual_results.push(ResultadoCenario::Navegador(res.clone()));

                    if let Some(ref esperados) = expected_results {
                        if idx < esperados.len() {
                            if let ResultadoCenario::Navegador(ref esperado) = esperados[idx] {
                                if esperado.arquivo_gerado != res.arquivo_gerado {
                                    println!("FALHOU");
                                    println!(
                                        "  arquivo_gerado esperado: {}",
                                        esperado.arquivo_gerado
                                    );
                                    println!("  arquivo_gerado obtido:   {}", res.arquivo_gerado);
                                } else if !screenshot_path.exists() {
                                    println!("FALHOU (arquivo de screenshot não foi gerado)");
                                } else {
                                    println!("PASSOU");
                                    passed += 1;
                                }
                            } else {
                                println!(
                                    "FALHOU (tipo incompatível no snapshot, esperado Navegador)"
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
