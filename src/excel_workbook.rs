use indicatif::MultiProgress;
use rust_xlsxwriter::Worksheet;

use crate::{
    AnaliseDosCreditos, ConsolidacaoCST, DocsFiscais, EFDResult, FormatRegistry, SheetType,
    get_worksheets,
};

/// Gera todas as planilhas em paralelo usando Rayon scope.
pub fn get_all_worksheets(
    data_efd: &[DocsFiscais],
    data_cst: &[ConsolidacaoCST],
    data_nat: &[AnaliseDosCreditos],
    registry: &FormatRegistry,
    multiprogressbar: &MultiProgress,
) -> EFDResult<Vec<Worksheet>> {
    let mut res_efd = Ok(vec![]);
    let mut res_cst = Ok(vec![]);
    let mut res_nat = Ok(vec![]);

    rayon::scope(|s| {
        s.spawn(|_| {
            res_efd = get_worksheets(
                data_efd,
                SheetType::ItensDocsFiscais,
                registry,
                multiprogressbar,
                0,
            )
        });
        s.spawn(|_| {
            res_cst = get_worksheets(
                data_cst,
                SheetType::ConsolidacaoCST,
                registry,
                multiprogressbar,
                1,
            )
        });
        s.spawn(|_| {
            res_nat = get_worksheets(
                data_nat,
                SheetType::AnaliseCreditos,
                registry,
                multiprogressbar,
                2,
            )
        });
    });

    [res_efd, res_cst, res_nat]
        .into_iter()
        .collect::<EFDResult<Vec<Vec<Worksheet>>>>()
        .map(|v| v.into_iter().flatten().collect())
}
