use crate::{
    CodigoSituacaoTributaria, EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToDecimal,
    ToNaiveDate, impl_reg_methods,
};
use chrono::NaiveDate;
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "M625";

#[derive(Debug, Clone)]
pub struct RegistroM625 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub det_valor_aj: Option<CompactString>,          // 2
    pub cst_cofins: Option<CodigoSituacaoTributaria>, // 3
    pub det_bc_cred: Option<Decimal>,                 // 4 (Assumindo que DET_BC_CRED é um valor)
    pub det_aliq: Option<Decimal>,                    // 5 (Assumindo que DET_ALIQ é uma alíquota)
    pub dt_oper_aj: Option<NaiveDate>,                // 6
    pub desc_aj: Option<CompactString>,               // 7
    pub cod_cta: Option<CompactString>,               // 8
    pub info_compl: Option<CompactString>,            // 9
}

impl_reg_methods!(RegistroM625);

impl SpedParser for RegistroM625 {
    type Output = RegistroM625;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro M625 possui 9 campos de dados + 2 delimitadores = 11.
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

        // --- Closures auxiliares para campos comuns ---

        // Closure para campos decimais (Option<Decimal>)
        let get_decimal = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path, line_number, field_name)
        };

        // Closure para campos de data (Option<NaiveDate>)
        let get_date = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_optional_date(file_path, line_number, field_name)
        };

        let det_valor_aj = fields.get(2).to_compact_string();
        let cst_cofins = fields
            .get(3)
            .to_efd_field(file_path, line_number, "CST_COFINS")?;
        let det_bc_cred = get_decimal(4, "DET_BC_CRED")?;
        let det_aliq = get_decimal(5, "DET_ALIQ")?;
        let dt_oper_aj = get_date(6, "DT_OPER_AJ")?;
        let desc_aj = fields.get(7).to_compact_string();
        let cod_cta = fields.get(8).to_compact_string();
        let info_compl = fields.get(9).to_compact_string();

        let reg = RegistroM625 {
            nivel: 5,
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
