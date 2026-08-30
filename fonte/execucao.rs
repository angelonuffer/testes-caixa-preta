mod comandos;
#[path = "execucao/navegação.rs"]
mod navegação;

use crate::modelos::{Cenario, ResultadoCenario};
use comandos::testar_comandos;
use navegação::testar_navegador;

pub fn executar_cenario(
    caso: &Cenario,
    idx: usize,
    expected_results: &Option<Vec<ResultadoCenario>>,
    actual_results: &mut Vec<ResultadoCenario>,
    passed: &mut usize,
    total: &mut usize,
    config: &Option<crate::modelos::Configuracao>,
) {
    match caso {
        Cenario::Comandos(cenario_comandos) => {
            testar_comandos(
                cenario_comandos,
                idx,
                expected_results,
                actual_results,
                passed,
                total,
            );
        }
        Cenario::Navegador(cenario_navegador) => {
            testar_navegador(
                cenario_navegador,
                idx,
                expected_results,
                actual_results,
                passed,
                total,
                config,
            );
        }
    }
}
