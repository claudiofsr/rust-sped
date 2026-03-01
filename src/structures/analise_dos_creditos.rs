use compact_str::CompactString;
use csv::StringRecord;
use rayon::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize, Serializer};
use serde_aux::prelude::serde_introspect;
use struct_iterable::Iterable;

use std::{
    collections::HashMap,
    //cmp::Reverse,
    fmt::Display,
    hash::Hash,
    ops::{Add, AddAssign, Mul},
};

use tabled::{
    Table, Tabled,
    settings::{
        Alignment, Modify, Style,
        object::{Columns, Rows, Segment},
    },
};

use crate::{
    AppConfig, CSTOption, CodigoDoCredito, CodigoSituacaoTributaria, DECIMAL_ALIQ, DecimalExt,
    Despise, DocsFiscais, EFDResult, ExcelExtension, MesesDoAno, NatBCOption, NaturezaBaseCalculo,
    RowStyle, TipoDeCredito, TipoDeOperacao, TipoDeRateio, Tributo, apurar_receita_bruta,
    consolidar_registros, display_cst, display_decimal, display_mes, display_value,
    realizar_somas_trimestrais, serialize_cst, serialize_decimal, serialize_natureza_opt,
    serialize_option_decimal, verificar_periodo_multiplo,
};

use CodigoSituacaoTributaria::*;
use NaturezaBaseCalculo::*;

use claudiofsr_lib::CFOP_DE_EXPORTACAO;

#[derive(Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Serialize, Deserialize)]
pub struct Chaves {
    // CNPJ Base tem 8 chars. Cabe inteiramente na Stack, zero alocação de Heap.
    //pub path: CompactString,
    pub cnpj_base: CompactString,
    pub ano: Option<i32>,
    pub trimestre: Option<u32>,
    pub mes: Option<MesesDoAno>,
    pub tipo_de_operacao: Option<TipoDeOperacao>,
    pub tipo_de_credito: Option<TipoDeCredito>,
    pub cst: Option<CodigoSituacaoTributaria>,
    pub cfop: Option<u16>,
    pub aliq_pis: Option<Decimal>,
    pub aliq_cofins: Option<Decimal>,
    pub natureza_bc: Option<NaturezaBaseCalculo>,
}

impl Chaves {
    /**
    Checks if the CFOP corresponds to an export operation.

    CFOP de Exportacao:

    . Grupo 7:
        valores entre 7000 e 7999;

    . Fim específico de exportação.


    The `CFOP_DE_EXPORTACAO` constant array is assumed to be sorted
    for efficient binary search.
    */
    pub fn cfop_de_exportacao(&self) -> bool {
        self.cfop
            .is_some_and(|cfop_value| CFOP_DE_EXPORTACAO.binary_search(&cfop_value).is_ok())
    }
}

#[derive(Debug, Default, PartialEq, PartialOrd, Copy, Clone, Serialize, Deserialize)]
pub struct Valores {
    pub valor_item: Decimal,
    pub valor_bc: Decimal,
    pub valor_rbnc_trib: Decimal,
    pub valor_rbnc_ntrib: Decimal,
    pub valor_rbnc_exp: Decimal,
    pub valor_rb_cum: Decimal,
}

impl Add for Valores {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            valor_item: self.valor_item + other.valor_item,
            valor_bc: self.valor_bc + other.valor_bc,
            valor_rbnc_trib: self.valor_rbnc_trib + other.valor_rbnc_trib,
            valor_rbnc_ntrib: self.valor_rbnc_ntrib + other.valor_rbnc_ntrib,
            valor_rbnc_exp: self.valor_rbnc_exp + other.valor_rbnc_exp,
            valor_rb_cum: self.valor_rb_cum + other.valor_rb_cum,
        }
    }
}

// https://doc.rust-lang.org/std/ops/trait.AddAssign.html
/// Executa a operação +=
impl AddAssign for Valores {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

// Multiplying Struct (Valores) by scalars (Decimal) as in linear algebra
impl Mul<Decimal> for Valores {
    type Output = Self;
    fn mul(self, value: Decimal) -> Self {
        Self {
            valor_item: (self.valor_item * value),
            valor_bc: (self.valor_bc * value),
            valor_rbnc_trib: (self.valor_rbnc_trib * value),
            valor_rbnc_ntrib: (self.valor_rbnc_ntrib * value),
            valor_rbnc_exp: (self.valor_rbnc_exp * value),
            valor_rb_cum: (self.valor_rb_cum * value),
        }
    }
}

impl Valores {
    /// Cria uma nova instância de Valores.
    ///
    /// Se os argumentos forem `None`, assume-se 0.0 (o padrão para Decimal).
    pub fn new(valor_item: Option<Decimal>, valor_bc: Option<Decimal>) -> Self {
        Self {
            // unwrap_or_default() retorna o valor interno ou 0.0 (Decimal::default())
            valor_item: valor_item.unwrap_or_default(),
            valor_bc: valor_bc.unwrap_or_default(),

            // Preenche os campos restantes (rbnc_trib, rb_cum, etc.) com 0.0
            ..Self::default()
        }
    }

    /// Calcula a Receita Bruta Não Cumulativa (soma das parcelas)
    #[inline]
    pub fn rec_bruta_nao_cumulativa(&self) -> Decimal {
        self.valor_rbnc_trib + self.valor_rbnc_ntrib + self.valor_rbnc_exp
    }

    /// Calcula Receita Bruta Cumulativa por diferença.
    ///
    /// Receita Bruta Total = Receita Bruta Cumulativa + Receita Bruta Não Cumulativa ->
    /// Receita Bruta Cumulativa = Receita Bruta Total - Receita Bruta Não Cumulativa
    ///
    /// Fórmula: RB Cumulativa = Valor BC (Total) - RB Não Cumulativa Total
    /// Retorna 0.0 se a diferença for negativa ou menor que a tolerância de erro.
    pub fn calcular_rb_cumulativa(&self) -> Decimal {
        let rec_bruta_cumulativa = self.valor_bc - self.rec_bruta_nao_cumulativa();

        if rec_bruta_cumulativa < dec!(0.10) {
            Decimal::ZERO
        } else {
            rec_bruta_cumulativa
        }
    }

    /// Auxiliar para retornar todos os campos como um array.
    /// Útil para operações em lote (somas, comparações, etc) sem alocar memória no Heap.
    #[inline]
    pub fn as_array(&self) -> [Decimal; 6] {
        [
            self.valor_item,
            self.valor_bc,
            self.valor_rbnc_trib,
            self.valor_rbnc_ntrib,
            self.valor_rbnc_exp,
            self.valor_rb_cum,
        ]
    }

    /// Compara se todos os campos são aproximadamente iguais a outro objeto.
    ///
    /// Comparação otimizada sem alocação de Vec.
    ///
    /// # Parameters
    /// - `other`: o outro objeto do qual você está comparando
    /// - `delta`: a tolerância de erro permitida
    ///
    /// # Returns
    /// `true` se os valores forem aproximadamente iguais, `false` caso contrário
    pub fn aproximadamente_iguais(&self, other: &Self, delta: Decimal) -> bool {
        self.as_array()
            .iter()
            .zip(other.as_array())
            // Uso de all: O método all do iterador é "curto-circuito".
            // Se o primeiro campo já for diferente além do delta, ele para a execução ali mesmo, sendo muito eficiente.
            .all(|(a, b)| (a - b).abs() <= delta)
    }

    /// Retorna uma referência mutável para o campo correspondente ao tipo de rateio.
    pub fn obter_campo_de_rateio_mut(&mut self, rateio: TipoDeRateio) -> &mut Decimal {
        match rateio {
            TipoDeRateio::RecBrutaNCumTribMercInterno => &mut self.valor_rbnc_trib,
            TipoDeRateio::RecBrutaNCumNTribMercInterno => &mut self.valor_rbnc_ntrib,
            TipoDeRateio::RecBrutaNCumDeExportacao => &mut self.valor_rbnc_exp,
            TipoDeRateio::RecBrutaCumulativa => &mut self.valor_rb_cum,
        }
    }

    /// Distribui os valores de acordo com o rateio usando CodigoDoCredito
    pub fn distribuir_conforme_rateio(
        &mut self,
        cod: Option<CodigoDoCredito>,
        valor_opt: Option<Decimal>,
    ) {
        if let (Some(codigo_do_credito), Some(valor)) = (cod, valor_opt) {
            *self.obter_campo_de_rateio_mut(codigo_do_credito.rateio) = valor;
        }
    }
}

/// Análise dos Créditos
#[derive(
    Debug, Default, PartialEq, PartialOrd, Clone, Serialize, Deserialize, Tabled, Iterable,
)]
pub struct AnaliseDosCreditos {
    //#[serde(rename = "Arquivo da EFD Contribuições")] // skip
    //#[tabled(rename = "Arquivo da EFD Contribuições", skip)]
    //pub path: CompactString,
    #[serde(rename = "CNPJ Base")]
    #[tabled(rename = "CNPJ Base")]
    pub cnpj_base: CompactString,

    #[serde(rename = "Ano do Período de Apuração")]
    #[tabled(rename = "Ano", display = "display_value")]
    pub ano: Option<i32>,

    #[serde(rename = "Trimestre do Período de Apuração")]
    #[tabled(rename = "Trim", display = "display_value")]
    pub trimestre: Option<u32>,

    #[serde(rename = "Mês do Período de Apuração")]
    #[tabled(rename = "Mês", display = "display_mes")]
    pub mes: Option<MesesDoAno>,

    #[serde(rename = "Tipo de Operação")]
    #[tabled(rename = "Tipo de Operação", display = "display_option")]
    pub tipo_de_operacao: Option<TipoDeOperacao>,

    #[serde(
        rename = "Tipo de Crédito",
        serialize_with = "serialize_tipo_de_credito"
    )]
    #[tabled(rename = "Tipo de Crédito", display = "display_option")]
    pub tipo_de_credito: Option<TipoDeCredito>,

    #[serde(rename = "CST", serialize_with = "serialize_cst")]
    #[tabled(rename = "CST", display = "display_cst")]
    pub cst: Option<CodigoSituacaoTributaria>,

    #[serde(
        rename = "Alíquota de PIS/PASEP",
        serialize_with = "serialize_option_decimal"
    )]
    #[tabled(rename = "Alíquota PIS/PASEP", display = "display_aliquota")]
    pub aliq_pis: Option<Decimal>,

    #[serde(
        rename = "Alíquota de COFINS",
        serialize_with = "serialize_option_decimal"
    )]
    #[tabled(rename = "Alíquota COFINS", display = "display_aliquota")]
    pub aliq_cofins: Option<Decimal>,

    #[serde(
        rename = "Natureza da Base de Cálculo dos Créditos",
        serialize_with = "serialize_natureza_opt"
    )]
    #[tabled(
        rename = "Natureza da Base de Cálculo dos Créditos",
        display = "display_natureza"
    )]
    pub natureza_bc: Option<NaturezaBaseCalculo>,

    #[serde(rename = "Base de Cálculo", serialize_with = "serialize_decimal")]
    #[tabled(rename = "Base de Cálculo", display = "display_decimal")]
    pub valor_bc: Decimal,

    #[serde(
        rename = "Crédito vinculado à Receita Bruta Não Cumulativa: Tributada",
        serialize_with = "serialize_decimal"
    )]
    #[tabled(rename = "RBNC_Trib", display = "display_decimal")]
    pub valor_rbnc_trib: Decimal,

    #[serde(
        rename = "Crédito vinculado à Receita Bruta Não Cumulativa: Não Tributada",
        serialize_with = "serialize_decimal"
    )]
    #[tabled(rename = "RBNC_NTrib", display = "display_decimal")]
    pub valor_rbnc_ntrib: Decimal,

    #[serde(
        rename = "Crédito vinculado à Receita Bruta Não Cumulativa: de Exportação",
        serialize_with = "serialize_decimal"
    )]
    #[tabled(rename = "RBNC_Exp", display = "display_decimal")]
    pub valor_rbnc_exp: Decimal,

    #[serde(
        rename = "Crédito vinculado à Receita Bruta Cumulativa",
        serialize_with = "serialize_decimal"
    )]
    #[tabled(rename = "RB_Cum", display = "display_decimal")]
    pub valor_rb_cum: Decimal,
}

impl ExcelExtension for AnaliseDosCreditos {
    fn row_style(&self) -> RowStyle {
        match self.natureza_bc {
            // "Soma" - Intervalo 101 a 199 ou 300
            // Utiliza o método auxiliar para o range 101..=199 e verifica o 300 explicitamente
            Some(n) if (n.eh_soma_de_bc() || n == BaseSomaValorTotal) => RowStyle::Soma,

            // "Crédito Disponível após Descontos" (221 | 225)
            Some(CreditoAposDescontosPis | CreditoAposDescontosCofins) => RowStyle::Desconto,

            // "Saldo de Crédito Passível de Desconto ou Ressarcimento" (301 | 305)
            Some(SaldoDisponivelPis | SaldoDisponivelCofins) => RowStyle::Saldo,

            // Caso padrão (Naturezas de operação 01..18, ajustes, outros ou None)
            _ => RowStyle::Default,
        }
    }
}

impl AnaliseDosCreditos {
    pub fn get_headers() -> StringRecord {
        // use serde_aux::prelude::serde_introspect;
        let colunas_vec = serde_introspect::<AnaliseDosCreditos>();
        StringRecord::from(colunas_vec)
    }
}

// Helper genérico para o Tabled (opcional, pois Tabled não usa Serde por padrão)
pub fn display_option<T: Display>(opt: &Option<T>) -> String {
    match opt {
        Some(s) => s.to_string(), // Chama o Display do Enum (que retorna a descrição)
        None => String::new(),
    }
}

fn serialize_tipo_de_credito<S>(
    tipo_opt: &Option<TipoDeCredito>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    tipo_opt
        .filter(|tipo| (1..100).contains(&tipo.code())) // Filtra apenas códigos válidos (01-99)
        //.map(|tipo| tipo.descricao_com_codigo()) // Transforma em String formatada
        .serialize(serializer) // Serializa o Option resultante (Some ou None)
}

pub fn display_natureza(nat_opt: &Option<NaturezaBaseCalculo>) -> String {
    nat_opt
        .map(|n| n.descricao_com_codigo())
        .unwrap_or_default()
}

pub fn display_aliquota(valor: &Option<Decimal>) -> String {
    match valor {
        Some(val) => format!("{}%", val.to_formatted_string(DECIMAL_ALIQ)),
        None => String::new(),
    }
}

// ==============================================================================
// Lógica Principal
// ==============================================================================

pub fn consolidar_natureza_da_base_de_calculo(
    config: &AppConfig,
    linhas: &[DocsFiscais],
) -> EFDResult<(Option<String>, Option<String>, Vec<AnaliseDosCreditos>)> {
    // 1. Agregação Inicial (Map-Reduce Genérico)
    let chaves_consolidadas: HashMap<Chaves, Valores> = consolidar_registros(
        linhas,
        |line| line.entrada_de_credito() || line.saida_de_receita_bruta(),
        obter_chaves_valores,
    );

    // 2. Pré-alocação Heurística
    // Assume que nenhum grupo (Receitas ou Créditos) terá mais de 60% dos dados.
    // O '+ 1' garante que não alocamos 0 se o mapa estiver vazio.
    let capacity = chaves_consolidadas.len() / 2 + 1; // 50% + 1

    let mut receita_bruta = HashMap::with_capacity(capacity); //  1 <= CST <= 49
    let mut base_creditos = HashMap::with_capacity(capacity); // 50 <= CST <= 66

    // 3. Separação de Categorias (Partitioning via Move)
    // Consome 'chaves_consolidadas' evitando clones de Strings (Zero-Copy para as chaves)
    for (k, v) in chaves_consolidadas {
        if k.cst.eh_receita_bruta(config.excluir_cst_49) {
            receita_bruta.insert(k, v);
        } else if k.cst.eh_base_de_credito() {
            base_creditos.insert(k, v);
        }
        // Itens que não são nem receita nem crédito são descartados aqui, liberando memória.
    }

    // 4. Processamento de Receita (Delegado para receita_bruta_segregada.rs)
    distribuir_creditos_rateados(linhas, &mut base_creditos);

    let informacoes_de_receita_bruta = apurar_receita_bruta(&receita_bruta);
    let tabela_da_receita_bruta = gerar_tabela_rec(&informacoes_de_receita_bruta);

    // 5. Ajustes e Descontos em Paralelo usando Rayon Scope
    // Inicializamos as variáveis para capturar os resultados das threads
    let mut bcparcial = HashMap::new();
    let mut ajustes = HashMap::new();
    let mut descontos = HashMap::new();

    rayon::scope(|s| {
        // Tarefa 1: Soma da base de cálculo parcial
        // Nota: Passamos o borrow de base_creditos, que é seguro pois o scope garante
        // que as threads terminem antes de base_creditos ser modificado novamente.
        s.spawn(|_| bcparcial = somar_base_de_calculo_valor_parcial(&base_creditos));

        // Tarefa 2: Ajustes rateados
        s.spawn(|_| ajustes = distribuir_ajustes_rateados(linhas));

        // Tarefa 3: Descontos rateados
        s.spawn(|_| descontos = distribuir_descontos_rateados(linhas));
    });

    // 6. Merge dos resultados (Executado após o encerramento do scope)
    // O extend consome os HashMaps temporários e move os dados para base_creditos
    base_creditos.extend(bcparcial);
    base_creditos.extend(ajustes);
    base_creditos.extend(descontos);

    // 7. Cálculos em Cadeia
    base_creditos.extend(apurar_credito_das_contribuicoes(&base_creditos));
    base_creditos.extend(calcular_credito_apos_ajustes(&base_creditos));
    base_creditos.extend(calcular_credito_apos_descontos(&base_creditos));
    base_creditos.extend(somar_base_de_calculo_valor_total(&base_creditos));
    base_creditos.extend(calcular_saldo_de_credito_passivel_de_ressarcimento(
        &base_creditos,
    ));

    // 8. Somas Trimestrais (usando função do utils.rs)
    if verificar_periodo_multiplo(&base_creditos) {
        realizar_somas_trimestrais(&mut base_creditos);
    }

    let base_creditos_ordenado: Vec<(Chaves, Valores)> = ordenar(base_creditos);
    let base_creditos_estruturado: Vec<AnaliseDosCreditos> = get_analises(&base_creditos_ordenado);
    let tabela_de_base_creditos = gerar_tabela_nat(&base_creditos_estruturado);

    Ok((
        tabela_de_base_creditos,
        tabela_da_receita_bruta,
        base_creditos_estruturado,
    ))
}

// Funções de geração de tabela
fn gerar_tabela_rec<'a, T>(lines: &[T]) -> Option<String>
where
    T: Tabled + Deserialize<'a>,
{
    if lines.is_empty() {
        return None;
    }

    // O limit define até qual coluna aplicaremos o alinhamento central
    let limit = 4;

    Some(
        Table::new(lines)
            .with(Modify::new(Columns::new(..limit)).with(Alignment::center()))
            .with(Modify::new(Columns::one(limit)).with(Alignment::left()))
            .with(Modify::new(Columns::new(limit + 1..)).with(Alignment::right()))
            .with(Modify::new(Rows::first()).with(Alignment::center()))
            .with(Style::rounded())
            .to_string(),
    )
}

fn gerar_tabela_nat<'a, T>(lines: &[T]) -> Option<String>
where
    T: Tabled + Deserialize<'a>,
{
    if lines.is_empty() {
        return None;
    }

    // O limit define qual coluna aplicaremos o alinhamento à esquerda
    let limit = 9;

    Some(
        Table::new(lines)
            .with(Modify::new(Segment::all()).with(Alignment::center()))
            .with(Modify::new(Columns::one(limit)).with(Alignment::left()))
            .with(Modify::new(Columns::new(limit + 1..)).with(Alignment::right()))
            .with(Modify::new(Rows::first()).with(Alignment::center()))
            .with(Style::rounded())
            //.with(Modify::new(Rows::one(0)).with(Format::new(|s| s.blue().to_string())))
            //.with(Modify::new(Rows::new(..)).with(Format::new(|s| s.blue().to_string())))
            .to_string(),
    )
}

pub fn obter_chaves_valores(linha: &DocsFiscais) -> (Chaves, Valores) {
    // Informações de CFOP serão utilizadas na segregação da Receita Bruta.
    // Informações de CFOP não serão utilizadas na discriminação da
    // Natureza da Base de Cálculo dos Créditos.
    let cfop = if linha.cst_de_credito() {
        None // Remover informações de CFOP
    } else {
        linha.cfop
    };

    let chaves = Chaves {
        //path: linha.arquivo_efd.to_compact_string(),
        cnpj_base: linha.get_cnpj_base(),
        ano: linha.ano,
        trimestre: linha.trimestre,
        mes: linha.mes,
        tipo_de_operacao: linha.tipo_de_operacao,
        tipo_de_credito: linha.tipo_de_credito,
        cst: linha.cst,
        cfop,
        aliq_pis: linha.aliq_pis,
        aliq_cofins: linha.aliq_cofins,
        natureza_bc: linha.natureza_bc,
    };

    let valores = Valores::new(linha.valor_item, linha.valor_bc);

    (chaves, valores)
}

fn distribuir_creditos_rateados(
    linhas: &[DocsFiscais],
    base_creditos: &mut HashMap<Chaves, Valores>,
) {
    // Transmitir cod_credito (crédito com informação de rateio) para base_creditos.
    // Distribuir valores de Creditos rateados nas colunas correspondentes: Trib, NTrib e Exportação.

    for linha in linhas
        .iter()
        .filter(|&linha| linha.tipo_de_operacao == Some(TipoDeOperacao::Detalhamento))
    {
        let chaves = Chaves {
            //path: linha.arquivo_efd.to_compact_string(),
            cnpj_base: linha.get_cnpj_base(),
            ano: linha.ano,
            trimestre: linha.trimestre,
            mes: linha.mes,
            tipo_de_operacao: Some(TipoDeOperacao::Entrada), // atualizar valor: Entrada
            tipo_de_credito: linha.tipo_de_credito,
            cst: linha.cst,
            cfop: linha.cfop, // None
            aliq_pis: linha.aliq_pis,
            aliq_cofins: linha.aliq_cofins,
            natureza_bc: linha.natureza_bc,
        };

        // usar base_creditos.get_mut(&chaves) para obter uma referência mutável do valor associado à chave.

        //println!("chaves: {chaves:#?}");
        //println!("cod_credito: {:?} ({:?})", linha.cod_credito, linha.cod_credito.map(|c| c.to_u16()));
        if let Some(valores) = base_creditos.get_mut(&chaves) {
            //println!("valores: {valores:#?}\n");
            valores.distribuir_conforme_rateio(linha.cod_credito, linha.valor_bc);
        }
        //println!("### --- ###");
    }

    //std::process::exit(0);
}

/// Distribuir valores de Ajustes rateados nas colunas correspondentes: Trib, NTrib e Exportação.
fn distribuir_ajustes_rateados(linhas: &[DocsFiscais]) -> HashMap<Chaves, Valores> {
    consolidar_registros(
        linhas,
        |linha| linha.tipo_de_operacao.is_some_and(|t| t.is_ajuste()),
        |linha| {
            // Tipo de Operação, 3: "Ajuste de Acréscimo", 4: "Ajuste de Redução"

            // 1. Define o tributo
            let tributo = if linha.aliq_pis.is_some() {
                Tributo::Pis
            } else {
                Tributo::Cofins
            };

            // Obtém a NaturezaBaseCalculo
            let natureza_bc = linha
                .tipo_de_operacao
                .and_then(|tipo| NaturezaBaseCalculo::from_ajustes(tipo, tributo));

            let chaves = Chaves {
                //path: linha.arquivo_efd.to_compact_string(),
                cnpj_base: linha.get_cnpj_base(),
                ano: linha.ano,
                trimestre: linha.trimestre,
                mes: linha.mes,
                tipo_de_operacao: linha.tipo_de_operacao,
                tipo_de_credito: linha.tipo_de_credito,
                cst: linha.cst,
                cfop: linha.cfop,
                aliq_pis: linha.aliq_pis,
                aliq_cofins: linha.aliq_cofins,
                natureza_bc,
            };

            let mut valores = Valores::new(linha.valor_item, linha.valor_item);
            valores.distribuir_conforme_rateio(linha.cod_credito, linha.valor_item);

            (chaves, valores)
        },
    )
}

/// Distribuir valores de Descontos rateados nas colunas correspondentes: Trib, NTrib e Exportação.
/// Transmitir cod_credito (crédito com informação de rateio) para base_creditos.
///
/// 5: "Desconto da Contribuição Apurada no Próprio Período"
///
/// 6: "Desconto Efetuado em Período Posterior"
fn distribuir_descontos_rateados(linhas: &[DocsFiscais]) -> HashMap<Chaves, Valores> {
    consolidar_registros(
        linhas,
        |linha| linha.tipo_de_operacao.is_some_and(|t| t.is_desconto()),
        |linha| {
            // Tenta obter a natureza correta. Se o registro for inválido,
            // a linha será ignorada na consolidação (via filter_map implícito ou tratamento de Option).
            let natureza_bc = linha
                .tipo_de_operacao
                .and_then(|tipo| NaturezaBaseCalculo::from_tipo_de_operacao(tipo, &linha.registro));

            let chaves = Chaves {
                //path: linha.arquivo_efd.to_compact_string(),
                cnpj_base: linha.get_cnpj_base(),
                ano: linha.ano,
                trimestre: linha.trimestre,
                mes: linha.mes,
                tipo_de_operacao: linha.tipo_de_operacao,
                tipo_de_credito: linha.tipo_de_credito,
                cst: linha.cst,
                cfop: linha.cfop,
                aliq_pis: linha.aliq_pis,
                aliq_cofins: linha.aliq_cofins,
                natureza_bc,
            };

            let mut valores = Valores::new(linha.valor_item, linha.valor_item);
            valores.distribuir_conforme_rateio(linha.cod_credito, linha.valor_item);

            (chaves, valores)
        },
    )
}

fn somar_base_de_calculo_valor_parcial(
    base_creditos: &HashMap<Chaves, Valores>,
) -> HashMap<Chaves, Valores> {
    base_creditos
        .iter()
        .filter_map(|(chaves, &valores)| {
            // Mapeamento direto via método do Enum
            let natureza_soma = chaves.tipo_de_credito?.para_natureza_soma()?;

            let mut chaves_bc = chaves.clone();
            chaves_bc.cst = Some(SomaParcialDaBaseCalculo);
            chaves_bc.natureza_bc = Some(natureza_soma);

            let mut valores_soma = valores;
            valores_soma.valor_bc = valores.rec_bruta_nao_cumulativa();

            Some((chaves_bc, valores_soma))
        })
        .fold(HashMap::new(), |mut acc, (k, v)| {
            *acc.entry(k).or_default() += v;
            acc
        })
}

// Assinatura alterada para retornar novo HashMap (Functional Style)
fn apurar_credito_das_contribuicoes(
    base_creditos: &HashMap<Chaves, Valores>,
) -> HashMap<Chaves, Valores> {
    base_creditos
        .iter()
        .filter(|(chaves, _)| {
            // Filtrar 'Base de Cálculo dos Créditos', valores entre 101 e 199.
            chaves.natureza_bc.eh_soma_de_bc()
        })
        .flat_map(|(chaves, &valores)| {
            // Configuração das regras para cada tributo (Tributo, Alíquota, NatBC, CST)
            let regras = [
                (
                    Tributo::Pis,
                    chaves.aliq_pis,
                    NaturezaBaseCalculo::CreditoApuradoPis,
                    CodigoSituacaoTributaria::CSTApuradoPIS,
                ),
                (
                    Tributo::Cofins,
                    chaves.aliq_cofins,
                    NaturezaBaseCalculo::CreditoApuradoCofins,
                    CodigoSituacaoTributaria::CSTApuradoCofins,
                ),
            ];

            regras
                .into_iter()
                .filter_map(move |(tributo, aliq_opt, nova_nat, novo_cst)| {
                    // Se a alíquota for None, o filter_map descarta esta iteração automaticamente
                    let aliquota = aliq_opt?;

                    // Lógica de construção da nova chave
                    let mut nova_chave = chaves.clone();
                    nova_chave.natureza_bc = Some(nova_nat);
                    nova_chave.cst = Some(novo_cst);

                    match tributo {
                        Tributo::Pis => nova_chave.aliq_cofins = None,
                        Tributo::Cofins => nova_chave.aliq_pis = None,
                    }

                    // Crédito Apurado no Período (PIS/PASEP)
                    // Crédito Apurado no Período (COFINS)
                    // Foi implementada a função de multiplicação (mul) entre Valores e Decimal.
                    let valor_apurado = valores * (aliquota / dec!(100.0));
                    Some((nova_chave, valor_apurado))
                })
        })
        .fold(HashMap::new(), |mut acc, (k, v)| {
            *acc.entry(k).or_default() += v;
            acc
        })
}

fn calcular_credito_apos_ajustes(
    base_creditos: &HashMap<Chaves, Valores>,
) -> HashMap<Chaves, Valores> {
    base_creditos
        .iter()
        .filter_map(|(chaves, &valores)| {
            // Obtemos a natureza. Se for None, filtramos para fora.
            let natureza = chaves.natureza_bc?;

            // Usamos pattern matching para identificar se é PIS ou COFINS
            // e já definir os novos valores de CST e Natureza.
            let (novo_cst, nova_natureza) = if natureza.eh_ajuste_de_pis() {
                (CSTApuradoPIS, CreditoAposAjustesPis)
            } else if natureza.eh_ajuste_de_cofins() {
                (CSTApuradoCofins, CreditoAposAjustesCofins)
            } else {
                return None;
            };

            let chaves_bc = Chaves {
                aliq_pis: None,
                aliq_cofins: None,
                // Para fins de acumulação e ordenação foi escolhido TipoDeOperacao::AjusteReducao
                tipo_de_operacao: Some(TipoDeOperacao::AjusteReducao),
                cst: Some(novo_cst),
                natureza_bc: Some(nova_natureza),
                ..chaves.clone()
            };

            Some((chaves_bc, valores))
        })
        // Agrupamento (fold) permanece funcional e eficiente
        .fold(HashMap::new(), |mut acc, (k, v)| {
            *acc.entry(k).or_default() += v;
            acc
        })
}

fn calcular_credito_apos_descontos(
    base_creditos: &HashMap<Chaves, Valores>,
) -> HashMap<Chaves, Valores> {
    base_creditos
        .iter()
        .filter_map(|(chaves, &valores)| {
            // 1. Extrai a natureza ou interrompe o fluxo
            let natureza = chaves.natureza_bc?;

            // 2. Mapeia a Natureza de origem para o par (CST, Nova Natureza)
            // Agrupamos aqui as regras de PIS (211, 51, 61) e COFINS (215, 55, 65)
            let (novo_cst, nova_natureza) = if natureza.eh_desconto_de_pis() {
                (CSTApuradoPIS, CreditoAposDescontosPis)
            } else if natureza.eh_desconto_de_cofins() {
                (CSTApuradoCofins, CreditoAposDescontosCofins)
            } else {
                return None;
            };

            // 3. Criação da nova chave usando a sintaxe de atualização de struct
            let nova_chave = Chaves {
                aliq_pis: None,
                aliq_cofins: None,
                // Para fins de acumulação e ordenação foi escolhido TipoDeOperacao::DescontoPosterior
                tipo_de_operacao: Some(TipoDeOperacao::DescontoPosterior),
                cst: Some(novo_cst),
                natureza_bc: Some(nova_natureza),
                ..chaves.clone()
            };

            Some((nova_chave, valores))
        })
        // 4. Agrega os resultados somando os valores para chaves idênticas
        .fold(HashMap::new(), |mut acc, (k, v)| {
            *acc.entry(k).or_default() += v;
            acc
        })
}

fn somar_base_de_calculo_valor_total(
    base_creditos: &HashMap<Chaves, Valores>,
) -> HashMap<Chaves, Valores> {
    base_creditos
        .iter()
        .filter(|(chaves, _)| chaves.natureza_bc.eh_soma_de_bc())
        .map(|(chaves, &valores)| {
            let bc_soma = Chaves {
                tipo_de_operacao: None,
                tipo_de_credito: None,
                aliq_pis: None,
                aliq_cofins: None,
                natureza_bc: Some(NaturezaBaseCalculo::BaseSomaValorTotal),
                ..chaves.clone()
            };

            (bc_soma, valores)
        })
        .fold(HashMap::new(), |mut acc, (k, v)| {
            *acc.entry(k).or_default() += v;
            acc
        })
}

fn calcular_saldo_de_credito_passivel_de_ressarcimento(
    base_creditos: &HashMap<Chaves, Valores>,
) -> HashMap<Chaves, Valores> {
    base_creditos
        .iter()
        .filter_map(|(chaves, &valores)| match chaves.natureza_bc {
            Some(NaturezaBaseCalculo::CreditoAposDescontosPis) => {
                let k = Chaves {
                    tipo_de_operacao: None,
                    tipo_de_credito: None,
                    aliq_pis: None,
                    aliq_cofins: None,
                    natureza_bc: Some(NaturezaBaseCalculo::SaldoDisponivelPis),
                    ..chaves.clone()
                };
                Some((k, valores))
            }
            Some(NaturezaBaseCalculo::CreditoAposDescontosCofins) => {
                let k = Chaves {
                    tipo_de_operacao: None,
                    tipo_de_credito: None,
                    aliq_pis: None,
                    aliq_cofins: None,
                    natureza_bc: Some(NaturezaBaseCalculo::SaldoDisponivelCofins),
                    ..chaves.clone()
                };
                Some((k, valores))
            }
            _ => None,
        })
        .fold(HashMap::new(), |mut acc, (k, v)| {
            *acc.entry(k).or_default() += v;
            acc
        })
}

// Em TipoDeCredito desejo que None (representa soma de valores totais) fique por último na ordenacao.
// O tipo Option implementa Ord de forma que None é sempre o menor valor (None < Some).
// Na função ordenar, alteramos a chave de ordenação para que o None (totais) seja tratado
// como o maior valor possível, forçando-o a ir para o final.
// Assim, desejo "empurrar" None para o final da ordenação.
fn ordenar(hmap: HashMap<Chaves, Valores>) -> Vec<(Chaves, Valores)> {
    // transform hashmap to vec
    let mut vec_from_hash: Vec<(Chaves, Valores)> = hmap.into_iter().collect();

    vec_from_hash.sort_unstable_by_key(|(chaves, _valores)| {
        (
            //chaves.path.clone(),
            chaves.cnpj_base.clone(),
            chaves.ano,
            chaves.trimestre,
            // CRITÉRIO DE OURO:
            // 1. mes.is_none() -> false (0) para meses reais, true (1) para o total.
            // Isso garante que o None (total) fique no final do trimestre.
            chaves.mes.is_none(),
            chaves.mes, // Ordenação secundária entre os meses (Jan, Fev...)
            // LÓGICA DE ORDENAÇÃO CUSTOMIZADA:
            // 1. O booleano .is_none() retorna 'false' para Some e 'true' para None.
            // 2. No Sort, 'false' (0) vem antes de 'true' (1).
            // 3. Isso joga todos os None (totais) para baixo dos itens preenchidos.
            // Idiomático: Some primeiro, None por último
            chaves.tipo_de_credito.is_none(),
            chaves.tipo_de_credito, // Ordenação secundária pelo código (1, 2, 8...)
            // chaves.tipo_de_credito.map(|tipo| tipo.code()).unwrap_or(u16::MAX), // None vai para o final na ordenação
            chaves.tipo_de_operacao,
            // Prioriza a ordem lógica (Saídas -> Total Saídas -> Créditos -> Total Créditos)
            // chaves.cst.map(|c| c.get_ordem()),
            chaves.cst,
            //(chaves.cst.is_none(), 100),
            //(chaves.cst.is_some(), Reverse(chaves.cst)),
            //(Reverse(chaves.cst), chaves.natureza_bc <= Some(18)),
            chaves.natureza_bc,
            chaves.aliq_pis,
            chaves.aliq_cofins,
        )
    });

    // Remove CSTs fictícios (usando o Enum e Ord)
    vec_from_hash.par_iter_mut().for_each(|(chaves, _valores)| {
        // Estes CSTs fictícios foram adicionados com a finalidade de ordenação.
        // Ao final, remover valores temporários de CST.
        if let Some(cst) = chaves.cst
            && cst.deve_limpar_cst()
        {
            chaves.cst = None;
        }
    });

    vec_from_hash
}

fn get_analises(info_ordenada: &[(Chaves, Valores)]) -> Vec<AnaliseDosCreditos> {
    let mut lines: Vec<AnaliseDosCreditos> = Vec::new();

    for (chaves, valores) in info_ordenada {
        let receita_bruta_cumulativa = valores.calcular_rb_cumulativa();

        let mut line = AnaliseDosCreditos {
            //path: chaves.path.clone(),
            cnpj_base: chaves.cnpj_base.clone(),
            ano: chaves.ano,
            trimestre: chaves.trimestre,
            mes: chaves.mes,
            tipo_de_operacao: chaves.tipo_de_operacao,
            tipo_de_credito: chaves.tipo_de_credito,
            cst: chaves.cst,
            aliq_pis: chaves.aliq_pis,
            aliq_cofins: chaves.aliq_cofins,
            natureza_bc: chaves.natureza_bc,
            valor_bc: valores.valor_bc,
            valor_rbnc_trib: valores.valor_rbnc_trib,
            valor_rbnc_ntrib: valores.valor_rbnc_ntrib,
            valor_rbnc_exp: valores.valor_rbnc_exp,
            valor_rb_cum: receita_bruta_cumulativa,
        };

        line.despise_small_values();

        lines.push(line);
    }

    lines
}

//----------------------------------------------------------------------------//
//                                   Tests                                    //
//----------------------------------------------------------------------------//
//
// cargo test -- --help
// cargo test -- --nocapture
// cargo test -- --show-output

/// Run tests with:
/// cargo test -- --show-output analise_dos_creditos_tests
#[cfg(test)]
#[path = "../tests/analise_dos_creditos_tests.rs"]
mod analise_dos_creditos_tests;
