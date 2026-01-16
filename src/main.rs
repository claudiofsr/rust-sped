use chrono::{DateTime, Local};
use clap::Parser;
use claudiofsr_lib::my_print;
use efd_contribuicoes::{Arguments, DATE_FORMAT, EFDResult, executar_programa};
use execution_time::ExecutionTime;
use log::{LevelFilter, debug};
use std::{io::Write, process};

/*
Como armazenar as strings de forma mais eficiente possível nos registros?

Os registros parseam as linhas que segue o padrao:
|C100|0|1|865322|01|00|002|16798||15012018|26012018|2541,39|0|||2541,39|9||0|||||||25,52|117,57|0|0|

Após o parse, sobre os itens dos registros são executados alguams lógicas de negocios.
Ao final, estas informações são padronizadas em Excel e CSV e escritas em arquivos finais.

    # uso de find + sed para alterar variáveis dos muitos registros.
    # exemplo: impl_sped_record_trait para impl_reg_methods
    find ./src/blocos -type f -name "*.rs" -exec sed -i -E 's/impl_sped_record_trait/impl_reg_methods/' {} +

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

    # Para ver os avisos (warn) e informações (info)
    RUST_LOG=warn cargo test -- --nocapture | rg warn

    RUST_LOG=trace cargo run

    # Teste de Debug:
    cargo run -- -d -f
*/

fn main() -> EFDResult<()> {
    // 1. Parse dos argumentos (Rápido e sem efeitos colaterais)
    // Build Arguments struct
    let args = Arguments::parse();

    // 2. Configuração do Logger
    let mut builder = env_logger::Builder::from_default_env();

    // Customização do formato para o padrão do Brasil
    builder.format(|buf, record| {
        use std::io::Write;

        // Data formatada como DD/MM/YYYY HH:MM:SS
        let ts = Local::now().format(DATE_FORMAT);

        writeln!(
            buf,
            "[{ts} {:5} {}] {}",
            record.level(),
            record.target(),
            record.args()
        )
    });

    if args.debug {
        builder.filter_level(LevelFilter::Debug);
    } else if std::env::var("RUST_LOG").is_err() {
        builder.filter_level(LevelFilter::Info);
    }
    builder.init(); // <-- O Logger só "nasce" aqui

    // 3. Execução do programa passando os argumentos já processados
    // Padrão idiomático para lidar com erros no main
    if let Err(error) = run(args) {
        log::error!("\n[ERRO FATAL] {error}");
        process::exit(1);
    }

    // O Rust retorna exit code 0 automaticamente ao chegar ao fim do main
    Ok(())
}

fn run(mut args: Arguments) -> EFDResult<()> {
    let timer = ExecutionTime::start();
    let mut buffer = Vec::new();

    args.build()?;

    if args.debug {
        debug!("Modo DEBUG ativado! Detalhes técnicos serão exibidos.");
    }

    // Lógica de busca ou execução
    if args.find {
        args.print_arquivos_efd(&mut buffer)?;
    } else {
        args.get_range()?;
        executar_programa(&args, &mut buffer)?;
    }

    // Imprimir relatório de saída
    let output_file = format!("{}-output.txt", args.get_app_name());
    let dt_local_now: DateTime<Local> = Local::now();

    writeln!(buffer, "Write EFD Output file: {:?}\n", output_file)?;
    writeln!(buffer, "Data Local: {}", dt_local_now.format(DATE_FORMAT))?;
    writeln!(
        buffer,
        "Tempo de Execução Total: {}",
        timer.get_elapsed_time()
    )?;

    // Imprime no console e salva no arquivo de output
    my_print(&buffer, output_file)?;
    Ok(())
}
