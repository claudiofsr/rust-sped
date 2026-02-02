use crate::{
    EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToDecimal, impl_reg_methods,
};
use rust_decimal::Decimal;
use std::{path::Path, sync::Arc};

const REGISTRO: &str = "0145";

#[derive(Debug, Clone)]
pub struct Registro0145 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: Arc<str>,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cod_inc_trib: Option<Arc<str>>,      // 2
    pub vl_rec_tot: Option<Decimal>,         // 3
    pub vl_rec_ativ: Option<Decimal>,        // 4
    pub vl_rec_demais_ativ: Option<Decimal>, // 5
    pub info_compl: Option<Arc<str>>,        // 6
}

impl_reg_methods!(Registro0145);

impl SpedParser for Registro0145 {
    type Output = Registro0145;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        if len != 8 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 8,
                tamanho_encontrado: len,
            })
            .loc();
        }

        let get_decimal = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path, line_number, field_name)
        };

        let cod_inc_trib = fields.get(2).to_arc();
        let vl_rec_tot = get_decimal(3, "VL_REC_TOT")?;
        let vl_rec_ativ = get_decimal(4, "VL_REC_ATIV")?;
        let vl_rec_demais_ativ = get_decimal(5, "VL_REC_DEMAIS_ATIV")?;
        let info_compl = fields.get(6).to_arc();

        let reg = Registro0145 {
            nivel: 3,
            bloco: '0',
            registro: REGISTRO.into(),
            line_number,
            cod_inc_trib,
            vl_rec_tot,
            vl_rec_ativ,
            vl_rec_demais_ativ,
            info_compl,
        };

        Ok(reg)
    }
}
