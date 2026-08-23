use crate::modelos::{CenarioComando, ResultadoCenario, ResultadoComando};
use std::io::Write;
use std::process::{Command, Stdio};

pub fn testar_comando(
    caso_comando: &CenarioComando,
    idx: usize,
    expected_results: &Option<Vec<ResultadoCenario>>,
    actual_results: &mut Vec<ResultadoCenario>,
    passed: &mut usize,
    total: &mut usize,
) {
    *total += 1;
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
            return;
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
            return;
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

    if let Some(esperados) = expected_results {
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
                    *passed += 1;
                }
            } else {
                println!("FALHOU (tipo incompatível no snapshot, esperado Comando)");
            }
        } else {
            println!("FALHOU (não há saída correspondente no arquivo de snapshot)");
        }
    } else {
        println!("GERADO");
        *passed += 1;
    }
}
