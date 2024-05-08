mod args;
mod sped_efd;
mod analyze_all;
mod analyze_one;
mod dispatch_table;
mod regex;
mod sped_registros;
mod structures;
mod excel_worksheets;

pub use self::{
    args::*,
    sped_efd::*,
    analyze_all::*,
    analyze_one::*,
    dispatch_table::*,
    regex::*,
    sped_registros::*,
    structures::analise_dos_creditos::*,
    structures::docs_fiscais::*,
    structures::consolidacao_cst::*,
    structures::info::*,
    structures::traits::*,
    excel_worksheets::*,
};

// https://crates.io/crates/cfg-if
cfg_if::cfg_if! {
    if #[cfg(feature = "old")] {
        mod excel_alternative;
        pub use excel_alternative::*;
    } else {
        // default: "new"
        mod excel_workbook;
        pub use excel_workbook::*;
    }
}

pub const ZERO: f64 = 0.00000000;
pub const SMALL_VALUE: f64 = 0.009; // usado em fn despise_small_values; menor que um centavo
pub const DELIMITER_CHAR: char = '|';
pub const ALIQ_BASICA_PIS: f64 = 1.65;
pub const ALIQ_BASICA_COF: f64 = 7.60;
pub const DECIMAL_VALOR: usize = 2;
pub const DECIMAL_ALIQ:  usize = 4;
pub const NEWLINE_BYTE: u8 = b'\n';
pub const OUTPUT_DIRECTORY: &str = "novo";
pub const OUTPUT_FILENAME: &str = "Info do Contribuinte EFD Contribuicoes";

pub type MyError = Box<dyn std::error::Error + Send + Sync>;
pub type MyResult<T> = Result<T, MyError>;