use crate::{
    EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToCNPJ, ToNaiveDate,
    ToOptionalInteger, impl_reg_methods,
};
use chrono::NaiveDate;
use std::{path::Path, sync::Arc};

const EXPECTED_FIELDS: usize = 16;
const REGISTRO: &str = "0000";

#[derive(Debug, Clone)]
pub struct Registro0000 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: Arc<str>,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    // dt_ini, dt_fin e cnpj são itens obrigatórios
    pub cod_ver: Option<u8>,                // 2
    pub tipo_escrit: Option<u8>,            // 3
    pub ind_sit_esp: Option<u8>,            // 4
    pub num_rec_anterior: Option<Arc<str>>, // 5
    pub dt_ini: NaiveDate,                  // 6
    pub dt_fin: NaiveDate,                  // 7
    pub nome: Option<Arc<str>>,             // 8
    pub cnpj: Option<Arc<str>>,             // 9
    pub uf: Option<Arc<str>>,               // 10
    pub cod_mun: Option<Arc<str>>,          // 11
    pub suframa: Option<Arc<str>>,          // 12
    pub ind_nat_pj: Option<Arc<str>>,       // 13
    pub ind_ativ: Option<Arc<str>>,         // 14
}

impl_reg_methods!(Registro0000);

impl Registro0000 {
    /// Retorna o CNPJ completo (já validado e limpo durante o parse).
    #[inline]
    pub fn get_cnpj(&self) -> Arc<str> {
        self.cnpj.clone().unwrap_or_default()
    }

    /// Extrai o CNPJ Base (8 primeiros caracteres).
    /// Como o CNPJ foi validado no parse, o slice [..8] é seguro.
    #[inline]
    pub fn get_cnpj_base(&self) -> Arc<str> {
        self.cnpj
            .as_deref()
            .and_then(|s| s.get(..8))
            .map(Arc::from)
            .unwrap_or_default()
    }

    /// Retorna o nome da empresa, ou uma string vazia/padrão se for None.
    #[inline]
    pub fn get_nome(&self) -> Arc<str> {
        // Option<Arc<str>> implementa Clone, incrementando o contador atômico se Some.
        // unwrap_or_default retorna um Arc::from("") estático/barato se None.
        self.nome.clone().unwrap_or_default()
    }

    /// Retorna o Período de Apuração da EFD analisada
    #[inline]
    pub fn obter_periodo_de_apuracao(&self) -> NaiveDate {
        self.dt_ini
    }
}

impl SpedParser for Registro0000 {
    type Output = Registro0000;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro 0000 tipicamente tem 14 campos de dados (fora os delimitadores).
        // Se a linha começa com '|' e termina com '|', fields.len() deve ser 14 + 2 = 16.
        // Verifique o número real de campos para o seu formato SPED.

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

        // Closure for mandatory date fields (returns Result<NaiveDate, EFDError>)
        let get_required_date = |idx: usize, field_name: &str| {
            fields.get(idx).to_date(file_path, line_number, field_name)
        };

        // --- Closure auxiliar para campos u8 ---
        let get_integer = |idx: usize, field_name: &str| {
            fields
                .get(idx) // fields.get(idx) retorna Option<&&str>
                .to_optional_integer(file_path, line_number, field_name)
        };

        // --- Closure auxiliar para CNPJ (Limpeza + Validação + Erro específico) ---
        let get_full_cnpj = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_optional_cnpj(file_path, line_number, REGISTRO, field_name)
        };

        let cod_ver = get_integer(2, "COD_VER")?; // O '?' propagará o erro se houver
        let tipo_escrit = get_integer(3, "TIPO_ESCRIT")?;
        let ind_sit_esp = get_integer(4, "IND_SIT_ESP")?;
        let num_rec_anterior = fields.get(5).to_arc();

        // Using the closure for the MANDATORY date field
        let dt_ini = get_required_date(6, "DT_INI")?; // Will error if empty or invalid date
        let dt_fin = get_required_date(7, "DT_FIN")?;

        //let nome = fields.get(8).map(|n| Arc::from(n.to_uppercase()));
        let nome = fields.get(8).to_upper_arc(); // Normaliza nome para Uppercase

        // Aplicando a trait ToCNPJ no campo de índice 9
        let cnpj = get_full_cnpj(9, "CNPJ")?;

        let uf = fields.get(10).to_upper_arc(); // UF sempre Uppercase
        let cod_mun = fields.get(11).to_arc();
        let suframa = fields.get(12).to_arc();
        let ind_nat_pj = fields.get(13).to_arc();
        let ind_ativ = fields.get(14).to_arc();

        Ok(Registro0000 {
            nivel: 0,
            bloco: '0',
            registro: REGISTRO.into(),
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
        })
    }
}
