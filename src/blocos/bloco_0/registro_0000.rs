use crate::{
    EFDError, EFDResult, SpedParser, ToCnpj, ToNaiveDate, ToOptionalInteger, ToOptionalString,
    impl_sped_record_trait,
};
use chrono::NaiveDate;
use std::path::Path;

const EXPECTED_FIELDS: usize = 16;

#[derive(Debug)]
pub struct Registro0000 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cod_ver: Option<u8>,              // 2
    pub tipo_escrit: Option<u8>,          // 3
    pub ind_sit_esp: Option<u8>,          // 4
    pub num_rec_anterior: Option<String>, // 5
    pub dt_ini: NaiveDate,                // 6
    pub dt_fin: NaiveDate,                // 7
    pub nome: Option<String>,             // 8
    pub cnpj: String,                     // 9
    pub uf: Option<String>,               // 10
    pub cod_mun: Option<String>,          // 11
    pub suframa: Option<String>,          // 12
    pub ind_nat_pj: Option<String>,       // 13
    pub ind_ativ: Option<String>,         // 14
}

impl_sped_record_trait!(Registro0000);

impl SpedParser for Registro0000 {
    type Output = Registro0000;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        // O registro 0000 tipicamente tem 14 campos de dados (fora os delimitadores).
        // Se a linha começa com '|' e termina com '|', fields.len() deve ser 14 + 2 = 16.
        // Verifique o número real de campos para o seu formato SPED.

        if len != EXPECTED_FIELDS {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: registro.clone(), // Aqui precisa do clone porque `registro` será usado depois.
                tamanho_esperado: EXPECTED_FIELDS,
                tamanho_encontrado: len,
            });
        }

        // --- Closures auxiliares para campos comuns ---

        // Closure for mandatory date fields (returns Result<NaiveDate, EFDError>)
        let get_required_date_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_date(file_path.to_path_buf(), line_number, field_name)
        };

        // --- Closure auxiliar para campos u8 ---
        let get_integer_field = |idx: usize, field_name: &str| {
            fields
                .get(idx) // fields.get(idx) retorna Option<&&str>
                .to_optional_integer(file_path.to_path_buf(), line_number, field_name)
        };

        // --- Closure auxiliar para obter CNPJ ---
        let get_cnpj_field = |idx: usize, field_name: &str| {
            fields
                .get(idx) // fields.get(idx) retorna Option<&&str>
                .to_cnpj(file_path.to_path_buf(), line_number, &registro, field_name)
        };

        let cod_ver = get_integer_field(2, "COD_VER")?; // O '?' propagará o erro se houver
        let tipo_escrit = get_integer_field(3, "TIPO_ESCRIT")?;
        let ind_sit_esp = get_integer_field(4, "IND_SIT_ESP")?;
        let num_rec_anterior = fields.get(5).to_optional_string();

        // Using the closure for the MANDATORY date field
        let dt_ini = get_required_date_field(6, "DT_INI")?; // Will error if empty or invalid date
        let dt_fin = get_required_date_field(7, "DT_FIN")?;

        let nome = fields.get(8).to_optional_string();
        let cnpj = get_cnpj_field(9, "CNPJ")?;
        let uf = fields.get(10).to_optional_string();
        let cod_mun = fields.get(11).to_optional_string();
        let suframa = fields.get(12).to_optional_string();
        let ind_nat_pj = fields.get(13).to_optional_string();
        let ind_ativ = fields.get(14).to_optional_string();

        let reg = Registro0000 {
            nivel: 0,
            bloco: '0',
            registro,
            line_number,
            cod_ver,
            tipo_escrit,
            ind_sit_esp,
            num_rec_anterior,
            dt_ini,
            dt_fin,
            nome,
            cnpj,
            uf,
            cod_mun,
            suframa,
            ind_nat_pj,
            ind_ativ,
        };

        Ok(reg)
    }
}
