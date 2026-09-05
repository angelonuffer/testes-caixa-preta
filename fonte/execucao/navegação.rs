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
    config: &Option<crate::modelos::Configuracao>,
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

    let mut telas = BTreeMap::new();

    let chrome_bin = if std::path::Path::new("/usr/bin/chromium-browser").exists() {
        "chromium-browser"
    } else if std::path::Path::new("/usr/bin/google-chrome").exists() {
        "/usr/bin/google-chrome"
    } else {
        "chromium-browser"
    };

    let options = headless_chrome::LaunchOptions::default_builder()
        .path(Some(std::path::PathBuf::from(chrome_bin)))
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

    fn aplicar_modo(tab: &headless_chrome::Tab, modo: &str) {
        let value = match modo.to_lowercase().as_str() {
            "escuro" | "dark" => "dark",
            "claro" | "light" => "light",
            _ => "light",
        };
        let _ = tab.call_method(
            headless_chrome::protocol::cdp::Emulation::SetEmulatedMedia {
                media: None,
                features: Some(vec![
                    headless_chrome::protocol::cdp::Emulation::MediaFeature {
                        name: "prefers-color-scheme".to_string(),
                        value: value.to_string(),
                    },
                ]),
            },
        );
    }

    // O padrão deve ser o modo claro
    aplicar_modo(&tab, "claro");

    let cur_dir = std::env::current_dir().unwrap_or_default();

    for passo in &cenario_navegador.navegação {
        if let Some(m) = &passo.modo {
            aplicar_modo(&tab, m);
        }

        if let Some(endereço) = &passo.navegar_para {
            let url = if let Some(cfg) = config {
                if let Some(base) = &cfg.url_base {
                    let trimmed_base = base.trim_end_matches('/');
                    let trimmed_path = endereço.trim_start_matches('/');
                    format!("{}/{}", trimmed_base, trimmed_path)
                } else {
                    let path = cur_dir.join(endereço);
                    format!("file://{}", path.display())
                }
            } else {
                let path = cur_dir.join(endereço);
                format!("file://{}", path.display())
            };

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
        }

        if let Some(form) = &passo.enviar_formulario {
            for (id, val) in form {
                let selector = format!("#{}", id);
                if let Err(e) = tab.evaluate(
                    &format!(
                        "let el = document.querySelector('{}'); if (el) {{ el.value = '{}'; el.dispatchEvent(new Event('input')); el.dispatchEvent(new Event('change')); }}",
                        selector, val
                    ),
                    false,
                ) {
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

        if let Some(texto) = &passo.esperar_aparecer {
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

        if let Some(texto) = &passo.esperar_sumir {
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

        if let Some(tela) = &passo.capturar_tela {
            let screenshot_path = telas_dir.join(tela);
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

            telas.insert(tela.clone(), hash_str);
        }
    }

    let res = ResultadoNavegador { telas };

    actual_results.push(ResultadoCenario::Navegador(res.clone()));

    if let Some(esperados) = expected_results {
        if idx < esperados.len() {
            if let ResultadoCenario::Navegador(ref esperado) = esperados[idx] {
                if esperado.telas != res.telas {
                    println!("\x1b[1;31m❌ FALHOU\x1b[0m");
                    let mut diff_keys = std::collections::BTreeSet::new();

                    for (tela, hash_esperado) in &esperado.telas {
                        if res.telas.get(tela) != Some(hash_esperado) {
                            diff_keys.insert(tela.clone());
                        }
                    }
                    for (tela, hash_obtido) in &res.telas {
                        if esperado.telas.get(tela) != Some(hash_obtido) {
                            diff_keys.insert(tela.clone());
                        }
                    }

                    for tela in diff_keys {
                        println!("    tela: {}", tela);
                        let id_esperado = esperado
                            .telas
                            .get(&tela)
                            .cloned()
                            .unwrap_or_else(|| "Nenhum".to_string());
                        let id_obtido = res
                            .telas
                            .get(&tela)
                            .cloned()
                            .unwrap_or_else(|| "Nenhum".to_string());
                        println!("      id esperado: {}", id_esperado);
                        println!("      id obtido:   {}", id_obtido);
                    }
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
        for (tela, hash) in &res.telas {
            println!("    {}: {}", tela, hash);
        }
        *passed += 1;
    }
}
