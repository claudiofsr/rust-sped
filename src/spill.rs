//! Modo streaming (`--stream`): processa um número ilimitado de arquivos EFD
//! com memória limitada.
//!
//! MAP (paralelo): para cada arquivo -> análise -> CSV de detalhe (`-p`) ->
//! acumuladores (somas por chave) -> temporário `.agg` em `novo/spill` -> descarta tudo.
//!
//! REDUCE: funde os acumuladores, aplica as etapas derivadas (`finalizar_*`)
//! e escreve tabelas/Excel de consolidação (único por padrão, ou por CNPJ base
//! se exceder a RAM disponível).
//!
//! Formato do `.agg`: linha 1 = cabeçalho JSON (curto); linha 2 = corpo JSON.
//! As chaves de consolidação (`Keys` e `Chaves`) serializam sem perda diretamente.

use std::{
    collections::{BTreeMap, HashMap},
    fs::{self, File},
    io::{self, BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
    sync::{
        Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    thread,
};

use chrono::{Datelike, NaiveDate};
use indicatif::MultiProgress;
use serde::{Deserialize, Serialize};

use crate::{
    AppConfig, BUFFER_CAPACITY, DELIMITER_CHAR, DocsFiscais, EFDError, EFDResult, ResultExt,
    analyze_all::{
        escrever_tabelas, imprimir_resumo_periodo, print_arquivos_selecionados, should_keep_record,
        write_csv,
    },
    analyze_one_file,
    structures::{
        analise_dos_creditos::{
            AcumuladoNatureza, Chaves, Rateio, Valores, acumular_natureza, finalizar_natureza,
        },
        consolidacao_cst::{AcumuladoCST, Keys, Values, acumular_cst, finalizar_cst},
    },
    write_xlsx, write_xlsx_com_progresso,
};

const KIB: u64 = 1024;
const MIB: u64 = 1024 * KIB;
pub(crate) const GIB: u64 = 1024 * MIB;

fn formatar_bytes(bytes: u64) -> String {
    if bytes >= GIB {
        format!("{:.2} GiB", bytes as f64 / GIB as f64)
    } else if bytes >= MIB {
        format!("{:.2} MiB", bytes as f64 / MIB as f64)
    } else if bytes >= KIB {
        format!("{:.1} KiB", bytes as f64 / KIB as f64)
    } else {
        format!("{bytes} B")
    }
}

/// Fração do orçamento de RAM usada no planejamento (sobra para SO e Excel).
const SAFETY_MARGIN: f64 = 0.85;

fn orcamento_ram(config: &AppConfig) -> u64 {
    (config.max_ram_bytes as f64 * SAFETY_MARGIN) as u64
}

// ============================================================================
// Formato dos checkpoints
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CabecalhoShard {
    cnpj_base: u32,
    ano: i32,
    mes: u32,
    /// Nº de documentos fiscais extraídos ANTES do filtro de saída (0 = ignorado).
    documentos: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct CorpoShard {
    arquivo: String,
    mensagens: String,
    cst: Vec<(Keys, Values)>,
    nat: NatShard,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct NatShard {
    base: Vec<(Chaves, Valores)>,
    rateio: Vec<(Chaves, Rateio)>,
    ajustes: Vec<(Chaves, Valores)>,
    descontos: Vec<(Chaves, Valores)>,
}

impl From<AcumuladoNatureza> for NatShard {
    fn from(a: AcumuladoNatureza) -> Self {
        Self {
            base: a.base.into_iter().collect(),
            rateio: a.rateio.into_iter().collect(),
            ajustes: a.ajustes.into_iter().collect(),
            descontos: a.descontos.into_iter().collect(),
        }
    }
}

impl From<NatShard> for AcumuladoNatureza {
    fn from(s: NatShard) -> Self {
        Self {
            base: s.base.into_iter().collect(),
            rateio: s.rateio.into_iter().collect(),
            ajustes: s.ajustes.into_iter().collect(),
            descontos: s.descontos.into_iter().collect(),
        }
    }
}

// ============================================================================
// Arquivos auxiliares (checkpoint, nomes)
// ============================================================================

fn invalid_data<E: std::error::Error + Send + Sync + 'static>(e: E) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, e)
}

fn com_caminho(e: io::Error, path: &Path) -> io::Error {
    io::Error::new(e.kind(), format!("{}: {e}", path.display()))
}

fn tamanho_do_arquivo(path: &Path) -> u64 {
    fs::metadata(path).map(|m| m.len()).unwrap_or(0)
}

/// Nome-base estável do checkpoint: `<nome do arquivo>-<8 hex do hash do caminho>`.
fn nome_base(arquivo: &Path) -> String {
    let stem = arquivo
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "efd".to_string());
    let hash = blake3::hash(arquivo.to_string_lossy().as_bytes()).to_hex();
    format!("{stem}-{}", &hash.as_str()[..8])
}

struct CaminhosShard {
    agg: PathBuf,
    csv: PathBuf,
    xlsx: PathBuf,
}

fn caminhos_do_shard(spill_dir: &Path, arquivo: &Path) -> CaminhosShard {
    let base = nome_base(arquivo);
    CaminhosShard {
        agg: spill_dir.join(format!("{base}.agg")),
        csv: spill_dir.join(format!("{base}.csv")),
        xlsx: spill_dir.join(format!("{base}.xlsx")),
    }
}

fn ler_cabecalho(path: &Path) -> Option<CabecalhoShard> {
    let file = File::open(path).ok()?;
    let mut linha = String::new();
    BufReader::new(file).read_line(&mut linha).ok()?;
    serde_json::from_str(linha.trim_end()).ok()
}

fn ler_corpo(path: &Path) -> io::Result<CorpoShard> {
    let file = File::open(path).map_err(|e| com_caminho(e, path))?;
    let mut leitor = BufReader::new(file);
    let mut cabecalho = String::new();
    leitor.read_line(&mut cabecalho)?;
    serde_json::from_reader(leitor).map_err(|e| com_caminho(invalid_data(e), path))
}

/// Escrita atômica: grava em `.tmp` e renomeia (o `.agg` é o marcador de "concluído").
fn gravar_shard(path: &Path, cab: &CabecalhoShard, corpo: &CorpoShard) -> io::Result<()> {
    let tmp = path.with_extension("agg.tmp");
    {
        let mut w = BufWriter::new(File::create(&tmp).map_err(|e| com_caminho(e, &tmp))?);
        serde_json::to_writer(&mut w, cab).map_err(invalid_data)?;
        w.write_all(b"\n")?;
        serde_json::to_writer(&mut w, corpo).map_err(invalid_data)?;
        w.write_all(b"\n")?;
        w.flush()?;
    }
    fs::rename(&tmp, path)
}

fn limpar_spill(dir: &Path) -> io::Result<()> {
    for entrada in fs::read_dir(dir)? {
        let path = entrada?.path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if path.is_file() && matches!(ext, "agg" | "csv" | "xlsx" | "tmp") {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

// ============================================================================
// Planejamento
// ============================================================================

/// Nº de arquivos analisados em paralelo: cada um pode usar ~ tamanho x `mem_factor`
/// de RAM; nada fica retido depois (tudo vai para o disco).
fn planejar_workers(config: &AppConfig, files: &[PathBuf]) -> usize {
    let maior = files
        .iter()
        .map(|p| tamanho_do_arquivo(p))
        .max()
        .unwrap_or(0);
    let pico = (maior as f64 * config.mem_factor.max(1.0)) as u64;
    let orcamento = orcamento_ram(config);

    log::debug!(
        "RAM: orçamento utilizável {}; pico estimado do maior arquivo {}.",
        formatar_bytes(orcamento),
        formatar_bytes(pico)
    );

    let por_ram = match orcamento.checked_div(pico) {
        None => usize::MAX, // pico == 0: sem limite de RAM
        Some(0) => {
            // orcamento / pico == 0 significa exatamente pico > orcamento
            log::warn!(
                "Pico estimado de um arquivo ({}) excede o orçamento ({}): \
                 processando um arquivo por vez. Ajuste --mem-factor ou --jobs.",
                formatar_bytes(pico),
                formatar_bytes(orcamento)
            );
            1
        }
        Some(n) => usize::try_from(n).unwrap_or(usize::MAX),
    };

    config
        .jobs
        .unwrap_or_else(|| thread::available_parallelism().map_or(1, |n| n.get()))
        .max(1)
        .min(por_ram)
        .min(files.len().max(1))
}

// ============================================================================
// MAP: um arquivo
// ============================================================================

fn processar_arquivo(
    config: &AppConfig,
    spill_dir: &Path,
    multi: &MultiProgress,
    arquivo: &Path,
    index: usize,
    total: usize,
) -> EFDResult<()> {
    let paths = caminhos_do_shard(spill_dir, arquivo);

    let mut info = match analyze_one_file(multi, arquivo, index, total) {
        Ok(info) => info,
        Err(error) => {
            return Err(EFDError::AnalyzeFileError {
                source: Box::new(error),
                arquivo: arquivo.to_path_buf(),
            })
            .loc();
        }
    };

    let documentos = info.all_docs.len();

    // Numeração local ao arquivo; `concatenar_csv_global` a desloca para a numeração global.
    for (j, doc) in info.all_docs.iter_mut().enumerate() {
        doc.linhas = j + 2;
    }

    // Acumuladores sobre TODAS as linhas (o filtro de saída vem depois).
    let cst = acumular_cst(&info.all_docs);
    let nat = acumular_natureza(&info.all_docs);

    if documentos > 0 && (config.print_csv || config.xlsx_mensal) {
        let consolidacao_mensal = if config.xlsx_mensal {
            let (_, consolidacao_cst) = finalizar_cst(cst.clone())?;
            let (_, _, consolidacao_nat) = finalizar_natureza(config, nat.clone())?;
            Some((consolidacao_cst, consolidacao_nat))
        } else {
            None
        };

        info.all_docs.retain(|doc| should_keep_record(doc, config));

        if config.print_csv {
            let tmp = paths.csv.with_extension("csv.tmp");
            write_csv(&tmp, &info.all_docs)?;
            fs::rename(&tmp, &paths.csv)?;
        }

        if let Some((consolidacao_cst, consolidacao_nat)) = consolidacao_mensal {
            let tmp = paths.xlsx.with_extension("xlsx.tmp");
            write_xlsx_com_progresso(
                &tmp,
                &info.all_docs,
                &consolidacao_cst,
                &consolidacao_nat,
                config.memory_mode,
                multi,
            )?;
            fs::rename(&tmp, &paths.xlsx)?;
        }
    }

    let cab = CabecalhoShard {
        cnpj_base: info.cnpj_base,
        ano: info.periodo_de_apuracao.year(),
        mes: info.periodo_de_apuracao.month(),
        documentos,
    };

    let corpo = CorpoShard {
        arquivo: arquivo.display().to_string(),
        mensagens: info.messages,
        cst: cst.into_iter().collect(),
        nat: NatShard::from(nat),
    };

    gravar_shard(&paths.agg, &cab, &corpo)?;
    Ok(())
}

fn mapear(
    config: &AppConfig,
    spill_dir: &Path,
    files: &[PathBuf],
    workers: usize,
) -> EFDResult<()> {
    let total = files.len();

    // Barras de progresso por arquivo IDÊNTICAS às do modo normal (criadas por
    // `analyze_one_file` neste MultiProgress), sem limite de quantidade de arquivos.
    let multi = MultiProgress::new();

    let proximo = AtomicUsize::new(0);
    let abortar = AtomicBool::new(false);
    let primeiro_erro: Mutex<Option<EFDError>> = Mutex::new(None);

    thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| {
                while !abortar.load(Ordering::Relaxed) {
                    let index = proximo.fetch_add(1, Ordering::Relaxed);
                    let Some(arquivo) = files.get(index) else {
                        break;
                    };

                    match processar_arquivo(config, spill_dir, &multi, arquivo, index, total) {
                        Ok(()) => {}
                        Err(error) => {
                            abortar.store(true, Ordering::Relaxed);
                            primeiro_erro
                                .lock()
                                .unwrap_or_else(|e| e.into_inner())
                                .get_or_insert(error);
                        }
                    }
                }
            });
        }
    });

    primeiro_erro
        .into_inner()
        .unwrap_or_else(|e| e.into_inner())
        .map_or(Ok(()), Err)
}

// ============================================================================
// REDUCE: por CNPJ base
// ============================================================================

struct ItemShard {
    index: usize,
    cab: CabecalhoShard,
    agg: PathBuf,
    csv: PathBuf,
}

fn numero_de_linha(campo: &[u8]) -> io::Result<usize> {
    std::str::from_utf8(campo)
        .ok()
        .and_then(|texto| texto.parse::<usize>().ok())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "valor inválido na coluna \"Linhas\": {:?}",
                    String::from_utf8_lossy(campo)
                ),
            )
        })
}

/// CSV final único, BYTE A BYTE igual ao do modo global original:
/// * mesma ordem das linhas: (cnpj, ano, mês, ordem dos arquivos);
/// * mesmo formato (delimitador, aspas `NonNumeric`, cabeçalho);
/// * coluna "Linhas" renumerada globalmente: cada arquivo é deslocado pelo nº de
///   documentos (ANTES do filtro de saída) dos arquivos anteriores, como em
///   `update_line_counter` (as linhas filtradas deixam "buracos" na numeração).
///
/// `itens` deve estar na ordem global. É erro faltar o CSV de algum arquivo.
fn concatenar_csv_global(destino: &Path, itens: &[&ItemShard]) -> EFDResult<()> {
    let cabecalho = DocsFiscais::get_headers();

    let col_linhas = cabecalho
        .iter()
        .position(|nome| nome == "Linhas")
        .map_loc(|_| EFDError::Other("coluna \"Linhas\" ausente no cabeçalho".into()))?;

    let arquivo = File::create(destino).map_err(|e| com_caminho(e, destino))?;
    let buffer = BufWriter::with_capacity(BUFFER_CAPACITY, arquivo);

    // Mesmas opções de `write_csv` (analyze_all.rs).
    let mut saida = csv::WriterBuilder::new()
        .delimiter(DELIMITER_CHAR as u8)
        .has_headers(true)
        .quote_style(csv::QuoteStyle::NonNumeric)
        .from_writer(buffer);

    saida.write_record(&cabecalho)?;

    let mut deslocamento: usize = 0;
    let mut registro = csv::ByteRecord::new();
    let mut novo = csv::ByteRecord::new();

    for item in itens {
        let arquivo = File::open(&item.csv).map_err(|e| com_caminho(e, &item.csv))?;

        // has_headers(true): o cabeçalho de cada CSV por arquivo é descartado.
        let mut leitor = csv::ReaderBuilder::new()
            .delimiter(DELIMITER_CHAR as u8)
            .has_headers(true)
            .from_reader(BufReader::with_capacity(BUFFER_CAPACITY, arquivo));

        while leitor.read_byte_record(&mut registro)? {
            novo.clear();
            for (i, campo) in registro.iter().enumerate() {
                if i == col_linhas {
                    let global = numero_de_linha(campo)? + deslocamento;
                    novo.push_field(global.to_string().as_bytes());
                } else {
                    novo.push_field(campo);
                }
            }
            saida.write_byte_record(&novo)?;
        }

        deslocamento += item.cab.documentos;
    }

    saida.flush()?;
    Ok(())
}

/// Estimativa HEURÍSTICA: RAM ~ tamanho dos .agg (JSON) x fator. O JSON repete
/// nomes de campos e grava decimais como texto (maior que a RAM por chave), mas
/// `finalizar_*` cria vários mapas derivados e tabelas de texto. Calibre no 1º lote.
const AGG_RAM_FACTOR: f64 = 4.0;

/// Funde os checkpoints de `itens` (já na ordem global), finaliza a consolidação
/// e escreve Excel em `dir_saida`. O texto enviado a `write` reproduz o do modo
/// normal: por arquivo (cabeçalho, mensagens e tabelas), resumo do período e,
/// com mais de um arquivo, as tabelas globais.
fn consolidar_itens(
    config: &AppConfig,
    itens: &[&ItemShard],
    dir_saida: &Path,
    write: &mut dyn Write,
) -> EFDResult<()> {
    let mut cst: AcumuladoCST = HashMap::new();
    let mut nat = AcumuladoNatureza::default();
    let mut periodos: Vec<NaiveDate> = Vec::with_capacity(itens.len());

    for (n, item) in itens.iter().enumerate() {
        let corpo = ler_corpo(&item.agg)?;

        writeln!(write, "EFD {:02}: {}", n + 1, corpo.arquivo)?;
        writeln!(
            write,
            "Período de Apuração: {:02}/{}\n",
            item.cab.mes, item.cab.ano
        )?;
        write.write_all(corpo.mensagens.as_bytes())?;

        let cst_arquivo: AcumuladoCST = corpo.cst.into_iter().collect();
        let nat_arquivo: AcumuladoNatureza = corpo.nat.into();

        // Consolidação individual por arquivo (sempre impressa, como no modo normal).
        let (tabela_cst, _) = finalizar_cst(cst_arquivo.clone())?;
        let (tabela_nat, tabela_rb, _) = finalizar_natureza(config, nat_arquivo.clone())?;
        escrever_tabelas(write, tabela_rb, tabela_cst, tabela_nat)?;

        for (chave, valor) in cst_arquivo {
            *cst.entry(chave).or_default() += valor;
        }
        nat.mesclar(nat_arquivo);

        if let Some(data) = NaiveDate::from_ymd_opt(item.cab.ano, item.cab.mes, 1) {
            periodos.push(data);
        }
    }

    imprimir_resumo_periodo(&periodos, write)?;

    let (tabela_cst, consolidacao_cst) = finalizar_cst(cst)?;
    let (tabela_nat, tabela_rb, consolidacao_nat) = finalizar_natureza(config, nat)?;

    // Tabelas globais somente com mais de um arquivo (como no modo normal).
    if itens.len() > 1 {
        escrever_tabelas(write, tabela_rb, tabela_cst, tabela_nat)?;
    }

    fs::create_dir_all(dir_saida)?;

    if !config.no_excel {
        let path_xlsx = dir_saida.join(&config.base_name).with_extension("xlsx");
        writeln!(write, "Write Excel xlsx file: {:?}\n", path_xlsx.display())?;

        // Somente as abas de consolidação: o detalhe fica nos CSV por arquivo.
        write_xlsx(
            &path_xlsx,
            &[],
            &consolidacao_cst,
            &consolidacao_nat,
            config.memory_mode,
        )?;
    }

    Ok(())
}

/// Checagem definitiva (exata nos .agg, heurística no fator): decide entre
/// consolidação global única ou fallback automático por CNPJ base.
fn cabe_no_orcamento_global(config: &AppConfig, itens: &[&ItemShard]) -> bool {
    let bytes_agg: u64 = itens.iter().map(|i| tamanho_do_arquivo(&i.agg)).sum();
    let estimado = (bytes_agg as f64 * AGG_RAM_FACTOR) as u64;
    let orcamento = orcamento_ram(config);

    log::debug!(
        "Consolidação global: {} checkpoints ({} em disco); RAM estimada {}; \
         orçamento {} (estimativa heurística, fator {AGG_RAM_FACTOR}).",
        itens.len(),
        formatar_bytes(bytes_agg),
        formatar_bytes(estimado),
        formatar_bytes(orcamento)
    );

    if estimado > orcamento {
        log::warn!(
            "RAM estimada para consolidação global ({}) excede o orçamento ({}): \
             gerando um Excel por CNPJ base (fallback automático).",
            formatar_bytes(estimado),
            formatar_bytes(orcamento)
        );
        return false;
    }

    true
}

fn consolidar_por_cnpj(
    config: &AppConfig,
    spill_dir: &Path,
    files: &[PathBuf],
    write: &mut dyn Write,
) -> EFDResult<()> {
    // 1. Agrupar temporários por CNPJ base (lê só a 1a linha de cada .agg).
    let mut grupos: BTreeMap<u32, Vec<ItemShard>> = BTreeMap::new();

    for (index, arquivo) in files.iter().enumerate() {
        let paths = caminhos_do_shard(spill_dir, arquivo);
        let Some(cab) = ler_cabecalho(&paths.agg) else {
            log::warn!("Sem arquivo temporário para {}", arquivo.display());
            continue;
        };
        if cab.documentos == 0 {
            continue;
        }
        grupos.entry(cab.cnpj_base).or_default().push(ItemShard {
            index,
            cab,
            agg: paths.agg,
            csv: paths.csv,
        });
    }

    // Mesma ordem do modo global: (cnpj, ano, mês), estável pela ordem dos arquivos.
    for itens in grupos.values_mut() {
        itens.sort_by_key(|i| (i.cab.ano, i.cab.mes, i.index));
    }

    let total_grupos = grupos.len();
    let todos: Vec<&ItemShard> = grupos.values().flatten().collect();

    if cabe_no_orcamento_global(config, &todos) {
        // 2a. Padrão: uma única consolidação (idêntica ao modo global original).
        consolidar_itens(config, &todos, &config.output_dir, write)?;
    } else {
        // 2b. Fallback por falta de RAM: um CNPJ por vez (O(chaves de um CNPJ)).
        for (cnpj, itens) in &grupos {
            writeln!(write, "CNPJ base {cnpj:08}: {} arquivos\n", itens.len())?;
            let refs: Vec<&ItemShard> = itens.iter().collect();
            let dir_cnpj = config.output_dir.join(format!("{cnpj:08}"));
            consolidar_itens(config, &refs, &dir_cnpj, write)?;
        }
    }

    if config.print_csv && config.concat_csv {
        let destino = config.path_csv();
        writeln!(write, "Write csv file: {:?}\n", destino.display())?;
        concatenar_csv_global(&destino, &todos)?;
    }

    log::debug!(
        "Modo streaming concluído: {total_grupos} CNPJ base. CSV por arquivo em: {}",
        spill_dir.display()
    );

    Ok(())
}

// ============================================================================
// Ponto de entrada
// ============================================================================

pub(crate) fn executar_streaming(config: &AppConfig, write: &mut dyn Write) -> EFDResult<()> {
    let spill_dir = config.spill_dir();
    for dir in [&config.output_dir, &spill_dir] {
        fs::create_dir_all(dir)
            .map_loc(|e| EFDError::DirectoryCreationFailed(dir.display().to_string(), e))?;
    }

    limpar_spill(&spill_dir)?;

    let files: &[PathBuf] = &config.all_files;
    let workers = planejar_workers(config, files);

    // Mesmo texto inicial do modo normal (lista dos arquivos selecionados).
    print_arquivos_selecionados(files, write)?;

    log::debug!(
        "Modo streaming: {} arquivos, {workers} em paralelo. Spill: {}",
        files.len(),
        spill_dir.display()
    );

    mapear(config, &spill_dir, files, workers)?;
    consolidar_por_cnpj(config, &spill_dir, files, write)
}

//----------------------------------------------------------------------------//
//                                   Tests                                    //
//----------------------------------------------------------------------------//

/// Run tests with:
/// cargo test -- --show-output tests_spill
#[cfg(test)]
mod tests_spill {
    use super::*;
    use crate::{
        CodigoSituacaoTributaria, TipoDeOperacao,
        structures::{
            analise_dos_creditos::consolidar_natureza_da_base_de_calculo,
            consolidacao_cst::consolidar_operacoes_por_cst,
        },
    };
    use indicatif::ProgressDrawTarget;
    use rust_decimal_macros::dec;

    const TIPOS: [TipoDeOperacao; 7] = [
        TipoDeOperacao::Entrada,
        TipoDeOperacao::Saida,
        TipoDeOperacao::AjusteAcrescimo,
        TipoDeOperacao::AjusteReducao,
        TipoDeOperacao::DescontoNoPeriodo,
        TipoDeOperacao::DescontoPosterior,
        TipoDeOperacao::Detalhamento,
    ];

    #[test]
    fn chaves_preserva_tipo_de_operacao() {
        for tipo in TIPOS {
            let chaves = Chaves {
                cnpj_base: "12345678".into(),
                ano: Some(2020),
                tipo_de_operacao: Some(tipo),
                aliq_pis: Some(dec!(1.65)),
                ..Default::default()
            };

            // Ida e volta pelo JSON, como no checkpoint.
            let json = serde_json::to_string(&chaves).unwrap();
            let volta: Chaves = serde_json::from_str(&json).unwrap();

            assert_eq!(volta, chaves, "tipo: {tipo:?}");
        }
    }

    #[test]
    fn rateio_o_ultimo_vence() {
        let mut a = Rateio::default();
        a.atribuir(crate::TipoDeRateio::RecBrutaNCumTribMercInterno, dec!(1));

        let mut b = Rateio::default();
        b.atribuir(crate::TipoDeRateio::RecBrutaNCumTribMercInterno, dec!(2));
        b.atribuir(crate::TipoDeRateio::RecBrutaNCumDeExportacao, dec!(3));

        a.sobrepor(b);

        assert_eq!(a.trib, Some(dec!(2)));
        assert_eq!(a.exp, Some(dec!(3)));
        assert_eq!(a.ntrib, None);
    }

    #[test]
    fn corpo_shard_ida_e_volta_json() {
        let chave = Keys {
            cnpj_base: "12345678".into(),
            ano: Some(2020),
            cst: Some(CodigoSituacaoTributaria::TotalReceitasSaidas),
            ..Default::default()
        };
        let corpo = CorpoShard {
            arquivo: "a.txt".into(),
            mensagens: "linha 1\nlinha \"2\"\n".into(),
            cst: vec![(chave.clone(), Values::default())],
            nat: NatShard::default(),
        };

        let json = serde_json::to_string(&corpo).unwrap();
        let volta: CorpoShard = serde_json::from_str(&json).unwrap();

        assert_eq!(volta.cst[0].0, chave);
        assert_eq!(volta.mensagens, corpo.mensagens);
        assert_eq!(serde_json::to_string(&volta).unwrap(), json);
    }

    /// Fundir acumuladores de duas metades == consolidar tudo de uma vez.
    #[test]
    fn acumuladores_fundidos_equivalem_a_consolidacao_global() -> EFDResult<()> {
        let multi = MultiProgress::with_draw_target(ProgressDrawTarget::hidden());
        let info = analyze_one_file(&multi, Path::new("examples/efd_data_random"), 0, 1)?;
        let docs = info.all_docs;
        let config = AppConfig::default();

        let (_, cst_global) = consolidar_operacoes_por_cst(&docs)?;
        let (_, _, nat_global) = consolidar_natureza_da_base_de_calculo(&config, &docs)?;

        let (a, b) = docs.split_at(docs.len() / 2);

        let mut cst = acumular_cst(a);
        for (k, v) in acumular_cst(b) {
            *cst.entry(k).or_default() += v;
        }
        let (_, cst_fundido) = finalizar_cst(cst)?;

        let mut nat = acumular_natureza(a);
        nat.mesclar(acumular_natureza(b));
        let (_, _, nat_fundido) = finalizar_natureza(&config, nat)?;

        assert_eq!(
            serde_json::to_string(&cst_global).unwrap(),
            serde_json::to_string(&cst_fundido).unwrap()
        );
        assert_eq!(
            serde_json::to_string(&nat_global).unwrap(),
            serde_json::to_string(&nat_fundido).unwrap()
        );

        Ok(())
    }

    /// O CSV concatenado do modo --stream deve ser IDÊNTICO ao do modo global.
    /// O mesmo arquivo é usado duas vezes para exercitar o deslocamento de "Linhas".
    #[test]
    fn csv_concatenado_e_identico_ao_modo_global() -> EFDResult<()> {
        let base = std::env::temp_dir().join(format!("efd_spill_test_{}", std::process::id()));
        let dir_global = base.join("global");
        let dir_stream = base.join("stream");
        fs::create_dir_all(&dir_global)?;
        fs::create_dir_all(&dir_stream)?;

        let arquivo = PathBuf::from("examples/efd_data_random");
        let ambos = vec![arquivo.clone(), arquivo];

        let global = AppConfig {
            all_files: ambos.clone(),
            output_dir: dir_global,
            print_csv: true,
            no_excel: true,
            ..Default::default()
        };
        let stream = AppConfig {
            all_files: ambos,
            output_dir: dir_stream,
            print_csv: true,
            no_excel: true,
            stream: true,
            concat_csv: true,
            jobs: Some(1),
            ..Default::default()
        };

        let mut log: Vec<u8> = Vec::new();
        crate::executar_programa(&global, &mut log)?;
        crate::executar_programa(&stream, &mut log)?;

        let csv_global = fs::read(global.path_csv())?;
        let csv_stream = fs::read(stream.path_csv())?;
        let _ = fs::remove_dir_all(&base);

        assert!(!csv_global.is_empty());
        assert!(
            csv_global == csv_stream,
            "o CSV do modo streaming difere do CSV do modo global"
        );

        Ok(())
    }

    #[test]
    fn nome_base_e_estavel_e_distingue_caminhos() {
        let a = nome_base(Path::new("x/PISCOFINS_1.txt"));
        let b = nome_base(Path::new("y/PISCOFINS_1.txt"));
        assert_eq!(a, nome_base(Path::new("x/PISCOFINS_1.txt")));
        assert_ne!(a, b);
        assert!(a.starts_with("PISCOFINS_1-"));
    }

    #[test]
    fn formatar_bytes_escolhe_unidade_adequada() {
        assert_eq!(formatar_bytes(512), "512 B");
        assert_eq!(formatar_bytes(2 * KIB), "2.0 KiB");
        assert_eq!(formatar_bytes(5 * MIB / 2), "2.50 MiB");
        assert_eq!(formatar_bytes(3 * GIB / 2), "1.50 GiB");
    }

    #[test]
    fn fallback_por_cnpj_quando_excede_orcamento() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(&[b'x'; 100]).unwrap();
        tmp.flush().unwrap();

        let item = ItemShard {
            index: 0,
            cab: CabecalhoShard {
                cnpj_base: 12345678,
                ano: 2024,
                mes: 1,
                documentos: 1,
            },
            agg: tmp.path().to_path_buf(),
            csv: PathBuf::new(),
        };

        let config_pequeno = AppConfig {
            max_ram_bytes: 100,
            ..Default::default()
        };
        assert!(!cabe_no_orcamento_global(&config_pequeno, &[&item]));

        let config_grande = AppConfig {
            max_ram_bytes: 1000,
            ..Default::default()
        };
        assert!(cabe_no_orcamento_global(&config_grande, &[&item]));
    }

    #[test]
    fn xlsx_mensal_e_gerado_no_spill_e_limpo_por_limpar_spill() -> EFDResult<()> {
        let base =
            std::env::temp_dir().join(format!("efd_xlsx_mensal_test_{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);

        let arquivo = PathBuf::from("examples/efd_data_random");
        let config = AppConfig {
            all_files: vec![arquivo.clone()],
            output_dir: base.clone(),
            no_excel: true,
            stream: true,
            xlsx_mensal: true,
            jobs: Some(1),
            ..Default::default()
        };

        let mut log: Vec<u8> = Vec::new();
        crate::executar_programa(&config, &mut log)?;

        let paths = caminhos_do_shard(&config.spill_dir(), &arquivo);
        assert!(
            paths.xlsx.is_file(),
            "xlsx mensal não foi gerado em {}",
            paths.xlsx.display()
        );
        assert!(
            !config.path_xlsx().exists(),
            "-n não deve gerar Excel consolidado"
        );

        limpar_spill(&config.spill_dir())?;
        assert!(!paths.xlsx.exists(), "limpar_spill deve remover .xlsx");

        let _ = fs::remove_dir_all(&base);
        Ok(())
    }
}
