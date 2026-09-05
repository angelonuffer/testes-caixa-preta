use crate::modelos::{Cenario, CenarioNavegador, PassoNavegacao};
use std::collections::HashMap;

pub fn parse_markdown(content: &str) -> Result<Vec<Cenario>, String> {
    let mut title = String::new();
    let mut steps_navegador = Vec::new();
    let mut in_yaml = false;
    let mut current_yaml = String::new();

    for line in content.lines() {
        if let Some(stripped) = line.strip_prefix("# ") {
            title = stripped.trim().to_string();
        } else if line.starts_with("```yaml") {
            in_yaml = true;
            current_yaml.clear();
        } else if line.starts_with("```") && in_yaml {
            in_yaml = false;
            let parsed: Result<Vec<HashMap<String, serde_yaml::Value>>, _> =
                serde_yaml::from_str(&current_yaml);
            if let Ok(parsed) = parsed {
                let mut passo = PassoNavegacao::default();
                let mut is_navegador = false;

                for map in parsed {
                    if let Some(val) = map.get("navegar para") {
                        passo.navegar_para = Some(val.as_str().unwrap_or("").to_string());
                        is_navegador = true;
                    } else if let Some(val) = map.get("modo") {
                        passo.modo = Some(val.as_str().unwrap_or("").to_string());
                        is_navegador = true;
                    } else if let Some(val) = map.get("enviar formulário") {
                        if let Some(mapping) = val.as_mapping() {
                            let mut form = HashMap::new();
                            for (k, v) in mapping {
                                if let (Some(k_str), Some(v_str)) = (k.as_str(), v.as_str()) {
                                    form.insert(k_str.to_string(), v_str.to_string());
                                }
                            }
                            passo.enviar_formulario = Some(form);
                        }
                    } else if let Some(val) = map.get("esperar aparecer") {
                        passo.esperar_aparecer = Some(val.as_str().unwrap_or("").to_string());
                    } else if let Some(val) = map.get("esperar sumir") {
                        passo.esperar_sumir = Some(val.as_str().unwrap_or("").to_string());
                    }
                }
                if is_navegador {
                    steps_navegador.push(passo);
                }
            } else {
                return Err(format!(
                    "Erro ao ler YAML no markdown: {}",
                    parsed.err().unwrap()
                ));
            }
        } else if in_yaml {
            current_yaml.push_str(line);
            current_yaml.push('\n');
        } else if line.starts_with("![") {
            if let Some(start) = line.find("](")
                && let Some(end) = line[start..].find(')')
            {
                let path = &line[start + 2..start + end];
                let filename = path.split('/').next_back().unwrap_or("").to_string();
                if let Some(last_step) = steps_navegador.last_mut()
                    && last_step.capturar_tela.is_none()
                {
                    last_step.capturar_tela = Some(filename);
                }
            }
        } else if line.trim().starts_with("<!--") && line.trim().ends_with("-->") {
            let trimmed = line.trim();
            let hash_str = trimmed[4..trimmed.len() - 3].trim().to_string();
            if let Some(last_step) = steps_navegador.last_mut()
                && last_step.capturar_tela.is_some()
            {
                last_step.hash_esperado = Some(hash_str);
            }
        }
    }

    if !steps_navegador.is_empty() {
        Ok(vec![Cenario::Navegador(CenarioNavegador {
            cenario: title,
            navegação: steps_navegador,
        })])
    } else {
        Err("Nenhum cenário suportado (navegação) encontrado no arquivo markdown".to_string())
    }
}
