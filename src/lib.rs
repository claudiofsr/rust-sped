mod analyze_all;
mod analyze_one;
mod args;
mod dispatch_table;
mod error;
mod excel_worksheets;
mod regex;
mod sped_efd;
mod sped_registros;
mod structures;
mod traits;

mod model;
mod parser;
mod registros;

pub use self::{
    analyze_all::*, analyze_one::*, args::*, dispatch_table::*, error::*, excel_worksheets::*,
    model::*, parser::*, regex::*, registros::*, sped_efd::*, sped_registros::*,
    structures::analise_dos_creditos::*, structures::consolidacao_cst::*,
    structures::docs_fiscais::*, structures::info::*, traits::*,
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
pub const DECIMAL_ALIQ: usize = 4;
pub const NEWLINE_BYTE: u8 = b'\n';
pub const OUTPUT_DIRECTORY: &str = "novo";
pub const OUTPUT_FILENAME: &str = "Info do Contribuinte EFD Contribuicoes";
