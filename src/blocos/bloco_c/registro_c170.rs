use crate::{
    CodigoSituacaoTributaria, EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToDecimal,
    impl_reg_methods,
};
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const EXPECTED_FIELDS: usize = 39;
const REGISTRO: &str = "C170";

/// Complemento do Documento - Itens do Documento
#[derive(Debug, Clone)]
pub struct RegistroC170 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub num_item: Option<u16>,                        // 2
    pub cod_item: Option<CompactString>,              // 3
    pub descr_compl: Option<CompactString>,           // 4
    pub qtd: Option<Decimal>,                         // 5
    pub unid: Option<CompactString>,                  // 6
    pub vl_item: Option<Decimal>,                     // 7
    pub vl_desc: Option<Decimal>,                     // 8
    pub ind_mov: Option<char>,                        // 9
    pub cst_icms: Option<u16>,                        // 10
    pub cfop: Option<u16>,                            // 11
    pub cod_nat: Option<CompactString>,               // 12
    pub vl_bc_icms: Option<Decimal>,                  // 13
    pub aliq_icms: Option<Decimal>,                   // 14
    pub vl_icms: Option<Decimal>,                     // 15
    pub vl_bc_icms_st: Option<Decimal>,               // 16
    pub aliq_st: Option<Decimal>,                     // 17
    pub vl_icms_st: Option<Decimal>,                  // 18
    pub ind_apur: Option<CompactString>,              // 19
    pub cst_ipi: Option<u16>,                         // 20
    pub cod_enq: Option<CompactString>,               // 21
    pub vl_bc_ipi: Option<Decimal>,                   // 22
    pub aliq_ipi: Option<Decimal>,                    // 23
    pub vl_ipi: Option<Decimal>,                      // 24
    pub cst_pis: Option<CodigoSituacaoTributaria>,    // 25
    pub vl_bc_pis: Option<Decimal>,                   // 26
    pub aliq_pis: Option<Decimal>,                    // 27
    pub quant_bc_pis: Option<CompactString>,          // 28
    pub aliq_pis_quant: Option<Decimal>,              // 29
    pub vl_pis: Option<Decimal>,                      // 30
    pub cst_cofins: Option<CodigoSituacaoTributaria>, // 31
    pub vl_bc_cofins: Option<Decimal>,                // 32
    pub aliq_cofins: Option<Decimal>,                 // 33
    pub quant_bc_cofins: Option<CompactString>,       // 34
    pub aliq_cofins_quant: Option<Decimal>,           // 35
    pub vl_cofins: Option<Decimal>,                   // 36
    pub cod_cta: Option<CompactString>,               // 37
}

impl_reg_methods!(RegistroC170);

impl SpedParser for RegistroC170 {
    type Output = RegistroC170;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro C170 possui 37 campos de dados + 2 delimitadores = 39.
        if len != EXPECTED_FIELDS {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: EXPECTED_FIELDS,
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
        let qtd = get_decimal(5, "QTD")?; // Assumindo QTD como Decimal
        let unid = fields.get(6).to_compact_string();
        let vl_item = get_decimal(7, "VL_ITEM")?;
        let vl_desc = get_decimal(8, "VL_DESC")?;
        let ind_mov = fields.get(9).parse_opt();
        let cst_icms = fields.get(10).parse_opt();
        let cfop = fields.get(11).parse_opt();
        let cod_nat = fields.get(12).to_compact_string();
        let vl_bc_icms = get_decimal(13, "VL_BC_ICMS")?;
        let aliq_icms = get_decimal(14, "ALIQ_ICMS")?;
        let vl_icms = get_decimal(15, "VL_ICMS")?;
        let vl_bc_icms_st = get_decimal(16, "VL_BC_ICMS_ST")?;
        let aliq_st = get_decimal(17, "ALIQ_ST")?;
        let vl_icms_st = get_decimal(18, "VL_ICMS_ST")?;
        let ind_apur = fields.get(19).to_compact_string();
        let cst_ipi = fields.get(20).parse_opt();
        let cod_enq = fields.get(21).to_compact_string();
        let vl_bc_ipi = get_decimal(22, "VL_BC_IPI")?;
        let aliq_ipi = get_decimal(23, "ALIQ_IPI")?;
        let vl_ipi = get_decimal(24, "VL_IPI")?;
        let cst_pis = fields
            .get(25)
            //.parse_opt();
            .to_efd_field(file_path, line_number, "CST_PIS")?;
        let vl_bc_pis = get_decimal(26, "VL_BC_PIS")?;
        let aliq_pis = get_decimal(27, "ALIQ_PIS")?;
        let quant_bc_pis = fields.get(28).to_compact_string(); // Pode ser String ou Decimal
        let aliq_pis_quant = get_decimal(29, "ALIQ_PIS_QUANT")?;
        let vl_pis = get_decimal(30, "VL_PIS")?;
        let cst_cofins = fields
            .get(31)
            //.parse_opt();
            .to_efd_field(file_path, line_number, "CST_COFINS")?;
        let vl_bc_cofins = get_decimal(32, "VL_BC_COFINS")?;
        let aliq_cofins = get_decimal(33, "ALIQ_COFINS")?;
        let quant_bc_cofins = fields.get(34).to_compact_string(); // Pode ser String ou Decimal
        let aliq_cofins_quant = get_decimal(35, "ALIQ_COFINS_QUANT")?;
        let vl_cofins = get_decimal(36, "VL_COFINS")?;
        let cod_cta = fields.get(37).to_compact_string();

        let reg = RegistroC170 {
            nivel: 4,
            bloco: 'C',
            registro: REGISTRO.into(),
            line_number,
            num_item,
            cod_item,
            descr_compl,
            qtd,
            unid,
            vl_item,
            vl_desc,
            ind_mov,
            cst_icms,
            cfop,
            cod_nat,
            vl_bc_icms,
            aliq_icms,
            vl_icms,
            vl_bc_icms_st,
            aliq_st,
            vl_icms_st,
            ind_apur,
            cst_ipi,
            cod_enq,
            vl_bc_ipi,
            aliq_ipi,
            vl_ipi,
            cst_pis,
            vl_bc_pis,
            aliq_pis,
            quant_bc_pis,
            aliq_pis_quant,
            vl_pis,
            cst_cofins,
            vl_bc_cofins,
            aliq_cofins,
            quant_bc_cofins,
            aliq_cofins_quant,
            vl_cofins,
            cod_cta,
        };

        Ok(reg)
    }
}
