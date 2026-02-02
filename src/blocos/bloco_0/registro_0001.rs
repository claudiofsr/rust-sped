use crate::{EFDError, EFDResult, ResultExt, SpedParser, StringParser, impl_reg_methods};
use std::{path::Path, sync::Arc};

const EXPECTED_FIELDS: usize = 4;
const REGISTRO: &str = "0001";
/*
Registro 0001: Abertura do Bloco 0

Nº Campo    Descrição                   Tipo    Tam     Dec Obrig
01 REG      Texto fixo contendo “0001”. C       004*    -   S
02 IND_MOV  Indicador de movimento:     N       001     -   S
            0 - Bloco com dados informados;
            1 – Bloco sem dados informados.

Observações: Registro obrigatório. Deve ser gerado para abertura do Bloco 0 e indica se há informações previstas para este bloco.

Nível hierárquico - 1
Ocorrência - um (por arquivo)

Campo 01 - Valor Válido: [0001]

Campo 02 - Valor Válido: [0,1]
Considerando que na escrituração do Bloco “0” deve ser escriturado, no mínimo, os registros “0110 - Regimes de
Apuração da Contribuição Social e de Apropriação de Crédito” e “0140 – Tabela de Cadastro de Estabelecimento”,
deve sempre ser informado, no Campo 02, o indicador “0 – Bloco com dados informados”.
*/

#[derive(Debug, Clone)]
pub struct Registro0001 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: Arc<str>,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub ind_mov: Option<u8>, // 2
}

impl_reg_methods!(Registro0001);

impl SpedParser for Registro0001 {
    type Output = Registro0001;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro 0001 tipicamente tem 2 campos de dados (fora os delimitadores).
        // Se a linha começa com '|' e termina com '|', fields.len() deve ser 2 + 2 = 4.
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

        let ind_mov = fields.get(2).parse_opt();

        Ok(Registro0001 {
            nivel: 1,
            bloco: '0',
            line_number,
            registro: REGISTRO.into(),
            ind_mov,
        })
    }
}
