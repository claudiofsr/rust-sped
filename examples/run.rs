/*
cargo build --examples
./target/debug/examples/run examples/efd_data_random

// or

cargo build --examples --release
./target/release/examples/run examples/efd_data_random

// See the new files:
// efd_contribuicoes-output.txt
// novo/Info do Contribuinte EFD Contribuicoes.xlsx
// novo/Info do Contribuinte EFD Contribuicoes.csv
*/

use claudiofsr_lib::{blake3_hash, my_print};
use efd_contribuicoes::{AppConfig, EFDResult, OUTPUT_DIRECTORY, executar_programa};
use rust_decimal_macros::dec;
use std::{env, io::Write, path::PathBuf};

fn main() -> EFDResult<()> {
    // cargo test -- --show-output test_executar_programa

    let path = env::args()
        .nth(1)
        .expect("supply a single path as the program argument");

    let arquivo: PathBuf = PathBuf::from(path);

    let mut write_buffer: Vec<u8> = vec![];
    let mut write: Box<&mut dyn Write> = Box::new(&mut write_buffer);

    let config = AppConfig {
        all_files: vec![arquivo.clone()],
        print_csv: true,
        ..Default::default()
    };

    let all_lines = executar_programa(&config, &mut write)?;
    // println!("all_lines:\n{all_lines:#?}");

    let output_name = [&config.app_name, "-output.txt"].concat();
    let output_file: PathBuf = PathBuf::from(output_name);

    my_print(&write_buffer, output_file.clone())?;

    let csv_file: PathBuf = [
        OUTPUT_DIRECTORY,
        "Info do Contribuinte EFD Contribuicoes.csv",
    ]
    .iter()
    .collect();

    let arq_file_hash = blake3_hash(&arquivo)?;
    let out_file_hash = blake3_hash(&output_file)?;
    let csv_file_hash = blake3_hash(&csv_file)?;

    assert_eq!(all_lines[0].descr_item, "MANTER DE 50ºC À 90ºC".into());
    assert_eq!(
        all_lines[1].descr_item,
        "“ASPAS”, SÍMBOLO EUROPEU (€) E TRAÇOS FANTASIA (– E —)".into()
    );
    assert_eq!(all_lines[20].num_linha_efd, Some(115));
    assert_eq!(all_lines[20].registro, "C170".into());
    assert_eq!(
        all_lines[20].chave_doc,
        "90-0315-29.446.564/7701-69-19-048-060.204.494-849.351.589-8".into()
    );
    assert_eq!(all_lines[20].valor_item, Some(dec!(38752.0)));
    assert_eq!(all_lines[40].num_linha_efd, Some(178));
    assert_eq!(all_lines[40].valor_item, Some(dec!(44818.0)));
    assert_eq!(
        "2612118ea298d2365a30808e9e2227a7c210d9e91e6580ec3efc6ef875ca35c7",
        arq_file_hash
    );
    assert_eq!(
        "99cb92fe2e10073b9a560752c4a4b9da80bb7aca794a293cad76da81ae486e45",
        out_file_hash
    );
    assert_eq!(
        "8f575fbb3a2b710749d2e875b221a1d233aed840359be8dad20bc821a7703641",
        csv_file_hash
    );

    Ok(())
}
