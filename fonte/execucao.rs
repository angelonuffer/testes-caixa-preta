mod comandos;
#[path = "execucao/navegação.rs"]
mod navegação;

use crate::modelos::Cenario;
use comandos::testar_comandos;
use navegação::testar_navegador;

pub fn executar_cenario(
    caso: &Cenario,
    passed: &mut usize,
    total: &mut usize,
    config: &Option<crate::modelos::Configuracao>,
) {
    match caso {
        Cenario::Comandos(cenario_comandos) => {
            testar_comandos(cenario_comandos, passed, total);
        }
        Cenario::Navegador(cenario_navegador) => {
            testar_navegador(cenario_navegador, passed, total, config);
        }
    }
}
