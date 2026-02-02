use crate::{
    EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToNaiveDate, impl_reg_methods,
};
use chrono::NaiveDate;
use compact_str::CompactString;
use std::path::Path;

const REGISTRO: &str = "M515";

#[derive(Debug, Clone)]
pub struct RegistroM515 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub det_valor_aj: Option<CompactString>, // 2
    pub cst_cofins: Option<u16>,             // 3
    pub det_bc_cred: Option<CompactString>,  // 4
    pub det_aliq: Option<CompactString>,     // 5 (Pode ser String ou Decimal)
    pub dt_oper_aj: Option<NaiveDate>,       // 6
    pub desc_aj: Option<CompactString>,      // 7
    pub cod_cta: Option<CompactString>,      // 8
    pub info_compl: Option<CompactString>,   // 9
}

impl_reg_methods!(RegistroM515);

impl SpedParser for RegistroM515 {
    type Output = RegistroM515;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro M515 possui 9 campos de dados + 2 delimitadores = 11.
        if len != 11 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 11,
                tamanho_encontrado: len,
            })
            .loc();
        }

        // Closure para campos de data (Option<NaiveDate>)
        let get_date = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_optional_date(file_path, line_number, field_name)
        };

        let det_valor_aj = fields.get(2).to_compact_string();
        let cst_cofins = fields.get(3).parse_opt();
        let det_bc_cred = fields.get(4).to_compact_string();
        let det_aliq = fields.get(5).to_compact_string();
        let dt_oper_aj = get_date(6, "DT_OPER_AJ")?;
        let desc_aj = fields.get(7).to_compact_string();
        let cod_cta = fields.get(8).to_compact_string();
        let info_compl = fields.get(9).to_compact_string();

        let reg = RegistroM515 {
            nivel: 4,
            bloco: 'M',
            registro: REGISTRO.into(),
            line_number,
            det_valor_aj,
            cst_cofins,
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
