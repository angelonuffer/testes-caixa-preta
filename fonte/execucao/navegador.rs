use crate::modelos::{CenarioNavegador, ResultadoCenario, ResultadoNavegador};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::process::Command;

pub fn testar_navegador(
    cenario_navegador: &CenarioNavegador,
    idx: usize,
    expected_results: &Option<Vec<ResultadoCenario>>,
    actual_results: &mut Vec<ResultadoCenario>,
    passed: &mut usize,
    total: &mut usize,
) {
    *total += 1;
    print!(
        "Testando cenário de navegador: '{}' (`{}`) ... ",
        cenario_navegador.cenario, cenario_navegador.endereço
    );

    let mut hasher = DefaultHasher::new();
    cenario_navegador.endereço.hash(&mut hasher);
    let hash_str = format!("{:x}", hasher.finish());

    let telas_dir = Path::new("./testes/telas");
    if !telas_dir.exists()
        && let Err(err) = fs::create_dir_all(telas_dir)
    {
        println!("FALHOU (erro ao criar diretório telas: {})", err);
        return;
    }

    let screenshot_path = telas_dir.join(format!("{}.png", hash_str));

    let mut cmd = Command::new("chromium-browser");
    cmd.arg("--headless")
        .arg("--disable-gpu")
        .arg("--no-sandbox")
        .arg(format!(
            "--screenshot={}",
            screenshot_path.to_str().unwrap()
        ))
        .arg(&cenario_navegador.endereço);

    let child = match cmd.spawn() {
        Ok(c) => c,
        Err(err) => {
            println!("FALHOU (erro ao iniciar chromium-browser: {})", err);
            return;
        }
    };

    let output = match child.wait_with_output() {
        Ok(o) => o,
        Err(err) => {
            println!("FALHOU (erro ao aguardar processo: {})", err);
            return;
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        println!("FALHOU (comando falhou: {})", stderr);
        return;
    }

    let res = ResultadoNavegador {
        arquivo_gerado: format!("testes/telas/{}.png", hash_str),
    };

    actual_results.push(ResultadoCenario::Navegador(res.clone()));

    if let Some(esperados) = expected_results {
        if idx < esperados.len() {
            if let ResultadoCenario::Navegador(ref esperado) = esperados[idx] {
                if esperado.arquivo_gerado != res.arquivo_gerado {
                    println!("FALHOU");
                    println!("  arquivo_gerado esperado: {}", esperado.arquivo_gerado);
                    println!("  arquivo_gerado obtido:   {}", res.arquivo_gerado);
                } else if !screenshot_path.exists() {
                    println!("FALHOU (arquivo de screenshot não foi gerado)");
                } else {
                    println!("PASSOU");
                    *passed += 1;
                }
            } else {
                println!("FALHOU (tipo incompatível no snapshot, esperado Navegador)");
            }
        } else {
            println!("FALHOU (não há saída correspondente no arquivo de snapshot)");
        }
    } else {
        println!("GERADO");
        *passed += 1;
    }
}
