use indicatif::MultiProgress;
use rust_xlsxwriter::{Format, Worksheet};
use std::collections::HashMap;

use crate::{
    AnaliseDosCreditos, ConsolidacaoCST, DocsFiscais, EFDResult, ResultExt, SheetType,
    get_worksheets,
};

/// Generate worksheets using Rayon scope for heterogeneous tasks
pub fn get_all_worksheets(
    data_efd: &[DocsFiscais],
    data_cst: &[ConsolidacaoCST],
    data_nat: &[AnaliseDosCreditos],
    _formats: &HashMap<String, Format>, // Mantido para paridade de interface
    multiprogressbar: &MultiProgress,
) -> EFDResult<Vec<Worksheet>> {
    // Inicializamos os resultados como Erro ou Vazio para garantir a segurança de memória
    let mut res_efd = Ok(vec![]);
    let mut res_cst = Ok(vec![]);
    let mut res_nat = Ok(vec![]);

    // Rayon scope: usa o pool de threads existente (i9-9900K - 16 threads)
    // É a alternativa idiomática para N tarefas diferentes sem aninhar 'join'
    rayon::scope(|s| {
        s.spawn(|_| {
            res_efd = get_worksheets(data_efd, SheetType::ItensDocsFiscais, multiprogressbar, 0)
        });
        s.spawn(|_| {
            res_cst = get_worksheets(data_cst, SheetType::ConsolidacaoCST, multiprogressbar, 1)
        });
        s.spawn(|_| {
            res_nat = get_worksheets(data_nat, SheetType::AnaliseCreditos, multiprogressbar, 2)
        });
    });

    // Processamento funcional dos resultados:
    // 1. Coloca os 3 resultados em um array
    // 2. 'collect' transforma Vec<Result<Vec<T>>> em Result<Vec<Vec<T>>> (Early return se houver erro)
    // 3. 'flatten' achata a estrutura para o retorno final
    [res_efd, res_cst, res_nat]
        .into_iter()
        .collect::<EFDResult<Vec<Vec<Worksheet>>>>()
        .map(|v| v.into_iter().flatten().collect())
        .map_loc(|e| {
            eprintln!("Erro em get_all_worksheets: {e}");
            e
        })
}
