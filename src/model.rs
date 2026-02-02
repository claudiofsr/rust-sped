use crate::{EFDError, EFDResult, ResultExt, SpedRecordTrait, blocos::*};
use rayon::prelude::*;

// ============================================================================
// Hierarquia de Enums (Model)
// ============================================================================

/*
1.2 - Organização dos Blocos

Os blocos devem ser organizados e dispostos na sequência estabelecida no item 2.5 do Manual do Leiaute
da EFD-Contribuições e alterações, ou seja, inicia-se com o bloco 0 e seus registros, na sequência o
bloco A e registros correspondentes, depois os blocos C, D, F, I, M, P e 1 e, ao final, o bloco 9,
que encerra o arquivo digital da escrituração.

    /// Enum Principal: Representa os blocos do arquivo EFD Contribuições.
    #[derive(Debug, Clone)]
    pub enum SpedRecord {
        Bloco0(Box<Bloco0>),
        BlocoA(Box<BlocoA>),
        BlocoC(Box<BlocoC>),
        BlocoD(Box<BlocoD>),
        BlocoF(Box<BlocoF>),
        BlocoI(Box<BlocoI>),
        BlocoM(Box<BlocoM>),
        BlocoP(Box<BlocoP>),
        Bloco1(Box<Bloco1>),
        Bloco9(Box<Bloco9>),
    }

    impl SpedRecord {
    ...
    }
    ...

    /// Documentos Fiscais - Serviços (ISS)
    #[derive(Debug, Clone)]
    pub enum BlocoA {
        RA001(RegistroA001),
        RA010(RegistroA010),
        RA100(RegistroA100),
        RA110(RegistroA110),
        RA111(RegistroA111),
        RA120(RegistroA120),
        RA170(RegistroA170),
        RA990(RegistroA990),
    }
    ...
*/

/// Macro para implementar enum SpedRecord e métodos em enums de blocos.
///
/// Gera as funções line_number() e registro_name() para cada registro.
macro_rules! define_sped_record {
    (
        $(
            $(#[$meta:meta])* // Captura doc comments (///) ou atributos (#[...])
            $bloco:ident {
                $( $variant:ident => ($registro:ident, $reg_name:expr) ),* $(,)?
            }
        ),*
    ) => {
        // 1. Geração dos Enums de cada Bloco (Bloco0, BlocoA, BlocoC, etc)
        $(
            $(#[$meta])* // Aplica os comentários/atributos capturados ao Enum do bloco
            #[derive(Debug, Clone)]
            pub enum $bloco {
                $( $variant($registro), )*
            }

            impl $bloco {
                #[inline(always)]
                pub fn nivel(&self) -> u16 {
                    match self { $( Self::$variant(r) => r.nivel, )* }
                }

                #[inline(always)]
                pub fn line_number(&self) -> usize {
                    match self { $( Self::$variant(r) => r.line_number, )* }
                }

                #[inline(always)]
                pub fn registro_name(&self) -> &str {
                    // O nome do registro é uma constante estática no binário
                    match self { $( Self::$variant(_) => $reg_name, )* }
                }

                /// Obtém o caractere do Bloco (primeiro caractere do registro)
                #[inline(always)]
                pub fn bloco(&self) -> char {
                    match self {
                        // Extrai o primeiro char da string literal em tempo de compilação
                        $( Self::$variant(_) => $reg_name.chars().next().unwrap_or('?'), )*
                    }
                }

                /// Permite o downcast genérico acessando a struct interna
                #[inline(always)]
                pub fn as_any(&self) -> &dyn std::any::Any {
                    match self {
                        $( Self::$variant(r) => r, )*
                    }
                }

                /// Retorna a struct interna como Any para downcast mutável
                #[inline(always)]
                pub fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                    match self {
                        $( Self::$variant(r) => r, )*
                    }
                }
            }

            // Macro para implementar a SpedRecordTrait em cada registro.
            //
            // Esta macro vincula os campos internos da struct (nivel, bloco, registro, line_number)
            // aos métodos do trait e fornece a implementação de as_any, necessária para o
            // funcionamento do downcast no SpedFile.

            // Implementa o Trait para o Bloco
            impl $crate::traits::SpedRecordTrait for $bloco {
                #[inline]
                fn nivel(&self) -> u16 {
                    self.nivel()
                }

                #[inline]
                fn line_number(&self) -> usize {
                    self.line_number()
                }

                #[inline]
                fn registro_name(&self) -> &str {
                    self.registro_name()
                }

                #[inline]
                fn bloco(&self) -> char {
                    self.bloco()
                }

                // Estas implementações permitem que o SpedRecord (Enum)
                // converta a referência de volta para a struct concreta.
                #[inline]
                fn as_any(&self) -> &dyn std::any::Any {
                    self.as_any()
                }

                #[inline]
                fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                    self.as_any_mut() // Chama o método do Enum que faz o match
                }
            }
        )*

        // 2. Enum Global SpedRecord com Dupla Indireção (Box)
        /// Representa qualquer registro do arquivo.
        /// Otimizado para 16 bytes na stack, ideal para processamento paralelo.
        #[derive(Debug, Clone)]
        pub enum SpedRecord {
            $( $bloco(Box<$bloco>), )*
        }

        impl SpedRecord {
            /// Retorna o número da linha original no arquivo texto.
            #[inline]
            pub fn line_number(&self) -> usize {
                match self { $( Self::$bloco(b) => b.line_number(), )* }
            }

            /// Retorna o nome do registro (Ex: "C100", "0000").
            #[inline]
            pub fn registro_name(&self) -> &str {
                match self { $( Self::$bloco(b) => b.registro_name(), )* }
            }

            /// Identifica a qual bloco o registro pertence.
            #[inline]
            pub fn bloco(&self) -> char {
                match self { $( Self::$bloco(b) => b.bloco(), )* }
            }

            #[inline]
            pub fn as_any(&self) -> &dyn std::any::Any {
                match self { $( Self::$bloco(b) => b.as_any(), )* }
            }

            #[inline]
            pub fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                match self { $( Self::$bloco(b) => b.as_any_mut(), )* }
            }

            /// Downcast genérico para converter SpedRecord em uma struct concreta (ex: RegistroC100).
            ///
            /// Se o registro atual não for do tipo `T`, retorna `EFDError::RecordCastError`
            /// contendo o nome do registro que **realmente** está armazenado.
            #[inline]
            pub fn downcast_ref<T: 'static>(&self) -> EFDResult<&T> {
                self.as_any().downcast_ref::<T>().map_loc(|_|
                    EFDError::RecordCastError(self.registro_name().to_string())
                )
            }

            /// Método de impressão para visualização formatada.
            ///
            /// Aproveita o #[derive(Debug)] implementado recursivamente em todos os blocos e registros.
            #[inline]
            pub fn println(&self) {
                // Como todos os enums internos (Bloco0, BlocoC, etc) implementam Debug,
                // o Rust imprimirá a estrutura completa, ex: BlocoC(RC100(RegistroC100 { ... }))
                match self { $( Self::$bloco(reg) => println!("{reg:#?}"), )* }
            }

            /// Método de impressão para visualização formatada.
            ///
            /// Acrescenta a tag SpedRecord:: do início da impressão.
            #[inline]
            pub fn println_v2(&self) {
                println!("{self:#?}");
            }
        }
    };
}

// ============================================================================
// Definições de Enums por Bloco e Aplicação da Macro
// ============================================================================

// Impede que o cargo fmt quebre as linhas desta macro
#[rustfmt::skip]
define_sped_record! {
    /// Abertura, Identificação e Referências
    Bloco0 {
        R0000 => (Registro0000, "0000"), R0001 => (Registro0001, "0001"),
        R0035 => (Registro0035, "0035"), R0100 => (Registro0100, "0100"),
        R0110 => (Registro0110, "0110"), R0111 => (Registro0111, "0111"),
        R0120 => (Registro0120, "0120"), R0140 => (Registro0140, "0140"),
        R0145 => (Registro0145, "0145"), R0150 => (Registro0150, "0150"),
        R0190 => (Registro0190, "0190"), R0200 => (Registro0200, "0200"),
        R0205 => (Registro0205, "0205"), R0206 => (Registro0206, "0206"),
        R0208 => (Registro0208, "0208"), R0400 => (Registro0400, "0400"),
        R0450 => (Registro0450, "0450"), R0500 => (Registro0500, "0500"),
        R0600 => (Registro0600, "0600"), R0900 => (Registro0900, "0900"),
        R0990 => (Registro0990, "0990"),
    },

    /// Documentos Fiscais - Serviços (ISS)
    BlocoA {
        RA001 => (RegistroA001, "A001"), RA010 => (RegistroA010, "A010"),
        RA100 => (RegistroA100, "A100"), RA110 => (RegistroA110, "A110"),
        RA111 => (RegistroA111, "A111"), RA120 => (RegistroA120, "A120"),
        RA170 => (RegistroA170, "A170"), RA990 => (RegistroA990, "A990"),
    },

    /// Documentos Fiscais I – Mercadorias (ICMS/IPI)
    BlocoC {
        RC001 => (RegistroC001, "C001"), RC010 => (RegistroC010, "C010"),
        RC100 => (RegistroC100, "C100"), RC110 => (RegistroC110, "C110"),
        RC111 => (RegistroC111, "C111"), RC120 => (RegistroC120, "C120"),
        RC170 => (RegistroC170, "C170"), RC175 => (RegistroC175, "C175"),
        RC180 => (RegistroC180, "C180"), RC181 => (RegistroC181, "C181"),
        RC185 => (RegistroC185, "C185"), RC188 => (RegistroC188, "C188"),
        RC190 => (RegistroC190, "C190"), RC191 => (RegistroC191, "C191"),
        RC195 => (RegistroC195, "C195"), RC198 => (RegistroC198, "C198"),
        RC199 => (RegistroC199, "C199"), RC380 => (RegistroC380, "C380"),
        RC381 => (RegistroC381, "C381"), RC385 => (RegistroC385, "C385"),
        RC395 => (RegistroC395, "C395"), RC396 => (RegistroC396, "C396"),
        RC400 => (RegistroC400, "C400"), RC405 => (RegistroC405, "C405"),
        RC481 => (RegistroC481, "C481"), RC485 => (RegistroC485, "C485"),
        RC489 => (RegistroC489, "C489"), RC490 => (RegistroC490, "C490"),
        RC491 => (RegistroC491, "C491"), RC495 => (RegistroC495, "C495"),
        RC499 => (RegistroC499, "C499"), RC500 => (RegistroC500, "C500"),
        RC501 => (RegistroC501, "C501"), RC505 => (RegistroC505, "C505"),
        RC509 => (RegistroC509, "C509"), RC600 => (RegistroC600, "C600"),
        RC601 => (RegistroC601, "C601"), RC605 => (RegistroC605, "C605"),
        RC609 => (RegistroC609, "C609"), RC800 => (RegistroC800, "C800"),
        RC810 => (RegistroC810, "C810"), RC820 => (RegistroC820, "C820"),
        RC830 => (RegistroC830, "C830"), RC860 => (RegistroC860, "C860"),
        RC870 => (RegistroC870, "C870"), RC880 => (RegistroC880, "C880"),
        RC890 => (RegistroC890, "C890"), RC990 => (RegistroC990, "C990"),
    },

    /// Documentos Fiscais II – Serviços (ICMS)
    BlocoD {
        RD001 => (RegistroD001, "D001"), RD010 => (RegistroD010, "D010"),
        RD100 => (RegistroD100, "D100"), RD101 => (RegistroD101, "D101"),
        RD105 => (RegistroD105, "D105"), RD111 => (RegistroD111, "D111"),
        RD200 => (RegistroD200, "D200"), RD201 => (RegistroD201, "D201"),
        RD205 => (RegistroD205, "D205"), RD209 => (RegistroD209, "D209"),
        RD300 => (RegistroD300, "D300"), RD309 => (RegistroD309, "D309"),
        RD350 => (RegistroD350, "D350"), RD359 => (RegistroD359, "D359"),
        RD500 => (RegistroD500, "D500"), RD501 => (RegistroD501, "D501"),
        RD505 => (RegistroD505, "D505"), RD509 => (RegistroD509, "D509"),
        RD600 => (RegistroD600, "D600"), RD601 => (RegistroD601, "D601"),
        RD605 => (RegistroD605, "D605"), RD609 => (RegistroD609, "D609"),
        RD990 => (RegistroD990, "D990"),
    },

    /// Demais Documentos e Operações
    BlocoF {
        RF001 => (RegistroF001, "F001"), RF010 => (RegistroF010, "F010"),
        RF100 => (RegistroF100, "F100"), RF111 => (RegistroF111, "F111"),
        RF120 => (RegistroF120, "F120"), RF129 => (RegistroF129, "F129"),
        RF130 => (RegistroF130, "F130"), RF139 => (RegistroF139, "F139"),
        RF150 => (RegistroF150, "F150"), RF200 => (RegistroF200, "F200"),
        RF205 => (RegistroF205, "F205"), RF210 => (RegistroF210, "F210"),
        RF211 => (RegistroF211, "F211"), RF500 => (RegistroF500, "F500"),
        RF509 => (RegistroF509, "F509"), RF510 => (RegistroF510, "F510"),
        RF519 => (RegistroF519, "F519"), RF525 => (RegistroF525, "F525"),
        RF550 => (RegistroF550, "F550"), RF559 => (RegistroF559, "F559"),
        RF560 => (RegistroF560, "F560"), RF569 => (RegistroF569, "F569"),
        RF600 => (RegistroF600, "F600"), RF700 => (RegistroF700, "F700"),
        RF800 => (RegistroF800, "F800"), RF990 => (RegistroF990, "F990"),
    },

    /// Operações das Instituições Financeiras e Assemelhadas
    BlocoI {
        RI001 => (RegistroI001, "I001"), RI010 => (RegistroI010, "I010"),
        RI100 => (RegistroI100, "I100"), RI199 => (RegistroI199, "I199"),
        RI200 => (RegistroI200, "I200"), RI299 => (RegistroI299, "I299"),
        RI300 => (RegistroI300, "I300"), RI399 => (RegistroI399, "I399"),
        RI990 => (RegistroI990, "I990"),
    },

    /// Apuração da Contribuição e Crédito de PIS/PASEP e da COFINS
    BlocoM {
        RM001 => (RegistroM001, "M001"), RM100 => (RegistroM100, "M100"),
        RM105 => (RegistroM105, "M105"), RM110 => (RegistroM110, "M110"),
        RM115 => (RegistroM115, "M115"), RM200 => (RegistroM200, "M200"),
        RM205 => (RegistroM205, "M205"), RM210 => (RegistroM210, "M210"),
        RM210Antigo => (RegistroM210Antigo, "M210"),
        RM211 => (RegistroM211, "M211"), RM215 => (RegistroM215, "M215"),
        RM220 => (RegistroM220, "M220"), RM225 => (RegistroM225, "M225"),
        RM230 => (RegistroM230, "M230"), RM300 => (RegistroM300, "M300"),
        RM350 => (RegistroM350, "M350"), RM400 => (RegistroM400, "M400"),
        RM410 => (RegistroM410, "M410"), RM500 => (RegistroM500, "M500"),
        RM505 => (RegistroM505, "M505"), RM510 => (RegistroM510, "M510"),
        RM515 => (RegistroM515, "M515"), RM600 => (RegistroM600, "M600"),
        RM605 => (RegistroM605, "M605"), RM610 => (RegistroM610, "M610"),
        RM610Antigo => (RegistroM610Antigo, "M610"),
        RM611 => (RegistroM611, "M611"), RM615 => (RegistroM615, "M615"),
        RM620 => (RegistroM620, "M620"), RM625 => (RegistroM625, "M625"),
        RM630 => (RegistroM630, "M630"), RM700 => (RegistroM700, "M700"),
        RM800 => (RegistroM800, "M800"), RM810 => (RegistroM810, "M810"),
        RM990 => (RegistroM990, "M990"),
    },

    /// Apuração da Contribuição Previdenciária sobre a Receita Bruta
    BlocoP {
        RP001 => (RegistroP001, "P001"), RP010 => (RegistroP010, "P010"),
        RP100 => (RegistroP100, "P100"), RP110 => (RegistroP110, "P110"),
        RP199 => (RegistroP199, "P199"), RP200 => (RegistroP200, "P200"),
        RP210 => (RegistroP210, "P210"), RP990 => (RegistroP990, "P990"),
    },

    /// Complemento da Escrituração – Controle de Saldos de Créditos e de Retenções
    Bloco1 {
        R1001 => (Registro1001, "1001"), R1010 => (Registro1010, "1010"),
        R1011 => (Registro1011, "1011"), R1020 => (Registro1020, "1020"),
        R1050 => (Registro1050, "1050"), R1100 => (Registro1100, "1100"),
        R1101 => (Registro1101, "1101"), R1102 => (Registro1102, "1102"),
        R1200 => (Registro1200, "1200"), R1210 => (Registro1210, "1210"),
        R1220 => (Registro1220, "1220"), R1300 => (Registro1300, "1300"),
        R1500 => (Registro1500, "1500"), R1501 => (Registro1501, "1501"),
        R1502 => (Registro1502, "1502"), R1600 => (Registro1600, "1600"),
        R1610 => (Registro1610, "1610"), R1620 => (Registro1620, "1620"),
        R1700 => (Registro1700, "1700"), R1800 => (Registro1800, "1800"),
        R1809 => (Registro1809, "1809"), R1900 => (Registro1900, "1900"),
        R1990 => (Registro1990, "1990"),
    },

    /// Controle e Encerramento do Arquivo Digital
    Bloco9 {
        R9001 => (Registro9001, "9001"), R9900 => (Registro9900, "9900"),
        R9990 => (Registro9990, "9990"), R9999 => (Registro9999, "9999"),
    }
}

// ============================================================================
// Estrutura Principal de Dados
// ============================================================================

/// Estrutura principal para armazenar todos os registros SPED, agrupados por bloco.
#[derive(Debug, Default)]
pub struct SpedFile {
    pub bloco_0: Vec<Bloco0>,
    pub bloco_a: Vec<BlocoA>,
    pub bloco_c: Vec<BlocoC>,
    pub bloco_d: Vec<BlocoD>,
    pub bloco_f: Vec<BlocoF>,
    pub bloco_i: Vec<BlocoI>,
    pub bloco_m: Vec<BlocoM>,
    pub bloco_p: Vec<BlocoP>,
    pub bloco_1: Vec<Bloco1>,
    pub bloco_9: Vec<Bloco9>,
}

impl SpedFile {
    pub fn new() -> Self {
        Self::default()
    }

    /// Extrai o Bloco0 de dentro do SpedFile, deixando um Vec vazio no lugar.
    ///
    /// Isso evita clonar os dados; você está apenas movendo o ponteiro da heap.
    /// Use std::mem::take para "roubar" o Bloco0 de dentro da estrutura SpedFile antes de colocá-la no Arc.
    /// Isso deixa um vetor vazio (que não ocupa quase nada de RAM) no lugar do Bloco0.
    ///
    /// Útil para processar SpedContext e liberar memória antes de processar os demais blocos.
    pub fn take_bloco_0(&mut self) -> Vec<Bloco0> {
        std::mem::take(&mut self.bloco_0)
    }

    /// Retorna uma referência direta ao vetor do BlocoA.
    ///
    /// Getters diretos para os blocos (Custo zero)
    pub fn get_bloco_a(&self) -> &[BlocoA] {
        &self.bloco_a
    }
    pub fn get_bloco_c(&self) -> &[BlocoC] {
        &self.bloco_c
    }
    pub fn get_bloco_d(&self) -> &[BlocoD] {
        &self.bloco_d
    }
    pub fn get_bloco_f(&self) -> &[BlocoF] {
        &self.bloco_f
    }
    pub fn get_bloco_i(&self) -> &[BlocoI] {
        &self.bloco_i
    }
    pub fn get_bloco_m(&self) -> &[BlocoM] {
        &self.bloco_m
    }
    pub fn get_bloco_p(&self) -> &[BlocoP] {
        &self.bloco_p
    }
    pub fn get_bloco_1(&self) -> &[Bloco1] {
        &self.bloco_1
    }
    pub fn get_bloco_9(&self) -> &[Bloco9] {
        &self.bloco_9
    }

    /// Une outro SpedFile a este de forma ultra-eficiente.
    ///
    /// O seu uso de append é mais eficiente que extend para este caso específico.
    ///
    /// - extend: Trabalha com qualquer iterador.
    ///   Ele percorre os elementos um a um. Se você passar um Vec, ele terá que lidar com o iterador do vetor.
    ///
    /// - append: É uma operação de "baixo nível" entre dois Vecs.
    ///   Ele move os elementos em bloco (bitcopy/memcpy) e reseta o vetor de origem para o estado vazio.
    ///   É a forma mais rápida de fundir dois vetores em Rust quando você tem acesso mutável a ambos.
    pub fn merge(&mut self, mut other: Self) {
        // Função auxiliar interna para fundir vetores sem clonagem e com realocação mínima.
        // O compilador fará o inline disto perfeitamente.
        #[inline]
        fn merge_vecs<T>(dest: &mut Vec<T>, src: &mut Vec<T>) {
            // Se a origem estiver vazia, não há nada a fazer.
            if src.is_empty() {
                return;
            }

            if dest.is_empty() {
                // O(1): Apenas troca os ponteiros da Heap (ponteiro, len e cap).
                // O vetor 'src' torna-se o 'dest' instantaneamente.
                std::mem::swap(dest, src);
            } else {
                // Se ambos têm dados, garantimos a capacidade total de uma vez só.
                dest.reserve(src.len());
                // O(N): Move os elementos de src para o final de dest.
                // append() é preferível a extend() para Vec -> Vec.
                dest.append(src);
            }
        }

        merge_vecs(&mut self.bloco_0, &mut other.bloco_0);
        merge_vecs(&mut self.bloco_a, &mut other.bloco_a);
        merge_vecs(&mut self.bloco_c, &mut other.bloco_c);
        merge_vecs(&mut self.bloco_d, &mut other.bloco_d);
        merge_vecs(&mut self.bloco_f, &mut other.bloco_f);
        merge_vecs(&mut self.bloco_i, &mut other.bloco_i);
        merge_vecs(&mut self.bloco_m, &mut other.bloco_m);
        merge_vecs(&mut self.bloco_p, &mut other.bloco_p);
        merge_vecs(&mut self.bloco_1, &mut other.bloco_1);
        merge_vecs(&mut self.bloco_9, &mut other.bloco_9);
    }

    /// Adiciona um registro ao bloco correto usando pattern matching no Enum.
    /// Adiciona o registro movendo-o para o vetor otimizado.
    pub fn add_record(&mut self, record: SpedRecord) {
        match record {
            SpedRecord::Bloco0(r) => self.bloco_0.push(*r),
            SpedRecord::BlocoA(r) => self.bloco_a.push(*r),
            SpedRecord::BlocoC(r) => self.bloco_c.push(*r),
            SpedRecord::BlocoD(r) => self.bloco_d.push(*r),
            SpedRecord::BlocoF(r) => self.bloco_f.push(*r),
            SpedRecord::BlocoI(r) => self.bloco_i.push(*r),
            SpedRecord::BlocoM(r) => self.bloco_m.push(*r),
            SpedRecord::BlocoP(r) => self.bloco_p.push(*r),
            SpedRecord::Bloco1(r) => self.bloco_1.push(*r),
            SpedRecord::Bloco9(r) => self.bloco_9.push(*r),
        }
    }

    /// Ordena todos os registros de todos os blocos pelo número da linha (Serial).
    pub fn sort_records_by_line_number_serial(&mut self) {
        self.bloco_0.sort_unstable_by_key(|r| r.line_number());
        self.bloco_a.sort_unstable_by_key(|r| r.line_number());
        self.bloco_c.sort_unstable_by_key(|r| r.line_number());
        self.bloco_d.sort_unstable_by_key(|r| r.line_number());
        self.bloco_f.sort_unstable_by_key(|r| r.line_number());
        self.bloco_i.sort_unstable_by_key(|r| r.line_number());
        self.bloco_m.sort_unstable_by_key(|r| r.line_number());
        self.bloco_p.sort_unstable_by_key(|r| r.line_number());
        self.bloco_1.sort_unstable_by_key(|r| r.line_number());
        self.bloco_9.sort_unstable_by_key(|r| r.line_number());
    }

    /// Ordenação paralela eficiente usando Rayon sobre os campos.
    pub fn sort_records_by_line_number(&mut self) {
        // Como os tipos são diferentes, ordenamos campo a campo em paralelo.
        rayon::scope(|s| {
            s.spawn(|_| self.bloco_0.par_sort_unstable_by_key(|r| r.line_number()));
            s.spawn(|_| self.bloco_a.par_sort_unstable_by_key(|r| r.line_number()));
            s.spawn(|_| self.bloco_c.par_sort_unstable_by_key(|r| r.line_number()));
            s.spawn(|_| self.bloco_d.par_sort_unstable_by_key(|r| r.line_number()));
            s.spawn(|_| self.bloco_f.par_sort_unstable_by_key(|r| r.line_number()));
            s.spawn(|_| self.bloco_i.par_sort_unstable_by_key(|r| r.line_number()));
            s.spawn(|_| self.bloco_m.par_sort_unstable_by_key(|r| r.line_number()));
            s.spawn(|_| self.bloco_p.par_sort_unstable_by_key(|r| r.line_number()));
            s.spawn(|_| self.bloco_1.par_sort_unstable_by_key(|r| r.line_number()));
            s.spawn(|_| self.bloco_9.par_sort_unstable_by_key(|r| r.line_number()));
        });
    }

    /// Busca o primeiro registro imutável com máxima performance.
    /// O(1) para localizar o bloco e O(N) para localizar o registro dentro do bloco.
    ///
    /// Ideal para buscar registros únicos como '0000', '0100', etc.
    pub fn obter_registro<T: 'static>(&self, nome_reg: &str) -> EFDResult<&T> {
        let bloco_char = nome_reg
            .chars()
            .next()
            .map_loc(|_| EFDError::KeyNotFound(nome_reg.to_string()))?;

        // Macro local para busca estática (Zero Cost Abstraction) e manter o código DRY
        macro_rules! find_and_cast {
            ($bloco_vec:expr) => {
                $bloco_vec
                    .iter()
                    .find(|r| r.registro_name() == nome_reg)
                    .map_loc(|_| EFDError::RecordNotFound(nome_reg.to_string()))?
                    .as_any()
                    .downcast_ref::<T>()
                    .map_loc(|_| EFDError::RecordCastError(nome_reg.to_string()))
            };
        }

        match bloco_char {
            '0' => find_and_cast!(self.bloco_0),
            'A' => find_and_cast!(self.bloco_a),
            'C' => find_and_cast!(self.bloco_c),
            'D' => find_and_cast!(self.bloco_d),
            'F' => find_and_cast!(self.bloco_f),
            'I' => find_and_cast!(self.bloco_i),
            'M' => find_and_cast!(self.bloco_m),
            'P' => find_and_cast!(self.bloco_p),
            '1' => find_and_cast!(self.bloco_1),
            '9' => find_and_cast!(self.bloco_9),
            _ => Err(EFDError::KeyNotFound(nome_reg.to_string())).loc(),
        }
    }

    /// Obtém uma referência mutável para o primeiro registro encontrado com o nome solicitado.
    /// Realiza a busca diretamente no bloco correto para máxima performance.
    ///
    /// Exemplo de uso:
    /// if let Some(reg) = sped_file.obter_registro_mut::<Registro0000>("0000") {
    ///     reg.nome = Some(Arc::from("NOVO NOME"));
    /// }
    pub fn obter_registro_mut<T: 'static>(&mut self, nome_reg: &str) -> Option<&mut T> {
        // Identifica o bloco pelo primeiro caractere do nome do registro (ex: 'C' em "C100")
        let bloco_char = nome_reg.chars().next()?;

        // Macro local para evitar repetição exaustiva de código (DRY)
        // O operador '?' após o find garante que se não houver o registro, o braço do match retorna None
        macro_rules! find_and_cast {
            ($bloco_vec:expr) => {
                $bloco_vec
                    .iter_mut()
                    .find(|r| r.registro_name() == nome_reg)?
                    .as_any_mut()
                    .downcast_mut::<T>()
            };
        }

        // Despacho direto para o vetor específico.
        // Todos os braços retornam Option<&mut T>.
        match bloco_char {
            '0' => find_and_cast!(self.bloco_0),
            'A' => find_and_cast!(self.bloco_a),
            'C' => find_and_cast!(self.bloco_c),
            'D' => find_and_cast!(self.bloco_d),
            'F' => find_and_cast!(self.bloco_f),
            'I' => find_and_cast!(self.bloco_i),
            'M' => find_and_cast!(self.bloco_m),
            'P' => find_and_cast!(self.bloco_p),
            '1' => find_and_cast!(self.bloco_1),
            '9' => find_and_cast!(self.bloco_9),
            _ => None,
        }
    }

    /// Retorna uma lista filtrada de registros específicos em paralelo.
    ///
    /// Esta implementação é O(1) para encontrar o bloco e O(N/CPUs) para filtrar.
    /// Usa o 'as_any().downcast_ref::<T>()' para converter a referência do Enum
    /// para o tipo concreto solicitado.
    pub fn obter_lista_registros<T: 'static + Sync>(&self, nome_registro: &str) -> Vec<&T> {
        let bloco_char = nome_registro.chars().next().unwrap_or('?');

        // Macro interna para evitar repetição de código (DRY)
        // O Rayon precisa do tipo concreto em cada braço do match para otimizar.
        macro_rules! filter_parallel {
            ($bloco_vec:expr) => {
                $bloco_vec
                    .par_iter()
                    .filter(|r| r.registro_name() == nome_registro)
                    .filter_map(|r| r.as_any().downcast_ref::<T>())
                    .collect()
            };
        }

        match bloco_char {
            '0' => filter_parallel!(self.bloco_0),
            'A' => filter_parallel!(self.bloco_a),
            'C' => filter_parallel!(self.bloco_c),
            'D' => filter_parallel!(self.bloco_d),
            'F' => filter_parallel!(self.bloco_f),
            'I' => filter_parallel!(self.bloco_i),
            'M' => filter_parallel!(self.bloco_m),
            'P' => filter_parallel!(self.bloco_p),
            '1' => filter_parallel!(self.bloco_1),
            '9' => filter_parallel!(self.bloco_9),
            _ => Vec::new(),
        }
    }

    /// Imprime a árvore hierárquica do arquivo para debug.
    pub fn print_structure(&self) {
        println!("--- Sped File Structure ---");

        // Chamamos o helper para cada campo da struct.
        // A ordem aqui define a ordem da impressão (0, A, C... 9).
        self.print_bloco_helper('0', &self.bloco_0);
        self.print_bloco_helper('A', &self.bloco_a);
        self.print_bloco_helper('C', &self.bloco_c);
        self.print_bloco_helper('D', &self.bloco_d);
        self.print_bloco_helper('F', &self.bloco_f);
        self.print_bloco_helper('I', &self.bloco_i);
        self.print_bloco_helper('M', &self.bloco_m);
        self.print_bloco_helper('P', &self.bloco_p);
        self.print_bloco_helper('1', &self.bloco_1);
        self.print_bloco_helper('9', &self.bloco_9);

        println!("---------------------------");
    }

    /// Helper genérico para imprimir informações de qualquer bloco.
    /// T precisa implementar BlockRecordTrait (para nome e linha)
    /// e Debug (para a representação visual).
    fn print_bloco_helper<T>(&self, bloco_char: char, registros: &[T])
    where
        T: SpedRecordTrait + std::fmt::Debug,
    {
        if registros.is_empty() {
            return;
        }

        println!("Bloco {} ({} registros):", bloco_char, registros.len());
        const MAX: usize = 10;
        let spaces = registros
            .iter()
            .take(MAX)
            .map(|r| r.line_number())
            .max()
            .unwrap_or(1)
            .to_string()
            .len();

        // Imprime apenas os MAX primeiros registros
        for rec in registros.iter().take(MAX) {
            println!(
                "  [{:4}] L{: <spaces$} {:?}",
                rec.registro_name(),
                rec.line_number(),
                rec
            );
        }

        if registros.len() > MAX {
            println!("  ...");
        }
        println!(); // Linha em branco entre blocos
    }
}
