/// Representa as diferentes abas (worksheets) geradas no arquivo Excel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SheetType {
    /// Detalhamento dos itens constantes nos documentos fiscais.
    ItensDocsFiscais,
    /// Resumo consolidado por Código de Situação Tributária.
    ConsolidacaoCST,
    /// Detalhamento da análise de naturezas de crédito.
    AnaliseCreditos,
}

impl SheetType {
    /// Retorna o nome amigável da aba que aparecerá no Excel.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ItensDocsFiscais => "Itens de Docs Fiscais",
            Self::ConsolidacaoCST => "Consolidação CST",
            Self::AnaliseCreditos => "Análise dos Créditos",
        }
    }

    /// Verifica se o tipo atual é o de Itens, usado para lógica de formatação específica.
    pub fn is_itens(&self) -> bool {
        matches!(self, Self::ItensDocsFiscais)
    }
}

impl std::fmt::Display for SheetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
