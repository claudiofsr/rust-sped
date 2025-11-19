use crate::{EFDError, EFDResult, SpedRecordTrait};
use rayon::prelude::*;
use std::collections::HashMap;

#[derive(Debug)]
pub enum SpedRecord {
    /// Contém qualquer registro SPED que implemente SpedRecordTrait
    ///
    /// Send + Sync necessários para Rayon
    Generic(Box<dyn SpedRecordTrait + Send + Sync>),

    // Adicionar outros registros aqui
    Unknown(String), // Para linhas que não correspondem a um registro conhecido
}

impl SpedRecord {
    /// Construtor conveniente para o caso Generic
    pub fn new_generic<T: SpedRecordTrait + Send + Sync + 'static>(record: T) -> Self {
        SpedRecord::Generic(Box::new(record))
    }

    /// Retorna o número da linha associado a este registro.
    pub fn line_number(&self) -> usize {
        match self {
            SpedRecord::Generic(r) => r.line_number(),
            SpedRecord::Unknown(_) => 0, // Ou um valor de erro/default apropriado
        }
    }

    /// Retorna o bloco associado a este registro.
    pub fn bloco(&self) -> char {
        match self {
            SpedRecord::Generic(r) => r.bloco(),
            SpedRecord::Unknown(_) => 'U',
        }
    }

    /// Retorna o nome do registro (e.g., "0000", "C100").
    pub fn registro_name(&self) -> &str {
        match self {
            SpedRecord::Generic(r) => r.registro_name(),
            SpedRecord::Unknown(_) => "Não Definido",
        }
    }

    /// Tenta converter (downcast) o registro genérico para um tipo concreto `T`.
    ///
    /// Se o registro atual não for do tipo `T`, retorna `EFDError::RecordCastError`
    /// contendo o nome do registro que **realmente** está armazenado.
    pub fn downcast_ref<T: 'static>(&self) -> EFDResult<&T> {
        match self {
            SpedRecord::Generic(inner) => {
                // Tenta fazer o downcast usando as_any() definido no Trait
                inner.as_any().downcast_ref::<T>().ok_or_else(|| {
                    // Se falhar, retorna erro com o nome do registro encontrado
                    EFDError::RecordCastError(inner.registro_name().to_string())
                })
            }
            SpedRecord::Unknown(reg_name) => {
                // Se for Unknown, definitivamente não é T
                Err(EFDError::RecordCastError(reg_name.clone()))
            }
        }
    }

    /// Método de impressão para visualização
    pub fn println(&self) {
        match self {
            SpedRecord::Generic(reg) => println!("{reg:#?}"),
            SpedRecord::Unknown(reg_str) => println!("Unknown Record: {}", reg_str),
        }
    }
}

/// Estrutura para conter os registros de cada bloco.
/// Usamos um `Vec` para manter a ordem em que os registros foram lidos.
#[derive(Debug, Default)]
pub struct BlocoRegistros {
    pub registros: Vec<SpedRecord>,
}

impl BlocoRegistros {
    pub fn add_record(&mut self, record: SpedRecord) {
        // Para manter a ordem de linha, podemos inserir ordenadamente,
        // mas para arquivos SPED, geralmente os registros já estão ordenados.
        // Uma simples adição ao final é suficiente e mais eficiente se a ordem já estiver garantida.
        // Se a ordem for crucial e não garantida, uma inserção binária ou sort no final seria necessária.
        self.registros.push(record);
    }
}

/// Estrutura principal para armazenar todos os registros SPED, agrupados por bloco.
#[derive(Debug, Default)]
pub struct SpedFile {
    // Agora podemos usar um HashMap para todos os blocos, tornando-o mais dinâmico
    pub blocos: HashMap<char, BlocoRegistros>,
}

impl SpedFile {
    pub fn new() -> Self {
        SpedFile::default()
    }

    pub fn add_record(&mut self, record: SpedRecord) {
        self.blocos
            .entry(record.bloco())
            .or_default()
            .add_record(record);
    }

    /// Ordena todos os registros dentro de cada bloco pelo número da linha.
    pub fn sort_records_by_line_number_serial(&mut self) {
        self.blocos
            .iter_mut()
            .for_each(|(_bloco_char, bloco_registros)| {
                bloco_registros
                    .registros
                    .sort_by_key(|sped_record| sped_record.line_number());
            });
    }

    /// Ordena todos os registros dentro de cada bloco pelo número da linha em paralelo.
    pub fn sort_records_by_line_number(&mut self) {
        // par_iter_mut: Itera sobre os blocos simultaneamente
        self.blocos
            .par_iter_mut()
            .for_each(|(_bloco_char, bloco_registros)| {
                // par_sort_by_key: Ordena o vetor gigante do bloco (ex: Bloco C) usando várias threads
                bloco_registros
                    .registros
                    .par_sort_unstable_by_key(|sped_record| sped_record.line_number());
            });
    }

    pub fn obter_bloco_option(&self, name: char) -> Option<&Vec<SpedRecord>> {
        self.blocos
            .get(&name)
            .map(|bloco_registros| &bloco_registros.registros)
    }

    /// Obtém um registro específico usando o método downcast_ref
    pub fn obter_registro<T: 'static>(&self, nome_registro: &str) -> EFDResult<&T> {
        let bloco_char = nome_registro.chars().next().unwrap_or('?');

        self.blocos
            .get(&bloco_char)
            .and_then(|bloco| {
                // Procura o primeiro registro com o nome solicitado
                bloco
                    .registros
                    .iter()
                    .find(|r| r.registro_name() == nome_registro)
            })
            // Se não encontrar o registro na lista, erro NotFound
            .ok_or_else(|| EFDError::RecordNotFound(nome_registro.to_string()))
            // Se encontrar, tenta o downcast (pode retornar RecordCastError)
            .and_then(|record| record.downcast_ref::<T>())
    }

    /// Retorna lista de registros de um tipo específico usando filter_map e downcast
    pub fn obter_lista_registros<T: 'static + Sync>(&self, nome_registro: &str) -> Vec<&T> {
        let bloco_char = nome_registro.chars().next().unwrap_or('?');

        self.blocos
            .get(&bloco_char)
            .map(|bloco_registros| {
                bloco_registros
                    .registros
                    .par_iter()
                    .filter(|r| r.registro_name() == nome_registro)
                    // Aqui usamos ok() para transformar o Result em Option,
                    // descartando silenciosamente erros de cast (filter_map),
                    // pois queremos apenas os que correspondem a T.
                    .filter_map(|r| r.downcast_ref::<T>().ok())
                    .collect()
            })
            .unwrap_or_default()
    }

    // Método para imprimir a estrutura da SpedFile (opcional, para debug)
    pub fn print_structure(&self) {
        println!("--- Sped File Structure ---");
        let mut sorted_block_chars: Vec<&char> = self.blocos.keys().collect();
        sorted_block_chars.sort();

        for bloco_char in sorted_block_chars {
            if let Some(bloco_registros) = self.blocos.get(bloco_char) {
                println!(
                    "Bloco {} ({} registros):",
                    bloco_char,
                    bloco_registros.registros.len()
                );
                for rec in &bloco_registros.registros {
                    println!(
                        "  [{:4}] L{: <5} {:?}",
                        rec.registro_name(),
                        rec.line_number(),
                        rec
                    );
                }
            }
            println!();
        }
        println!("---------------------------");
    }
}
