use crate::{EFDError, EFDResult, SpedParser, StringParser, ToDecimal, impl_sped_record_trait};
use rust_decimal::Decimal;
use std::{path::Path, sync::Arc};

const REGISTRO: &str = "F130";

#[derive(Debug, Clone)]
pub struct RegistroF130 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub nat_bc_cred: Option<Arc<str>>,           // 2
    pub ident_bem_imob: Option<Arc<str>>,        // 3
    pub ind_orig_cred: Option<Arc<str>>,         // 4
    pub ind_util_bem_imob: Option<Arc<str>>,     // 5
    pub mes_oper_aquis: Option<Arc<str>>,        // 6
    pub vl_oper_aquis: Option<Decimal>,          // 7
    pub parc_oper_nao_bc_cred: Option<Arc<str>>, // 8 (Assumindo String, pode ser Decimal)
    pub vl_bc_cred: Option<Decimal>,             // 9
    pub ind_nr_parc: Option<Arc<str>>,           // 10
    pub cst_pis: Option<u16>,                    // 11
    pub vl_bc_pis: Option<Decimal>,              // 12
    pub aliq_pis: Option<Decimal>,               // 13
    pub vl_pis: Option<Decimal>,                 // 14
    pub cst_cofins: Option<u16>,                 // 15
    pub vl_bc_cofins: Option<Decimal>,           // 16
    pub aliq_cofins: Option<Decimal>,            // 17
    pub vl_cofins: Option<Decimal>,              // 18
    pub cod_cta: Option<Arc<str>>,               // 19
    pub cod_ccus: Option<Arc<str>>,              // 20
    pub desc_bem_imob: Option<Arc<str>>,         // 21
}

impl_sped_record_trait!(RegistroF130);

impl SpedParser for RegistroF130 {
    type Output = RegistroF130;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro F130 possui 21 campos de dados + 2 delimitadores = 23.
        if len != 23 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.to_string(),
                tamanho_esperado: 23,
                tamanho_encontrado: len,
            });
        }

        let get_decimal_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path, line_number, field_name)
        };

        let nat_bc_cred = fields.get(2).to_arc();
        let ident_bem_imob = fields.get(3).to_arc();
        let ind_orig_cred = fields.get(4).to_arc();
        let ind_util_bem_imob = fields.get(5).to_arc();
        let mes_oper_aquis = fields.get(6).to_arc();
        let vl_oper_aquis = get_decimal_field(7, "VL_OPER_AQUIS")?;
        let parc_oper_nao_bc_cred = fields.get(8).to_arc();
        let vl_bc_cred = get_decimal_field(9, "VL_BC_CRED")?;
        let ind_nr_parc = fields.get(10).to_arc();
        let cst_pis = fields.get(11).parse_opt();
        let vl_bc_pis = get_decimal_field(12, "VL_BC_PIS")?;
        let aliq_pis = get_decimal_field(13, "ALIQ_PIS")?;
        let vl_pis = get_decimal_field(14, "VL_PIS")?;
        let cst_cofins = fields.get(15).parse_opt();
        let vl_bc_cofins = get_decimal_field(16, "VL_BC_COFINS")?;
        let aliq_cofins = get_decimal_field(17, "ALIQ_COFINS")?;
        let vl_cofins = get_decimal_field(18, "VL_COFINS")?;
        let cod_cta = fields.get(19).to_arc();
        let cod_ccus = fields.get(20).to_arc();
        let desc_bem_imob = fields.get(21).to_arc();

        let reg = RegistroF130 {
            nivel: 3,
            bloco: 'F',
            registro: REGISTRO.to_string(),
            line_number,
            nat_bc_cred,
            ident_bem_imob,
            ind_orig_cred,
            ind_util_bem_imob,
            mes_oper_aquis,
            vl_oper_aquis,
            parc_oper_nao_bc_cred,
            vl_bc_cred,
            ind_nr_parc,
            cst_pis,
            vl_bc_pis,
            aliq_pis,
            vl_pis,
            cst_cofins,
            vl_bc_cofins,
            aliq_cofins,
            vl_cofins,
            cod_cta,
            cod_ccus,
            desc_bem_imob,
        };

        Ok(reg)
    }
}
