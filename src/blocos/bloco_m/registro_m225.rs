use crate::{EFDError, EFDResult, SpedParser, StringParser, ToNaiveDate, impl_sped_record_trait};
use chrono::NaiveDate;
use std::{path::Path, sync::Arc};

const REGISTRO: &str = "M225";

#[derive(Debug, Clone)]
pub struct RegistroM225 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub det_valor_aj: Option<Arc<str>>, // 2
    pub cst_pis: Option<u16>,           // 3
    pub det_bc_cred: Option<Arc<str>>,  // 4
    pub det_aliq: Option<Arc<str>>,     // 5 (Pode ser String ou Decimal)
    pub dt_oper_aj: Option<NaiveDate>,  // 6
    pub desc_aj: Option<Arc<str>>,      // 7
    pub cod_cta: Option<Arc<str>>,      // 8
    pub info_compl: Option<Arc<str>>,   // 9
}

impl_sped_record_trait!(RegistroM225);

impl SpedParser for RegistroM225 {
    type Output = RegistroM225;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro M225 possui 9 campos de dados + 2 delimitadores = 11.
        if len != 11 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.to_string(),
                tamanho_esperado: 11,
                tamanho_encontrado: len,
            });
        }

        // Closure para campos de data (Option<NaiveDate>)
        let get_date = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_optional_date(file_path, line_number, field_name)
        };

        let det_valor_aj = fields.get(2).to_arc();
        let cst_pis = fields.get(3).parse_opt();
        let det_bc_cred = fields.get(4).to_arc();
        let det_aliq = fields.get(5).to_arc();
        let dt_oper_aj = get_date(6, "DT_OPER_AJ")?;
        let desc_aj = fields.get(7).to_arc();
        let cod_cta = fields.get(8).to_arc();
        let info_compl = fields.get(9).to_arc();

        let reg = RegistroM225 {
            nivel: 5,
            bloco: 'M',
            registro: REGISTRO.to_string(),
            line_number,
            det_valor_aj,
            cst_pis,
            det_bc_cred,
            det_aliq,
            dt_oper_aj,
            desc_aj,
            cod_cta,
            info_compl,
        };

        Ok(reg)
    }
}
