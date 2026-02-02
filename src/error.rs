use glob::PatternError;
use rust_decimal::Error as DecimalError;
use rust_xlsxwriter::XlsxError;
use std::{
    error::Error,
    io,
    num::{ParseFloatError, ParseIntError, TryFromIntError},
    panic::Location,
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
    /// Erro ocorrido durante a análise de um único arquivo EFD.
    #[error(
        "Erro ao analisar o arquivo '{arquivo:?}'\n\
        {source}\n\
         Detalhes: Ocorreu uma falha durante o processamento de um arquivo Sped EFD Contribuições."
    )]
    AnalyzeFileError {
        #[source]
        source: Box<EFDError>, // Envolve outro EFDError
        arquivo: PathBuf,
    },

    /// Erro específico para falha na correlação entre PIS e COFINS (Bloco M).
    #[error(
        "Ausência de correlação entre as alíquotas de PIS/PASEP e COFINS.\n\
         Arquivo: {arquivo:?}\n\
         Linha nº: {linha_num}\n\
         Registro: {registro}\n\
         Alíquota COFINS fornecida: {aliq_cofins}\n\
         Detalhes: Não foi possível encontrar uma alíquota de PIS correspondente na tabela estática ou no cache dinâmico (M105)."
    )]
    CorrelationPISCOFINS {
        arquivo: PathBuf,
        linha_num: usize,
        registro: String,
        aliq_cofins: String,
    },

    /// Erro ao criar um diretório.
    #[error("Não foi possível criar o diretório '{0}': {1}")]
    DirectoryCreationFailed(String, io::Error),

    /// Erro que sinaliza o fim esperado do arquivo SPED (registro 9999).
    /// Usado para interromper o processamento.
    #[error("Fim do arquivo SPED (Registro 9999) alcançado.")]
    EndOfFile,

    #[error("O formato/estilo '{0}' não foi encontrado no mapa de formatos do Excel.")]
    FormatNotFound(String),

    /// Erro de formato ou conteúdo específico do arquivo/linha: CNPJ inválido encontrado.
    #[error(
        "CNPJ inválido.\n\
         Arquivo: {arquivo:?}\n\
         Linha nº: {linha_num}\n\
         Registro: {registro}\n\
         Campo: {campo_nome}\n\
         CNPJ: {cnpj}\n"
    )]
    InvalidCNPJ {
        arquivo: PathBuf,
        linha_num: usize,
        registro: String,
        campo_nome: String,
        cnpj: String,
    },

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
    #[error("Formato inválido (esperado MMYYYY para MesAno)")]
    InvalidDateFormat,

    /// Erro geral de validação ou parsing de data.
    #[error("Data inválida")]
    InvalidDate,

    #[error("Arquivo nº 0: opção inválida!")]
    InvalidFileIndex,

    #[error("Não foi encontrado o arquivo nº {index}! Escolha um número entre 1 a {total}.")]
    FileNotFoundInIndex { index: usize, total: usize },

    #[error("Nenhuma ação selecionada (utilize -r para analisar ou -f para listar).")]
    NoActionSelected,

    /// Erro disparado quando nenhum arquivo SPED é encontrado no diretório.
    #[error("Nenhum arquivo SPED EFD encontrado neste diretório!")]
    NoFilesFound,

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

    /// Erro ao fazer parse de um número decimal com contexto de arquivo e linha.
    ///
    /// Envolve `rust_decimal::Error`.
    #[error(
        "Erro ao fazer parse de valor decimal: '{valor_str}'.\n\
         Campo: '{campo_nome}'\n\
         Arquivo: '{arquivo:?}'\n\
         Nº da linha: {linha_num}.\n\
         A string fornecida não pôde ser interpretada como um número decimal.\n\
         Erro Original: {source}"
    )]
    ParseDecimalError {
        #[source] // Indica que este é o erro original
        source: DecimalError, // Agora aceita o erro de rust_decimal
        valor_str: String,
        campo_nome: String, // Adicionado para dar mais contexto de qual campo falhou
        arquivo: PathBuf,
        linha_num: usize,
    },

    /// Erro ao fazer parse de data com contexto de arquivo e linha.
    #[error(
        "Erro ao fazer parse de data: '{data_str}'.\n\
         Arquivo: '{arquivo:?}'\n\
         Nº da linha: {line_number}.\n\
         Campo: '{campo_nome}'\n\
         A string fornecida não pôde ser interpretada como uma data válida no formato esperado (DDMMYYYY).\n\
         Erro Original: {source}"
    )]
    ParseDateError {
        #[source]
        source: chrono::ParseError,
        data_str: String,
        campo_nome: String, // Adicionado para dar mais contexto de qual campo falhou
        arquivo: PathBuf,
        line_number: usize,
    },

    /// Erro ao fazer parse de String para u64.
    #[error(
        "Erro ao fazer parse da string '{data_str}' para U64.\n\
         Arquivo: '{arquivo:?}'\n\
         Nº da linha: {line_number}.\n\
         Campo: '{campo_nome}'\n\
         A string fornecida não pôde ser interpretada como um número inteiro.\n\
         Erro Original: {source}"
    )]
    ParseIntegerError {
        #[source]
        source: ParseIntError,
        data_str: String,
        campo_nome: String, // Adicionado para dar mais contexto de qual campo falhou
        arquivo: PathBuf,
        line_number: usize,
    },

    /// Envolve `std::num::ParseIntError` e adiciona contexto customizado (o valor que falhou o parsing).
    /// Requer mapeamento manual com `.map_err()` se o contexto for necessário.
    #[error(
        "Erro ao fazer parse de número inteiro: {1}\n\
         A string fornecida não pôde ser interpretada como um número inteiro.\n\
         Erro Original: {0}"
    )]
    ParseIntError(ParseIntError, String), // source error, value string

    /// Erro de decodificação UTF-8 e WINDOWS_1252 com contexto de arquivo/linha/IO.
    #[error(
        "Falha ao decodificar bytes em UTF-8 e em WINDOWS_1252.\n\
         Arquivo: '{arquivo:?}'\n\
         Nº da linha: {linha_num}\n\
         Erro UTF-8: {utf8_error}\n\
         Erro WINDOWS_1252: {win_1252_error}\n\
         Considere tentar outra decodificação."
    )]
    Utf8DecodeError {
        arquivo: PathBuf,
        linha_num: usize,
        utf8_error: Utf8Error,
        win_1252_error: io::Error,
    },

    /// Item (ex: período, registro, valor) não encontrado.
    #[error("Item não encontrado: {0:#?}")]
    KeyNotFound(String),

    /// Registro inválido.
    #[error(
        "Erro no campo '{campo}' no registro: {valor}\n\
         Detalhes: {detalhe:?}\n\
         Arquivo: '{arquivo:?}'\n\
         Nº da linha: {linha_num}"
    )]
    InvalidField {
        arquivo: PathBuf,
        linha_num: usize,
        campo: String,
        valor: String,
        detalhe: Option<String>,
    },

    /// Nº de campos insuficiente
    #[error(
        "Nº de campos insuficiente.\n\
         Arquivo: '{arquivo:?}'\n\
         Nº da linha: {linha_num}\n\
         Conteúdo da linha: '{line}'"
    )]
    InsufficientFieldNumber {
        arquivo: PathBuf,
        linha_num: usize,
        line: String,
    },

    /// Nº incorreto de campos
    #[error(
        "Erro: Nº incorreto de campos\n\
         Arquivo: '{arquivo:?}'\n\
         Nº da linha: {linha_num}\n\
         O Registro '{registro}' possui número incorreto de campos.\n\
         Esperado: {tamanho_esperado}\n\
         Encontrado: {tamanho_encontrado}\n"
    )]
    InvalidFieldCount {
        arquivo: PathBuf,
        linha_num: usize,
        registro: String,
        tamanho_esperado: usize,
        tamanho_encontrado: usize,
    },

    /// O período de apuração fornecido com data inválida.
    #[error(
        "Período de Apuração inválido. Data: '{year}-{month}-{day}'.\n\
        Esta data fornecida não pôde ser criada."
    )]
    InvalidDefaultPeriodoApuracao { year: i32, month: u32, day: u32 },

    /// Encontrou um tipo de registro SPED que não é suportado ou reconhecido.
    #[error(
        "Tipo de registro não suportado ou inesperado!\n\
         Registro: '{2}'\n\
         Arquivo: {0:?}\n\
         Nº da linha: {1}"
    )]
    UnsupportedRecordType(PathBuf, usize, String),

    /// Erro genérico para falhas de conversão de campo (se não cobertas por erros de parse específicos).
    #[error("Conversão de campo inválida!")]
    FieldConversion,

    /// Wrapper geral para `std::io::Error`.
    #[error("Erro de IO: {0}")]
    Io(#[from] io::Error), // source io::Error

    /// Wrapper geral para `std::io::Error` verbose.
    #[error("Erro de I/O em {path:?}: {source}")]
    InOut {
        #[source]
        source: io::Error,
        path: PathBuf,
    },

    /// Wrapper geral para `std::io::Error` verbose.
    #[error("Erro de I/O em {path:?}:[linha nº {line_number}] {source}")]
    InOutDetalhado {
        #[source]
        source: io::Error,
        path: PathBuf,
        line_number: usize,
    },

    /// Wrapper geral para `glob::GlobError`.
    #[error("Erro de Glob: {0}")]
    Glob(#[from] glob::GlobError),

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

    /// Erro para registros não definidos no arquivo EFD.
    #[error(
        "Registro '{record}' não definido conforme sped_efd.rs\n\
         Arquivo: '{arquivo:?}'\n\
         Linha nº: {linha_num}\n\
         Dados da linha: {fields:?}"
    )]
    UndefinedRecord {
        record: String,
        arquivo: PathBuf,
        linha_num: usize,
        fields: Vec<String>,
    },

    /// Erro quando um registro específico solicitado não é encontrado no arquivo.
    #[error("Registro '{0}' não encontrado.")]
    RecordNotFound(String),

    /// Erro ao tentar converter (downcast) um registro encontrado para o tipo Rust esperado.
    /// Ex: Encontrou o registro "C100", mas tentou converter para "Registro0000".
    #[error(
        "Erro de tipo: O registro '{0}' foi encontrado, mas não corresponde à struct solicitada."
    )]
    RecordCastError(String),

    /// Um catch-all para outros erros menos específicos não cobertos por variantes específicas.
    #[error("Outro erro subjacente: {0}")]
    Other(String), // Wrapped boxed error

    /// Variante para encapsular o erro com sua localização na fonte
    #[error("Erro em {file}:{line}\n=> {source}")]
    Position {
        source: Box<EFDError>,
        file: &'static str,
        line: u32,
    },
}

// Implementação de um helper para facilitar o encapsulamento
impl EFDError {
    /// O "Coração" do rastreamento: Adiciona contexto de localização ao erro.
    /// Chamado automaticamente pelos Traits de extensão.
    pub fn tag(self, loc: &'static Location<'static>) -> Self {
        EFDError::Position {
            source: Box::new(self),
            file: loc.file(),
            line: loc.line(),
        }
    }

    /// Desembrulha o erro recursivamente (útil para testes).
    pub fn flatten(self) -> Self {
        match self {
            Self::Position { source, .. } => source.flatten(),
            other => other,
        }
    }
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

/// Função auxiliar para formatar a cadeia de erros
pub fn format_error(err: &EFDError) -> String {
    match err {
        EFDError::Position { source, file, line } => {
            // Formatação: Mensagem do Erro + Localização
            format!("{}\nLocal: {}:{}", source, file, line)
        }
        _ => err.to_string(),
    }
}
