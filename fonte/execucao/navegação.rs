use crate::modelos::{CenarioNavegador, ResultadoCenario, ResultadoNavegador};
use std::collections::BTreeMap;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::Hasher;
use std::io::Write;
use std::path::Path;

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

    // Limpa o perfil do Chrome para garantir um estado limpo (ex: IndexedDB) a cada execução de cenário
    let profile_dir = Path::new("./testes/chrome-profile");
    if profile_dir.exists() {
        let _ = fs::remove_dir_all(profile_dir);
    }

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

    let mut arquivos = BTreeMap::new();

    let options = headless_chrome::LaunchOptions::default_builder()
        .path(Some(std::path::PathBuf::from("chromium-browser")))
        .port(Some(0))
        .args(vec![
            std::ffi::OsStr::new("--no-sandbox"),
            std::ffi::OsStr::new("--disable-gpu"),
            std::ffi::OsStr::new("--allow-file-access-from-files"),
            std::ffi::OsStr::new("--disable-web-security"),
            std::ffi::OsStr::new("--user-data-dir=./testes/chrome-profile"),
        ])
        .build()
        .unwrap_or_default();

    let browser = match headless_chrome::Browser::new(options) {
        Ok(b) => b,
        Err(e) => {
            println!(
                "\x1b[1;31m❌ FALHOU\x1b[0m (erro ao iniciar navegador: {})",
                e
            );
            return;
        }
    };

    let tab = match browser.new_tab() {
        Ok(t) => t,
        Err(e) => {
            println!("\x1b[1;31m❌ FALHOU\x1b[0m (erro ao abrir aba: {})", e);
            return;
        }
    };

    let cur_dir = std::env::current_dir().unwrap_or_default();

    for passo in &cenario_navegador.navegação {
        let screenshot_path = telas_dir.join(&passo.arquivo);
        let path = cur_dir.join(&passo.endereço);
        let url = format!("file://{}", path.display());

        if let Err(e) = tab.navigate_to(&url) {
            println!("\x1b[1;31m❌ FALHOU\x1b[0m (erro ao navegar: {})", e);
            return;
        }

        if let Err(e) = tab.wait_until_navigated() {
            println!(
                "\x1b[1;31m❌ FALHOU\x1b[0m (erro aguardando carregamento: {})",
                e
            );
            return;
        }

        if let Some(form) = &passo.formulário {
            for (id, val) in form {
                let selector = format!("#{}", id);
                if let Err(e) = tab.evaluate(&format!("let el = document.querySelector('{}'); el.value = '{}'; el.dispatchEvent(new Event('input')); el.dispatchEvent(new Event('change'));", selector, val), false) {
                    println!("Erro ao injetar valor no input {}: {:?}", id, e);
                }
            }
            if tab.find_element("button[type=\"submit\"]").is_ok() {
                let _ = tab.evaluate(
                    "document.querySelector('button[type=\"submit\"]').click()",
                    false,
                );
            }
        }

        if let Some(texto) = &passo.esperar_exibição {
            let mut tentativas = 0;
            let mut sucesso = false;
            let cond = format!(
                "document.body && document.body.innerText.includes(`{}`)",
                texto
            );
            while tentativas < 50 {
                if let Ok(res) = tab.evaluate(&format!("!!({})", cond), false)
                    && let Some(val) = res.value
                    && val.as_bool().unwrap_or(false)
                {
                    sucesso = true;
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
                tentativas += 1;
            }
            if !sucesso {
                println!(
                    "\x1b[1;31m❌ FALHOU\x1b[0m (tempo esgotado aguardando exibição de '{}')",
                    texto
                );
                return;
            }
        }

        if let Some(texto) = &passo.esperar_ocultação {
            let mut tentativas = 0;
            let mut sucesso = false;
            let cond = format!(
                "document.body && !document.body.innerText.includes(`{}`)",
                texto
            );
            while tentativas < 50 {
                if let Ok(res) = tab.evaluate(&format!("!!({})", cond), false)
                    && let Some(val) = res.value
                    && val.as_bool().unwrap_or(false)
                {
                    sucesso = true;
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
                tentativas += 1;
            }
            if !sucesso {
                println!(
                    "\x1b[1;31m❌ FALHOU\x1b[0m (tempo esgotado aguardando ocultação de '{}')",
                    texto
                );
                return;
            }
        }

        let png_data = match tab.capture_screenshot(
            headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption::Png,
            None,
            None,
            true,
        ) {
            Ok(d) => d,
            Err(e) => {
                println!(
                    "\x1b[1;31m❌ FALHOU\x1b[0m (erro ao tirar screenshot: {})",
                    e
                );
                return;
            }
        };

        if let Err(e) = fs::write(&screenshot_path, &png_data) {
            println!(
                "\x1b[1;31m❌ FALHOU\x1b[0m (erro ao salvar screenshot: {})",
                e
            );
            return;
        }

        let mut hasher = DefaultHasher::new();
        hasher.write(&png_data);
        let hash_str = format!("{:x}", hasher.finish());

        arquivos.insert(passo.arquivo.clone(), hash_str);
    }

    let res = ResultadoNavegador { arquivos };

    actual_results.push(ResultadoCenario::Navegador(res.clone()));

    if let Some(esperados) = expected_results {
        if idx < esperados.len() {
            if let ResultadoCenario::Navegador(ref esperado) = esperados[idx] {
                if esperado.arquivos != res.arquivos {
                    println!("\x1b[1;31m❌ FALHOU\x1b[0m");
                    println!("    arquivos esperado: {:?}", esperado.arquivos);
                    println!("    arquivos obtido:   {:?}", res.arquivos);
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
