use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ResultadoComando {
    pub saida_padrao: String,
    pub erro_padrao: String,
    pub codigo_saida: i32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ResultadoNavegador {
    pub arquivo_gerado: String,
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
    pub comandos: Vec<String>,
    pub entrada: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct CenarioNavegador {
    pub endereço: String,
}
