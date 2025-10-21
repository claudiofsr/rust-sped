use chrono::{DateTime, Local};
use claudiofsr_lib::my_print;
use efd_contribuicoes::{Arguments, EFDResult, executar_programa};
use execution_time::ExecutionTime;
use std::io::Write;

/*
    cargo test --features old
    cargo test -- --nocapture
    cargo test -- --show-output split_line
    cargo run -- -h
    cargo run --features old -- -cpr 1 20
    cargo run --example run --features old examples/efd_data_random
    cargo doc --open
    cargo clippy
    rustfmt src/main.rs
    cargo b -r && cargo install --path=.
    cargo b -r && cargo install --path=. --features old
*/

fn main() -> EFDResult<()> {
    let timer = ExecutionTime::start();
    let mut args = Arguments::build()?;

    let mut write_buffer: Vec<u8> = Vec::new();
    let mut write: Box<&mut dyn Write> = Box::new(&mut write_buffer);

    if args.find {
        args.print_arquivos_efd(&mut write)?;
    } else {
        args.get_range()?;
        executar_programa(&args, &mut write)?;
    }

    let dt_local_now: DateTime<Local> = Local::now();
    writeln!(write, "Data Local: {}", dt_local_now.format("%d/%m/%Y"))?;
    writeln!(
        write,
        "Tempo de Execução Total: {}",
        timer.get_elapsed_time()
    )?;

    let output_file: String = [args.get_app_name(), "-output.txt".to_string()].concat();
    my_print(&write_buffer, output_file)?;

    Ok(())
}
