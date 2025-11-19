use crate::{EFDError, EFDResult, SpedParser, ToDecimal, impl_sped_record_trait};
use rust_decimal::Decimal;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Registro0111 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub rec_bru_ncum_trib_mi: Option<Decimal>, // 2
    pub rec_bru_ncum_nt_mi: Option<Decimal>,   // 3
    pub rec_bru_ncum_exp: Option<Decimal>,     // 4
    pub rec_bru_cum: Option<Decimal>,          // 5
    pub rec_bru_total: Option<Decimal>,        // 6
}

impl_sped_record_trait!(Registro0111);

impl SpedParser for Registro0111 {
    type Output = Registro0111;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        if len != 8 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: registro.clone(),
                tamanho_esperado: 8,
                tamanho_encontrado: len,
            });
        }

        let get_decimal_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path.to_path_buf(), line_number, field_name)
        };

        let rec_bru_ncum_trib_mi = get_decimal_field(2, "REC_BRU_NCUM_TRIB_MI")?;
        let rec_bru_ncum_nt_mi = get_decimal_field(3, "REC_BRU_NCUM_NT_MI")?;
        let rec_bru_ncum_exp = get_decimal_field(4, "REC_BRU_NCUM_EXP")?;
        let rec_bru_cum = get_decimal_field(5, "REC_BRU_CUM")?;
        let rec_bru_total = get_decimal_field(6, "REC_BRU_TOTAL")?;

        let reg = Registro0111 {
            nivel: 3,
            bloco: '0',
            registro,
            line_number,
            rec_bru_ncum_trib_mi,
            rec_bru_ncum_nt_mi,
            rec_bru_ncum_exp,
            rec_bru_cum,
            rec_bru_total,
        };

        Ok(reg)
    }
}
