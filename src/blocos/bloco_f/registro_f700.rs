use crate::{EFDError, EFDResult, SpedParser, StringParser, ToDecimal, impl_reg_methods};
use rust_decimal::Decimal;
use std::{path::Path, sync::Arc};

const REGISTRO: &str = "F700";

#[derive(Debug, Clone)]
pub struct RegistroF700 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: Arc<str>,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub ind_ori_ded: Option<Arc<str>>,  // 2
    pub ind_nat_ded: Option<Arc<str>>,  // 3
    pub vl_ded_pis: Option<Decimal>,    // 4
    pub vl_ded_cofins: Option<Decimal>, // 5
    pub vl_bc_oper: Option<Decimal>,    // 6
    pub cnpj: Option<Arc<str>>,         // 7
    pub inf_comp: Option<Arc<str>>,     // 8
}

impl_reg_methods!(RegistroF700);

impl SpedParser for RegistroF700 {
    type Output = RegistroF700;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro F700 possui 8 campos de dados + 2 delimitadores = 10.
        if len != 10 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 10,
                tamanho_encontrado: len,
            });
        }

        let get_decimal = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path, line_number, field_name)
        };

        let ind_ori_ded = fields.get(2).to_arc();
        let ind_nat_ded = fields.get(3).to_arc();
        let vl_ded_pis = get_decimal(4, "VL_DED_PIS")?;
        let vl_ded_cofins = get_decimal(5, "VL_DED_COFINS")?;
        let vl_bc_oper = get_decimal(6, "VL_BC_OPER")?;
        let cnpj = fields.get(7).to_arc();
        let inf_comp = fields.get(8).to_arc();

        let reg = RegistroF700 {
            nivel: 3,
            bloco: 'F',
            registro: REGISTRO.into(),
            line_number,
            ind_ori_ded,
            ind_nat_ded,
            vl_ded_pis,
            vl_ded_cofins,
            vl_bc_oper,
            cnpj,
            inf_comp,
        };

        Ok(reg)
    }
}
