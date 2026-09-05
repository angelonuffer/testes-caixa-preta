use crate::modelos::{CenarioNavegador, ModoNavegador, ResultadoCenario, ResultadoNavegador};
use headless_chrome::protocol::cdp::Emulation::{MediaFeature, SetEmulatedMedia};
use headless_chrome::protocol::cdp::Page::AddScriptToEvaluateOnNewDocument;
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

    let chrome_bin = if std::process::Command::new("chromium-browser")
        .arg("--version")
        .output()
        .is_ok()
    {
        std::path::PathBuf::from("chromium-browser")
    } else if std::process::Command::new("google-chrome")
        .arg("--version")
        .output()
        .is_ok()
    {
        std::path::PathBuf::from("google-chrome")
    } else if std::path::Path::new("/opt/google/chrome/chrome").exists() {
        std::path::PathBuf::from("/opt/google/chrome/chrome")
    } else {
        std::path::PathBuf::from("chromium-browser")
    };

    let options = headless_chrome::LaunchOptions::default_builder()
        .path(Some(chrome_bin))
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

    if let Some(simuladores) = &cenario_navegador.simuladores
        && let Some(data_atual) = &simuladores.data_atual
    {
        let mock_script = format!(
            r#"(() => {{
                const TargetDate = {0:?};
                const FixedDate = class extends Date {{
                    constructor(...args) {{
                        if (args.length === 0) {{
                            super(TargetDate);
                        }} else {{
                            super(...args);
                        }}
                    }}
                    static now() {{
                        return new Date(TargetDate).getTime();
                    }}
                }};
                Object.getOwnPropertyNames(Date).forEach((prop) => {{
                    if (!(prop in FixedDate)) {{
                        try {{
                            FixedDate[prop] = Date[prop];
                        }} catch (e) {{}}
                    }}
                }});
                window.Date = FixedDate;
            }})();"#,
            data_atual
        );

        if let Err(e) = tab.call_method(AddScriptToEvaluateOnNewDocument {
            source: mock_script,
            world_name: None,
            include_command_line_api: None,
            run_immediately: Some(true),
        }) {
            eprintln!(
                "\x1b[1;33m⚠️ Aviso ao injetar mock de data atual: {}\x1b[0m",
                e
            );
        }
    }

    let cur_dir = std::env::current_dir().unwrap_or_default();

    for passo in &cenario_navegador.navegação {
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
