use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Configuracao {
    pub servidor: Option<String>,
    pub url_base: Option<String>,
    #[serde(default, alias = "tempo_espera_servidor")]
    pub tempo_espera: Option<u64>,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct ResultadoComando {
    #[serde(rename = "saída padrão", alias = "saída_padrão", default)]
    pub saida_padrao: String,
    #[serde(rename = "erro padrão", alias = "erro_padrão", default)]
    pub erro_padrao: String,
    #[serde(rename = "código saída", alias = "código_saída", default)]
    pub codigo_saida: i32,
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
    pub comandos: Vec<PassoComando>,
    pub entrada: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum PassoComando {
    Comando(String),
    Saida(ResultadoComando),
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
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
    #[serde(default, rename = "simular data")]
    pub simular_data: Option<String>,

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
}
