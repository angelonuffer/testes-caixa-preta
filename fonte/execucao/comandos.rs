use crate::modelos::{CenarioComandos, ResultadoCenario, ResultadoComando};
use std::io::Write;
use std::process::{Command, Stdio};

pub fn testar_comandos(
    cenario_comandos: &CenarioComandos,
    idx: usize,
    expected_results: &Option<Vec<ResultadoCenario>>,
    actual_results: &mut Vec<ResultadoCenario>,
    passed: &mut usize,
    total: &mut usize,
) {
    *total += 1;
    print!("Testando cenário '{}' ... ", cenario_comandos.cenario);

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
                println!("FALHOU (passo {} erro ao iniciar processo: {})", i + 1, err);
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

        if let Some(esperados) = expected_results {
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
                            if esperado[k].saida_padrao != cenario_results[k].saida_padrao
                                || esperado[k].erro_padrao != cenario_results[k].erro_padrao
                                || esperado[k].codigo_saida != cenario_results[k].codigo_saida
                            {
                                if !fail {
                                    println!("FALHOU");
                                    fail = true;
                                }
                                println!("  Passo do cenário: {}", k + 1);
                                if esperado[k].saida_padrao != cenario_results[k].saida_padrao {
                                    println!(
                                        "    saída_padrão esperada:\n{}",
                                        format_output(&esperado[k].saida_padrao)
                                    );
                                    println!(
                                        "    saída_padrão obtida:\n{}",
                                        format_output(&cenario_results[k].saida_padrao)
                                    );
                                }
                                if esperado[k].erro_padrao != cenario_results[k].erro_padrao {
                                    println!(
                                        "    erro_padrão esperado:\n{}",
                                        format_output(&esperado[k].erro_padrao)
                                    );
                                    println!(
                                        "    erro_padrão obtido:\n{}",
                                        format_output(&cenario_results[k].erro_padrao)
                                    );
                                }
                                if esperado[k].codigo_saida != cenario_results[k].codigo_saida {
                                    println!(
                                        "    código_saída esperado: {}",
                                        esperado[k].codigo_saida
                                    );
                                    println!(
                                        "    código_saída obtido:   {}",
                                        cenario_results[k].codigo_saida
                                    );
                                }
                            }
                        }
                    }
                    if !fail {
                        println!("PASSOU");
                        *passed += 1;
                    }
                } else {
                    println!("FALHOU (tipo incompatível no snapshot, esperado Comandos)");
                }
            } else {
                println!(
                    "FALHOU (não há saída correspondente no arquivo de snapshot para o cenário)"
                );
            }
        } else {
            println!("GERADO");
            *passed += 1;
        }
    }
}

fn format_output(s: &str) -> String {
    if s.is_empty() {
        "      (vazio)".to_string()
    } else {
        s.lines()
            .map(|l| format!("      | {}", l))
            .collect::<Vec<_>>()
            .join("\n")
    }
}
