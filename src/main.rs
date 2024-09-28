use std::{
    error::Error,
    time::Instant,
    io::Write,
};
use chrono::{
    DateTime,
    Local
};
use claudiofsr_lib::my_print;
use efd_contribuicoes::{
    Arguments,
    executar_programa,
};

/*
    clear && cargo test -- --nocapture
    clear && cargo run -- -h
    clear && cargo run -- -cpr 1 20
    cargo doc --open
    cargo clippy
    cargo b -r && cargo install --path=.
    cargo b -r && cargo install --path=. --features old
*/

fn main() -> Result<(), Box<dyn Error>> {
    let time = Instant::now();
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
    writeln!(write, "Tempo de Execução Total: {:?}",time.elapsed())?;

    let output_file: String = [args.get_app_name(), "-output.txt".to_string()].concat();
    my_print(&write_buffer, output_file)?;

    Ok(())
}