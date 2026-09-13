use crate::modelos::{CenarioComandos, PassoComando, ResultadoComando};
use std::io::Write;
use std::process::{Command, Stdio};

pub fn testar_comandos(cenario_comandos: &CenarioComandos, passed: &mut usize, total: &mut usize) {
    *total += 1;
    print!("  \x1b[1m{}\x1b[0m ... ", cenario_comandos.cenario);
    let _ = std::io::stdout().flush();

    let mut current_input = cenario_comandos.entrada.clone().unwrap_or_default();
    let mut cenario_falhou = false;
    let mut cenario_results = Vec::new();

    let mut expected_results = Vec::new();

    for passo in &cenario_comandos.comandos {
        let PassoComando::Comando(comando) = passo else {
            if let PassoComando::Saida(expected) = passo {
                expected_results.push(expected.clone());
            }
            continue;
        };

        let i = cenario_results.len();
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg(comando);

        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(err) => {
                println!(
                    "\x1b[1;31m❌ FALHOU\x1b[0m (passo {} erro ao iniciar processo: {})",
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
                    "\x1b[1;31m❌ FALHOU\x1b[0m (passo {} erro ao aguardar processo: {})",
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
        let mut fail = expected_results.len() != cenario_results.len();
        if fail {
            println!(
                "\x1b[1;31m❌ FALHOU\x1b[0m (cada comando deve ser seguido por sua saída esperada)"
            );
        } else {
            for k in 0..expected_results.len() {
                let esperado = &expected_results[k];
                let obtido = &cenario_results[k];
                if esperado.saida_padrao.trim() != obtido.saida_padrao
                    || esperado.erro_padrao.trim() != obtido.erro_padrao
                    || esperado.codigo_saida != obtido.codigo_saida
                {
                    if !fail {
                        println!("\x1b[1;31m❌ FALHOU\x1b[0m");
                        fail = true;
                    }
                    println!("    Passo do cenário: {}", k + 1);
                    if esperado.saida_padrao.trim() != obtido.saida_padrao {
                        println!(
                            "      saída padrão esperada:\n{}",
                            format_output(&esperado.saida_padrao)
                        );
                        println!(
                            "      saída padrão obtida:\n{}",
                            format_output(&obtido.saida_padrao)
                        );
                    }
                    if esperado.erro_padrao.trim() != obtido.erro_padrao {
                        println!(
                            "      erro padrão esperado:\n{}",
                            format_output(&esperado.erro_padrao)
                        );
                        println!(
                            "      erro padrão obtido:\n{}",
                            format_output(&obtido.erro_padrao)
                        );
                    }
                    if esperado.codigo_saida != obtido.codigo_saida {
                        println!("      código saída esperado: {}", esperado.codigo_saida);
                        println!("      código saída obtido:   {}", obtido.codigo_saida);
                    }
                }
            }
        }
        if !fail {
            println!("\x1b[1;32m✅ PASSOU\x1b[0m");
            *passed += 1;
        }
    }
}

fn format_output(s: &str) -> String {
    if s.is_empty() {
        "        (vazio)".to_string()
    } else {
        s.lines()
            .map(|l| format!("        | {}", l))
            .collect::<Vec<_>>()
            .join("\n")
    }
}
