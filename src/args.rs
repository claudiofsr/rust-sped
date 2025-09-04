use clap::{CommandFactory, Parser};
use clap_complete::{Generator, Shell, generate};
use claudiofsr_lib::clear_terminal_screen;
use colored::*;
use glob::{MatchOptions, glob_with};
use std::{
    cmp::Ordering,
    io::{self, Write},
    path::PathBuf,
    process, // process::exit(1)
    str,
};

use crate::EFDResult;

// https://stackoverflow.com/questions/74068168/clap-rs-not-printing-colors-during-help
pub fn get_styles() -> clap::builder::Styles {
    clap::builder::Styles::styled()
        .placeholder(
            anstyle::Style::new().fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Yellow))),
        )
        .usage(
            anstyle::Style::new()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Cyan)))
                .bold(),
        )
        .header(
            anstyle::Style::new()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Cyan)))
                .bold()
                .underline(),
        )
        .literal(
            anstyle::Style::new().fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Green))),
        )
}

// https://docs.rs/clap/latest/clap/struct.Command.html#method.help_template
const APPLET_TEMPLATE: &str = "\
{before-help}
{about}
{author-with-newline}
{usage-heading} {usage}

{all-args}
{after-help}";

#[derive(Parser, Debug)]
#[command(
    author, version, about, long_about = None,
    trailing_var_arg = true, next_line_help = true,
    help_template = APPLET_TEMPLATE,
    styles=get_styles(),
)]
pub struct Arguments {
    #[arg(short, long, value_parser, verbatim_doc_comment, hide = true)]
    pub all_files: Vec<PathBuf>,

    /// Clear the terminal screen before presenting the analysis of EFD files.
    #[arg(short('c'), long("clear_terminal"), default_value_t = false)]
    // action = ArgAction::SetTrue
    pub clear_terminal: bool,

    /// Delete output operations items from Excel and CSV files.
    ///
    /// Exclua itens de operações de saída de arquivos Excel e CSV.
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

    /// Find SPED EFD files
    #[arg(
        short,
        long,
        value_parser,
        verbatim_doc_comment,
        default_value_t = false
    )]
    pub find: bool,

    /**
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

    /// Print CSV (Comma Separated Values) file.
    ///
    /// Para imprimir o arquivo .csv, adicione a opção: --print_csv ou -p
    #[arg(
        short,
        long,
        value_parser,
        verbatim_doc_comment,
        default_value_t = false,
        requires = "range"
    )]
    pub print_csv: bool,

    /// Select SPED EFD files to analyze by specifying the range.
    ///
    /// (Selecione arquivos SPED EFD para analisar especificando o intervalo).
    ///
    /// Seja N o número de arquivos SPED EFD encontrados no diretório.
    ///
    /// Para analisar apenas um arquivo X (número inteiro de 1 a N) digite a opção: --range X ou -r X.
    ///
    /// Para analisar arquivos de X a Y (inteiros de 1 a N) digite a opção: --range X Y ou -r X Y.
    #[arg(short, long, value_parser, verbatim_doc_comment, required = false, num_args = 1..=2)]
    pub range: Option<Vec<usize>>,
}

impl Arguments {
    /// Build Arguments struct
    pub fn build() -> EFDResult<Arguments> {
        let mut args: Arguments = Arguments::parse();

        if let Some(generator) = args.generator {
            args.print_completions(generator);
        }

        if args.clear_terminal {
            clear_terminal_screen();
        }

        if args.find || args.range.is_some() {
            let pattern: &str = "**/PISCOFINS_[0-9][0-9]*_[0-9][0-9]*.txt";
            args.search_files(pattern)?;
        } else {
            args.print_help_msg();
            process::exit(1);
        };

        Ok(args)
    }

    pub fn get_app_name(&self) -> String {
        let app = Arguments::command();
        app.get_name().to_string()
    }

    fn print_help_msg(&self) {
        let mut app = Arguments::command();
        //app.print_long_help().ok();
        app.print_help().ok();
    }

    fn search_files(&mut self, pattern: &str) -> EFDResult<()> {
        let options = MatchOptions {
            case_sensitive: false,
            require_literal_separator: false,
            require_literal_leading_dot: false,
        };

        let paths: Vec<PathBuf> = glob_with(pattern, options)?.flatten().collect();

        self.all_files = paths;

        Ok(())
    }

    pub fn get_range(&mut self) -> EFDResult<()> {
        let number_of_files: usize = self.all_files.len();

        if number_of_files == 0 {
            self.print_help_msg();
            let msg = "Nenhum arquivo SPED EFD encontrado neste diretório!";
            println!("\n\t{}\n", msg.red());
            process::exit(1);
        }

        let mut range: Vec<usize> = self.range.clone().unwrap();

        range.sort();

        // range possui 1 ou 2 argumentos devido a opção num_args = 1..=2
        let (first, last): (usize, usize) = match range.len() {
            1 => (range[0], range[0]),
            _ => (range[0], range[1]),
        };

        if first == 0 {
            self.print_help_msg();
            let msg = "Arquivo nº 0: opção inválida!".red();
            println!("\n\t{msg}\n");
            process::exit(1);
        }

        if last > number_of_files {
            self.print_help_msg();
            let msg1 = format!("Não foi encontrado o arquivo nº {last}!");
            println!("\n\t{}", msg1.red());
            let msg2 = format!("Escolha um número entre 1 a {number_of_files}.");
            println!("\t{}\n", msg2.red());
            process::exit(1);
        }

        self.all_files = self.all_files[(first - 1)..last].to_vec();

        Ok(())
    }

    pub fn print_arquivos_efd(&self, write: &mut dyn Write) -> EFDResult<()> {
        let number_of_files = self.all_files.len();
        let number_of_digits = number_of_files.to_string().len();

        match number_of_files.cmp(&1) {
            Ordering::Less => {
                writeln!(write, "Nenhum arquivo SPED EFD encontrado neste diretório!")?
            }
            Ordering::Equal => writeln!(write, "Arquivo SPED EFD encontrado neste diretório:\n")?,
            Ordering::Greater => {
                writeln!(write, "Arquivos SPED EFD encontrados neste diretório:\n")?
            }
        }

        for (i, arquivo) in self.all_files.iter().enumerate() {
            writeln!(
                write,
                "   arquivo nº {:>number_of_digits$}: {}",
                i + 1,
                arquivo.display()
            )?;
        }

        writeln!(write)?;

        if number_of_files >= 1 {
            writeln!(write, "\tExemplos de uso:\n")?;

            writeln!(
                write,
                "\tPara analisar o primeiro arquivo SPED EFD utilize o comando:"
            )?;
            writeln!(write, "\tefd_contribuicoes -r 1\n")?;

            if number_of_files >= 4 {
                let q2 = number_of_files * 2 / 4;
                let q3 = number_of_files * 3 / 4;
                let delta = 1 + q3 - q2;
                writeln!(
                    write,
                    "\tPara analisar {delta} arquivos SPED EFD de {q2} a {q3} utilize o comando:"
                )?;
                writeln!(write, "\tefd_contribuicoes -r {q2} {q3}\n")?;
            }

            if number_of_files > 1 {
                writeln!(
                    write,
                    "\tPara analisar todos os arquivos SPED EFD utilize o comando:"
                )?;
                writeln!(write, "\tefd_contribuicoes -r 1 {number_of_files}\n")?;
            }
        }

        Ok(())
    }

    /// Print shell completions to standard output
    pub fn print_completions<G>(&self, r#gen: G)
    where
        G: Generator + std::fmt::Debug,
    {
        let mut cmd = Arguments::command();
        let cmd_name = cmd.get_name().to_string();
        let mut stdout = io::stdout();

        eprintln!("Generating completion file for {gen:?}...");
        generate(r#gen, &mut cmd, cmd_name, &mut stdout);
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    // cargo test -- --help
    // cargo test -- --nocapture
    // cargo test -- --show-output
    use super::*;

    #[test]
    fn verify_cli() {
        // cargo test -- --show-output verify_cli
        use clap::CommandFactory;
        Arguments::command().debug_assert()
    }
}
