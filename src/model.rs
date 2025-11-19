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

    /// Método de impressão para visualização
    pub fn println2(&self) {
        println!("{:#?}", self);
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
        let bloco_char = record.bloco();
        self.blocos
            .entry(bloco_char)
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

    /// Tenta obter uma referência para um único registro específico (ex: "0000").
    /// Estilo Funcional: Option chain -> Result conversion.
    pub fn obter_registro<T: 'static>(&self, nome_registro: &str) -> EFDResult<&T> {
        let bloco_char = nome_registro.chars().next().unwrap_or(' ');

        self.blocos
            .get(&bloco_char)
            // 1. Tenta encontrar o registro pelo nome dentro do bloco
            .and_then(|bloco_registros| {
                bloco_registros
                    .registros
                    .iter()
                    .find(|r| r.registro_name() == nome_registro)
            })
            // 2. Se não achou o bloco ou o registro, converte Option::None em Erro
            .ok_or_else(|| EFDError::RecordNotFound(nome_registro.to_string()))
            // 3. Se achou, tenta fazer o downcast
            .and_then(|record| {
                match record {
                    SpedRecord::Generic(inner) => inner
                        .as_any()
                        .downcast_ref::<T>()
                        // Se falhar o cast, retorna erro específico
                        .ok_or_else(|| EFDError::RecordCastError(nome_registro.to_string())),
                    // Se for Unknown ou outro tipo
                    _ => Err(EFDError::RecordCastError(nome_registro.to_string())),
                }
            })
    }

    /// Retorna um Vec com referências para todos os registros de um tipo (ex: "C100").
    /// Estilo Funcional: Iterator flattening e FilterMap.
    pub fn obter_lista_registros<T: 'static>(&self, nome_registro: &str) -> Vec<&T> {
        let bloco_char = nome_registro.chars().next().unwrap_or(' ');

        self.blocos
            .get(&bloco_char)
            .into_iter() // Converte Option<&Bloco> em Iterator (0 ou 1 item)
            .flat_map(|b| b.registros.iter()) // Abre o Vec de registros
            .filter(|r| r.registro_name() == nome_registro) // Filtra pelo nome (String)
            .filter_map(|r| {
                // Tenta o downcast e descarta os que falharem (retorna Option)
                match r {
                    SpedRecord::Generic(inner) => inner.as_any().downcast_ref::<T>(),
                    _ => None,
                }
            })
            .collect()
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
