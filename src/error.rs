use glob::PatternError;
use rust_xlsxwriter::XlsxError;
use std::{
    error::Error,
    io,
    num::{ParseFloatError, ParseIntError, TryFromIntError},
    path::PathBuf,
    str::Utf8Error,
};
use thiserror::Error;

/// Um tipo de resultado especializado para operações que podem retornar um `EFDError`.
pub type EFDResult<T> = Result<T, EFDError>;

// Nota: A implementação de `std::error::Error for EFDError` é automaticamente
// fornecida pelo `#[derive(Error)]` do `thiserror`.
// As implementações `From` para `ParseFloatError`, `io::Error`, `XlsxError`, `csv::Error`,
// `TryFromIntError` e `PatternError` são geradas automaticamente pelos atributos `#[from]`.

/// Enumera todos os possíveis erros que podem ocorrer durante o processamento do EFD SPED.
#[derive(Error, Debug)] // Derive Error do thiserror
pub enum EFDError {
    /// Erro de formato ou conteúdo específico do arquivo/linha: CNPJ inválido encontrado.
    #[error(
        "Erro ao processar informações do arquivo: CNPJ do estabelecimento não encontrado.\n\
         Arquivo: {0}\nLinha nº: {1}"
    )]
    InvalidCNPJ(String, usize), // filename, line_number
    /// Erro de formato ou conteúdo específico do arquivo/linha: Nome da empresa inválido encontrado.
    #[error(
        "Erro ao processar informações do arquivo: Nome do estabelecimento não encontrado.\n\
         Arquivo: {0}\nLinha nº: {1}"
    )]
    InvalidName(String, usize), // filename, line_number
    /// Erro relacionado a uma configuração de estilo ou formato de saída inválida.
    #[error("Estilo de saída inválido!")]
    InvalidStyle,
    /// Erro de validação ou formato específico para o período (PA).
    #[error("Período de Apuração (PA) inválido!")]
    InvalidPA,
    /// Erro geral de validação de formato (ex: formato MMYYYY esperado).
    #[error("Formato inválido (esperado MMYYYY ou similar)")]
    InvalidFormat,
    /// Erro geral de validação ou parsing de data.
    #[error("Data inválida")]
    InvalidDate,
    /// Erro ao fazer parse de um número decimal com contexto de arquivo e linha.
    ///
    /// Envolve `std::num::ParseFloatError`.
    #[error(
        "fn formatar_casas_decimais()\n\
        Erro ao fazer parse de número decimal: '{valor_str}'.\n\
        Arquivo: '{arquivo:?}'\n\
        Nº da linha: {linha_num}.\n\
        A string fornecida não pôde ser interpretada como um número decimal.\n\
        Erro Original: {source}"
    )]
    ParseFloatError {
        #[source] // Indica que este é o erro original
        source: ParseFloatError,
        valor_str: String,
        arquivo: PathBuf,
        linha_num: usize,
    },
    /// Envolve `std::num::ParseIntError` e adiciona contexto customizado (o valor que falhou o parsing).
    /// Requer mapeamento manual com `.map_err()` se o contexto for necessário.
    #[error(
        "Erro ao fazer parse de número inteiro: {1}\n\
         A string fornecida não pôde ser interpretada como um número inteiro.\n\
         Erro Original: {0}"
    )]
    ParseIntError(ParseIntError, String), // source error, value string
    /// Erro detalhado para problemas de decodificação UTF-8 com contexto de arquivo/linha/IO.
    #[error(
        "Erro de decodificação UTF-8 no arquivo '{0:?}' na linha {1}:\n\
         Falha ao decodificar bytes como UTF-8.\n\
         Codificação tentada (provavelmente): WINDOWS-1252.\n\
         Erro UTF-8: {2}\n\
         Contexto do Erro de IO: {3}\n\
         Considere tentar outra codificação."
    )]
    Utf8DecodeError(PathBuf, usize, Utf8Error, io::Error), // path, line_number, Utf8Error, io::Error context
    /// Item (ex: período, registro, valor) não encontrado.
    #[error("Item não encontrado (ex: Período não encontrado)")]
    NotFound,
    /// Uma linha inteira de um arquivo era inválida, carregando o conteúdo da linha.
    #[error("Linha inválida!\nConteúdo da Linha: {0:#?}")]
    InvalidLine(String), // line content
    /// Um tipo de registro específico tinha um comprimento inesperado.
    #[error("Comprimento inválido!\nRegistro: {0}\nComprimento: {1}")]
    InvalidLength(String, usize), // record name, actual length
    /// Encontrou um tipo de registro SPED que não é suportado ou reconhecido.
    #[error("Tipo de registro não suportado!\nRegistro: {0}")]
    UnsupportedRecordType(String), // record type found
    /// Erro genérico para falhas de conversão de campo (se não cobertas por erros de parse específicos).
    #[error("Conversão de campo inválida!")]
    FieldConversion,
    /// Wrapper geral para `std::io::Error`.
    #[error("Erro de IO: {0}")]
    Io(#[from] io::Error), // source io::Error
    /// Envolve `rust_xlsxwriter::XlsxError`.
    #[error("Erro Xlsx: {0}")]
    XlsxError(#[from] XlsxError), // source XlsxError
    /// Envolve `csv::Error`.
    #[error("Erro CSV: {0}")]
    CsvError(#[from] csv::Error), // source csv::Error
    /// Envolve `std::num::TryFromIntError`.
    #[error("Erro de conversão de inteiro: {0}")]
    TryFromIntError(#[from] TryFromIntError), // source TryFromIntError
    /// Envolve `glob::PatternError`.
    #[error("Erro de padrão (glob): {0}")]
    PatternError(#[from] PatternError), // source PatternError
    /// Um catch-all para outros erros menos específicos não cobertos por variantes específicas.
    #[error("Outro erro subjacente: {0}")]
    Other(String), // Wrapped boxed error
}

// Implement From<String> para EFDError, caso precise converter strings genéricas em erros.
impl From<String> for EFDError {
    fn from(err: String) -> Self {
        EFDError::Other(err)
    }
}

// Implementa a conversão de Box<dyn Error + Send + Sync> para EFDError
impl From<Box<dyn Error + Send + Sync>> for EFDError {
    fn from(err: Box<dyn Error + Send + Sync>) -> Self {
        EFDError::Other(err.to_string())
    }
}
