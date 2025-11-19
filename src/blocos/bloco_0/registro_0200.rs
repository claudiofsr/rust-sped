use crate::{EFDError, EFDResult, SpedParser, ToDecimal, ToOptionalString, impl_sped_record_trait};
use rust_decimal::Decimal;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Registro0200 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cod_item: Option<String>,     // 2
    pub descr_item: Option<String>,   // 3
    pub cod_barra: Option<String>,    // 4
    pub cod_ant_item: Option<String>, // 5
    pub unid_inv: Option<String>,     // 6
    pub tipo_item: Option<String>,    // 7
    pub cod_ncm: Option<String>,      // 8
    pub ex_ipi: Option<String>,       // 9
    pub cod_gen: Option<String>,      // 10
    pub cod_lst: Option<String>,      // 11
    pub aliq_icms: Option<Decimal>,   // 12
}

impl_sped_record_trait!(Registro0200);

impl SpedParser for Registro0200 {
    type Output = Registro0200;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        if len != 14 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: registro.clone(),
                tamanho_esperado: 14,
                tamanho_encontrado: len,
            });
        }

        let get_decimal_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path.to_path_buf(), line_number, field_name)
        };

        let cod_item = fields.get(2).to_optional_string();
        let descr_item = fields.get(3).to_optional_string();
        let cod_barra = fields.get(4).to_optional_string();
        let cod_ant_item = fields.get(5).to_optional_string();
        let unid_inv = fields.get(6).to_optional_string();
        let tipo_item = fields.get(7).to_optional_string();
        let cod_ncm = fields.get(8).to_optional_string();
        let ex_ipi = fields.get(9).to_optional_string();
        let cod_gen = fields.get(10).to_optional_string();
        let cod_lst = fields.get(11).to_optional_string();
        let aliq_icms = get_decimal_field(12, "ALIQ_ICMS")?;

        let reg = Registro0200 {
            nivel: 3,
            bloco: '0',
            registro,
            line_number,
            cod_item,
            descr_item,
            cod_barra,
            cod_ant_item,
            unid_inv,
            tipo_item,
            cod_ncm,
            ex_ipi,
            cod_gen,
            cod_lst,
            aliq_icms,
        };

        Ok(reg)
    }
}
