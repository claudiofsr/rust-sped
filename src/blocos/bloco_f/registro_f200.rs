use crate::{
    CodigoSituacaoTributaria, EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToDecimal,
    ToNaiveDate, impl_reg_methods,
};
use chrono::NaiveDate;
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "F200";

#[derive(Debug, Clone)]
pub struct RegistroF200 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub ind_oper: Option<CompactString>,              // 2
    pub unid_imob: Option<CompactString>,             // 3
    pub ident_emp: Option<CompactString>,             // 4
    pub desc_unid_imob: Option<CompactString>,        // 5
    pub num_cont: Option<CompactString>,              // 6
    pub cpf_cnpj_adqu: Option<CompactString>,         // 7
    pub dt_oper: Option<NaiveDate>,                   // 8
    pub vl_tot_vend: Option<Decimal>,                 // 9
    pub vl_rec_acum: Option<Decimal>,                 // 10
    pub vl_tot_rec: Option<Decimal>,                  // 11
    pub cst_pis: Option<CodigoSituacaoTributaria>,    // 12
    pub vl_bc_pis: Option<Decimal>,                   // 13
    pub aliq_pis: Option<Decimal>,                    // 14
    pub vl_pis: Option<Decimal>,                      // 15
    pub cst_cofins: Option<CodigoSituacaoTributaria>, // 16
    pub vl_bc_cofins: Option<Decimal>,                // 17
    pub aliq_cofins: Option<Decimal>,                 // 18
    pub vl_cofins: Option<Decimal>,                   // 19
    pub perc_rec_receb: Option<CompactString>,        // 20 (Assumindo String, pode ser Decimal)
    pub ind_nat_emp: Option<CompactString>,           // 21
    pub inf_comp: Option<CompactString>,              // 22
}

impl_reg_methods!(RegistroF200);

impl SpedParser for RegistroF200 {
    type Output = RegistroF200;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro F200 possui 22 campos de dados + 2 delimitadores = 24.
        if len != 24 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 24,
                tamanho_encontrado: len,
            })
            .loc();
        }

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

        let ind_oper = fields.get(2).to_compact_string();
        let unid_imob = fields.get(3).to_compact_string();
        let ident_emp = fields.get(4).to_compact_string();
        let desc_unid_imob = fields.get(5).to_compact_string();
        let num_cont = fields.get(6).to_compact_string();
        let cpf_cnpj_adqu = fields.get(7).to_compact_string();
        let dt_oper = get_date(8, "DT_OPER")?;
        let vl_tot_vend = get_decimal(9, "VL_TOT_VEND")?;
        let vl_rec_acum = get_decimal(10, "VL_REC_ACUM")?;
        let vl_tot_rec = get_decimal(11, "VL_TOT_REC")?;
        let cst_pis = fields
            .get(12)
            .to_efd_field(file_path, line_number, "CST_PIS")?;
        let vl_bc_pis = get_decimal(13, "VL_BC_PIS")?;
        let aliq_pis = get_decimal(14, "ALIQ_PIS")?;
        let vl_pis = get_decimal(15, "VL_PIS")?;
        let cst_cofins = fields
            .get(16)
            .to_efd_field(file_path, line_number, "CST_COFINS")?;
        let vl_bc_cofins = get_decimal(17, "VL_BC_COFINS")?;
        let aliq_cofins = get_decimal(18, "ALIQ_COFINS")?;
        let vl_cofins = get_decimal(19, "VL_COFINS")?;
        let perc_rec_receb = fields.get(20).to_compact_string();
        let ind_nat_emp = fields.get(21).to_compact_string();
        let inf_comp = fields.get(22).to_compact_string();

        let reg = RegistroF200 {
            nivel: 3,
            bloco: 'F',
            registro: REGISTRO.into(),
            line_number,
            ind_oper,
            unid_imob,
            ident_emp,
            desc_unid_imob,
            num_cont,
            cpf_cnpj_adqu,
            dt_oper,
            vl_tot_vend,
            vl_rec_acum,
            vl_tot_rec,
            cst_pis,
            vl_bc_pis,
            aliq_pis,
            vl_pis,
            cst_cofins,
            vl_bc_cofins,
            aliq_cofins,
            vl_cofins,
            perc_rec_receb,
            ind_nat_emp,
            inf_comp,
        };

        Ok(reg)
    }
}
