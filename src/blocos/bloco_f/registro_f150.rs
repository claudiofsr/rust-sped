use crate::{
    CodigoSituacaoTributaria, EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToDecimal,
    impl_reg_methods,
};
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "F150";

#[derive(Debug, Clone)]
pub struct RegistroF150 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub nat_bc_cred: Option<u16>,                     // 2
    pub vl_tot_est: Option<Decimal>,                  // 3
    pub est_imp: Option<CompactString>,               // 4
    pub vl_bc_est: Option<Decimal>,                   // 5
    pub vl_bc_men_est: Option<Decimal>,               // 6
    pub cst_pis: Option<CodigoSituacaoTributaria>,    // 7
    pub aliq_pis: Option<Decimal>,                    // 8
    pub vl_cred_pis: Option<Decimal>,                 // 9
    pub cst_cofins: Option<CodigoSituacaoTributaria>, // 10
    pub aliq_cofins: Option<Decimal>,                 // 11
    pub vl_cred_cofins: Option<Decimal>,              // 12
    pub desc_est: Option<CompactString>,              // 13
    pub cod_cta: Option<CompactString>,               // 14
}

impl_reg_methods!(RegistroF150);

impl SpedParser for RegistroF150 {
    type Output = RegistroF150;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro F150 possui 14 campos de dados + 2 delimitadores = 16.
        if len != 16 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 16,
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
        let vl_tot_est = get_decimal(3, "VL_TOT_EST")?;
        let est_imp = fields.get(4).to_compact_string();
        let vl_bc_est = get_decimal(5, "VL_BC_EST")?;
        let vl_bc_men_est = get_decimal(6, "VL_BC_MEN_EST")?;
        let cst_pis = fields
            .get(7)
            .to_efd_field(file_path, line_number, "CST_PIS")?;
        let aliq_pis = get_decimal(8, "ALIQ_PIS")?;
        let vl_cred_pis = get_decimal(9, "VL_CRED_PIS")?;
        let cst_cofins = fields
            .get(10)
            .to_efd_field(file_path, line_number, "CST_COFINS")?;
        let aliq_cofins = get_decimal(11, "ALIQ_COFINS")?;
        let vl_cred_cofins = get_decimal(12, "VL_CRED_COFINS")?;
        let desc_est = fields.get(13).to_compact_string();
        let cod_cta = fields.get(14).to_compact_string();

        let reg = RegistroF150 {
            nivel: 3,
            bloco: 'F',
            registro: REGISTRO.into(),
            line_number,
            nat_bc_cred,
            vl_tot_est,
            est_imp,
            vl_bc_est,
            vl_bc_men_est,
            cst_pis,
            aliq_pis,
            vl_cred_pis,
            cst_cofins,
            aliq_cofins,
            vl_cred_cofins,
            desc_est,
            cod_cta,
        };

        Ok(reg)
    }
}
