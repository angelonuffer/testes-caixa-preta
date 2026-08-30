use crate::modelos::{Cenario, CenarioNavegador, PassoNavegacao};
use std::collections::HashMap;

pub fn parse_markdown(content: &str) -> Result<Vec<Cenario>, String> {
    let mut title = String::new();
    let mut steps_navegador = Vec::new();
    let mut in_yaml = false;
    let mut current_yaml = String::new();
    
    for line in content.lines() {
        if line.starts_with("# ") {
            title = line[2..].trim().to_string();
        } else if line.starts_with("```yaml") {
            in_yaml = true;
            current_yaml.clear();
        } else if line.starts_with("```") && in_yaml {
            in_yaml = false;
            let parsed: Result<Vec<HashMap<String, serde_yaml::Value>>, _> = serde_yaml::from_str(&current_yaml);
            if let Ok(parsed) = parsed {
                let mut passo = PassoNavegacao {
                    endereço: String::new(),
                    arquivo: String::new(),
                    formulário: None,
                    esperar_exibição: None,
                    esperar_ocultação: None,
                };
                let mut is_navegador = false;
                
                for map in parsed {
                    if let Some(val) = map.get("navegar para") {
                        passo.endereço = val.as_str().unwrap_or("").to_string();
                        is_navegador = true;
                    } else if let Some(val) = map.get("preencher formulário") {
                        if let Some(mapping) = val.as_mapping() {
                            let mut form = HashMap::new();
                            for (k, v) in mapping {
                                if let (Some(k_str), Some(v_str)) = (k.as_str(), v.as_str()) {
                                    form.insert(k_str.to_string(), v_str.to_string());
                                }
                            }
                            passo.formulário = Some(form);
                        }
                    } else if let Some(val) = map.get("esperar aparecer") {
                        passo.esperar_exibição = Some(val.as_str().unwrap_or("").to_string());
                    } else if let Some(val) = map.get("esperar sumir") {
                        passo.esperar_ocultação = Some(val.as_str().unwrap_or("").to_string());
                    }
                }
                if is_navegador {
                    steps_navegador.push(passo);
                }
            } else {
                return Err(format!("Erro ao ler YAML no markdown: {}", parsed.err().unwrap()));
            }
        } else if in_yaml {
            current_yaml.push_str(line);
            current_yaml.push('\n');
        } else if line.starts_with("![") {
            if let Some(start) = line.find("](") {
                if let Some(end) = line[start..].find(")") {
                    let path = &line[start + 2..start + end];
                    let filename = path.split('/').last().unwrap_or("").to_string();
                    if let Some(last_step) = steps_navegador.last_mut() {
                        if last_step.arquivo.is_empty() {
                            last_step.arquivo = filename;
                        }
                    }
                }
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
