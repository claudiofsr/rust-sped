use crate::{
    EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToDecimal, ToNaiveDate,
    impl_reg_methods,
};
use chrono::NaiveDate;
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "F130";

/// Registro F130: Bens Incorporados ao Ativo Imobilizado
///
/// Operações Geradoras de Créditos com Base no Valor de Aquisição/Contribuição
#[derive(Debug, Clone)]
pub struct RegistroF130 {
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
    pub mes_oper_aquis: Option<NaiveDate>,            // 6
    pub vl_oper_aquis: Option<Decimal>,               // 7
    pub parc_oper_nao_bc_cred: Option<CompactString>, // 8 (Assumindo String, pode ser Decimal)
    pub vl_bc_cred: Option<Decimal>,                  // 9
    pub ind_nr_parc: Option<CompactString>,           // 10
    pub cst_pis: Option<u16>,                         // 11
    pub vl_bc_pis: Option<Decimal>,                   // 12
    pub aliq_pis: Option<Decimal>,                    // 13
    pub vl_pis: Option<Decimal>,                      // 14
    pub cst_cofins: Option<u16>,                      // 15
    pub vl_bc_cofins: Option<Decimal>,                // 16
    pub aliq_cofins: Option<Decimal>,                 // 17
    pub vl_cofins: Option<Decimal>,                   // 18
    pub cod_cta: Option<CompactString>,               // 19
    pub cod_ccus: Option<CompactString>,              // 20
    pub desc_bem_imob: Option<CompactString>,         // 21
}

impl_reg_methods!(RegistroF130);

impl SpedParser for RegistroF130 {
    type Output = RegistroF130;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro F130 possui 21 campos de dados + 2 delimitadores = 23.
        if len != 23 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 23,
                tamanho_encontrado: len,
            })
            .loc();
        }

        // --- Closures auxiliares para campos comuns ---

        // Closure para campos de data (Option<NaiveDate>)
        let get_date = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_optional_date(file_path, line_number, field_name)
        };

        let get_decimal = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path, line_number, field_name)
        };

        let nat_bc_cred = fields.get(2).parse_opt();
        let ident_bem_imob = fields.get(3).to_compact_string();
        let ind_orig_cred = fields.get(4).to_compact_string();
        let ind_util_bem_imob = fields.get(5).to_compact_string();

        let mes_oper_aquis = get_date(6, "MES_OPER_AQUIS")?;

        let vl_oper_aquis = get_decimal(7, "VL_OPER_AQUIS")?;
        let parc_oper_nao_bc_cred = fields.get(8).to_compact_string();
        let vl_bc_cred = get_decimal(9, "VL_BC_CRED")?;
        let ind_nr_parc = fields.get(10).to_compact_string();
        let cst_pis = fields.get(11).parse_opt();
        let vl_bc_pis = get_decimal(12, "VL_BC_PIS")?;
        let aliq_pis = get_decimal(13, "ALIQ_PIS")?;
        let vl_pis = get_decimal(14, "VL_PIS")?;
        let cst_cofins = fields.get(15).parse_opt();
        let vl_bc_cofins = get_decimal(16, "VL_BC_COFINS")?;
        let aliq_cofins = get_decimal(17, "ALIQ_COFINS")?;
        let vl_cofins = get_decimal(18, "VL_COFINS")?;
        let cod_cta = fields.get(19).to_compact_string();
        let cod_ccus = fields.get(20).to_compact_string();
        let desc_bem_imob = fields.get(21).to_compact_string();

        let reg = RegistroF130 {
            nivel: 3,
            bloco: 'F',
            registro: REGISTRO.into(),
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
