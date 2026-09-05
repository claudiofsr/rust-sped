use crate::{
    EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToDecimal, ToNaiveDate,
    impl_reg_methods,
};
use chrono::NaiveDate;
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "M215";

#[derive(Debug, Clone)]
pub struct RegistroM215 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub ind_aj_bc: Option<CompactString>,   // 2
    pub vl_aj_bc: Option<Decimal>,          // 3
    pub cod_aj_bc: Option<CompactString>,   // 4
    pub num_doc: Option<usize>,             // 5
    pub descr_aj_bc: Option<CompactString>, // 6
    pub dt_ref: Option<NaiveDate>,          // 7
    pub cod_cta: Option<CompactString>,     // 8
    pub cnpj: Option<CompactString>,        // 9
    pub info_compl: Option<CompactString>,  // 10
}

impl_reg_methods!(RegistroM215);

impl SpedParser for RegistroM215 {
    type Output = RegistroM215;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro M215 possui 10 campos de dados + 2 delimitadores = 12.
        if len != 12 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 12,
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

        // Closure para campos decimais (Option<Decimal>)
        let get_decimal = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path, line_number, field_name)
        };

        let ind_aj_bc = fields.get(2).to_compact_string();
        let vl_aj_bc = get_decimal(3, "VL_AJ_BC")?;
        let cod_aj_bc = fields.get(4).to_compact_string();
        let num_doc = fields.get(5).parse_opt();
        let descr_aj_bc = fields.get(6).to_compact_string();
        let dt_ref = get_date(7, "DT_REF")?;
        let cod_cta = fields.get(8).to_compact_string();
        let cnpj = fields.get(9).to_compact_string();
        let info_compl = fields.get(10).to_compact_string();

        let reg = RegistroM215 {
            nivel: 4,
            bloco: 'M',
            registro: REGISTRO.into(),
            line_number,
            ind_aj_bc,
            vl_aj_bc,
            cod_aj_bc,
            num_doc,
            descr_aj_bc,
            dt_ref,
            cod_cta,
            cnpj,
            info_compl,
        };

        Ok(reg)
    }
}
