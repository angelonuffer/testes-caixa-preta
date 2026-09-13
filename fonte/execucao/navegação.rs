use crate::modelos::{CenarioNavegador, ModoNavegador};
use headless_chrome::protocol::cdp::Emulation::{MediaFeature, SetEmulatedMedia};
use headless_chrome::protocol::cdp::Page::AddScriptToEvaluateOnNewDocument;
use std::fs;
use std::io::Write;
use std::path::Path;

pub fn testar_navegador(
    cenario_navegador: &CenarioNavegador,
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

    let mut cenario_falhou = false;

    let mut args = vec![
        std::ffi::OsStr::new("--no-sandbox"),
        std::ffi::OsStr::new("--disable-gpu"),
        std::ffi::OsStr::new("--allow-file-access-from-files"),
        std::ffi::OsStr::new("--disable-web-security"),
        std::ffi::OsStr::new("--user-data-dir=./testes/chrome-profile"),
    ];

    if cenario_navegador.modo == ModoNavegador::Escuro {
        args.push(std::ffi::OsStr::new("--force-dark-mode"));
    }

    let options = headless_chrome::LaunchOptions::default_builder()
        .path(Some(std::path::PathBuf::from("chromium-browser")))
        .port(Some(0))
        .args(args)
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

    let esquema_cor = match cenario_navegador.modo {
        ModoNavegador::Claro => "light",
        ModoNavegador::Escuro => "dark",
    };
    let media_feature = MediaFeature {
        name: "prefers-color-scheme".to_string(),
        value: esquema_cor.to_string(),
    };
    if let Err(e) = tab.call_method(SetEmulatedMedia {
        media: None,
        features: Some(vec![media_feature]),
    }) {
        eprintln!(
            "\x1b[1;33m⚠️ Aviso ao definir modo de cor (prefers-color-scheme: {}): {}\x1b[0m",
            esquema_cor, e
        );
    }

    let cur_dir = std::env::current_dir().unwrap_or_default();

    for passo in &cenario_navegador.navegação {
        if let Some(data_simulada) = &passo.simular_data
            && let Err(erro) = aplicar_mock_data(&tab, data_simulada)
        {
            println!(
                "\x1b[1;31m❌ FALHOU\x1b[0m (erro ao simular data '{}': {})",
                data_simulada, erro
            );
            return;
        }

        if let Some(endereço) = &passo.navegar_para {
            let endereço_resolvido = if let Ok(porta) = std::env::var("PORTA") {
                endereço
                    .replace("${PORTA}", &porta)
                    .replace("$PORTA", &porta)
            } else {
                endereço.clone()
            };

            let url = if let Some(cfg) = config {
                if let Some(base) = &cfg.url_base {
                    let trimmed_base = base.trim_end_matches('/');
                    let trimmed_path = endereço_resolvido.trim_start_matches('/');
                    format!("{}/{}", trimmed_base, trimmed_path)
                } else {
                    let path = cur_dir.join(&endereço_resolvido);
                    format!("file://{}", path.display())
                }
            } else {
                let path = cur_dir.join(&endereço_resolvido);
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
                let mut tentativas = 0;
                let mut sucesso = false;
                let script = format!(
                    r#"(() => {{
                        try {{
                            let el = document.querySelector('{0}');
                            if (el) {{
                                el.focus();
                                el.value = '{1}';
                                el.dispatchEvent(new Event('input', {{ bubbles: true }}));
                                el.dispatchEvent(new Event('change', {{ bubbles: true }}));
                                return true;
                            }}
                        }} catch (e) {{}}
                        return false;
                    }})()"#,
                    selector,
                    val.replace('\\', "\\\\").replace('\'', "\\'")
                );
                while tentativas < 50 {
                    if let Ok(res) = tab.evaluate(&script, false)
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
                    println!("Erro ao injetar valor no input {}", id);
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
            let submit_script = r#"(() => {
                try {
                    let btn = document.querySelector('button[type="submit"]');
                    if (btn) {
                        btn.click();
                        return true;
                    }
                } catch (e) {}
                return false;
            })()"#;
            let mut submit_tentativas = 0;
            while submit_tentativas < 20 {
                if let Ok(res) = tab.evaluate(submit_script, false)
                    && let Some(val) = res.value
                    && val.as_bool().unwrap_or(false)
                {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
                submit_tentativas += 1;
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

        if let Some(alvo) = &passo.clicar_em {
            let mut tentativas = 0;
            let mut sucesso = false;
            let script = format!(
                r#"(() => {{
                    try {{
                        let el = document.querySelector('{0}')
                            || document.querySelector('[title="{0}"]')
                            || Array.from(document.querySelectorAll('a, button, [role="button"], input[type="button"], input[type="submit"]')).find(e => (e.innerText && e.innerText.trim() === '{0}') || e.getAttribute('title') === '{0}');
                        if (el) {{
                            el.click();
                            return true;
                        }}
                    }} catch (e) {{}}
                    return false;
                }})()"#,
                alvo.replace('\'', "\\'")
            );
            while tentativas < 50 {
                if let Ok(res) = tab.evaluate(&script, false)
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
                    "\x1b[1;31m❌ FALHOU\x1b[0m (tempo esgotado ao tentar clicar em '{}')",
                    alvo
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

            let hash_str = hash_png(&png_data);

            match &passo.hash_esperado {
                Some(hash_esperado) if hash_esperado == &hash_str => {}
                Some(hash_esperado) => {
                    println!("\x1b[1;31m❌ FALHOU\x1b[0m");
                    println!("    tela: {}", tela);
                    println!("      hash esperado: {}", hash_esperado);
                    println!("      hash obtido:   {}", hash_str);
                    cenario_falhou = true;
                }
                None => {
                    println!(
                        "\x1b[1;31m❌ FALHOU\x1b[0m (a captura '{}' não especifica 'hash esperado')",
                        tela
                    );
                    cenario_falhou = true;
                }
            }
        }
    }

    if !cenario_falhou {
        println!("\x1b[1;32m✅ PASSOU\x1b[0m");
        *passed += 1;
    }
}

fn aplicar_mock_data(
    tab: &headless_chrome::Tab,
    data_simulada: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let data_json = serde_json::to_string(data_simulada)?;
    let script = format!(
        r#"
(() => {{
    const NativeDate = globalThis.__testesCaixaPretaNativeDate || Date;
    globalThis.__testesCaixaPretaNativeDate = NativeDate;

    const dataFixa = new NativeDate({data});
    const tempoFixo = dataFixa.getTime();
    if (Number.isNaN(tempoFixo)) {{
        throw new Error("data inválida");
    }}

    class MockDate extends NativeDate {{
        constructor(...args) {{
            super(...(args.length === 0 ? [tempoFixo] : args));
        }}

        static now() {{
            return tempoFixo;
        }}
    }}

    Object.setPrototypeOf(MockDate, NativeDate);
    Object.defineProperty(MockDate, 'parse', {{ value: NativeDate.parse }});
    Object.defineProperty(MockDate, 'UTC', {{ value: NativeDate.UTC }});
    Object.defineProperty(globalThis, 'Date', {{
        configurable: true,
        writable: true,
        value: MockDate
    }});
}})();
"#,
        data = data_json
    );

    tab.call_method(AddScriptToEvaluateOnNewDocument {
        source: script.clone(),
        world_name: None,
        include_command_line_api: None,
        run_immediately: Some(true),
    })?;
    tab.evaluate(&script, false)?;
    Ok(())
}

fn hash_png(png_data: &[u8]) -> String {
    use sha2::Digest;

    let hash = sha2::Sha256::digest(png_data);
    hash.iter().map(|byte| format!("{byte:02x}")).collect()
}
