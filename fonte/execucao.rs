mod comando;
mod comandos;
mod navegador;

use crate::modelos::{Cenario, ResultadoCenario};
use comando::testar_comando;
use comandos::testar_comandos;
use navegador::testar_navegador;

pub fn executar_cenario(
    caso: &Cenario,
    idx: usize,
    expected_results: &Option<Vec<ResultadoCenario>>,
    actual_results: &mut Vec<ResultadoCenario>,
    passed: &mut usize,
    total: &mut usize,
) {
    match caso {
        Cenario::Comando(caso_comando) => {
            testar_comando(
                caso_comando,
                idx,
                expected_results,
                actual_results,
                passed,
                total,
            );
        }
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
            );
        }
    }
}
