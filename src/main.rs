use chrono::{DateTime, Local};
use clap::Parser;
use claudiofsr_lib::my_print;
use colored::*;
use efd_contribuicoes::{
    AppConfig, Arguments, DATE_FORMAT, EFDResult, executar_programa, format_error,
};
use execution_time::ExecutionTime;
use log::{LevelFilter, debug};
use std::{io::Write, process::ExitCode};

/*
Como armazenar as strings de forma mais eficiente possível nos registros?

Os registros parseam as linhas que segue o padrao:
|C100|0|1|865322|01|00|002|16798||15012018|26012018|2541,39|0|||2541,39|9||0|||||||25,52|117,57|0|0|

Após o parse, sobre os itens dos registros são executados alguams lógicas de negocios.
Ao final, estas informações são padronizadas em Excel e CSV e escritas em arquivos finais.

    # uso de find + sed para alterar variáveis dos muitos registros.
    # exemplo: impl_sped_record_trait para impl_reg_methods
    find ./src/blocos -type f -name "*.rs" -exec sed -i -E 's/impl_sped_record_trait/impl_reg_methods/' {} +

    start='let cst_cofins = fields\.get\(([0-9]+)\)\.parse_opt\(\);'
    final='let cst_cofins = fields.get(\1).to_efd_field(file_path, line_number, "CST_COFINS")?;'

    start='pub cst_cofins: Option<u16>'
    final='pub cst_cofins: Option<CodigoSituacaoTributaria>'

    find ./src/blocos/bloco_[a-z1-9] -type f -name "*.rs" -exec sed -i -E "s/$start/$final/" {} +

    # Para os Imports
    start='use crate::{'
    final='use crate::{ResultExt, '

    # Se não houver ResultExt, substitui start por final
    find ./src/blocos -type f -name "registro_*.rs" -exec sed -i "/ResultExt/ ! s/$start/$final/" {} +

    # Para os Erros
    # Procura o bloco do erro e, se não tiver .loc(), adiciona no fechamento });

    context='Err(EFDError'
    start='});'
    final='}).loc();'

    find ./src/blocos -type f -name "registro_*.rs" -exec sed -i "/$context/,/$start/ { /\.loc()/ ! s/$start/$final/ }" {} +

    cargo test -- --nocapture
    cargo test -- --show-output split_line
    cargo run -- -h
    cargo run -- -cpr 1 20
    cargo run --example run examples/efd_data_random
    cargo doc --open
    cargo clippy
    rustfmt src/main.rs
    cargo b -r && cargo install --path=.

    # Para ver os avisos (warn) e informações (info)
    RUST_LOG=warn cargo test -- --nocapture | rg warn

    RUST_LOG=trace cargo run

    # Teste de Debug:
    cargo run -- -d -f
*/

fn main() -> ExitCode {
    let args = Arguments::parse();
    setup_logger(args.debug);

    // ExitCode em vez de `process::exit`: o retorno normal permite que os
    // destrutores (flush de buffers, barras de progresso) executem antes de sair.
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            log::error!("\n\n{}\n", format_error(&error).red().bold());
            ExitCode::FAILURE
        }
    }
}

fn run(args: &Arguments) -> EFDResult<()> {
    args.start()?;

    let timer = ExecutionTime::start();
    let mut buffer = Vec::new();

    // 1. Transformar argumentos em Configuração (Busca e Valida arquivos)
    // Isso centraliza o IO de busca no início da execução
    let config = AppConfig::try_from_args(args)?;

    if config.debug {
        debug!("Modo DEBUG ativado! Detalhes técnicos serão exibidos.");
    }

    // Se o usuário usou a flag -f (find), apenas listamos e encerramos.
    // Caso contrário, executamos a análise completa
    if args.find {
        config.print_summary(&mut buffer)?;
    } else {
        executar_programa(&config, &mut buffer)?;
    }

    // 3. Relatório de Finalização
    finalizar_execucao(&config, timer, &mut buffer)?;

    Ok(())
}

/// Configura o env_logger com filtros e formatos customizados.
fn setup_logger(debug_mode: bool) {
    // Configuração do Logger
    let mut builder = env_logger::Builder::from_default_env();

    // Customização do formato para o padrão do Brasil
    builder.format(|buf, record| {
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

    if debug_mode {
        builder.filter_level(LevelFilter::Debug);
    } else if std::env::var("RUST_LOG").is_err() {
        builder.filter_level(LevelFilter::Info);
    }

    builder.init(); // <-- O Logger só "nasce" aqui
}

/// Gera o rodapé do relatório, imprime no console e salva em arquivo.
fn finalizar_execucao(
    config: &AppConfig,
    timer: ExecutionTime,
    buffer: &mut Vec<u8>,
) -> EFDResult<()> {
    let output_file = format!("{}-output.txt", config.app_name);
    let dt_now: DateTime<Local> = Local::now();

    writeln!(buffer, "Write EFD Output file: {:?}\n", output_file)?;
    writeln!(buffer, "Data Local: {}", dt_now.format(DATE_FORMAT))?;
    writeln!(
        buffer,
        "Tempo de Execução Total: {}",
        timer.get_elapsed_time()
    )?;

    // claudiofsr_lib::my_print: Envia para stdout e para o arquivo simultaneamente
    my_print(buffer, output_file)?;

    Ok(())
}
