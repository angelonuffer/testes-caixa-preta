use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ResultadoComando {
    #[serde(rename = "saída_padrão")]
    pub saida_padrao: String,
    #[serde(rename = "erro_padrão")]
    pub erro_padrao: String,
    #[serde(rename = "código_saída")]
    pub codigo_saida: i32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ResultadoNavegador {
    pub arquivos: BTreeMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(untagged)]
pub enum ResultadoCenario {
    Comandos(Vec<ResultadoComando>),
    Navegador(ResultadoNavegador),
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Cenario {
    Comandos(CenarioComandos),
    Navegador(CenarioNavegador),
}

#[derive(Deserialize, Debug)]
pub struct CenarioComandos {
    #[serde(rename = "cenário")]
    pub cenario: String,
    pub comandos: Vec<String>,
    pub entrada: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct CenarioNavegador {
    #[serde(rename = "cenário")]
    pub cenario: String,
    pub navegação: Vec<PassoNavegacao>,
}

#[derive(Deserialize, Debug)]
pub struct PassoNavegacao {
    pub endereço: String,
    pub arquivo: String,
    pub formulário: Option<std::collections::HashMap<String, String>>,
    pub esperar_exibição: Option<String>,
    pub esperar_ocultação: Option<String>,
}
