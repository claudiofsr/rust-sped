use crate::{EFDError, EfdRaise};
use crate::{EFDResult, args::Arguments};
use claudiofsr_lib::Colors;
use colored::*;
use glob::{MatchOptions, glob_with};
use std::cmp::Ordering;
use std::io::Write;
use std::path::PathBuf;

pub const OUTPUT_DIRECTORY: &str = "novo";
pub const BASE_NAME: &str = "Info do Contribuinte EFD Contribuicoes";
const EFD_PATTERN: &str = "**/PISCOFINS_[0-9][0-9]*_[0-9][0-9]*.txt";

/// Configuração central da aplicação.
///
/// Esta estrutura separa as preocupações de "entrada do usuário" (CLI)
/// das "regras de negócio e caminhos" (Domínio).
#[derive(Debug)]
pub struct AppConfig {
    /// Nome do Programa extraído do binário
    pub app_name: String,

    /// Lista final de arquivos a serem processados.
    pub all_files: Vec<PathBuf>,

    // Flags de filtragem e processamento
    /// Se true, remove itens de saída do relatório final.
    pub excluir_saidas: bool,

    /// Se true, limita o rateio aos CSTs 01 a 09.
    pub excluir_cst_49: bool,

    /// Se true, mantém apenas operações que geram crédito (CST 50 a 66).
    pub operacoes_de_creditos: bool,

    /// Se true, não gera o arquivo .xlsx (Excel).
    pub no_excel: bool,

    /// Se true, gera o arquivo .csv.
    pub print_csv: bool,

    /// Ativa logs detalhados de depuração.
    pub debug: bool,

    // Caminhos de saída centralizados
    /// Diretório onde os resultados serão salvos.
    pub output_dir: PathBuf,

    /// Nome base para os arquivos de saída (sem extensão).
    pub base_name: String,
}

/// Implementação manual do Default para suportar valores customizados
impl Default for AppConfig {
    fn default() -> Self {
        Self {
            app_name: String::new(),
            all_files: Vec::new(),
            excluir_saidas: false,
            excluir_cst_49: false,
            operacoes_de_creditos: false,
            no_excel: false,
            print_csv: false,
            debug: false,
            output_dir: PathBuf::from(OUTPUT_DIRECTORY),
            base_name: BASE_NAME.to_string(),
        }
    }
}

impl AppConfig {
    /// Construtor principal que transforma Arguments (CLI) em AppConfig (Domínio).
    ///
    /// Realiza a busca física de arquivos e validação de intervalos (range).
    pub fn try_from_args(args: &Arguments) -> EFDResult<Self> {
        // 1. Localizar arquivos no sistema seguindo o padrão SPED
        let mut found_files = Self::search_files(EFD_PATTERN)?;

        // 2. Validação de existência
        if found_files.is_empty() {
            // Exibe o help antes de sair
            args.print_help_msg()?;
            //return Err(EFDError::NoFilesFound.tag(file!(), line!()));
            return EFDError::NoFilesFound.raise();
        }

        // 3. Lógica de Seleção (Range) ou Busca (Find)
        if let Some(ref range_vals) = args.range {
            found_files = Self::apply_range_filter(found_files, range_vals, args)?;
        } else if !args.find {
            // Se não houver range nem find, mostra help e sai
            args.print_help_msg()?;
            return EFDError::NoActionSelected.raise();
        }

        Ok(Self {
            app_name: args.get_app_name(),
            all_files: found_files,
            excluir_saidas: args.excluir_saidas,
            excluir_cst_49: args.excluir_cst_49,
            operacoes_de_creditos: args.operacoes_de_creditos,
            no_excel: args.no_excel,
            print_csv: args.print_csv,
            debug: args.debug,
            ..Self::default() // output_dir e base_name vêm do impl Default acima
        })
    }

    /// Executa a busca por arquivos TXT usando o padrão global.
    fn search_files(pattern: &str) -> EFDResult<Vec<PathBuf>> {
        let options = MatchOptions {
            case_sensitive: false,
            require_literal_separator: false,
            require_literal_leading_dot: false,
        };

        let paths: Vec<PathBuf> = glob_with(pattern, options)?.flatten().collect();

        Ok(paths)
    }

    /// Filtra a lista de arquivos baseada no intervalo [X Y] fornecido.
    ///
    /// Mantém as mensagens de erro exatas da implementação original.
    fn apply_range_filter(
        mut files: Vec<PathBuf>,
        range: &[usize],
        args: &Arguments,
    ) -> EFDResult<Vec<PathBuf>> {
        let number_of_files = files.len();

        // Ordenação para garantir [Início Fim]
        let mut sorted_range = range.to_vec();
        sorted_range.sort_unstable();

        // Se o usuário digitar -r 5 10, o vetor é [5, 10].
        // Se o usuário digitar -r 10 5, o vetor é [10, 5].
        // Ao ordenar (sort_unstable), o programa garante que o resultado será sempre [5, 10].
        // Isso evita que o código quebre tentando processar um intervalo que começa no fim e termina no início.

        // Se o usuário digitou -r 1 5 8, o código pegará o 1 e o 5 (ignorando o resto de forma segura).
        // first: 1
        // last: 5

        let (first, last) = match sorted_range.len() {
            1 => (sorted_range[0], sorted_range[0]),
            _ => (sorted_range[0], sorted_range[1]),
        };

        // Validação do Primeiro Índice (Mensagem original)
        if first == 0 {
            args.print_help_msg()?;
            return EFDError::InvalidFileIndex.raise();
        }

        // Validação do Último Índice (Mensagem original)
        if last > number_of_files {
            args.print_help_msg()?;
            return EFDError::FileNotFoundInIndex {
                index: last,
                total: number_of_files,
            }
            .raise();
        }

        // Otimização: drain() move os PathBuf sem clonar as Strings internas
        let start_idx = first - 1;
        Ok(files.drain(start_idx..last).collect())
    }

    /// Imprime a lista de arquivos e exemplos de uso (Lógica identica à original).
    pub fn print_summary(&self, write: &mut dyn Write) -> EFDResult<()> {
        let number_of_files = self.all_files.len();
        let number_of_digits = number_of_files.to_string().len();

        // Títulos baseados na quantidade (Cores blue() aplicadas conforme solicitado)
        match number_of_files.cmp(&1) {
            Ordering::Less => writeln!(
                write,
                "{}",
                "Nenhum arquivo SPED EFD encontrado neste diretório!".red()
            )?,
            Ordering::Equal => writeln!(write, "Arquivo SPED EFD encontrado neste diretório:\n")?,
            Ordering::Greater => writeln!(
                write,
                "{} arquivos SPED EFD encontrados neste diretório:\n",
                number_of_files.blue()
            )?,
        }

        // Listagem numerada com alinhamento à direita
        for (i, arquivo) in self.all_files.iter().enumerate() {
            writeln!(
                write,
                "   arquivo nº {:>number_of_digits$}: {}",
                i + 1,
                arquivo.display()
            )?;
        }

        writeln!(write)?;

        // Seção de Exemplos de Uso (Lógica matemática original preservada)
        if number_of_files >= 1 {
            writeln!(write, "\tExemplos de uso:\n")?;

            writeln!(
                write,
                "\tPara analisar o primeiro arquivo SPED EFD utilize o comando:"
            )?;
            writeln!(write, "\t{} -r 1\n", self.app_name)?;

            // Lógica de Delta para múltiplos arquivos
            if number_of_files >= 4 {
                let q2 = number_of_files * 2 / 4;
                let q3 = number_of_files * 3 / 4;
                let delta = 1 + q3 - q2;
                writeln!(
                    write,
                    "\tPara analisar {delta} arquivos SPED EFD de {q2} a {q3} utilize o comando:"
                )?;
                writeln!(write, "\t{} -r {q2} {q3}\n", self.app_name)?;
            }

            // Opção para analisar tudo
            if number_of_files > 1 {
                writeln!(
                    write,
                    "\tPara analisar todos os arquivos SPED EFD utilize o comando:"
                )?;
                writeln!(write, "\t{} -r 1 {number_of_files}\n", self.app_name)?;
            }
        }

        Ok(())
    }

    /// Helpers de Caminho (Zero-cost abstraction)
    pub fn path_csv(&self) -> PathBuf {
        self.output_dir.join(&self.base_name).with_extension("csv")
    }

    pub fn path_xlsx(&self) -> PathBuf {
        self.output_dir.join(&self.base_name).with_extension("xlsx")
    }
}
