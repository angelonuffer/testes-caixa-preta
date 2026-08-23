use crate::modelos::{CenarioNavegador, ResultadoCenario, ResultadoNavegador};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

pub fn testar_navegador(
    cenario_navegador: &CenarioNavegador,
    idx: usize,
    expected_results: &Option<Vec<ResultadoCenario>>,
    actual_results: &mut Vec<ResultadoCenario>,
    passed: &mut usize,
    total: &mut usize,
) {
    *total += 1;
    print!("  \x1b[1m{}\x1b[0m ... ", cenario_navegador.cenario);
    let _ = std::io::stdout().flush();

    let telas_dir = Path::new("./testes/telas");
    if !telas_dir.exists()
        && let Err(err) = fs::create_dir_all(telas_dir)
    {
        println!(
            "\x1b[1;31m❌ FALHOU\x1b[0m (erro ao criar diretório telas: {})",
            err
        );
        return;
    }

    let mut arquivos_gerados = Vec::new();

    for passo in &cenario_navegador.navegação {
        let mut hasher = DefaultHasher::new();
        passo.endereço.hash(&mut hasher);
        let hash_str = format!("{:x}", hasher.finish());

        let screenshot_path = telas_dir.join(format!("{}.png", hash_str));

        let mut cmd = Command::new("chromium-browser");
        cmd.arg("--headless")
            .arg("--disable-gpu")
            .arg("--no-sandbox")
            .arg(format!(
                "--screenshot={}",
                screenshot_path.to_str().unwrap()
            ))
            .arg(&passo.endereço)
            .stdout(Stdio::null())
            .stderr(Stdio::piped());

        let child = match cmd.spawn() {
            Ok(c) => c,
            Err(err) => {
                println!(
                    "\x1b[1;31m❌ FALHOU\x1b[0m (erro ao iniciar chromium-browser: {})",
                    err
                );
                return;
            }
        };

        let output = match child.wait_with_output() {
            Ok(o) => o,
            Err(err) => {
                println!(
                    "\x1b[1;31m❌ FALHOU\x1b[0m (erro ao aguardar processo: {})",
                    err
                );
                return;
            }
        };

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            println!("\x1b[1;31m❌ FALHOU\x1b[0m (comando falhou: {})", stderr);
            return;
        }

        if !screenshot_path.exists() {
            println!("\x1b[1;31m❌ FALHOU\x1b[0m (arquivo de screenshot não foi gerado)");
            return;
        }

        arquivos_gerados.push(format!("testes/telas/{}.png", hash_str));
    }

    let res = ResultadoNavegador { arquivos_gerados };

    actual_results.push(ResultadoCenario::Navegador(res.clone()));

    if let Some(esperados) = expected_results {
        if idx < esperados.len() {
            if let ResultadoCenario::Navegador(ref esperado) = esperados[idx] {
                if esperado.arquivos_gerados != res.arquivos_gerados {
                    println!("\x1b[1;31m❌ FALHOU\x1b[0m");
                    println!(
                        "    arquivos_gerados esperado: {:?}",
                        esperado.arquivos_gerados
                    );
                    println!("    arquivos_gerados obtido:   {:?}", res.arquivos_gerados);
                } else {
                    println!("\x1b[1;32m✅ PASSOU\x1b[0m");
                    *passed += 1;
                }
            } else {
                println!(
                    "\x1b[1;31m❌ FALHOU\x1b[0m (tipo incompatível no snapshot, esperado Navegador)"
                );
            }
        } else {
            println!(
                "\x1b[1;31m❌ FALHOU\x1b[0m (não há saída correspondente no arquivo de snapshot)"
            );
        }
    } else {
        println!("\x1b[1;33m📝 GERADO\x1b[0m");
        *passed += 1;
    }
}
