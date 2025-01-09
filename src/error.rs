use std::{
    error::Error,
    io::Error as io_Error,
    num::{ParseFloatError, ParseIntError},
    path::PathBuf,
    str::Utf8Error,
};

#[derive(Debug)]
pub enum EFDError {
    InvalidFile(String, PathBuf, Box<dyn Error>),

    InvalidCNPJ(String, usize),

    InvalidName(String, usize),

    InvalidStyle,

    InvalidPA,

    // We will defer to the parse error implementation for their error.
    // Supplying extra info requires adding more data to the type.
    // <https://doc.rust-lang.org/rust-by-example/error/multiple_error_types/wrap_error.html>
    ParseFloatError(ParseFloatError),

    ParseIntError(ParseIntError, String),

    /// Error utf8 decoding.
    Utf8DecodeError(PathBuf, usize, Utf8Error, io_Error),

    InvalidFormat,
    InvalidDate,
    NotFound,
}

impl std::fmt::Display for EFDError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            EFDError::InvalidFile(name, arquivo, error) => writeln!(
                f,
                "
                Erro ao executar a função: {name}\n\
                Arquivo inválido!\n\
                Arquivo: {a}\n\
                Error: {error}\n\
                ",
                a = arquivo.display()
            ),
            EFDError::InvalidCNPJ(filename, num) => writeln!(
                f,
                "
                Erro ao executar a função: parse_file_info()\n\
                Não foi encontrado o CNPJ do estabelecimento.\n\
                Arquivo: {filename}\n\
                linha nº: {num}\n\
                "
            ),
            EFDError::InvalidName(filename, num) => writeln!(
                f,
                "
                Erro ao executar a função: parse_file_info()\n\
                Não foi encontrado o Nome do estabelecimento.\n\
                Arquivo: {filename}\n\
                linha nº: {num}\n\
                "
            ),
            EFDError::InvalidStyle => writeln!(f, "Invalid Style!"),
            EFDError::InvalidPA => writeln!(f, "Período de Apuração Inválido!"),
            EFDError::ParseFloatError(error) => writeln!(
                f,
                "
                Erro ao executar a função: formatar_casas_decimais()\n\
                The provided string could not be parsed as float.\n\
                Error: {error}\n\
                "
            ),
            EFDError::ParseIntError(error, msg) => writeln!(
                f,
                "
                {msg}
                The provided string could not be parsed as Integer.\n\
                Error: {error}\n\
                "
            ),
            EFDError::Utf8DecodeError(path, line_number, error1, error2) => writeln!(
                f,
                "
                Erro ao executar a função: get_string_utf8()\n\
                Try to decode the bytes as UTF-8!\n\
                File: {path:?}\n\
                Line nº {line_number}\n\
                Used encoding type: WINDOWS_1252.\n\
                Failed to convert data from WINDOWS_1252 to UTF-8!\n\
                Utf8Error: {error1}\n\
                DecodeError: {error2}\n\
                Try another encoding type!\n\
                "
            ),
            EFDError::InvalidFormat => write!(f, "Formato MMYYYY inválido"),
            EFDError::InvalidDate => write!(f, "Data inválida"),
            EFDError::NotFound => write!(f, "Período de apuração não encontrado"),
        }
    }
}

/// If we want to use std::error::Error in main, we need to implement it for EFDError
impl std::error::Error for EFDError {}

// Implement the conversion from `ParseFloatError` to `EFDError`.
// This will be automatically called by `?` if a `ParseFloatError`
// needs to be converted into a `EFDError`.
// <https://doc.rust-lang.org/rust-by-example/error/multiple_error_types/wrap_error.html>
impl From<ParseFloatError> for EFDError {
    fn from(err: ParseFloatError) -> EFDError {
        EFDError::ParseFloatError(err)
    }
}

#[derive(Debug)]
pub enum SpedError {
    InvalidLine(String),
    InvalidLenght(String, usize),
    FieldConversion,
    FileNotFound,
    UnsupportedRecordType(String),
    Io(std::io::Error),
    Utf8(std::str::Utf8Error),
}

impl std::fmt::Display for SpedError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SpedError::InvalidLine(line) => writeln!(f, "Invalid Line!\nLine: {line:#?}"),
            SpedError::InvalidLenght(reg, size) => writeln!(f, "Invalid Lenght!\nRegistro: {reg}\nLenght: {size}"),
            SpedError::FieldConversion => writeln!(f, "Invalid Field Conversion!"),
            SpedError::FileNotFound => writeln!(f, "File Not Found!"),
            SpedError::UnsupportedRecordType(registro) => writeln!(f, "Unsupported Record Type!\nRegistro: {registro}"),
            SpedError::Io(error) => writeln!(f, "Io Error: {error}"),
            SpedError::Utf8(utf8_error) => writeln!(f, "Utf8 Error: {utf8_error}"),
        }
    }
}

/// If we want to use std::error::Error in main, we need to implement it for SpedError
impl std::error::Error for SpedError {}

impl From<std::io::Error> for SpedError {
    fn from(err: std::io::Error) -> Self {
        SpedError::Io(err)
    }
}

impl From<std::str::Utf8Error> for SpedError {
    fn from(err: std::str::Utf8Error) -> Self {
        SpedError::Utf8(err)
    }
}