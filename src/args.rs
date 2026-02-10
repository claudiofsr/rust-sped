use clap::{CommandFactory, Parser};
use clap_complete::{Generator, Shell, generate};
use claudiofsr_lib::clear_terminal_screen;
use std::{
    io,
    process, // process::exit(1)
    str,
};

use crate::EFDResult;

/// Define os estilos de cores para o terminal (Interface Moderna).
///
/// <https://stackoverflow.com/questions/74068168/clap-rs-not-printing-colors-during-help>
pub fn get_styles() -> clap::builder::Styles {
    // colors
    let yellow = anstyle::Color::Ansi(anstyle::AnsiColor::Yellow);
    let cyan = anstyle::Color::Ansi(anstyle::AnsiColor::Cyan);
    let green = anstyle::Color::Ansi(anstyle::AnsiColor::Green);

    clap::builder::Styles::styled()
        .placeholder(anstyle::Style::new().fg_color(Some(yellow)))
        .usage(anstyle::Style::new().fg_color(Some(cyan)).bold())
        .header(
            anstyle::Style::new()
                .fg_color(Some(cyan))
                .bold()
                .underline(),
        )
        .literal(anstyle::Style::new().fg_color(Some(green)))
}

/// Template para customizar a exibição do Help.
const APPLET_TEMPLATE: &str = "\
{before-help}
{about}
{author-with-newline}
{usage-heading} {usage}

{all-args}
{after-help}";

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about,
    long_about = "Lê, processa e consolida múltiplos arquivos TXT do SPED, \
    gerando relatórios em Excel e CSV.",
    next_line_help = true,
    help_template = APPLET_TEMPLATE,
    styles = get_styles(),
)]
pub struct Arguments {
    /// Limpa a tela do terminal antes de apresentar a análise.
    ///
    /// Clear the terminal screen before presenting the analysis of EFD files.
    #[arg(short('c'), long("clear_terminal"), default_value_t = false)]
    pub clear_terminal: bool,

    /// Ativar mensagens de debug (ex: detalhes de correlações do Bloco M).
    #[arg(short = 'd', long)]
    pub debug: bool,

    /// Exclui itens de operações de SAÍDA dos arquivos finais (Excel/CSV).
    ///
    /// Delete output operations items from Excel and CSV files.
    ///
    /// Ou seja, não imprimir nos arquivos finais as operações em que:
    ///
    /// Tipo de Operação = Saídas
    #[arg(
        short,
        long,
        value_parser,
        verbatim_doc_comment,
        default_value_t = false
    )]
    pub excluir_saidas: bool,

    /// Excluir CST 49 do Rateio da Receita Bruta.
    ///
    /// Ou seja, restringir o intervalo de '1 <= CST <= 49'
    /// para '1 <= CST <= 9' no cálculo da Receita Bruta utilizada
    /// no rateio dos créditos.
    #[arg(
        short('t'),
        long,
        value_parser,
        verbatim_doc_comment,
        default_value_t = false
    )]
    pub excluir_cst_49: bool,

    /// Listar arquivos SPED EFD encontrados no diretório atual.
    ///
    /// Find SPED EFD files.
    #[arg(
        short,
        long,
        value_parser,
        verbatim_doc_comment,
        default_value_t = false
    )]
    pub find: bool,

    /**
    Gera o arquivo de auto-complete para o shell especificado (bash, zsh, fish, etc).

    If provided, outputs the completion file for given shell.

    Usage example with Z-shell ou Zsh:

    #### Example (as root):

    Generate completions to efd_contribuicoes.

    Visible to all system users.

    ```console

    mkdir -p /usr/local/share/zsh/site-functions

    efd_contribuicoes -g zsh > /usr/local/share/zsh/site-functions/_efd_contribuicoes

    rustup completions zsh > /usr/local/share/zsh/site-functions/_rustp

    rustup completions zsh cargo > /usr/local/share/zsh/site-functions/_cargo

    compinit && zsh

    ```

    See `rustup completions` for detailed help.
    */
    #[arg(short('g'), long("generate"), value_enum)]
    pub generator: Option<Shell>,

    /// Retém apenas itens que geram crédito (50 <= CST <= 66).
    ///
    /// Retain only credit entries (50 <= CST <= 66)
    ///
    /// Reter apenas itens de operações de crédito em arquivos Excel e CSV.
    ///
    /// Ou seja, imprimir nos arquivos finais itens de operações com alguna
    ///
    /// Natureza da Base de Cálculo.
    #[arg(
        short,
        long,
        value_parser,
        verbatim_doc_comment,
        default_value_t = false
    )]
    pub operacoes_de_creditos: bool,

    /// Gerar arquivo CSV.
    ///
    /// Print CSV (Comma Separated Values) file.
    #[arg(
        short('p'),
        long("csv"),
        value_parser,
        verbatim_doc_comment,
        default_value_t = false,
        requires = "range"
    )]
    pub print_csv: bool,

    /// Desativar a criação da planilha Excel (habilitada por padrão).
    ///
    /// Do NOT generate the Excel (.xlsx) file.
    ///
    /// Por padrão, o programa sempre gera o arquivo Excel.
    /// Use esta opção para desativar essa geração.
    #[arg(
        short,
        long,
        verbatim_doc_comment,
        default_value_t = false,
        requires = "range"
    )]
    pub no_excel: bool,

    /// Selecione arquivos SPED EFD para analisar especificando o intervalo
    ///
    /// Select SPED EFD files to analyze by specifying the range.
    ///
    /// Seja N o número de arquivos SPED EFD encontrados no diretório.
    ///
    /// Para analisar apenas um arquivo X (número inteiro de 1 a N) digite a opção: --range X ou -r X.
    ///
    /// Para analisar arquivos de X a Y (inteiros de 1 a N) digite a opção: --range X Y ou -r X Y.
    ///
    /// Exemplo: '-r 1' (apenas o primeiro), '-r 1 5' (do primeiro ao quinto).
    #[arg(short, long, value_parser, verbatim_doc_comment, required = false, num_args = 1..=2)]
    pub range: Option<Vec<usize>>,
}

impl Arguments {
    /// Executa comandos iniciais
    pub fn start(&self) -> EFDResult<()> {
        if let Some(generator) = self.generator {
            self.print_completions(generator);
        }

        if self.clear_terminal {
            clear_terminal_screen();
        }

        Ok(())
    }

    /// Retorna o nome do executável definido no Cargo.toml.
    pub fn get_app_name(&self) -> String {
        Arguments::command().get_name().to_string()
    }

    /// Print shell completions to standard output
    pub fn print_completions<G>(&self, r#gen: G)
    where
        G: Generator + std::fmt::Debug,
    {
        let mut cmd = Arguments::command();
        let cmd_name = cmd.get_name().to_string();
        let mut stdout = io::stdout();

        log::info!("Gerando arquivo de auto-complete para {gen:?}...");
        generate(r#gen, &mut cmd, cmd_name, &mut stdout);
        process::exit(0);
    }

    /// Exibe a mensagem de ajuda padrão do clap.
    pub fn print_help_msg(&self) -> EFDResult<()> {
        let mut cmd = Arguments::command();
        cmd.print_help()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests_args {
    // cargo test -- --help
    // cargo test -- --nocapture
    // cargo test -- --show-output
    use super::*;

    /// Verifica se a estrutura do CLI está íntegra e sem conflitos de nomes/flags.
    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Arguments::command().debug_assert()
    }
}
