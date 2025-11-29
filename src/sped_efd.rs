use std::{
    collections::{HashMap, HashSet},
    sync::LazyLock,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tipo {
    C,     // Campo Alfanumérico, exceto "|" (pipe ou barra vertical)
    N,     // Campo Numérico [0-9] + vírgula como separador decimal. Exemplo "12345,6789"
    Valor, // Float com duas casas decimais
    Aliquota, // Float com quatro casas decimais
           //DataMMYYYY,
           //DataDDMMYYYY,
}

/// (Registro, nº de campos)
pub static REGISTROS_ANTIGOS: LazyLock<HashSet<(&'static str, usize)>> =
    LazyLock::new(|| HashSet::from([("M210", 13), ("M610", 13)]));

pub fn registros_antigos(registro: &str, campos_len: usize) -> bool {
    REGISTROS_ANTIGOS.contains(&(registro, campos_len))
}

pub fn registros() -> HashMap<&'static str, HashMap<u16, (&'static str, Tipo)>> {
    // (N°, (Campo, Tipo: C, N, Valor, Aliquota))

    let registro_0000: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("0", Tipo::N)), // "0": nivel hierárquico
        (1, ("REG", Tipo::C)),
        (2, ("COD_VER", Tipo::C)),
        (3, ("TIPO_ESCRIT", Tipo::C)),
        (4, ("IND_SIT_ESP", Tipo::C)),
        (5, ("NUM_REC_ANTERIOR", Tipo::C)),
        (6, ("DT_INI", Tipo::C)),
        (7, ("DT_FIN", Tipo::C)),
        (8, ("NOME", Tipo::C)),
        (9, ("CNPJ", Tipo::C)),
        (10, ("UF", Tipo::C)),
        (11, ("COD_MUN", Tipo::C)),
        (12, ("SUFRAMA", Tipo::C)),
        (13, ("IND_NAT_PJ", Tipo::C)),
        (14, ("IND_ATIV", Tipo::C)),
    ]);

    let registro_0001: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("1", Tipo::N)), // "1": nivel hierárquico
        (1, ("REG", Tipo::C)),
        (2, ("IND_MOV", Tipo::C)),
    ]);

    let registro_0035: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("2", Tipo::N)), // "2": nivel hierárquico
        (1, ("REG", Tipo::C)),
        (2, ("COD_SCP", Tipo::C)),
        (3, ("DESC_SCP", Tipo::C)),
        (4, ("INF_COMP", Tipo::C)),
    ]);

    let registro_0100: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("2", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("NOME", Tipo::C)),
        (3, ("CPF", Tipo::C)),
        (4, ("CRC", Tipo::C)),
        (5, ("CNPJ", Tipo::C)),
        (6, ("CEP", Tipo::C)),
        (7, ("END", Tipo::C)),
        (8, ("NUM", Tipo::C)),
        (9, ("COMPL", Tipo::C)),
        (10, ("BAIRRO", Tipo::C)),
        (11, ("FONE", Tipo::C)),
        (12, ("FAX", Tipo::C)),
        (13, ("EMAIL", Tipo::C)),
        (14, ("COD_MUN", Tipo::C)),
    ]);

    let registro_0110: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("2", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("COD_INC_TRIB", Tipo::C)),
        (3, ("IND_APRO_CRED", Tipo::C)),
        (4, ("COD_TIPO_CONT", Tipo::C)),
        (5, ("IND_REG_CUM", Tipo::C)),
    ]);

    let registro_0111: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("REC_BRU_NCUM_TRIB_MI", Tipo::Valor)),
        (3, ("REC_BRU_NCUM_NT_MI", Tipo::Valor)),
        (4, ("REC_BRU_NCUM_EXP", Tipo::Valor)),
        (5, ("REC_BRU_CUM", Tipo::Valor)),
        (6, ("REC_BRU_TOTAL", Tipo::Valor)),
    ]);

    let registro_0120: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("2", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("MES_REFER", Tipo::C)),
        (3, ("INF_COMP", Tipo::C)),
    ]);

    let registro_0140: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("2", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("COD_EST", Tipo::C)),
        (3, ("NOME", Tipo::C)),
        (4, ("CNPJ", Tipo::C)),
        (5, ("UF", Tipo::C)),
        (6, ("IE", Tipo::C)),
        (7, ("COD_MUN", Tipo::C)),
        (8, ("IM", Tipo::C)),
        (9, ("SUFRAMA", Tipo::C)),
    ]);

    let registro_0145: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("COD_INC_TRIB", Tipo::C)),
        (3, ("VL_REC_TOT", Tipo::Valor)),
        (4, ("VL_REC_ATIV", Tipo::Valor)),
        (5, ("VL_REC_DEMAIS_ATIV", Tipo::Valor)),
        (6, ("INFO_COMPL", Tipo::C)),
    ]);

    let registro_0150: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("COD_PART", Tipo::C)),
        (3, ("NOME", Tipo::C)),
        (4, ("COD_PAIS", Tipo::C)),
        (5, ("CNPJ", Tipo::C)),
        (6, ("CPF", Tipo::C)),
        (7, ("IE", Tipo::C)),
        (8, ("COD_MUN", Tipo::C)),
        (9, ("SUFRAMA", Tipo::C)),
        (10, ("END", Tipo::C)),
        (11, ("NUM", Tipo::C)),
        (12, ("COMPL", Tipo::C)),
        (13, ("BAIRRO", Tipo::C)),
    ]);

    let registro_0190: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("UNID", Tipo::C)),
        (3, ("DESCR", Tipo::C)),
    ]);

    let registro_0200: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("COD_ITEM", Tipo::C)),
        (3, ("DESCR_ITEM", Tipo::C)),
        (4, ("COD_BARRA", Tipo::C)),
        (5, ("COD_ANT_ITEM", Tipo::C)),
        (6, ("UNID_INV", Tipo::C)),
        (7, ("TIPO_ITEM", Tipo::C)),
        (8, ("COD_NCM", Tipo::C)),
        (9, ("EX_IPI", Tipo::C)),
        (10, ("COD_GEN", Tipo::C)),
        (11, ("COD_LST", Tipo::C)),
        (12, ("ALIQ_ICMS", Tipo::Aliquota)),
    ]);

    let registro_0205: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("DESCR_ANT_ITEM", Tipo::C)),
        (3, ("DT_INI", Tipo::C)),
        (4, ("DT_FIM", Tipo::C)),
        (5, ("COD_ANT_ITEM", Tipo::C)),
    ]);

    let registro_0206: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("COD_COMB", Tipo::C)),
    ]);

    let registro_0208: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("COD_TAB", Tipo::C)),
        (2, ("COD_GRU", Tipo::C)),
        (2, ("MARCA_COM", Tipo::C)),
    ]);

    let registro_0400: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("COD_NAT", Tipo::C)),
        (3, ("DESCR_NAT", Tipo::C)),
    ]);

    let registro_0450: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("COD_INF", Tipo::C)),
        (3, ("TXT", Tipo::C)),
    ]);

    let registro_0500: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("2", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("DT_ALT", Tipo::C)),
        (3, ("COD_NAT_CC", Tipo::C)),
        (4, ("IND_CTA", Tipo::C)),
        (5, ("NIVEL", Tipo::C)),
        (6, ("COD_CTA", Tipo::C)),
        (7, ("NOME_CTA", Tipo::C)),
        (8, ("COD_CTA_REF", Tipo::C)),
        (9, ("CNPJ_EST", Tipo::C)),
    ]);

    let registro_0600: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("2", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("DT_ALT", Tipo::C)),
        (3, ("COD_CCUS", Tipo::C)),
        (4, ("CCUS", Tipo::C)),
    ]);

    let registro_0900: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("2", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("REC_TOTAL_BLOCO_A", Tipo::Valor)),
        (3, ("REC_NRB_BLOCO_A", Tipo::Valor)),
        (4, ("REC_TOTAL_BLOCO_C", Tipo::Valor)),
        (5, ("REC_NRB_BLOCO_C", Tipo::Valor)),
        (6, ("REC_TOTAL_BLOCO_D", Tipo::Valor)),
        (7, ("REC_NRB_BLOCO_D", Tipo::Valor)),
        (8, ("REC_TOTAL_BLOCO_F", Tipo::Valor)),
        (9, ("REC_NRB_BLOCO_F", Tipo::Valor)),
        (10, ("REC_TOTAL_BLOCO_I", Tipo::Valor)),
        (11, ("REC_NRB_BLOCO_I", Tipo::Valor)),
        (12, ("REC_TOTAL_BLOCO_1", Tipo::Valor)),
        (13, ("REC_NRB_BLOCO_1", Tipo::Valor)),
        (14, ("REC_TOTAL_PERIODO", Tipo::Valor)),
        (15, ("REC_TOTAL_NRB_PERIODO", Tipo::Valor)),
    ]);

    let registro_0990: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("1", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("QTD_LIN_0", Tipo::C)),
    ]);

    let registro_a001: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("1", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("IND_MOV", Tipo::C)),
    ]);

    let registro_a010: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("2", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("CNPJ", Tipo::C)),
    ]);

    let registro_a100: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("IND_OPER", Tipo::C)),
        (3, ("IND_EMIT", Tipo::C)),
        (4, ("COD_PART", Tipo::C)),
        (5, ("COD_SIT", Tipo::C)),
        (6, ("SER", Tipo::C)),
        (7, ("SUB", Tipo::C)),
        (8, ("NUM_DOC", Tipo::C)),
        (9, ("CHV_NFSE", Tipo::C)),
        (10, ("DT_DOC", Tipo::C)),
        (11, ("DT_EXE_SERV", Tipo::C)),
        (12, ("VL_DOC", Tipo::Valor)),
        (13, ("IND_PGTO", Tipo::C)),
        (14, ("VL_DESC", Tipo::Valor)),
        (15, ("VL_BC_PIS", Tipo::Valor)),
        (16, ("VL_PIS", Tipo::Valor)),
        (17, ("VL_BC_COFINS", Tipo::Valor)),
        (18, ("VL_COFINS", Tipo::Valor)),
        (19, ("VL_PIS_RET", Tipo::Valor)),
        (20, ("VL_COFINS_RET", Tipo::Valor)),
        (21, ("VL_ISS", Tipo::Valor)),
    ]);

    let registro_a110: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("COD_INF", Tipo::C)),
        (3, ("TXT_COMPL", Tipo::C)),
    ]);

    let registro_a111: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("NUM_PROC", Tipo::C)),
        (3, ("IND_PROC", Tipo::C)),
    ]);

    let registro_a120: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("VL_TOT_SERV", Tipo::Valor)),
        (3, ("VL_BC_PIS", Tipo::Valor)),
        (4, ("VL_PIS_IMP", Tipo::Valor)),
        (5, ("DT_PAG_PIS", Tipo::C)),
        (6, ("VL_BC_COFINS", Tipo::Valor)),
        (7, ("VL_COFINS_IMP", Tipo::Valor)),
        (8, ("DT_PAG_COFINS", Tipo::C)),
        (9, ("LOC_EXE_SERV", Tipo::C)),
    ]);

    let registro_a170: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("NUM_ITEM", Tipo::C)),
        (3, ("COD_ITEM", Tipo::C)),
        (4, ("DESCR_COMPL", Tipo::C)),
        (5, ("VL_ITEM", Tipo::Valor)),
        (6, ("VL_DESC", Tipo::Valor)),
        (7, ("NAT_BC_CRED", Tipo::C)),
        (8, ("IND_ORIG_CRED", Tipo::C)),
        (9, ("CST_PIS", Tipo::C)),
        (10, ("VL_BC_PIS", Tipo::Valor)),
        (11, ("ALIQ_PIS", Tipo::Aliquota)),
        (12, ("VL_PIS", Tipo::Valor)),
        (13, ("CST_COFINS", Tipo::C)),
        (14, ("VL_BC_COFINS", Tipo::Valor)),
        (15, ("ALIQ_COFINS", Tipo::Aliquota)),
        (16, ("VL_COFINS", Tipo::Valor)),
        (17, ("COD_CTA", Tipo::C)),
        (18, ("COD_CCUS", Tipo::C)),
    ]);

    let registro_a990: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("1", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("QTD_LIN_A", Tipo::C)),
    ]);

    let registro_c001: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("1", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("IND_MOV", Tipo::C)),
    ]);

    let registro_c010: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("2", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("CNPJ", Tipo::C)),
        (3, ("IND_ESCRI", Tipo::C)),
    ]);

    let registro_c100: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("IND_OPER", Tipo::C)),
        (3, ("IND_EMIT", Tipo::C)),
        (4, ("COD_PART", Tipo::C)),
        (5, ("COD_MOD", Tipo::C)),
        (6, ("COD_SIT", Tipo::C)),
        (7, ("SER", Tipo::C)),
        (8, ("NUM_DOC", Tipo::C)),
        (9, ("CHV_NFE", Tipo::C)),
        (10, ("DT_DOC", Tipo::C)),
        (11, ("DT_E_S", Tipo::C)),
        (12, ("VL_DOC", Tipo::Valor)),
        (13, ("IND_PGTO", Tipo::C)),
        (14, ("VL_DESC", Tipo::Valor)),
        (15, ("VL_ABAT_NT", Tipo::Valor)),
        (16, ("VL_MERC", Tipo::Valor)),
        (17, ("IND_FRT", Tipo::C)),
        (18, ("VL_FRT", Tipo::Valor)),
        (19, ("VL_SEG", Tipo::Valor)),
        (20, ("VL_OUT_DA", Tipo::Valor)),
        (21, ("VL_BC_ICMS", Tipo::Valor)),
        (22, ("VL_ICMS", Tipo::Valor)),
        (23, ("VL_BC_ICMS_ST", Tipo::Valor)),
        (24, ("VL_ICMS_ST", Tipo::Valor)),
        (25, ("VL_IPI", Tipo::Valor)),
        (26, ("VL_PIS", Tipo::Valor)),
        (27, ("VL_COFINS", Tipo::Valor)),
        (28, ("VL_PIS_ST", Tipo::Valor)),
        (29, ("VL_COFINS_ST", Tipo::Valor)),
    ]);

    let registro_c110: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("COD_INF", Tipo::C)),
        (3, ("TXT_COMPL", Tipo::C)),
    ]);

    let registro_c111: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("NUM_PROC", Tipo::C)),
        (3, ("IND_PROC", Tipo::C)),
    ]);

    let registro_c120: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("COD_DOC_IMP", Tipo::C)),
        (3, ("NUM_DOC_IMP", Tipo::C)),
        (4, ("VL_PIS_IMP", Tipo::Valor)),
        (5, ("VL_COFINS_IMP", Tipo::Valor)),
        (6, ("NUM_ACDRAW", Tipo::C)),
    ]);

    let registro_c170: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("NUM_ITEM", Tipo::C)),
        (3, ("COD_ITEM", Tipo::C)),
        (4, ("DESCR_COMPL", Tipo::C)),
        (5, ("QTD", Tipo::C)),
        (6, ("UNID", Tipo::C)),
        (7, ("VL_ITEM", Tipo::Valor)),
        (8, ("VL_DESC", Tipo::Valor)),
        (9, ("IND_MOV", Tipo::C)),
        (10, ("CST_ICMS", Tipo::C)),
        (11, ("CFOP", Tipo::C)),
        (12, ("COD_NAT", Tipo::C)),
        (13, ("VL_BC_ICMS", Tipo::Valor)),
        (14, ("ALIQ_ICMS", Tipo::Aliquota)),
        (15, ("VL_ICMS", Tipo::Valor)),
        (16, ("VL_BC_ICMS_ST", Tipo::Valor)),
        (17, ("ALIQ_ST", Tipo::Aliquota)),
        (18, ("VL_ICMS_ST", Tipo::Valor)),
        (19, ("IND_APUR", Tipo::C)),
        (20, ("CST_IPI", Tipo::C)),
        (21, ("COD_ENQ", Tipo::C)),
        (22, ("VL_BC_IPI", Tipo::Valor)),
        (23, ("ALIQ_IPI", Tipo::Aliquota)),
        (24, ("VL_IPI", Tipo::Valor)),
        (25, ("CST_PIS", Tipo::C)),
        (26, ("VL_BC_PIS", Tipo::Valor)),
        (27, ("ALIQ_PIS", Tipo::Aliquota)),
        (28, ("QUANT_BC_PIS", Tipo::C)),
        (29, ("ALIQ_PIS_QUANT", Tipo::Aliquota)),
        (30, ("VL_PIS", Tipo::Valor)),
        (31, ("CST_COFINS", Tipo::C)),
        (32, ("VL_BC_COFINS", Tipo::Valor)),
        (33, ("ALIQ_COFINS", Tipo::Aliquota)),
        (34, ("QUANT_BC_COFINS", Tipo::C)),
        (35, ("ALIQ_COFINS_QUANT", Tipo::Aliquota)),
        (36, ("VL_COFINS", Tipo::Valor)),
        (37, ("COD_CTA", Tipo::C)),
    ]);

    let registro_c175: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("CFOP", Tipo::C)),
        (3, ("VL_OPR", Tipo::Valor)),
        (4, ("VL_DESC", Tipo::Valor)),
        (5, ("CST_PIS", Tipo::C)),
        (6, ("VL_BC_PIS", Tipo::Valor)),
        (7, ("ALIQ_PIS", Tipo::Aliquota)),
        (8, ("QUANT_BC_PIS", Tipo::C)),
        (9, ("ALIQ_PIS_QUANT", Tipo::Aliquota)),
        (10, ("VL_PIS", Tipo::Valor)),
        (11, ("CST_COFINS", Tipo::C)),
        (12, ("VL_BC_COFINS", Tipo::Valor)),
        (13, ("ALIQ_COFINS", Tipo::Aliquota)),
        (14, ("QUANT_BC_COFINS", Tipo::C)),
        (15, ("ALIQ_COFINS_QUANT", Tipo::Aliquota)),
        (16, ("VL_COFINS", Tipo::Valor)),
        (17, ("COD_CTA", Tipo::C)),
        (18, ("INFO_COMPL", Tipo::C)),
    ]);

    let registro_c180: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("COD_MOD", Tipo::C)),
        (3, ("DT_DOC_INI", Tipo::C)),
        (4, ("DT_DOC_FIN", Tipo::C)),
        (5, ("COD_ITEM", Tipo::C)),
        (6, ("COD_NCM", Tipo::C)),
        (7, ("EX_IPI", Tipo::C)),
        (8, ("VL_TOT_ITEM", Tipo::Valor)),
    ]);

    let registro_c181: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("CST_PIS", Tipo::C)),
        (3, ("CFOP", Tipo::C)),
        (4, ("VL_ITEM", Tipo::Valor)),
        (5, ("VL_DESC", Tipo::Valor)),
        (6, ("VL_BC_PIS", Tipo::Valor)),
        (7, ("ALIQ_PIS", Tipo::Aliquota)),
        (8, ("QUANT_BC_PIS", Tipo::C)),
        (9, ("ALIQ_PIS_QUANT", Tipo::Aliquota)),
        (10, ("VL_PIS", Tipo::Valor)),
        (11, ("COD_CTA", Tipo::C)),
    ]);

    let registro_c185: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("CST_COFINS", Tipo::C)),
        (3, ("CFOP", Tipo::C)),
        (4, ("VL_ITEM", Tipo::Valor)),
        (5, ("VL_DESC", Tipo::Valor)),
        (6, ("VL_BC_COFINS", Tipo::Valor)),
        (7, ("ALIQ_COFINS", Tipo::Aliquota)),
        (8, ("QUANT_BC_COFINS", Tipo::C)),
        (9, ("ALIQ_COFINS_QUANT", Tipo::Aliquota)),
        (10, ("VL_COFINS", Tipo::Valor)),
        (11, ("COD_CTA", Tipo::C)),
    ]);

    let registro_c188: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("NUM_PROC", Tipo::C)),
        (3, ("IND_PROC", Tipo::C)),
    ]);

    let registro_c190: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("COD_MOD", Tipo::C)),
        (3, ("DT_REF_INI", Tipo::C)),
        (4, ("DT_REF_FIN", Tipo::C)),
        (5, ("COD_ITEM", Tipo::C)),
        (6, ("COD_NCM", Tipo::C)),
        (7, ("EX_IPI", Tipo::C)),
        (8, ("VL_TOT_ITEM", Tipo::Valor)),
    ]);

    let registro_c191: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("CNPJ_CPF_PART", Tipo::C)),
        (3, ("CST_PIS", Tipo::C)),
        (4, ("CFOP", Tipo::C)),
        (5, ("VL_ITEM", Tipo::Valor)),
        (6, ("VL_DESC", Tipo::Valor)),
        (7, ("VL_BC_PIS", Tipo::Valor)),
        (8, ("ALIQ_PIS", Tipo::Aliquota)),
        (9, ("QUANT_BC_PIS", Tipo::C)),
        (10, ("ALIQ_PIS_QUANT", Tipo::Aliquota)),
        (11, ("VL_PIS", Tipo::Valor)),
        (12, ("COD_CTA", Tipo::C)),
    ]);

    let registro_c195: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("CNPJ_CPF_PART", Tipo::C)),
        (3, ("CST_COFINS", Tipo::C)),
        (4, ("CFOP", Tipo::C)),
        (5, ("VL_ITEM", Tipo::Valor)),
        (6, ("VL_DESC", Tipo::Valor)),
        (7, ("VL_BC_COFINS", Tipo::Valor)),
        (8, ("ALIQ_COFINS", Tipo::Aliquota)),
        (9, ("QUANT_BC_COFINS", Tipo::C)),
        (10, ("ALIQ_COFINS_QUANT", Tipo::Aliquota)),
        (11, ("VL_COFINS", Tipo::Valor)),
        (12, ("COD_CTA", Tipo::C)),
    ]);

    let registro_c198: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("NUM_PROC", Tipo::C)),
        (3, ("IND_PROC", Tipo::C)),
    ]);

    let registro_c199: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("COD_DOC_IMP", Tipo::C)),
        (3, ("NUM_DOC_IMP", Tipo::C)),
        (4, ("VL_PIS_IMP", Tipo::Valor)),
        (5, ("VL_COFINS_IMP", Tipo::Valor)),
        (6, ("NUM_ACDRAW", Tipo::C)),
    ]);

    let registro_c380: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("COD_MOD", Tipo::C)),
        (3, ("DT_DOC_INI", Tipo::C)),
        (4, ("DT_DOC_FIN", Tipo::C)),
        (5, ("NUM_DOC_INI", Tipo::C)),
        (6, ("NUM_DOC_FIN", Tipo::C)),
        (7, ("VL_DOC", Tipo::Valor)),
        (8, ("VL_DOC_CANC", Tipo::Valor)),
    ]);

    let registro_c381: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("CST_PIS", Tipo::C)),
        (3, ("COD_ITEM", Tipo::C)),
        (4, ("VL_ITEM", Tipo::Valor)),
        (5, ("VL_BC_PIS", Tipo::Valor)),
        (6, ("ALIQ_PIS", Tipo::Aliquota)),
        (7, ("QUANT_BC_PIS", Tipo::C)),
        (8, ("ALIQ_PIS_QUANT", Tipo::Aliquota)),
        (9, ("VL_PIS", Tipo::Valor)),
        (10, ("COD_CTA", Tipo::C)),
    ]);

    let registro_c385: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("CST_COFINS", Tipo::C)),
        (3, ("COD_ITEM", Tipo::C)),
        (4, ("VL_ITEM", Tipo::Valor)),
        (5, ("VL_BC_COFINS", Tipo::Valor)),
        (6, ("ALIQ_COFINS", Tipo::Aliquota)),
        (7, ("QUANT_BC_COFINS", Tipo::C)),
        (8, ("ALIQ_COFINS_QUANT", Tipo::Aliquota)),
        (9, ("VL_COFINS", Tipo::Valor)),
        (10, ("COD_CTA", Tipo::C)),
    ]);

    let registro_c395: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("COD_MOD", Tipo::C)),
        (3, ("COD_PART", Tipo::C)),
        (4, ("SER", Tipo::C)),
        (5, ("SUB_SER", Tipo::C)),
        (6, ("NUM_DOC", Tipo::C)),
        (7, ("DT_DOC", Tipo::C)),
        (8, ("VL_DOC", Tipo::Valor)),
    ]);

    let registro_c396: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("COD_ITEM", Tipo::C)),
        (3, ("VL_ITEM", Tipo::Valor)),
        (4, ("VL_DESC", Tipo::Valor)),
        (5, ("NAT_BC_CRED", Tipo::C)),
        (6, ("CST_PIS", Tipo::C)),
        (7, ("VL_BC_PIS", Tipo::Valor)),
        (8, ("ALIQ_PIS", Tipo::Aliquota)),
        (9, ("VL_PIS", Tipo::Valor)),
        (10, ("CST_COFINS", Tipo::C)),
        (11, ("VL_BC_COFINS", Tipo::Valor)),
        (12, ("ALIQ_COFINS", Tipo::Aliquota)),
        (13, ("VL_COFINS", Tipo::Valor)),
        (14, ("COD_CTA", Tipo::C)),
    ]);

    let registro_c400: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("COD_MOD", Tipo::C)),
        (3, ("ECF_MOD", Tipo::C)),
        (4, ("ECF_FAB", Tipo::C)),
        (5, ("ECF_CX", Tipo::C)),
    ]);

    let registro_c405: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("DT_DOC", Tipo::C)),
        (3, ("CRO", Tipo::C)),
        (4, ("CRZ", Tipo::C)),
        (5, ("NUM_COO_FIN", Tipo::C)),
        (6, ("GT_FIN", Tipo::C)),
        (7, ("VL_BRT", Tipo::Valor)),
    ]);

    let registro_c481: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("5", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("CST_PIS", Tipo::C)),
        (3, ("VL_ITEM", Tipo::Valor)),
        (4, ("VL_BC_PIS", Tipo::Valor)),
        (5, ("ALIQ_PIS", Tipo::Aliquota)),
        (6, ("QUANT_BC_PIS", Tipo::C)),
        (7, ("ALIQ_PIS_QUANT", Tipo::Aliquota)),
        (8, ("VL_PIS", Tipo::Valor)),
        (9, ("COD_ITEM", Tipo::C)),
        (10, ("COD_CTA", Tipo::C)),
    ]);

    let registro_c485: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("5", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("CST_COFINS", Tipo::C)),
        (3, ("VL_ITEM", Tipo::Valor)),
        (4, ("VL_BC_COFINS", Tipo::Valor)),
        (5, ("ALIQ_COFINS", Tipo::Aliquota)),
        (6, ("QUANT_BC_COFINS", Tipo::C)),
        (7, ("ALIQ_COFINS_QUANT", Tipo::Aliquota)),
        (8, ("VL_COFINS", Tipo::Valor)),
        (9, ("COD_ITEM", Tipo::C)),
        (10, ("COD_CTA", Tipo::C)),
    ]);

    let registro_c489: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("NUM_PROC", Tipo::C)),
        (3, ("IND_PROC", Tipo::C)),
    ]);

    let registro_c490: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("DT_DOC_INI", Tipo::C)),
        (3, ("DT_DOC_FIN", Tipo::C)),
        (4, ("COD_MOD", Tipo::C)),
    ]);

    let registro_c491: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("COD_ITEM", Tipo::C)),
        (3, ("CST_PIS", Tipo::C)),
        (4, ("CFOP", Tipo::C)),
        (5, ("VL_ITEM", Tipo::Valor)),
        (6, ("VL_BC_PIS", Tipo::Valor)),
        (7, ("ALIQ_PIS", Tipo::Aliquota)),
        (8, ("QUANT_BC_PIS", Tipo::C)),
        (9, ("ALIQ_PIS_QUANT", Tipo::Aliquota)),
        (10, ("VL_PIS", Tipo::Valor)),
        (11, ("COD_CTA", Tipo::C)),
    ]);

    let registro_c495: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("COD_ITEM", Tipo::C)),
        (3, ("CST_COFINS", Tipo::C)),
        (4, ("CFOP", Tipo::C)),
        (5, ("VL_ITEM", Tipo::Valor)),
        (6, ("VL_BC_COFINS", Tipo::Valor)),
        (7, ("ALIQ_COFINS", Tipo::Aliquota)),
        (8, ("QUANT_BC_COFINS", Tipo::C)),
        (9, ("ALIQ_COFINS_QUANT", Tipo::Aliquota)),
        (10, ("VL_COFINS", Tipo::Valor)),
        (11, ("COD_CTA", Tipo::C)),
    ]);

    let registro_c499: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("NUM_PROC", Tipo::C)),
        (3, ("IND_PROC", Tipo::C)),
    ]);

    let registro_c500: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("COD_PART", Tipo::C)),
        (3, ("COD_MOD", Tipo::C)),
        (4, ("COD_SIT", Tipo::C)),
        (5, ("SER", Tipo::C)),
        (6, ("SUB", Tipo::C)),
        (7, ("NUM_DOC", Tipo::C)),
        (8, ("DT_DOC", Tipo::C)),
        (9, ("DT_ENT", Tipo::C)),
        (10, ("VL_DOC", Tipo::Valor)),
        (11, ("VL_ICMS", Tipo::Valor)),
        (12, ("COD_INF", Tipo::C)),
        (13, ("VL_PIS", Tipo::Valor)),
        (14, ("VL_COFINS", Tipo::Valor)),
        (15, ("CHV_DOCe", Tipo::C)),
    ]);

    let registro_c501: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("CST_PIS", Tipo::C)),
        (3, ("VL_ITEM", Tipo::Valor)),
        (4, ("NAT_BC_CRED", Tipo::C)),
        (5, ("VL_BC_PIS", Tipo::Valor)),
        (6, ("ALIQ_PIS", Tipo::Aliquota)),
        (7, ("VL_PIS", Tipo::Valor)),
        (8, ("COD_CTA", Tipo::C)),
    ]);

    let registro_c505: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("CST_COFINS", Tipo::C)),
        (3, ("VL_ITEM", Tipo::Valor)),
        (4, ("NAT_BC_CRED", Tipo::C)),
        (5, ("VL_BC_COFINS", Tipo::Valor)),
        (6, ("ALIQ_COFINS", Tipo::Aliquota)),
        (7, ("VL_COFINS", Tipo::Valor)),
        (8, ("COD_CTA", Tipo::C)),
    ]);

    let registro_c509: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("NUM_PROC", Tipo::C)),
        (3, ("IND_PROC", Tipo::C)),
    ]);

    let registro_c600: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("COD_MOD", Tipo::C)),
        (3, ("COD_MUN", Tipo::C)),
        (4, ("SER", Tipo::C)),
        (5, ("SUB", Tipo::C)),
        (6, ("COD_CONS", Tipo::C)),
        (7, ("QTD_CONS", Tipo::C)),
        (8, ("QTD_CANC", Tipo::C)),
        (9, ("DT_DOC", Tipo::C)),
        (10, ("VL_DOC", Tipo::Valor)),
        (11, ("VL_DESC", Tipo::Valor)),
        (12, ("CONS", Tipo::C)),
        (13, ("VL_FORN", Tipo::Valor)),
        (14, ("VL_SERV_NT", Tipo::Valor)),
        (15, ("VL_TERC", Tipo::Valor)),
        (16, ("VL_DA", Tipo::Valor)),
        (17, ("VL_BC_ICMS", Tipo::Valor)),
        (18, ("VL_ICMS", Tipo::Valor)),
        (19, ("VL_BC_ICMS_ST", Tipo::Valor)),
        (20, ("VL_ICMS_ST", Tipo::Valor)),
        (21, ("VL_PIS", Tipo::Valor)),
        (22, ("VL_COFINS", Tipo::Valor)),
    ]);

    let registro_c601: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("CST_PIS", Tipo::C)),
        (3, ("VL_ITEM", Tipo::Valor)),
        (4, ("VL_BC_PIS", Tipo::Valor)),
        (5, ("ALIQ_PIS", Tipo::Aliquota)),
        (6, ("VL_PIS", Tipo::Valor)),
        (7, ("COD_CTA", Tipo::C)),
    ]);

    let registro_c605: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("CST_COFINS", Tipo::C)),
        (3, ("VL_ITEM", Tipo::Valor)),
        (4, ("VL_BC_COFINS", Tipo::Valor)),
        (5, ("ALIQ_COFINS", Tipo::Aliquota)),
        (6, ("VL_COFINS", Tipo::Valor)),
        (7, ("COD_CTA", Tipo::C)),
    ]);

    let registro_c609: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("NUM_PROC", Tipo::C)),
        (3, ("IND_PROC", Tipo::C)),
    ]);

    let registro_c800: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("COD_MOD", Tipo::C)),
        (3, ("COD_SIT", Tipo::C)),
        (4, ("NUM_CFE", Tipo::C)),
        (5, ("DT_DOC", Tipo::C)),
        (6, ("VL_CFE", Tipo::Valor)),
        (7, ("VL_PIS", Tipo::Valor)),
        (8, ("VL_COFINS", Tipo::Valor)),
        (9, ("CNPJ_CPF", Tipo::C)),
        (10, ("NR_SAT", Tipo::C)),
        (11, ("CHV_CFE", Tipo::C)),
        (12, ("VL_DESC", Tipo::Valor)),
        (13, ("VL_MERC", Tipo::Valor)),
        (14, ("VL_OUT_DA", Tipo::Valor)),
        (15, ("VL_ICMS", Tipo::Valor)),
        (16, ("VL_PIS_ST", Tipo::Valor)),
        (17, ("VL_COFINS_ST", Tipo::Valor)),
    ]);

    let registro_c810: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("CFOP", Tipo::C)),
        (3, ("VL_ITEM", Tipo::Valor)),
        (4, ("COD_ITEM", Tipo::C)),
        (5, ("CST_PIS", Tipo::C)),
        (6, ("VL_BC_PIS", Tipo::Valor)),
        (7, ("ALIQ_PIS", Tipo::Aliquota)),
        (8, ("VL_PIS", Tipo::Valor)),
        (9, ("CST_COFINS", Tipo::C)),
        (10, ("VL_BC_COFINS", Tipo::Valor)),
        (11, ("ALIQ_COFINS", Tipo::Aliquota)),
        (12, ("VL_COFINS", Tipo::Valor)),
        (13, ("COD_CTA", Tipo::C)),
    ]);

    let registro_c820: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("CFOP", Tipo::C)),
        (3, ("VL_ITEM", Tipo::Valor)),
        (4, ("COD_ITEM", Tipo::C)),
        (5, ("CST_PIS", Tipo::C)),
        (6, ("QUANT_BC_PIS", Tipo::C)),
        (7, ("ALIQ_PIS_QUANT", Tipo::Aliquota)),
        (8, ("VL_PIS", Tipo::Valor)),
        (9, ("CST_COFINS", Tipo::C)),
        (10, ("QUANT_BC_COFINS", Tipo::C)),
        (11, ("ALIQ_COFINS_QUANT", Tipo::Aliquota)),
        (12, ("VL_COFINS", Tipo::Valor)),
        (13, ("COD_CTA", Tipo::C)),
    ]);

    let registro_c830: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("NUM_PROC", Tipo::C)),
        (3, ("IND_PROC", Tipo::C)),
    ]);

    let registro_c860: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("COD_MOD", Tipo::C)),
        (3, ("NR_SAT", Tipo::C)),
        (4, ("DT_DOC", Tipo::C)),
        (5, ("DOC_INI", Tipo::C)),
        (6, ("DOC_FIM", Tipo::C)),
    ]);

    let registro_c870: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("COD_ITEM", Tipo::C)),
        (3, ("CFOP", Tipo::C)),
        (4, ("VL_ITEM", Tipo::Valor)),
        (5, ("VL_DESC", Tipo::Valor)),
        (6, ("CST_PIS", Tipo::C)),
        (7, ("VL_BC_PIS", Tipo::Valor)),
        (8, ("ALIQ_PIS", Tipo::Aliquota)),
        (9, ("VL_PIS", Tipo::Valor)),
        (10, ("CST_COFINS", Tipo::C)),
        (11, ("VL_BC_COFINS", Tipo::Valor)),
        (12, ("ALIQ_COFINS", Tipo::Aliquota)),
        (13, ("VL_COFINS", Tipo::Valor)),
        (14, ("COD_CTA", Tipo::C)),
    ]);

    let registro_c880: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("COD_ITEM", Tipo::C)),
        (3, ("CFOP", Tipo::C)),
        (4, ("VL_ITEM", Tipo::Valor)),
        (5, ("VL_DESC", Tipo::Valor)),
        (6, ("CST_PIS", Tipo::C)),
        (7, ("QUANT_BC_PIS", Tipo::C)),
        (8, ("ALIQ_PIS_QUANT", Tipo::Aliquota)),
        (9, ("VL_PIS", Tipo::Valor)),
        (10, ("CST_COFINS", Tipo::C)),
        (11, ("QUANT_BC_COFINS", Tipo::C)),
        (12, ("ALIQ_COFINS_QUANT", Tipo::Aliquota)),
        (13, ("VL_COFINS", Tipo::Valor)),
        (14, ("COD_CTA", Tipo::C)),
    ]);

    let registro_c890: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("NUM_PROC", Tipo::C)),
        (3, ("IND_PROC", Tipo::C)),
    ]);

    let registro_c990: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("1", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("QTD_LIN_C", Tipo::C)),
    ]);

    let registro_d001: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("1", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("IND_MOV", Tipo::C)),
    ]);

    let registro_d010: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("2", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("CNPJ", Tipo::C)),
    ]);

    let registro_d100: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("IND_OPER", Tipo::C)),
        (3, ("IND_EMIT", Tipo::C)),
        (4, ("COD_PART", Tipo::C)),
        (5, ("COD_MOD", Tipo::C)),
        (6, ("COD_SIT", Tipo::C)),
        (7, ("SER", Tipo::C)),
        (8, ("SUB", Tipo::C)),
        (9, ("NUM_DOC", Tipo::C)),
        (10, ("CHV_CTE", Tipo::C)),
        (11, ("DT_DOC", Tipo::C)),
        (12, ("DT_A_P", Tipo::C)),
        (13, ("TP_CT-e", Tipo::C)),
        (14, ("CHV_CTE_REF", Tipo::C)),
        (15, ("VL_DOC", Tipo::Valor)),
        (16, ("VL_DESC", Tipo::Valor)),
        (17, ("IND_FRT", Tipo::C)),
        (18, ("VL_SERV", Tipo::Valor)),
        (19, ("VL_BC_ICMS", Tipo::Valor)),
        (20, ("VL_ICMS", Tipo::Valor)),
        (21, ("VL_NT", Tipo::Valor)),
        (22, ("COD_INF", Tipo::C)),
        (23, ("COD_CTA", Tipo::C)),
    ]);

    let registro_d101: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("IND_NAT_FRT", Tipo::C)),
        (3, ("VL_ITEM", Tipo::Valor)),
        (4, ("CST_PIS", Tipo::C)),
        (5, ("NAT_BC_CRED", Tipo::C)),
        (6, ("VL_BC_PIS", Tipo::Valor)),
        (7, ("ALIQ_PIS", Tipo::Aliquota)),
        (8, ("VL_PIS", Tipo::Valor)),
        (9, ("COD_CTA", Tipo::C)),
    ]);

    let registro_d105: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("IND_NAT_FRT", Tipo::C)),
        (3, ("VL_ITEM", Tipo::Valor)),
        (4, ("CST_COFINS", Tipo::C)),
        (5, ("NAT_BC_CRED", Tipo::C)),
        (6, ("VL_BC_COFINS", Tipo::Valor)),
        (7, ("ALIQ_COFINS", Tipo::Aliquota)),
        (8, ("VL_COFINS", Tipo::Valor)),
        (9, ("COD_CTA", Tipo::C)),
    ]);

    let registro_d111: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("NUM_PROC", Tipo::C)),
        (3, ("IND_PROC", Tipo::C)),
    ]);

    let registro_d200: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("COD_MOD", Tipo::C)),
        (3, ("COD_SIT", Tipo::C)),
        (4, ("SER", Tipo::C)),
        (5, ("SUB", Tipo::C)),
        (6, ("NUM_DOC_INI", Tipo::C)),
        (7, ("NUM_DOC_FIN", Tipo::C)),
        (8, ("CFOP", Tipo::C)),
        (9, ("DT_REF", Tipo::C)),
        (10, ("VL_DOC", Tipo::Valor)),
        (11, ("VL_DESC", Tipo::Valor)),
    ]);

    let registro_d201: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("CST_PIS", Tipo::C)),
        (3, ("VL_ITEM", Tipo::Valor)),
        (4, ("VL_BC_PIS", Tipo::Valor)),
        (5, ("ALIQ_PIS", Tipo::Aliquota)),
        (6, ("VL_PIS", Tipo::Valor)),
        (7, ("COD_CTA", Tipo::C)),
    ]);

    let registro_d205: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("CST_COFINS", Tipo::C)),
        (3, ("VL_ITEM", Tipo::Valor)),
        (4, ("VL_BC_COFINS", Tipo::Valor)),
        (5, ("ALIQ_COFINS", Tipo::Aliquota)),
        (6, ("VL_COFINS", Tipo::Valor)),
        (7, ("COD_CTA", Tipo::C)),
    ]);

    let registro_d209: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("NUM_PROC", Tipo::C)),
        (3, ("IND_PROC", Tipo::C)),
    ]);

    let registro_d300: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("COD_MOD", Tipo::C)),
        (3, ("SER", Tipo::C)),
        (4, ("SUB", Tipo::C)),
        (5, ("NUM_DOC_INI", Tipo::C)),
        (6, ("NUM_DOC_FIN", Tipo::C)),
        (7, ("CFOP", Tipo::C)),
        (8, ("DT_REF", Tipo::C)),
        (9, ("VL_DOC", Tipo::Valor)),
        (10, ("VL_DESC", Tipo::Valor)),
        (11, ("CST_PIS", Tipo::C)),
        (12, ("VL_BC_PIS", Tipo::Valor)),
        (13, ("ALIQ_PIS", Tipo::Aliquota)),
        (14, ("VL_PIS", Tipo::Valor)),
        (15, ("CST_COFINS", Tipo::C)),
        (16, ("VL_BC_COFINS", Tipo::Valor)),
        (17, ("ALIQ_COFINS", Tipo::Aliquota)),
        (18, ("VL_COFINS", Tipo::Valor)),
        (19, ("COD_CTA", Tipo::C)),
    ]);

    let registro_d309: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("NUM_PROC", Tipo::C)),
        (3, ("IND_PROC", Tipo::C)),
    ]);

    let registro_d350: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("COD_MOD", Tipo::C)),
        (3, ("ECF_MOD", Tipo::C)),
        (4, ("ECF_FAB", Tipo::C)),
        (5, ("DT_DOC", Tipo::C)),
        (6, ("CRO", Tipo::C)),
        (7, ("CRZ", Tipo::C)),
        (8, ("NUM_COO_FIN", Tipo::C)),
        (9, ("GT_FIN", Tipo::C)),
        (10, ("VL_BRT", Tipo::Valor)),
        (11, ("CST_PIS", Tipo::C)),
        (12, ("VL_BC_PIS", Tipo::Valor)),
        (13, ("ALIQ_PIS", Tipo::Aliquota)),
        (14, ("QUANT_BC_PIS", Tipo::C)),
        (15, ("ALIQ_PIS_QUANT", Tipo::Aliquota)),
        (16, ("VL_PIS", Tipo::Valor)),
        (17, ("CST_COFINS", Tipo::C)),
        (18, ("VL_BC_COFINS", Tipo::Valor)),
        (19, ("ALIQ_COFINS", Tipo::Aliquota)),
        (20, ("QUANT_BC_COFINS", Tipo::C)),
        (21, ("ALIQ_COFINS_QUANT", Tipo::Aliquota)),
        (22, ("VL_COFINS", Tipo::Valor)),
        (23, ("COD_CTA", Tipo::C)),
    ]);

    let registro_d359: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("NUM_PROC", Tipo::C)),
        (3, ("IND_PROC", Tipo::C)),
    ]);

    let registro_d500: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("IND_OPER", Tipo::C)),
        (3, ("IND_EMIT", Tipo::C)),
        (4, ("COD_PART", Tipo::C)),
        (5, ("COD_MOD", Tipo::C)),
        (6, ("COD_SIT", Tipo::C)),
        (7, ("SER", Tipo::C)),
        (8, ("SUB", Tipo::C)),
        (9, ("NUM_DOC", Tipo::C)),
        (10, ("DT_DOC", Tipo::C)),
        (11, ("DT_A_P", Tipo::C)),
        (12, ("VL_DOC", Tipo::Valor)),
        (13, ("VL_DESC", Tipo::Valor)),
        (14, ("VL_SERV", Tipo::Valor)),
        (15, ("VL_SERV_NT", Tipo::Valor)),
        (16, ("VL_TERC", Tipo::Valor)),
        (17, ("VL_DA", Tipo::Valor)),
        (18, ("VL_BC_ICMS", Tipo::Valor)),
        (19, ("VL_ICMS", Tipo::Valor)),
        (20, ("COD_INF", Tipo::C)),
        (21, ("VL_PIS", Tipo::Valor)),
        (22, ("VL_COFINS", Tipo::Valor)),
    ]);

    let registro_d501: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("CST_PIS", Tipo::C)),
        (3, ("VL_ITEM", Tipo::Valor)),
        (4, ("NAT_BC_CRED", Tipo::C)),
        (5, ("VL_BC_PIS", Tipo::Valor)),
        (6, ("ALIQ_PIS", Tipo::Aliquota)),
        (7, ("VL_PIS", Tipo::Valor)),
        (8, ("COD_CTA", Tipo::C)),
    ]);

    let registro_d505: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("CST_COFINS", Tipo::C)),
        (3, ("VL_ITEM", Tipo::Valor)),
        (4, ("NAT_BC_CRED", Tipo::C)),
        (5, ("VL_BC_COFINS", Tipo::Valor)),
        (6, ("ALIQ_COFINS", Tipo::Aliquota)),
        (7, ("VL_COFINS", Tipo::Valor)),
        (8, ("COD_CTA", Tipo::C)),
    ]);

    let registro_d509: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("NUM_PROC", Tipo::C)),
        (3, ("IND_PROC", Tipo::C)),
    ]);

    let registro_d600: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("COD_MOD", Tipo::C)),
        (3, ("COD_MUN", Tipo::C)),
        (4, ("SER", Tipo::C)),
        (5, ("SUB", Tipo::C)),
        (6, ("IND_REC", Tipo::C)),
        (7, ("QTD_CONS", Tipo::C)),
        (8, ("DT_DOC_INI", Tipo::C)),
        (9, ("DT_DOC_FIN", Tipo::C)),
        (10, ("VL_DOC", Tipo::Valor)),
        (11, ("VL_DESC", Tipo::Valor)),
        (12, ("VL_SERV", Tipo::Valor)),
        (13, ("VL_SERV_NT", Tipo::Valor)),
        (14, ("VL_TERC", Tipo::Valor)),
        (15, ("VL_DA", Tipo::Valor)),
        (16, ("VL_BC_ICMS", Tipo::Valor)),
        (17, ("VL_ICMS", Tipo::Valor)),
        (18, ("VL_PIS", Tipo::Valor)),
        (19, ("VL_COFINS", Tipo::Valor)),
    ]);

    let registro_d601: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("COD_CLASS", Tipo::C)),
        (3, ("VL_ITEM", Tipo::Valor)),
        (4, ("VL_DESC", Tipo::Valor)),
        (5, ("CST_PIS", Tipo::C)),
        (6, ("VL_BC_PIS", Tipo::Valor)),
        (7, ("ALIQ_PIS", Tipo::Aliquota)),
        (8, ("VL_PIS", Tipo::Valor)),
        (9, ("COD_CTA", Tipo::C)),
    ]);

    let registro_d605: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("COD_CLASS", Tipo::C)),
        (3, ("VL_ITEM", Tipo::Valor)),
        (4, ("VL_DESC", Tipo::Valor)),
        (5, ("CST_COFINS", Tipo::C)),
        (6, ("VL_BC_COFINS", Tipo::Valor)),
        (7, ("ALIQ_COFINS", Tipo::Aliquota)),
        (8, ("VL_COFINS", Tipo::Valor)),
        (9, ("COD_CTA", Tipo::C)),
    ]);

    let registro_d609: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("NUM_PROC", Tipo::C)),
        (3, ("IND_PROC", Tipo::C)),
    ]);

    let registro_d990: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("1", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("QTD_LIN_D", Tipo::C)),
    ]);

    let registro_f001: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("1", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("IND_MOV", Tipo::C)),
    ]);

    let registro_f010: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("2", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("CNPJ", Tipo::C)),
    ]);

    let registro_f100: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("IND_OPER", Tipo::C)),
        (3, ("COD_PART", Tipo::C)),
        (4, ("COD_ITEM", Tipo::C)),
        (5, ("DT_OPER", Tipo::C)),
        (6, ("VL_OPER", Tipo::Valor)),
        (7, ("CST_PIS", Tipo::C)),
        (8, ("VL_BC_PIS", Tipo::Valor)),
        (9, ("ALIQ_PIS", Tipo::Aliquota)),
        (10, ("VL_PIS", Tipo::Valor)),
        (11, ("CST_COFINS", Tipo::C)),
        (12, ("VL_BC_COFINS", Tipo::Valor)),
        (13, ("ALIQ_COFINS", Tipo::Aliquota)),
        (14, ("VL_COFINS", Tipo::Valor)),
        (15, ("NAT_BC_CRED", Tipo::C)),
        (16, ("IND_ORIG_CRED", Tipo::C)),
        (17, ("COD_CTA", Tipo::C)),
        (18, ("COD_CCUS", Tipo::C)),
        (19, ("DESC_DOC_OPER", Tipo::C)),
    ]);

    let registro_f111: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("NUM_PROC", Tipo::C)),
        (3, ("IND_PROC", Tipo::C)),
    ]);

    let registro_f120: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("NAT_BC_CRED", Tipo::C)),
        (3, ("IDENT_BEM_IMOB", Tipo::C)),
        (4, ("IND_ORIG_CRED", Tipo::C)),
        (5, ("IND_UTIL_BEM_IMOB", Tipo::C)),
        (6, ("VL_OPER_DEP", Tipo::Valor)),
        (7, ("PARC_OPER_NAO_BC_CRED", Tipo::C)),
        (8, ("CST_PIS", Tipo::C)),
        (9, ("VL_BC_PIS", Tipo::Valor)),
        (10, ("ALIQ_PIS", Tipo::Aliquota)),
        (11, ("VL_PIS", Tipo::Valor)),
        (12, ("CST_COFINS", Tipo::C)),
        (13, ("VL_BC_COFINS", Tipo::Valor)),
        (14, ("ALIQ_COFINS", Tipo::Aliquota)),
        (15, ("VL_COFINS", Tipo::Valor)),
        (16, ("COD_CTA", Tipo::C)),
        (17, ("COD_CCUS", Tipo::C)),
        (18, ("DESC_BEM_IMOB", Tipo::C)),
    ]);

    let registro_f129: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("NUM_PROC", Tipo::C)),
        (3, ("IND_PROC", Tipo::C)),
    ]);

    let registro_f130: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("NAT_BC_CRED", Tipo::C)),
        (3, ("IDENT_BEM_IMOB", Tipo::C)),
        (4, ("IND_ORIG_CRED", Tipo::C)),
        (5, ("IND_UTIL_BEM_IMOB", Tipo::C)),
        (6, ("MES_OPER_AQUIS", Tipo::C)),
        (7, ("VL_OPER_AQUIS", Tipo::Valor)),
        (8, ("PARC_OPER_NAO_BC_CRED", Tipo::C)),
        (9, ("VL_BC_CRED", Tipo::Valor)),
        (10, ("IND_NR_PARC", Tipo::C)),
        (11, ("CST_PIS", Tipo::C)),
        (12, ("VL_BC_PIS", Tipo::Valor)),
        (13, ("ALIQ_PIS", Tipo::Aliquota)),
        (14, ("VL_PIS", Tipo::Valor)),
        (15, ("CST_COFINS", Tipo::C)),
        (16, ("VL_BC_COFINS", Tipo::Valor)),
        (17, ("ALIQ_COFINS", Tipo::Aliquota)),
        (18, ("VL_COFINS", Tipo::Valor)),
        (19, ("COD_CTA", Tipo::C)),
        (20, ("COD_CCUS", Tipo::C)),
        (21, ("DESC_BEM_IMOB", Tipo::C)),
    ]);

    let registro_f139: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("NUM_PROC", Tipo::C)),
        (3, ("IND_PROC", Tipo::C)),
    ]);

    let registro_f150: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("NAT_BC_CRED", Tipo::C)),
        (3, ("VL_TOT_EST", Tipo::Valor)),
        (4, ("EST_IMP", Tipo::C)),
        (5, ("VL_BC_EST", Tipo::Valor)),
        (6, ("VL_BC_MEN_EST", Tipo::Valor)),
        (7, ("CST_PIS", Tipo::C)),
        (8, ("ALIQ_PIS", Tipo::Aliquota)),
        (9, ("VL_CRED_PIS", Tipo::Valor)),
        (10, ("CST_COFINS", Tipo::C)),
        (11, ("ALIQ_COFINS", Tipo::Aliquota)),
        (12, ("VL_CRED_COFINS", Tipo::Valor)),
        (13, ("DESC_EST", Tipo::C)),
        (14, ("COD_CTA", Tipo::C)),
    ]);

    let registro_f200: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("IND_OPER", Tipo::C)),
        (3, ("UNID_IMOB", Tipo::C)),
        (4, ("IDENT_EMP", Tipo::C)),
        (5, ("DESC_UNID_IMOB", Tipo::C)),
        (6, ("NUM_CONT", Tipo::C)),
        (7, ("CPF_CNPJ_ADQU", Tipo::C)),
        (8, ("DT_OPER", Tipo::C)),
        (9, ("VL_TOT_VEND", Tipo::Valor)),
        (10, ("VL_REC_ACUM", Tipo::Valor)),
        (11, ("VL_TOT_REC", Tipo::Valor)),
        (12, ("CST_PIS", Tipo::C)),
        (13, ("VL_BC_PIS", Tipo::Valor)),
        (14, ("ALIQ_PIS", Tipo::Aliquota)),
        (15, ("VL_PIS", Tipo::Valor)),
        (16, ("CST_COFINS", Tipo::C)),
        (17, ("VL_BC_COFINS", Tipo::Valor)),
        (18, ("ALIQ_COFINS", Tipo::Aliquota)),
        (19, ("VL_COFINS", Tipo::Valor)),
        (20, ("PERC_REC_RECEB", Tipo::C)),
        (21, ("IND_NAT_EMP", Tipo::C)),
        (22, ("INF_COMP", Tipo::C)),
    ]);

    let registro_f205: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("VL_CUS_INC_ACUM_ANT", Tipo::Valor)),
        (3, ("VL_CUS_INC_PER_ESC", Tipo::Valor)),
        (4, ("VL_CUS_INC_ACUM", Tipo::Valor)),
        (5, ("VL_EXC_BC_CUS_INC_ACUM", Tipo::Valor)),
        (6, ("VL_BC_CUS_INC", Tipo::Valor)),
        (7, ("CST_PIS", Tipo::C)),
        (8, ("ALIQ_PIS", Tipo::Aliquota)),
        (9, ("VL_CRED_PIS_ACUM", Tipo::Valor)),
        (10, ("VL_CRED_PIS_DESC_ANT", Tipo::Valor)),
        (11, ("VL_CRED_PIS_DESC", Tipo::Valor)),
        (12, ("VL_CRED_PIS_DESC_FUT", Tipo::Valor)),
        (13, ("CST_COFINS", Tipo::C)),
        (14, ("ALIQ_COFINS", Tipo::Aliquota)),
        (15, ("VL_CRED_COFINS_ACUM", Tipo::Valor)),
        (16, ("VL_CRED_COFINS_DESC_ANT", Tipo::Valor)),
        (17, ("VL_CRED_COFINS_DESC", Tipo::Valor)),
        (18, ("VL_CRED_COFINS_DESC_FUT", Tipo::Valor)),
    ]);

    let registro_f210: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("VL_CUS_ORC", Tipo::Valor)),
        (3, ("VL_EXC", Tipo::Valor)),
        (4, ("VL_CUS_ORC_AJU", Tipo::Valor)),
        (5, ("VL_BC_CRED", Tipo::Valor)),
        (6, ("CST_PIS", Tipo::C)),
        (7, ("ALIQ_PIS", Tipo::Aliquota)),
        (8, ("VL_CRED_PIS_UTIL", Tipo::Valor)),
        (9, ("CST_COFINS", Tipo::C)),
        (10, ("ALIQ_COFINS", Tipo::Aliquota)),
        (11, ("VL_CRED_COFINS_UTIL", Tipo::Valor)),
    ]);

    let registro_f211: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("NUM_PROC", Tipo::C)),
        (3, ("IND_PROC", Tipo::C)),
    ]);

    let registro_f500: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("VL_REC_CAIXA", Tipo::Valor)),
        (3, ("CST_PIS", Tipo::C)),
        (4, ("VL_DESC_PIS", Tipo::Valor)),
        (5, ("VL_BC_PIS", Tipo::Valor)),
        (6, ("ALIQ_PIS", Tipo::Aliquota)),
        (7, ("VL_PIS", Tipo::Valor)),
        (8, ("CST_COFINS", Tipo::C)),
        (9, ("VL_DESC_COFINS", Tipo::Valor)),
        (10, ("VL_BC_COFINS", Tipo::Valor)),
        (11, ("ALIQ_COFINS", Tipo::Aliquota)),
        (12, ("VL_COFINS", Tipo::Valor)),
        (13, ("COD_MOD", Tipo::C)),
        (14, ("CFOP", Tipo::C)),
        (15, ("COD_CTA", Tipo::C)),
        (16, ("INFO_COMPL", Tipo::C)),
    ]);

    let registro_f509: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("NUM_PROC", Tipo::C)),
        (3, ("IND_PROC", Tipo::C)),
    ]);

    let registro_f510: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("VL_REC_CAIXA", Tipo::Valor)),
        (3, ("CST_PIS", Tipo::C)),
        (4, ("VL_DESC_PIS", Tipo::Valor)),
        (5, ("QUANT_BC_PIS", Tipo::C)),
        (6, ("ALIQ_PIS_QUANT", Tipo::Aliquota)),
        (7, ("VL_PIS", Tipo::Valor)),
        (8, ("CST_COFINS", Tipo::C)),
        (9, ("VL_DESC_COFINS", Tipo::Valor)),
        (10, ("QUANT_BC_COFINS", Tipo::C)),
        (11, ("ALIQ_COFINS_QUANT", Tipo::Aliquota)),
        (12, ("VL_COFINS", Tipo::Valor)),
        (13, ("COD_MOD", Tipo::C)),
        (14, ("CFOP", Tipo::C)),
        (15, ("COD_CTA", Tipo::C)),
        (16, ("INFO_COMPL", Tipo::C)),
    ]);

    let registro_f519: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("NUM_PROC", Tipo::C)),
        (3, ("IND_PROC", Tipo::C)),
    ]);

    let registro_f525: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("VL_REC", Tipo::Valor)),
        (3, ("IND_REC", Tipo::C)),
        (4, ("CNPJ_CPF", Tipo::C)),
        (5, ("NUM_DOC", Tipo::C)),
        (6, ("COD_ITEM", Tipo::C)),
        (7, ("VL_REC_DET", Tipo::Valor)),
        (8, ("CST_PIS", Tipo::C)),
        (9, ("CST_COFINS", Tipo::C)),
        (10, ("INFO_COMPL", Tipo::C)),
        (11, ("COD_CTA", Tipo::C)),
    ]);

    let registro_f550: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("VL_REC_COMP", Tipo::Valor)),
        (3, ("CST_PIS", Tipo::C)),
        (4, ("VL_DESC_PIS", Tipo::Valor)),
        (5, ("VL_BC_PIS", Tipo::Valor)),
        (6, ("ALIQ_PIS", Tipo::Aliquota)),
        (7, ("VL_PIS", Tipo::Valor)),
        (8, ("CST_COFINS", Tipo::C)),
        (9, ("VL_DESC_COFINS", Tipo::Valor)),
        (10, ("VL_BC_COFINS", Tipo::Valor)),
        (11, ("ALIQ_COFINS", Tipo::Aliquota)),
        (12, ("VL_COFINS", Tipo::Valor)),
        (13, ("COD_MOD", Tipo::C)),
        (14, ("CFOP", Tipo::C)),
        (15, ("COD_CTA", Tipo::C)),
        (16, ("INFO_COMPL", Tipo::C)),
    ]);

    let registro_f559: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("NUM_PROC", Tipo::C)),
        (3, ("IND_PROC", Tipo::C)),
    ]);

    let registro_f560: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("VL_REC_COMP", Tipo::Valor)),
        (3, ("CST_PIS", Tipo::C)),
        (4, ("VL_DESC_PIS", Tipo::Valor)),
        (5, ("QUANT_BC_PIS", Tipo::C)),
        (6, ("ALIQ_PIS_QUANT", Tipo::Aliquota)),
        (7, ("VL_PIS", Tipo::Valor)),
        (8, ("CST_COFINS", Tipo::C)),
        (9, ("VL_DESC_COFINS", Tipo::Valor)),
        (10, ("QUANT_BC_COFINS", Tipo::C)),
        (11, ("ALIQ_COFINS_QUANT", Tipo::Aliquota)),
        (12, ("VL_COFINS", Tipo::Valor)),
        (13, ("COD_MOD", Tipo::C)),
        (14, ("CFOP", Tipo::C)),
        (15, ("COD_CTA", Tipo::C)),
        (16, ("INFO_COMPL", Tipo::C)),
    ]);

    let registro_f569: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("NUM_PROC", Tipo::C)),
        (3, ("IND_PROC", Tipo::C)),
    ]);

    let registro_f600: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("IND_NAT_RET", Tipo::C)),
        (3, ("DT_RET", Tipo::C)),
        (4, ("VL_BC_RET", Tipo::Valor)),
        (5, ("VL_RET", Tipo::Valor)),
        (6, ("COD_REC", Tipo::C)),
        (7, ("IND_NAT_REC", Tipo::C)),
        (8, ("CNPJ", Tipo::C)),
        (9, ("VL_RET_PIS", Tipo::Valor)),
        (10, ("VL_RET_COFINS", Tipo::Valor)),
        (11, ("IND_DEC", Tipo::C)),
    ]);

    let registro_f700: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("IND_ORI_DED", Tipo::C)),
        (3, ("IND_NAT_DED", Tipo::C)),
        (4, ("VL_DED_PIS", Tipo::Valor)),
        (5, ("VL_DED_COFINS", Tipo::Valor)),
        (6, ("VL_BC_OPER", Tipo::Valor)),
        (7, ("CNPJ", Tipo::C)),
        (8, ("INF_COMP", Tipo::C)),
    ]);

    let registro_f800: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("IND_NAT_EVEN", Tipo::C)),
        (3, ("DT_EVEN", Tipo::C)),
        (4, ("CNPJ_SUCED", Tipo::C)),
        (5, ("PA_CONT_CRED", Tipo::C)),
        (6, ("COD_CRED", Tipo::C)),
        (7, ("VL_CRED_PIS", Tipo::Valor)),
        (8, ("VL_CRED_COFINS", Tipo::Valor)),
        (9, ("PER_CRED_CIS", Tipo::C)),
    ]);

    let registro_f990: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("1", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("QTD_LIN_F", Tipo::C)),
    ]);

    let registro_i001: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("1", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("IND_MOV", Tipo::C)),
    ]);

    let registro_i010: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("2", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("CNPJ", Tipo::C)),
        (3, ("IND_ATIV", Tipo::C)),
        (4, ("INFO_COMPL", Tipo::C)),
    ]);

    let registro_i100: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("VL_REC", Tipo::Valor)),
        (3, ("CST_PIS_COFINS", Tipo::C)),
        (4, ("VL_TOT_DED_GER", Tipo::Valor)),
        (5, ("VL_TOT_DED_ESP", Tipo::Valor)),
        (6, ("VL_BC_PIS", Tipo::Valor)),
        (7, ("ALIQ_PIS", Tipo::Aliquota)),
        (8, ("VL_PIS", Tipo::Valor)),
        (9, ("VL_BC_COFINS", Tipo::Valor)),
        (10, ("ALIQ_COFINS", Tipo::Aliquota)),
        (11, ("VL_COFINS", Tipo::Valor)),
        (12, ("INFO_COMPL", Tipo::C)),
    ]);

    let registro_i199: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("NUM_PROC", Tipo::C)),
        (3, ("IND_PROC", Tipo::C)),
    ]);

    let registro_i200: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("NUM_CAMPO", Tipo::C)),
        (3, ("COD_DET", Tipo::C)),
        (4, ("DET_VALOR", Tipo::C)),
        (5, ("COD_CTA", Tipo::C)),
        (6, ("INFO_COMPL", Tipo::C)),
    ]);

    let registro_i299: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("5", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("NUM_PROC", Tipo::C)),
        (3, ("IND_PROC", Tipo::C)),
    ]);

    let registro_i300: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("5", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("COD_COMP", Tipo::C)),
        (3, ("DET_VALOR", Tipo::C)),
        (4, ("COD_CTA", Tipo::C)),
        (5, ("INFO_COMPL", Tipo::C)),
    ]);

    let registro_i399: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("6", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("NUM_PROC", Tipo::C)),
        (3, ("IND_PROC", Tipo::C)),
    ]);

    let registro_i990: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("1", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("QTD_LIN_I", Tipo::C)),
    ]);

    let registro_m001: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("1", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("IND_MOV", Tipo::C)),
    ]);

    let registro_m100: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("2", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("COD_CRED", Tipo::C)),
        (3, ("IND_CRED_ORI", Tipo::C)),
        (4, ("VL_BC_PIS", Tipo::Valor)),
        (5, ("ALIQ_PIS", Tipo::Aliquota)),
        (6, ("QUANT_BC_PIS", Tipo::C)),
        (7, ("ALIQ_PIS_QUANT", Tipo::Aliquota)),
        (8, ("VL_CRED", Tipo::Valor)),
        (9, ("VL_AJUS_ACRES", Tipo::Valor)),
        (10, ("VL_AJUS_REDUC", Tipo::Valor)),
        (11, ("VL_CRED_DIF", Tipo::Valor)),
        (12, ("VL_CRED_DISP", Tipo::Valor)),
        (13, ("IND_DESC_CRED", Tipo::C)),
        (14, ("VL_CRED_DESC", Tipo::Valor)),
        (15, ("SLD_CRED", Tipo::Valor)),
    ]);

    let registro_m105: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("NAT_BC_CRED", Tipo::C)),
        (3, ("CST_PIS", Tipo::C)),
        (4, ("VL_BC_PIS_TOT", Tipo::Valor)),
        (5, ("VL_BC_PIS_CUM", Tipo::Valor)),
        (6, ("VL_BC_PIS_NC", Tipo::Valor)),
        (7, ("VL_BC_PIS", Tipo::Valor)),
        (8, ("QUANT_BC_PIS_TOT", Tipo::C)),
        (9, ("QUANT_BC_PIS", Tipo::C)),
        (10, ("DESC_CRED", Tipo::C)),
    ]);

    let registro_m110: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("IND_AJ", Tipo::C)),
        (3, ("VL_AJ", Tipo::Valor)),
        (4, ("COD_AJ", Tipo::C)),
        (5, ("NUM_DOC", Tipo::C)),
        (6, ("DESCR_AJ", Tipo::C)),
        (7, ("DT_REF", Tipo::C)),
    ]);

    let registro_m115: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("DET_VALOR_AJ", Tipo::C)),
        (3, ("CST_PIS", Tipo::C)),
        (4, ("DET_BC_CRED", Tipo::C)),
        (5, ("DET_ALIQ", Tipo::C)),
        (6, ("DT_OPER_AJ", Tipo::C)),
        (7, ("DESC_AJ", Tipo::C)),
        (8, ("COD_CTA", Tipo::C)),
        (9, ("INFO_COMPL", Tipo::C)),
    ]);

    let registro_m200: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("2", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("VL_TOT_CONT_NC_PER", Tipo::Valor)),
        (3, ("VL_TOT_CRED_DESC", Tipo::Valor)),
        (4, ("VL_TOT_CRED_DESC_ANT", Tipo::Valor)),
        (5, ("VL_TOT_CONT_NC_DEV", Tipo::Valor)),
        (6, ("VL_RET_NC", Tipo::Valor)),
        (7, ("VL_OUT_DED_NC", Tipo::Valor)),
        (8, ("VL_CONT_NC_REC", Tipo::Valor)),
        (9, ("VL_TOT_CONT_CUM_PER", Tipo::Valor)),
        (10, ("VL_RET_CUM", Tipo::Valor)),
        (11, ("VL_OUT_DED_CUM", Tipo::Valor)),
        (12, ("VL_CONT_CUM_REC", Tipo::Valor)),
        (13, ("VL_TOT_CONT_REC", Tipo::Valor)),
    ]);

    let registro_m205: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("NUM_CAMPO", Tipo::C)),
        (3, ("COD_REC", Tipo::C)),
        (4, ("VL_DEBITO", Tipo::Valor)),
    ]);

    let registro_m210: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("COD_CONT", Tipo::C)),
        (3, ("VL_REC_BRT", Tipo::Valor)),
        (4, ("VL_BC_CONT", Tipo::Valor)),
        (5, ("VL_AJUS_ACRES_BC_PIS", Tipo::Valor)),
        (6, ("VL_AJUS_REDUC_BC_PIS", Tipo::Valor)),
        (7, ("VL_BC_CONT_AJUS", Tipo::Valor)),
        (8, ("ALIQ_PIS", Tipo::Aliquota)),
        (9, ("QUANT_BC_PIS", Tipo::C)),
        (10, ("ALIQ_PIS_QUANT", Tipo::Aliquota)),
        (11, ("VL_CONT_APUR", Tipo::Valor)),
        (12, ("VL_AJUS_ACRES", Tipo::Valor)),
        (13, ("VL_AJUS_REDUC", Tipo::Valor)),
        (14, ("VL_CONT_DIFER", Tipo::Valor)),
        (15, ("VL_CONT_DIFER_ANT", Tipo::Valor)),
        (16, ("VL_CONT_PER", Tipo::Valor)),
    ]);

    let registro_m210_antigo: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("COD_CONT", Tipo::C)),
        (3, ("VL_REC_BRT", Tipo::Valor)),
        (4, ("VL_BC_CONT", Tipo::Valor)),
        (5, ("ALIQ_PIS", Tipo::Aliquota)),
        (6, ("QUANT_BC_PIS", Tipo::C)),
        (7, ("ALIQ_PIS_QUANT", Tipo::Aliquota)),
        (8, ("VL_CONT_APUR", Tipo::Valor)),
        (9, ("VL_AJUS_ACRES", Tipo::Valor)),
        (10, ("VL_AJUS_REDUC", Tipo::Valor)),
        (11, ("VL_CONT_DIFER", Tipo::Valor)),
        (12, ("VL_CONT_DIFER_ANT", Tipo::Valor)),
        (13, ("VL_CONT_PER", Tipo::Valor)),
    ]);

    let registro_m211: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("IND_TIP_COOP", Tipo::C)),
        (3, ("VL_BC_CONT_ANT_EXC_COOP", Tipo::Valor)),
        (4, ("VL_EXC_COOP_GER", Tipo::Valor)),
        (5, ("VL_EXC_ESP_COOP", Tipo::Valor)),
        (6, ("VL_BC_CONT", Tipo::Valor)),
    ]);

    let registro_m215: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("IND_AJ_BC", Tipo::C)),
        (3, ("VL_AJ_BC", Tipo::Valor)),
        (4, ("COD_AJ_BC", Tipo::C)),
        (5, ("NUM_DOC", Tipo::C)),
        (6, ("DESCR_AJ_BC", Tipo::C)),
        (7, ("DT_REF", Tipo::C)),
        (8, ("COD_CTA", Tipo::C)),
        (9, ("CNPJ", Tipo::C)),
        (10, ("INFO_COMPL", Tipo::C)),
    ]);

    let registro_m220: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("IND_AJ", Tipo::C)),
        (3, ("VL_AJ", Tipo::Valor)),
        (4, ("COD_AJ", Tipo::C)),
        (5, ("NUM_DOC", Tipo::C)),
        (6, ("DESCR_AJ", Tipo::C)),
        (7, ("DT_REF", Tipo::C)),
    ]);

    let registro_m225: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("5", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("DET_VALOR_AJ", Tipo::C)),
        (3, ("CST_PIS", Tipo::C)),
        (4, ("DET_BC_CRED", Tipo::C)),
        (5, ("DET_ALIQ", Tipo::C)),
        (6, ("DT_OPER_AJ", Tipo::C)),
        (7, ("DESC_AJ", Tipo::C)),
        (8, ("COD_CTA", Tipo::C)),
        (9, ("INFO_COMPL", Tipo::C)),
    ]);

    let registro_m230: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("CNPJ", Tipo::C)),
        (3, ("VL_VEND", Tipo::Valor)),
        (4, ("VL_NAO_RECEB", Tipo::Valor)),
        (5, ("VL_CONT_DIF", Tipo::Valor)),
        (6, ("VL_CRED_DIF", Tipo::Valor)),
        (7, ("COD_CRED", Tipo::C)),
    ]);

    let registro_m300: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("2", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("COD_CONT", Tipo::C)),
        (3, ("VL_CONT_APUR_DIFER", Tipo::Valor)),
        (4, ("NAT_CRED_DESC", Tipo::C)),
        (5, ("VL_CRED_DESC_DIFER", Tipo::Valor)),
        (6, ("VL_CONT_DIFER_ANT", Tipo::Valor)),
        (7, ("PER_APUR", Tipo::C)),
        (8, ("DT_RECEB", Tipo::C)),
    ]);

    let registro_m350: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("2", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("VL_TOT_FOL", Tipo::Valor)),
        (3, ("VL_EXC_BC", Tipo::Valor)),
        (4, ("VL_TOT_BC", Tipo::Valor)),
        (5, ("ALIQ_PIS_FOL", Tipo::Aliquota)),
        (6, ("VL_TOT_CONT_FOL", Tipo::Valor)),
    ]);

    let registro_m400: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("2", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("CST_PIS", Tipo::C)),
        (3, ("VL_TOT_REC", Tipo::Valor)),
        (4, ("COD_CTA", Tipo::C)),
        (5, ("DESC_COMPL", Tipo::C)),
    ]);

    let registro_m410: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("NAT_REC", Tipo::C)),
        (3, ("VL_REC", Tipo::Valor)),
        (4, ("COD_CTA", Tipo::C)),
        (5, ("DESC_COMPL", Tipo::C)),
    ]);

    let registro_m500: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("2", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("COD_CRED", Tipo::C)),
        (3, ("IND_CRED_ORI", Tipo::C)),
        (4, ("VL_BC_COFINS", Tipo::Valor)),
        (5, ("ALIQ_COFINS", Tipo::Aliquota)),
        (6, ("QUANT_BC_COFINS", Tipo::C)),
        (7, ("ALIQ_COFINS_QUANT", Tipo::Aliquota)),
        (8, ("VL_CRED", Tipo::Valor)),
        (9, ("VL_AJUS_ACRES", Tipo::Valor)),
        (10, ("VL_AJUS_REDUC", Tipo::Valor)),
        (11, ("VL_CRED_DIFER", Tipo::Valor)),
        (12, ("VL_CRED_DISP", Tipo::Valor)),
        (13, ("IND_DESC_CRED", Tipo::C)),
        (14, ("VL_CRED_DESC", Tipo::Valor)),
        (15, ("SLD_CRED", Tipo::Valor)),
    ]);

    let registro_m505: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("NAT_BC_CRED", Tipo::C)),
        (3, ("CST_COFINS", Tipo::C)),
        (4, ("VL_BC_COFINS_TOT", Tipo::Valor)),
        (5, ("VL_BC_COFINS_CUM", Tipo::Valor)),
        (6, ("VL_BC_COFINS_NC", Tipo::Valor)),
        (7, ("VL_BC_COFINS", Tipo::Valor)),
        (8, ("QUANT_BC_COFINS_TOT", Tipo::C)),
        (9, ("QUANT_BC_COFINS", Tipo::C)),
        (10, ("DESC_CRED", Tipo::C)),
    ]);

    let registro_m510: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("IND_AJ", Tipo::C)),
        (3, ("VL_AJ", Tipo::Valor)),
        (4, ("COD_AJ", Tipo::C)),
        (5, ("NUM_DOC", Tipo::C)),
        (6, ("DESCR_AJ", Tipo::C)),
        (7, ("DT_REF", Tipo::C)),
    ]);

    let registro_m515: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("DET_VALOR_AJ", Tipo::C)),
        (3, ("CST_COFINS", Tipo::C)),
        (4, ("DET_BC_CRED", Tipo::C)),
        (5, ("DET_ALIQ", Tipo::C)),
        (6, ("DT_OPER_AJ", Tipo::C)),
        (7, ("DESC_AJ", Tipo::C)),
        (8, ("COD_CTA", Tipo::C)),
        (9, ("INFO_COMPL", Tipo::C)),
    ]);

    let registro_m600: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("2", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("VL_TOT_CONT_NC_PER", Tipo::Valor)),
        (3, ("VL_TOT_CRED_DESC", Tipo::Valor)),
        (4, ("VL_TOT_CRED_DESC_ANT", Tipo::Valor)),
        (5, ("VL_TOT_CONT_NC_DEV", Tipo::Valor)),
        (6, ("VL_RET_NC", Tipo::Valor)),
        (7, ("VL_OUT_DED_NC", Tipo::Valor)),
        (8, ("VL_CONT_NC_REC", Tipo::Valor)),
        (9, ("VL_TOT_CONT_CUM_PER", Tipo::Valor)),
        (10, ("VL_RET_CUM", Tipo::Valor)),
        (11, ("VL_OUT_DED_CUM", Tipo::Valor)),
        (12, ("VL_CONT_CUM_REC", Tipo::Valor)),
        (13, ("VL_TOT_CONT_REC", Tipo::Valor)),
    ]);

    let registro_m605: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("NUM_CAMPO", Tipo::C)),
        (3, ("COD_REC", Tipo::C)),
        (4, ("VL_DEBITO", Tipo::Valor)),
    ]);

    let registro_m610: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("COD_CONT", Tipo::C)),
        (3, ("VL_REC_BRT", Tipo::Valor)),
        (4, ("VL_BC_CONT", Tipo::Valor)),
        (5, ("VL_AJUS_ACRES_BC_COFINS", Tipo::Valor)),
        (6, ("VL_AJUS_REDUC_BC_COFINS", Tipo::Valor)),
        (7, ("VL_BC_CONT_AJUS", Tipo::Valor)),
        (8, ("ALIQ_COFINS", Tipo::Aliquota)),
        (9, ("QUANT_BC_COFINS", Tipo::C)),
        (10, ("ALIQ_COFINS_QUANT", Tipo::Aliquota)),
        (11, ("VL_CONT_APUR", Tipo::Valor)),
        (12, ("VL_AJUS_ACRES", Tipo::Valor)),
        (13, ("VL_AJUS_REDUC", Tipo::Valor)),
        (14, ("VL_CONT_DIFER", Tipo::Valor)),
        (15, ("VL_CONT_DIFER_ANT", Tipo::Valor)),
        (16, ("VL_CONT_PER", Tipo::Valor)),
    ]);

    let registro_m610_antigo: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("COD_CONT", Tipo::C)),
        (3, ("VL_REC_BRT", Tipo::Valor)),
        (4, ("VL_BC_CONT", Tipo::Valor)),
        (5, ("ALIQ_COFINS", Tipo::Aliquota)),
        (6, ("QUANT_BC_COFINS", Tipo::C)),
        (7, ("ALIQ_COFINS_QUANT", Tipo::Aliquota)),
        (8, ("VL_CONT_APUR", Tipo::Valor)),
        (9, ("VL_AJUS_ACRES", Tipo::Valor)),
        (10, ("VL_AJUS_REDUC", Tipo::Valor)),
        (11, ("VL_CONT_DIFER", Tipo::Valor)),
        (12, ("VL_CONT_DIFER_ANT", Tipo::Valor)),
        (13, ("VL_CONT_PER", Tipo::Valor)),
    ]);

    let registro_m611: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("IND_TIP_COOP", Tipo::C)),
        (3, ("VL_BC_CONT_ANT_EXC_COOP", Tipo::Valor)),
        (4, ("VL_EXC_COOP_GER", Tipo::Valor)),
        (5, ("VL_EXC_ESP_COOP", Tipo::Valor)),
        (6, ("VL_BC_CONT", Tipo::Valor)),
    ]);

    let registro_m615: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("IND_AJ_BC", Tipo::C)),
        (3, ("VL_AJ_BC", Tipo::Valor)),
        (4, ("COD_AJ_BC", Tipo::C)),
        (5, ("NUM_DOC", Tipo::C)),
        (6, ("DESCR_AJ_BC", Tipo::C)),
        (7, ("DT_REF", Tipo::C)),
        (8, ("COD_CTA", Tipo::C)),
        (9, ("CNPJ", Tipo::C)),
        (10, ("INFO_COMPL", Tipo::C)),
    ]);

    let registro_m620: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("IND_AJ", Tipo::C)),
        (3, ("VL_AJ", Tipo::Valor)),
        (4, ("COD_AJ", Tipo::C)),
        (5, ("NUM_DOC", Tipo::C)),
        (6, ("DESCR_AJ", Tipo::C)),
        (7, ("DT_REF", Tipo::C)),
    ]);

    let registro_m625: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("5", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("DET_VALOR_AJ", Tipo::C)),
        (3, ("CST_COFINS", Tipo::C)),
        (4, ("DET_BC_CRED", Tipo::C)),
        (5, ("DET_ALIQ", Tipo::C)),
        (6, ("DT_OPER_AJ", Tipo::C)),
        (7, ("DESC_AJ", Tipo::C)),
        (8, ("COD_CTA", Tipo::C)),
        (9, ("INFO_COMPL", Tipo::C)),
    ]);

    let registro_m630: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("CNPJ", Tipo::C)),
        (3, ("VL_VEND", Tipo::Valor)),
        (4, ("VL_NAO_RECEB", Tipo::Valor)),
        (5, ("VL_CONT_DIF", Tipo::Valor)),
        (6, ("VL_CRED_DIF", Tipo::Valor)),
        (7, ("COD_CRED", Tipo::C)),
    ]);

    let registro_m700: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("2", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("COD_CONT", Tipo::C)),
        (3, ("VL_CONT_APUR_DIFER", Tipo::Valor)),
        (4, ("NAT_CRED_DESC", Tipo::C)),
        (5, ("VL_CRED_DESC_DIFER", Tipo::Valor)),
        (6, ("VL_CONT_DIFER_ANT", Tipo::Valor)),
        (7, ("PER_APUR", Tipo::C)),
        (8, ("DT_RECEB", Tipo::C)),
    ]);

    let registro_m800: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("2", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("CST_COFINS", Tipo::C)),
        (3, ("VL_TOT_REC", Tipo::Valor)),
        (4, ("COD_CTA", Tipo::C)),
        (5, ("DESC_COMPL", Tipo::C)),
    ]);

    let registro_m810: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("NAT_REC", Tipo::C)),
        (3, ("VL_REC", Tipo::Valor)),
        (4, ("COD_CTA", Tipo::C)),
        (5, ("DESC_COMPL", Tipo::C)),
    ]);

    let registro_m990: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("1", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("QTD_LIN_M", Tipo::C)),
    ]);

    let registro_p001: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("1", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("IND_MOV", Tipo::C)),
    ]);

    let registro_p010: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("2", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("CNPJ", Tipo::C)),
    ]);

    let registro_p100: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("DT_INI", Tipo::C)),
        (3, ("DT_FIN", Tipo::C)),
        (4, ("VL_REC_TOT_EST", Tipo::Valor)),
        (5, ("COD_ATIV_ECON", Tipo::C)),
        (6, ("VL_REC_ATIV_ESTAB", Tipo::Valor)),
        (7, ("VL_EXC", Tipo::Valor)),
        (8, ("VL_BC_CONT", Tipo::Valor)),
        (9, ("ALIQ_CONT", Tipo::Aliquota)),
        (10, ("VL_CONT_APU", Tipo::Valor)),
        (11, ("COD_CTA", Tipo::C)),
        (12, ("INFO_COMPL", Tipo::C)),
    ]);

    let registro_p110: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("NUM_CAMPO", Tipo::C)),
        (3, ("COD_DET", Tipo::C)),
        (4, ("DET_VALOR", Tipo::C)),
        (5, ("INF_COMPL", Tipo::C)),
    ]);

    let registro_p199: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("NUM_PROC", Tipo::C)),
        (3, ("IND_PROC", Tipo::C)),
    ]);

    let registro_p200: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("2", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("PER_REF", Tipo::C)),
        (3, ("VL_TOT_CONT_APU", Tipo::Valor)),
        (4, ("VL_TOT_AJ_REDUC", Tipo::Valor)),
        (5, ("VL_TOT_AJ_ACRES", Tipo::Valor)),
        (6, ("VL_TOT_CONT_DEV", Tipo::Valor)),
        (7, ("COD_REC", Tipo::C)),
    ]);

    let registro_p210: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("IND_AJ", Tipo::C)),
        (3, ("VL_AJ", Tipo::Valor)),
        (4, ("COD_AJ", Tipo::C)),
        (5, ("NUM_DOC", Tipo::C)),
        (6, ("DESCR_AJ", Tipo::C)),
        (7, ("DT_REF", Tipo::C)),
    ]);

    let registro_p990: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("1", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("QTD_LIN_P", Tipo::C)),
    ]);

    let registro_1001: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("1", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("IND_MOV", Tipo::C)),
    ]);

    let registro_1010: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("2", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("NUM_PROC", Tipo::C)),
        (3, ("ID_SEC_JUD", Tipo::C)),
        (4, ("ID_VARA", Tipo::C)),
        (5, ("IND_NAT_ACAO", Tipo::C)),
        (6, ("DESC_DEC_JUD", Tipo::C)),
        (7, ("DT_SENT_JUD", Tipo::C)),
    ]);

    let registro_1011: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("REG_REF", Tipo::C)),
        (3, ("CHAVE_DOC", Tipo::C)),
        (4, ("COD_PART", Tipo::C)),
        (5, ("COD_ITEM", Tipo::C)),
        (6, ("DT_OPER", Tipo::C)),
        (7, ("VL_OPER", Tipo::Valor)),
        (8, ("CST_PIS", Tipo::C)),
        (9, ("VL_BC_PIS", Tipo::Valor)),
        (10, ("ALIQ_PIS", Tipo::Aliquota)),
        (11, ("VL_PIS", Tipo::Valor)),
        (12, ("CST_COFINS", Tipo::C)),
        (13, ("VL_BC_COFINS", Tipo::Valor)),
        (14, ("ALIQ_COFINS", Tipo::Aliquota)),
        (15, ("VL_COFINS", Tipo::Valor)),
        (16, ("CST_PIS_SUSP", Tipo::C)),
        (17, ("VL_BC_PIS_SUSP", Tipo::Valor)),
        (18, ("ALIQ_PIS_SUSP", Tipo::Aliquota)),
        (19, ("VL_PIS_SUSP", Tipo::Valor)),
        (20, ("CST_COFINS_SUSP", Tipo::C)),
        (21, ("VL_BC_COFINS_SUSP", Tipo::Valor)),
        (22, ("ALIQ_COFINS_SUSP", Tipo::Aliquota)),
        (23, ("VL_COFINS_SUSP", Tipo::Valor)),
        (24, ("COD_CTA", Tipo::C)),
        (25, ("COD_CCUS", Tipo::C)),
        (26, ("DESC_DOC_OPER", Tipo::C)),
    ]);

    let registro_1020: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("2", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("NUM_PROC", Tipo::C)),
        (3, ("IND_NAT_ACAO", Tipo::C)),
        (4, ("DT_DEC_ADM", Tipo::C)),
    ]);

    let registro_1050: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("2", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("DT_REF", Tipo::C)),
        (3, ("IND_AJ_BC", Tipo::C)),
        (4, ("CNPJ", Tipo::C)),
        (5, ("VL_AJ_TOT", Tipo::Valor)),
        (6, ("VL_AJ_CST01", Tipo::Valor)),
        (7, ("VL_AJ_CST02", Tipo::Valor)),
        (8, ("VL_AJ_CST03", Tipo::Valor)),
        (9, ("VL_AJ_CST04", Tipo::Valor)),
        (10, ("VL_AJ_CST05", Tipo::Valor)),
        (11, ("VL_AJ_CST06", Tipo::Valor)),
        (12, ("VL_AJ_CST07", Tipo::Valor)),
        (13, ("VL_AJ_CST08", Tipo::Valor)),
        (14, ("VL_AJ_CST09", Tipo::Valor)),
        (15, ("VL_AJ_CST49", Tipo::Valor)),
        (16, ("VL_AJ_CST99", Tipo::Valor)),
        (17, ("IND_APROP", Tipo::C)),
        (18, ("NUM_REC", Tipo::C)),
        (19, ("INFO_COMPL", Tipo::C)),
    ]);

    let registro_1100: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("2", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("PER_APU_CRED", Tipo::C)),
        (3, ("ORIG_CRED", Tipo::C)),
        (4, ("CNPJ_SUC", Tipo::C)),
        (5, ("COD_CRED", Tipo::C)),
        (6, ("VL_CRED_APU", Tipo::Valor)),
        (7, ("VL_CRED_EXT_APU", Tipo::Valor)),
        (8, ("VL_TOT_CRED_APU", Tipo::Valor)),
        (9, ("VL_CRED_DESC_PA_ANT", Tipo::Valor)),
        (10, ("VL_CRED_PER_PA_ANT", Tipo::Valor)),
        (11, ("VL_CRED_DCOMP_PA_ANT", Tipo::Valor)),
        (12, ("SD_CRED_DISP_EFD", Tipo::C)),
        (13, ("VL_CRED_DESC_EFD", Tipo::Valor)),
        (14, ("VL_CRED_PER_EFD", Tipo::Valor)),
        (15, ("VL_CRED_DCOMP_EFD", Tipo::Valor)),
        (16, ("VL_CRED_TRANS", Tipo::Valor)),
        (17, ("VL_CRED_OUT", Tipo::Valor)),
        (18, ("SLD_CRED_FIM", Tipo::Valor)),
    ]);

    let registro_1101: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("COD_PART", Tipo::C)),
        (3, ("COD_ITEM", Tipo::C)),
        (4, ("COD_MOD", Tipo::C)),
        (5, ("SER", Tipo::C)),
        (6, ("SUB_SER", Tipo::C)),
        (7, ("NUM_DOC", Tipo::C)),
        (8, ("DT_OPER", Tipo::C)),
        (9, ("CHV_NFE", Tipo::C)),
        (10, ("VL_OPER", Tipo::Valor)),
        (11, ("CFOP", Tipo::C)),
        (12, ("NAT_BC_CRED", Tipo::C)),
        (13, ("IND_ORIG_CRED", Tipo::C)),
        (14, ("CST_PIS", Tipo::C)),
        (15, ("VL_BC_PIS", Tipo::Valor)),
        (16, ("ALIQ_PIS", Tipo::Aliquota)),
        (17, ("VL_PIS", Tipo::Valor)),
        (18, ("COD_CTA", Tipo::C)),
        (19, ("COD_CCUS", Tipo::C)),
        (20, ("DESC_COMPL", Tipo::C)),
        (21, ("PER_ESCRIT", Tipo::C)),
        (22, ("CNPJ", Tipo::C)),
    ]);

    let registro_1102: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("VL_CRED_PIS_TRIB_MI", Tipo::Valor)),
        (3, ("VL_CRED_PIS_NT_MI", Tipo::Valor)),
        (4, ("VL_CRED_PIS_EXP", Tipo::Valor)),
    ]);

    let registro_1200: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("2", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("PER_APUR_ANT", Tipo::C)),
        (3, ("NAT_CONT_REC", Tipo::C)),
        (4, ("VL_CONT_APUR", Tipo::Valor)),
        (5, ("VL_CRED_PIS_DESC", Tipo::Valor)),
        (6, ("VL_CONT_DEV", Tipo::Valor)),
        (7, ("VL_OUT_DED", Tipo::Valor)),
        (8, ("VL_CONT_EXT", Tipo::Valor)),
        (9, ("VL_MUL", Tipo::Valor)),
        (10, ("VL_JUR", Tipo::Valor)),
        (11, ("DT_RECOL", Tipo::C)),
    ]);

    let registro_1210: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("CNPJ", Tipo::C)),
        (3, ("CST_PIS", Tipo::C)),
        (4, ("COD_PART", Tipo::C)),
        (5, ("DT_OPER", Tipo::C)),
        (6, ("VL_OPER", Tipo::Valor)),
        (7, ("VL_BC_PIS", Tipo::Valor)),
        (8, ("ALIQ_PIS", Tipo::Aliquota)),
        (9, ("VL_PIS", Tipo::Valor)),
        (10, ("COD_CTA", Tipo::C)),
        (11, ("DESC_COMPL", Tipo::C)),
    ]);

    let registro_1220: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("PER_APU_CRED", Tipo::C)),
        (3, ("ORIG_CRED", Tipo::C)),
        (4, ("COD_CRED", Tipo::C)),
        (5, ("VL_CRED", Tipo::Valor)),
    ]);

    let registro_1300: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("2", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("IND_NAT_RET", Tipo::C)),
        (3, ("PR_REC_RET", Tipo::C)),
        (4, ("VL_RET_APU", Tipo::Valor)),
        (5, ("VL_RET_DED", Tipo::Valor)),
        (6, ("VL_RET_PER", Tipo::Valor)),
        (7, ("VL_RET_DCOMP", Tipo::Valor)),
        (8, ("SLD_RET", Tipo::Valor)),
    ]);

    let registro_1500: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("2", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("PER_APU_CRED", Tipo::C)),
        (3, ("ORIG_CRED", Tipo::C)),
        (4, ("CNPJ_SUC", Tipo::C)),
        (5, ("COD_CRED", Tipo::C)),
        (6, ("VL_CRED_APU", Tipo::Valor)),
        (7, ("VL_CRED_EXT_APU", Tipo::Valor)),
        (8, ("VL_TOT_CRED_APU", Tipo::Valor)),
        (9, ("VL_CRED_DESC_PA_ANT", Tipo::Valor)),
        (10, ("VL_CRED_PER_PA_ANT", Tipo::Valor)),
        (11, ("VL_CRED_DCOMP_PA_ANT", Tipo::Valor)),
        (12, ("SD_CRED_DISP_EFD", Tipo::C)),
        (13, ("VL_CRED_DESC_EFD", Tipo::Valor)),
        (14, ("VL_CRED_PER_EFD", Tipo::Valor)),
        (15, ("VL_CRED_DCOMP_EFD", Tipo::Valor)),
        (16, ("VL_CRED_TRANS", Tipo::Valor)),
        (17, ("VL_CRED_OUT", Tipo::Valor)),
        (18, ("SLD_CRED_FIM", Tipo::Valor)),
    ]);

    let registro_1501: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("COD_PART", Tipo::C)),
        (3, ("COD_ITEM", Tipo::C)),
        (4, ("COD_MOD", Tipo::C)),
        (5, ("SER", Tipo::C)),
        (6, ("SUB_SER", Tipo::C)),
        (7, ("NUM_DOC", Tipo::C)),
        (8, ("DT_OPER", Tipo::C)),
        (9, ("CHV_NFE", Tipo::C)),
        (10, ("VL_OPER", Tipo::Valor)),
        (11, ("CFOP", Tipo::C)),
        (12, ("NAT_BC_CRED", Tipo::C)),
        (13, ("IND_ORIG_CRED", Tipo::C)),
        (14, ("CST_COFINS", Tipo::C)),
        (15, ("VL_BC_COFINS", Tipo::Valor)),
        (16, ("ALIQ_COFINS", Tipo::Aliquota)),
        (17, ("VL_COFINS", Tipo::Valor)),
        (18, ("COD_CTA", Tipo::C)),
        (19, ("COD_CCUS", Tipo::C)),
        (20, ("DESC_COMPL", Tipo::C)),
        (21, ("PER_ESCRIT", Tipo::C)),
        (22, ("CNPJ", Tipo::C)),
    ]);

    let registro_1502: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("4", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("VL_CRED_COFINS_TRIB_MI", Tipo::Valor)),
        (3, ("VL_CRED_COFINS_NT_MI", Tipo::Valor)),
        (4, ("VL_CRED_COFINS_EXP", Tipo::Valor)),
    ]);

    let registro_1600: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("2", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("PER_APUR_ANT", Tipo::C)),
        (3, ("NAT_CONT_REC", Tipo::C)),
        (4, ("VL_CONT_APUR", Tipo::Valor)),
        (5, ("VL_CRED_COFINS_DESC", Tipo::Valor)),
        (6, ("VL_CONT_DEV", Tipo::Valor)),
        (7, ("VL_OUT_DED", Tipo::Valor)),
        (8, ("VL_CONT_EXT", Tipo::Valor)),
        (9, ("VL_MUL", Tipo::Valor)),
        (10, ("VL_JUR", Tipo::Valor)),
        (11, ("DT_RECOL", Tipo::C)),
    ]);

    let registro_1610: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("CNPJ", Tipo::C)),
        (3, ("CST_COFINS", Tipo::C)),
        (4, ("COD_PART", Tipo::C)),
        (5, ("DT_OPER", Tipo::C)),
        (6, ("VL_OPER", Tipo::Valor)),
        (7, ("VL_BC_COFINS", Tipo::Valor)),
        (8, ("ALIQ_COFINS", Tipo::Aliquota)),
        (9, ("VL_COFINS", Tipo::Valor)),
        (10, ("COD_CTA", Tipo::C)),
        (11, ("DESC_COMPL", Tipo::C)),
    ]);

    let registro_1620: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("PER_APU_CRED", Tipo::C)),
        (3, ("ORIG_CRED", Tipo::C)),
        (4, ("COD_CRED", Tipo::C)),
        (5, ("VL_CRED", Tipo::Valor)),
    ]);

    let registro_1700: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("2", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("IND_NAT_RET", Tipo::C)),
        (3, ("PR_REC_RET", Tipo::C)),
        (4, ("VL_RET_APU", Tipo::Valor)),
        (5, ("VL_RET_DED", Tipo::Valor)),
        (6, ("VL_RET_PER", Tipo::Valor)),
        (7, ("VL_RET_DCOMP", Tipo::Valor)),
        (8, ("SLD_RET", Tipo::Valor)),
    ]);

    let registro_1800: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("2", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("INC_IMOB", Tipo::C)),
        (3, ("REC_RECEB_RET", Tipo::Valor)),
        (4, ("REC_FIN_RET", Tipo::Valor)),
        (5, ("BC_RET", Tipo::C)),
        (6, ("ALIQ_RET", Tipo::Aliquota)),
        (7, ("VL_REC_UNI", Tipo::Valor)),
        (8, ("DT_REC_UNI", Tipo::C)),
        (9, ("COD_REC", Tipo::C)),
    ]);

    let registro_1809: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("3", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("NUM_PROC", Tipo::C)),
        (3, ("IND_PROC", Tipo::C)),
    ]);

    let registro_1900: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("2", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("CNPJ", Tipo::C)),
        (3, ("COD_MOD", Tipo::C)),
        (4, ("SER", Tipo::C)),
        (5, ("SUB_SER", Tipo::C)),
        (6, ("COD_SIT", Tipo::C)),
        (7, ("VL_TOT_REC", Tipo::Valor)),
        (8, ("QUANT_DOC", Tipo::C)),
        (9, ("CST_PIS", Tipo::C)),
        (10, ("CST_COFINS", Tipo::C)),
        (11, ("CFOP", Tipo::C)),
        (12, ("INF_COMPL", Tipo::C)),
        (13, ("COD_CTA", Tipo::C)),
    ]);

    let registro_1990: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("1", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("QTD_LIN_1", Tipo::C)),
    ]);

    let registro_9001: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("1", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("IND_MOV", Tipo::C)),
    ]);

    let registro_9900: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("2", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("REG_BLC", Tipo::C)),
        (3, ("QTD_REG_BLC", Tipo::C)),
    ]);

    let registro_9990: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("1", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("QTD_LIN_9", Tipo::C)),
    ]);

    let registro_9999: HashMap<u16, (&str, Tipo)> = HashMap::from([
        (0, ("0", Tipo::N)),
        (1, ("REG", Tipo::C)),
        (2, ("QTD_LIN", Tipo::C)),
    ]);

    // Adicionar todos os registros em efd_blocos:
    let efd_blocos: HashMap<&'static str, HashMap<u16, (&'static str, Tipo)>> = HashMap::from([
        // Bloco 0:
        ("0000", registro_0000),
        ("0001", registro_0001),
        ("0035", registro_0035),
        ("0100", registro_0100),
        ("0110", registro_0110),
        ("0111", registro_0111),
        ("0120", registro_0120),
        ("0140", registro_0140),
        ("0145", registro_0145),
        ("0150", registro_0150),
        ("0190", registro_0190),
        ("0200", registro_0200),
        ("0205", registro_0205),
        ("0206", registro_0206),
        ("0208", registro_0208),
        ("0400", registro_0400),
        ("0450", registro_0450),
        ("0500", registro_0500),
        ("0600", registro_0600),
        ("0900", registro_0900),
        ("0990", registro_0990),
        // Bloco A:
        ("A001", registro_a001),
        ("A010", registro_a010),
        ("A100", registro_a100),
        ("A110", registro_a110),
        ("A111", registro_a111),
        ("A120", registro_a120),
        ("A170", registro_a170),
        ("A990", registro_a990),
        // Bloco C:
        ("C001", registro_c001),
        ("C010", registro_c010),
        ("C100", registro_c100),
        ("C110", registro_c110),
        ("C111", registro_c111),
        ("C120", registro_c120),
        ("C170", registro_c170),
        ("C175", registro_c175),
        ("C180", registro_c180),
        ("C181", registro_c181),
        ("C185", registro_c185),
        ("C188", registro_c188),
        ("C190", registro_c190),
        ("C191", registro_c191),
        ("C195", registro_c195),
        ("C198", registro_c198),
        ("C199", registro_c199),
        ("C380", registro_c380),
        ("C381", registro_c381),
        ("C385", registro_c385),
        ("C395", registro_c395),
        ("C396", registro_c396),
        ("C400", registro_c400),
        ("C405", registro_c405),
        ("C481", registro_c481),
        ("C485", registro_c485),
        ("C489", registro_c489),
        ("C490", registro_c490),
        ("C491", registro_c491),
        ("C495", registro_c495),
        ("C499", registro_c499),
        ("C500", registro_c500),
        ("C501", registro_c501),
        ("C505", registro_c505),
        ("C509", registro_c509),
        ("C600", registro_c600),
        ("C601", registro_c601),
        ("C605", registro_c605),
        ("C609", registro_c609),
        ("C800", registro_c800),
        ("C810", registro_c810),
        ("C820", registro_c820),
        ("C830", registro_c830),
        ("C860", registro_c860),
        ("C870", registro_c870),
        ("C880", registro_c880),
        ("C890", registro_c890),
        ("C990", registro_c990),
        // Bloco D:
        ("D001", registro_d001),
        ("D010", registro_d010),
        ("D100", registro_d100),
        ("D101", registro_d101),
        ("D105", registro_d105),
        ("D111", registro_d111),
        ("D200", registro_d200),
        ("D201", registro_d201),
        ("D205", registro_d205),
        ("D209", registro_d209),
        ("D300", registro_d300),
        ("D309", registro_d309),
        ("D350", registro_d350),
        ("D359", registro_d359),
        ("D500", registro_d500),
        ("D501", registro_d501),
        ("D505", registro_d505),
        ("D509", registro_d509),
        ("D600", registro_d600),
        ("D601", registro_d601),
        ("D605", registro_d605),
        ("D609", registro_d609),
        ("D990", registro_d990),
        // Bloco F:
        ("F001", registro_f001),
        ("F010", registro_f010),
        ("F100", registro_f100),
        ("F111", registro_f111),
        ("F120", registro_f120),
        ("F129", registro_f129),
        ("F130", registro_f130),
        ("F139", registro_f139),
        ("F150", registro_f150),
        ("F200", registro_f200),
        ("F205", registro_f205),
        ("F210", registro_f210),
        ("F211", registro_f211),
        ("F500", registro_f500),
        ("F509", registro_f509),
        ("F510", registro_f510),
        ("F519", registro_f519),
        ("F525", registro_f525),
        ("F550", registro_f550),
        ("F559", registro_f559),
        ("F560", registro_f560),
        ("F569", registro_f569),
        ("F600", registro_f600),
        ("F700", registro_f700),
        ("F800", registro_f800),
        ("F990", registro_f990),
        // Bloco I:
        ("I001", registro_i001),
        ("I010", registro_i010),
        ("I100", registro_i100),
        ("I199", registro_i199),
        ("I200", registro_i200),
        ("I299", registro_i299),
        ("I300", registro_i300),
        ("I399", registro_i399),
        ("I990", registro_i990),
        // Bloco M:
        ("M001", registro_m001),
        ("M100", registro_m100),
        ("M105", registro_m105),
        ("M110", registro_m110),
        ("M115", registro_m115),
        ("M200", registro_m200),
        ("M205", registro_m205),
        ("M210", registro_m210),
        ("M210_antigo", registro_m210_antigo),
        ("M211", registro_m211),
        ("M215", registro_m215),
        ("M220", registro_m220),
        ("M225", registro_m225),
        ("M230", registro_m230),
        ("M300", registro_m300),
        ("M350", registro_m350),
        ("M400", registro_m400),
        ("M410", registro_m410),
        ("M500", registro_m500),
        ("M505", registro_m505),
        ("M510", registro_m510),
        ("M515", registro_m515),
        ("M600", registro_m600),
        ("M605", registro_m605),
        ("M610", registro_m610),
        ("M610_antigo", registro_m610_antigo),
        ("M611", registro_m611),
        ("M615", registro_m615),
        ("M620", registro_m620),
        ("M625", registro_m625),
        ("M630", registro_m630),
        ("M700", registro_m700),
        ("M800", registro_m800),
        ("M810", registro_m810),
        ("M990", registro_m990),
        // Bloco P:
        ("P001", registro_p001),
        ("P010", registro_p010),
        ("P100", registro_p100),
        ("P110", registro_p110),
        ("P199", registro_p199),
        ("P200", registro_p200),
        ("P210", registro_p210),
        ("P990", registro_p990),
        // Bloco 1:
        ("1001", registro_1001),
        ("1010", registro_1010),
        ("1011", registro_1011),
        ("1020", registro_1020),
        ("1050", registro_1050),
        ("1100", registro_1100),
        ("1101", registro_1101),
        ("1102", registro_1102),
        ("1200", registro_1200),
        ("1210", registro_1210),
        ("1220", registro_1220),
        ("1300", registro_1300),
        ("1500", registro_1500),
        ("1501", registro_1501),
        ("1502", registro_1502),
        ("1600", registro_1600),
        ("1610", registro_1610),
        ("1620", registro_1620),
        ("1700", registro_1700),
        ("1800", registro_1800),
        ("1809", registro_1809),
        ("1900", registro_1900),
        ("1990", registro_1990),
        // Bloco 9:
        ("9001", registro_9001),
        ("9900", registro_9900),
        ("9990", registro_9990),
        ("9999", registro_9999),
    ]);

    efd_blocos
}

#[cfg(test)]
mod tests {
    // cargo test -- --help
    // cargo test -- --nocapture
    // cargo test -- --show-output
    use super::*;

    #[test]
    fn registros_sped_efd() {
        let registros_efd = registros(); // tabela de registros

        let mut registro = "0000";
        let mut index = 5;
        let (reg_0000_05, _tipo) = registros_efd[registro][&index];

        registro = "C170";
        index = 37;
        let (reg_c170_37, _tipo) = registros_efd[registro][&index];

        registro = "1610";
        index = 9;
        let (reg_1610_09, _tipo) = registros_efd[registro][&index];

        println!("reg_0000_05: {reg_0000_05}");
        println!("reg_C170_37: {reg_c170_37}");
        println!("reg_1610_09: {reg_1610_09}");

        assert_eq!(reg_0000_05, "NUM_REC_ANTERIOR");
        assert_eq!(reg_c170_37, "COD_CTA");
        assert_eq!(reg_1610_09, "VL_COFINS");
    }
}
