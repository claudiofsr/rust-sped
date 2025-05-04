use glob::PatternError;
use rust_xlsxwriter::XlsxError;
use std::{
    fmt::{self, Display},
    io,
    num::{ParseFloatError, ParseIntError, TryFromIntError},
    path::PathBuf,
    str::Utf8Error,
};

/// Define a common boxed trait object error type for flexibility.
/// Useful for interfacing with libraries returning Box<dyn Error + Send + Sync + 'static>.
pub type MyError = Box<dyn std::error::Error + Send + Sync + 'static>;

/// A specialized result type for operations that can return an `EFDError`.
pub type EFDResult<T> = Result<T, EFDError>;

/// Enumerates all possible errors that can occur during EFD SPED processing.
#[derive(Debug)]
pub enum EFDError {
    /// File/line specific format or content error: Invalid CNPJ found.
    InvalidCNPJ(String, usize), // filename, line_number
    /// File/line specific format or content error: Invalid company name found.
    InvalidName(String, usize), // filename, line_number
    /// Error related to an invalid output style or format setting.
    InvalidStyle,
    /// Specific validation or format error for the period (PA).
    InvalidPA,
    /// General format validation error (e.g., expected MMYYYY format).
    InvalidFormat,
    /// General date validation or parsing error.
    InvalidDate,
    /// Wraps `std::num::ParseFloatError`.
    ParseFloatError(ParseFloatError),
    /// Wraps `std::num::ParseIntError` and adds custom context (the value that failed parsing).
    /// Requires manual mapping with `.map_err()` if context is needed;
    /// a simple `From<ParseIntError>` cannot capture the value automatically.
    ParseIntError(ParseIntError, String), // source error, value string
    /// Detailed error for UTF-8 decoding issues with file/line/io context.
    Utf8DecodeError(PathBuf, usize, Utf8Error, io::Error), // path, line_number, Utf8Error, io::Error context
    /// Item (e.g., period, record, value) not found.
    NotFound,
    /// An entire line from a file was invalid, carrying the line content.
    InvalidLine(String), // line content
    /// A specific record type had an unexpected length.
    InvalidLength(String, usize), // record name, actual length
    /// Encountered a SPED record type that is not supported or recognized.
    UnsupportedRecordType(String), // record type found
    /// Generic error for field conversion failures (if not covered by specific parse errors).
    FieldConversion,
    /// General wrapper for `std::io::Error`.
    Io(io::Error), // source io::Error
    /// Wraps `rust_xlsxwriter::XlsxError`.
    XlsxError(XlsxError), // source XlsxError
    /// Wraps `csv::Error`.
    CsvError(csv::Error), // source csv::Error
    /// Wraps `std::num::TryFromIntError`.
    TryFromIntError(TryFromIntError), // source TryFromIntError
    /// Wraps `glob::PatternError`.
    PatternError(PatternError), // source PatternError
    /// Captures any boxed error (`MyError`) that doesn't specifically match another `EFDError` variant.
    /// This preserves the original error's type and source chain when converting from MyError.
    Other(MyError), // Wrapped boxed error
}

// Implement Display for user-facing messages.
impl Display for EFDError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EFDError::InvalidCNPJ(filename, num) => write!(
                f,
                "Error processing file info: Failed to find establishment CNPJ.\n\
                File: {}\nLine nº: {}\n",
                filename, num
            ),
            EFDError::InvalidName(filename, num) => write!(
                f,
                "Error processing file info: Failed to find establishment Name.\n\
                File: {}\nLine nº: {}\n",
                filename, num
            ),
            EFDError::InvalidStyle => write!(f, "Invalid Style!"),
            EFDError::InvalidPA => write!(f, "Invalid Period (PA)!"),
            EFDError::ParseFloatError(error) => write!(
                f,
                "Error parsing float: The provided string could not be parsed as float.\n\
                Source Error: {}\n",
                error
            ),
            EFDError::ParseIntError(error, msg) => write!(
                f,
                "Error parsing integer: {}\n\
                The provided string could not be parsed as integer.\n\
                Source Error: {}\n",
                msg, error
            ),
            EFDError::Utf8DecodeError(path, line_number, utf8_error, io_error) => write!(
                f,
                "UTF-8 decode error in file '{:?}' at line {}:\n\
                Failed to decode bytes as UTF-8.\n\
                Attempted encoding (likely): WINDOWS-1252.\n\
                Utf8 Error: {}\n\
                IO Error Context: {}\n\
                Consider trying another encoding.",
                path, line_number, utf8_error, io_error
            ),
            EFDError::InvalidFormat => write!(f, "Invalid Format (expected MMYYYY or similar)"),
            EFDError::InvalidDate => write!(f, "Invalid Date"),
            EFDError::NotFound => write!(f, "Item not found (e.g., Period not found)"),
            EFDError::InvalidLine(line) => write!(f, "Invalid Line!\nLine: {:#?}\n", line),
            EFDError::InvalidLength(reg, size) => {
                write!(f, "Invalid Length!\nRecord: {}\nLength: {}\n", reg, size)
            }
            EFDError::FieldConversion => write!(f, "Invalid Field Conversion!"),
            EFDError::UnsupportedRecordType(registro) => {
                write!(f, "Unsupported Record Type!\nRecord: {}\n", registro)
            }
            EFDError::Io(error) => write!(f, "IO Error: {}", error),
            EFDError::XlsxError(error) => write!(f, "Xlsx Error: {}", error),
            EFDError::CsvError(error) => write!(f, "CSV Error: {}", error),
            EFDError::TryFromIntError(error) => write!(f, "TryFromInt Error: {}", error),
            EFDError::PatternError(error) => write!(f, "Pattern Error: {}", error),
            EFDError::Other(error) => write!(f, "Other underlying error: {}", error),
        }
    }
}

// Implement std::error::Error for proper error propagation and chaining.
impl std::error::Error for EFDError {}

// --- Conversions FROM specific foreign error types INTO EFDError ---

/// Converts a `MyError` (Box<dyn std::error::Error + Send + Sync + 'static>) into `EFDError`.
///
/// It attempts to downcast the boxed error to a concrete `EFDError`. If successful, it
/// unboxes and returns the original `EFDError`. If not, it wraps the original `MyError`
/// in the `EFDError::Other` variant.
impl From<MyError> for EFDError {
    fn from(err: MyError) -> Self {
        // Attempt to downcast the MyError (Box<dyn Error + ...>) to a concrete EFDError.
        match err.downcast::<EFDError>() {
            Ok(boxed_efd_err) => {
                // If the original error inside the box *was* an EFDError, unbox it and return.
                *boxed_efd_err
            }
            Err(original_err) => {
                // If it was not an EFDError, wrap the original boxed error in the Other variant.
                EFDError::Other(original_err)
            }
        }
    }
}

// Converts a raw io::Error into the EFDError::Io variant.
impl From<io::Error> for EFDError {
    fn from(err: io::Error) -> Self {
        EFDError::Io(err)
    }
}

impl From<csv::Error> for EFDError {
    fn from(err: csv::Error) -> Self {
        EFDError::CsvError(err)
    }
}

impl From<XlsxError> for EFDError {
    fn from(err: XlsxError) -> Self {
        EFDError::XlsxError(err)
    }
}

impl From<PatternError> for EFDError {
    fn from(err: PatternError) -> Self {
        EFDError::PatternError(err)
    }
}

impl From<TryFromIntError> for EFDError {
    fn from(err: TryFromIntError) -> Self {
        EFDError::TryFromIntError(err)
    }
}

/*
/// Macro to implement From<$src_error> for EFDError variants
macro_rules! impl_from_efd {
    // Pattern for variants with just a `source` field
    ($src_error:ty, $efd_variant:ident) => {
        impl From<$src_error> for EFDError {
            fn from(err: $src_error) -> Self {
                EFDError::$efd_variant(err)
            }
        }
    };
}

impl_from_efd!(io::Error, Io);
impl_from_efd!(csv::Error, CsvError);
impl_from_efd!(XlsxError, XlsxError);
impl_from_efd!(PatternError, PatternError);
impl_from_efd!(TryFromIntError, TryFromIntError);
*/
