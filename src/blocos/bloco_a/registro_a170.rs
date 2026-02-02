use crate::{
    EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToDecimal, impl_reg_methods,
};
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "A170";

#[derive(Debug, Clone)]
pub struct RegistroA170 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub num_item: Option<u16>,                // 2
    pub cod_item: Option<CompactString>,      // 3
    pub descr_compl: Option<CompactString>,   // 4
    pub vl_item: Option<Decimal>,             // 5
    pub vl_desc: Option<Decimal>,             // 6
    pub nat_bc_cred: Option<u16>,             // 7
    pub ind_orig_cred: Option<CompactString>, // 8
    pub cst_pis: Option<u16>,                 // 9
    pub vl_bc_pis: Option<Decimal>,           // 10
    pub aliq_pis: Option<Decimal>,            // 11
    pub vl_pis: Option<Decimal>,              // 12
    pub cst_cofins: Option<u16>,              // 13
    pub vl_bc_cofins: Option<Decimal>,        // 14
    pub aliq_cofins: Option<Decimal>,         // 15
    pub vl_cofins: Option<Decimal>,           // 16
    pub cod_cta: Option<CompactString>,       // 17
    pub cod_ccus: Option<CompactString>,      // 18
}

impl_reg_methods!(RegistroA170);

impl SpedParser for RegistroA170 {
    type Output = RegistroA170;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro A170 possui 18 campos de dados + 2 delimitadores = 20.
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

        // --- Closures auxiliares para campos comuns ---

        // Closure para campos decimais (Option<Decimal>)
        let get_decimal = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path, line_number, field_name)
        };

        let num_item = fields.get(2).parse_opt();
        let cod_item = fields.get(3).to_compact_string();
        let descr_compl = fields.get(4).to_compact_string(); // .to_compact_string()
        let vl_item = get_decimal(5, "VL_ITEM")?;
        let vl_desc = get_decimal(6, "VL_DESC")?;
        let nat_bc_cred = fields.get(7).parse_opt();
        let ind_orig_cred = fields.get(8).to_compact_string();
        let cst_pis = fields.get(9).parse_opt();
        let vl_bc_pis = get_decimal(10, "VL_BC_PIS")?;
        let aliq_pis = get_decimal(11, "ALIQ_PIS")?;
        let vl_pis = get_decimal(12, "VL_PIS")?;
        let cst_cofins = fields.get(13).parse_opt();
        let vl_bc_cofins = get_decimal(14, "VL_BC_COFINS")?;
        let aliq_cofins = get_decimal(15, "ALIQ_COFINS")?;
        let vl_cofins = get_decimal(16, "VL_COFINS")?;
        let cod_cta = fields.get(17).to_compact_string();
        let cod_ccus = fields.get(18).to_compact_string();

        let reg = RegistroA170 {
            nivel: 4,
            bloco: 'A',
            registro: REGISTRO.into(),
            line_number,
            num_item,
            cod_item,
            descr_compl,
            vl_item,
            vl_desc,
            nat_bc_cred,
            ind_orig_cred,
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
        };

        Ok(reg)
    }
}
