use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Deserialize, Debug, Clone)]
pub struct Configuracao {
    pub servidor: Option<String>,
    pub url_base: Option<String>,
    #[serde(default, alias = "tempo_espera_servidor")]
    pub tempo_espera: Option<u64>,
}

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
    pub telas: BTreeMap<String, String>,
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

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ModoNavegador {
    #[default]
    #[serde(rename = "claro")]
    Claro,
    #[serde(rename = "escuro")]
    Escuro,
}

#[derive(Deserialize, Debug)]
pub struct CenarioNavegador {
    #[serde(rename = "cenário")]
    pub cenario: String,
    #[serde(default)]
    pub modo: ModoNavegador,
    pub navegação: Vec<PassoNavegacao>,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct PassoNavegacao {
    #[serde(default, rename = "navegar para")]
    pub navegar_para: Option<String>,

    #[serde(default, rename = "capturar tela")]
    pub capturar_tela: Option<String>,

    #[serde(default, rename = "hash esperado")]
    pub hash_esperado: Option<String>,

    #[serde(default, rename = "enviar formulário")]
    pub enviar_formulario: Option<std::collections::HashMap<String, String>>,

    #[serde(default, rename = "esperar aparecer")]
    pub esperar_aparecer: Option<String>,

    #[serde(default, rename = "esperar sumir")]
    pub esperar_sumir: Option<String>,

    #[serde(default, alias = "clicar", rename = "clicar em")]
    pub clicar_em: Option<String>,

    #[serde(default, rename = "descrição")]
    #[allow(dead_code)]
    pub descricao: Option<String>,
}
