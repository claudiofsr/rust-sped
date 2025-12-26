use chrono::{DateTime, Local};
use claudiofsr_lib::my_print;
use efd_contribuicoes::{Arguments, EFDResult, executar_programa};
use execution_time::ExecutionTime;
use log::debug;
use std::{io::Write, process};

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

fn main() {
    // Padrão idiomático para lidar com erros no main
    if let Err(error) = run() {
        eprintln!("Operation failed:");
        eprintln!("Error: {error}");
        process::exit(1);
    }
    // O Rust retorna exit code 0 automaticamente ao chegar ao fim do main
}

fn run() -> EFDResult<()> {
    let timer = ExecutionTime::start();
    let mut args = Arguments::build()?;
    let mut buffer = Vec::new();

    // --- INÍCIO DA CONFIGURAÇÃO DO LOGGER ---
    // Define o nível de log: Se args.debug for true, usa "debug", senão usa "info" (ou "warn")
    let log_level = if args.debug {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };

    // Inicializa o logger com formatação personalizada (opcional) ou padrão
    env_logger::builder()
        .filter_level(log_level)
        .format_timestamp(None) // Opcional: remove timestamp para limpar a saída
        .init();
    // --- FIM DA CONFIGURAÇÃO DO LOGGER ---

    // Exemplo de uso
    if args.debug {
        debug!("Modo DEBUG ativado!");
    }

    // --- FINAL DA CONFIGURAÇÃO DO LOGGER ---

    if args.find {
        args.print_arquivos_efd(&mut buffer)?;
    } else {
        args.get_range()?;
        executar_programa(&args, &mut buffer)?;
    }

    let dt_local_now: DateTime<Local> = Local::now();
    writeln!(buffer, "Data Local: {}", dt_local_now.format("%d/%m/%Y"))?;
    writeln!(
        buffer,
        "Tempo de Execução Total: {}",
        timer.get_elapsed_time()
    )?;

    let output_file = format!("{}-output.txt", args.get_app_name());
    my_print(&buffer, output_file)?;

    Ok(())
}
