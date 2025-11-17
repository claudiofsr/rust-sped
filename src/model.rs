use crate::SpedRecordTrait;
use std::collections::HashMap;

#[derive(Debug)]
pub enum SpedRecord {
    /// Contém qualquer registro SPED que implemente SpedRecordTrait
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
    pub fn sort_records_by_line_number(&mut self) {
        for (_, block) in self.blocos.iter_mut() {
            block.registros.sort_by_key(|record| record.line_number());
        }
    }

    pub fn obter_bloco_option(&self, name: char) -> Option<&Vec<SpedRecord>> {
        self.blocos
            .get(&name)
            .map(|bloco_registros| &bloco_registros.registros)
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
