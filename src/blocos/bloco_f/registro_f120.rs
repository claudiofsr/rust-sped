use crate::{
    CodigoSituacaoTributaria, EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToDecimal,
    impl_reg_methods,
};
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "F120";

#[derive(Debug, Clone)]
pub struct RegistroF120 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub nat_bc_cred: Option<u16>,                     // 2
    pub ident_bem_imob: Option<CompactString>,        // 3
    pub ind_orig_cred: Option<CompactString>,         // 4
    pub ind_util_bem_imob: Option<CompactString>,     // 5
    pub vl_oper_dep: Option<Decimal>,                 // 6
    pub parc_oper_nao_bc_cred: Option<CompactString>, // 7 (Assumindo String, pode ser Decimal)
    pub cst_pis: Option<CodigoSituacaoTributaria>,    // 8
    pub vl_bc_pis: Option<Decimal>,                   // 9
    pub aliq_pis: Option<Decimal>,                    // 10
    pub vl_pis: Option<Decimal>,                      // 11
    pub cst_cofins: Option<CodigoSituacaoTributaria>, // 12
    pub vl_bc_cofins: Option<Decimal>,                // 13
    pub aliq_cofins: Option<Decimal>,                 // 14
    pub vl_cofins: Option<Decimal>,                   // 15
    pub cod_cta: Option<CompactString>,               // 16
    pub cod_ccus: Option<CompactString>,              // 17
    pub desc_bem_imob: Option<CompactString>,         // 18
}

impl_reg_methods!(RegistroF120);

impl SpedParser for RegistroF120 {
    type Output = RegistroF120;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro F120 possui 18 campos de dados + 2 delimitadores = 20.
        if len != 20 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 20,
                tamanho_encontrado: len,
            })
            .loc();
        }

        let get_decimal = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path, line_number, field_name)
        };

        let nat_bc_cred = fields.get(2).parse_opt();
        let ident_bem_imob = fields.get(3).to_compact_string();
        let ind_orig_cred = fields.get(4).to_compact_string();
        let ind_util_bem_imob = fields.get(5).to_compact_string();
        let vl_oper_dep = get_decimal(6, "VL_OPER_DEP")?;
        let parc_oper_nao_bc_cred = fields.get(7).to_compact_string();
        let cst_pis = fields
            .get(8)
            .to_efd_field(file_path, line_number, "CST_PIS")?;
        let vl_bc_pis = get_decimal(9, "VL_BC_PIS")?;
        let aliq_pis = get_decimal(10, "ALIQ_PIS")?;
        let vl_pis = get_decimal(11, "VL_PIS")?;
        let cst_cofins = fields
            .get(12)
            .to_efd_field(file_path, line_number, "CST_COFINS")?;
        let vl_bc_cofins = get_decimal(13, "VL_BC_COFINS")?;
        let aliq_cofins = get_decimal(14, "ALIQ_COFINS")?;
        let vl_cofins = get_decimal(15, "VL_COFINS")?;
        let cod_cta = fields.get(16).to_compact_string();
        let cod_ccus = fields.get(17).to_compact_string();
        let desc_bem_imob = fields.get(18).to_compact_string();

        let reg = RegistroF120 {
            nivel: 3,
            bloco: 'F',
            registro: REGISTRO.into(),
            line_number,
            nat_bc_cred,
            ident_bem_imob,
            ind_orig_cred,
            ind_util_bem_imob,
            vl_oper_dep,
            parc_oper_nao_bc_cred,
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
