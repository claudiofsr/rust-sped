use std::{collections::{HashMap, HashSet}, sync::LazyLock};

/// (Registro, nº de campos)
pub static REGISTROS_ANTIGOS: LazyLock<HashSet<(&'static str, usize)>> = LazyLock::new(|| {
	HashSet::from([
		("M210", 13),
		("M610", 13),
	])
});

pub fn registros_antigos(registro: &str, campos_len: usize) -> bool {
    REGISTROS_ANTIGOS.contains(&(registro, campos_len))
}

pub fn registros() -> HashMap::<&'static str, HashMap<u16, (&'static str, &'static str)>> {

	// (N°, (Campo, Tipo: C, N, Valor, Aliquota))

	let registro_0000: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("0", "C")), // nivel hierárquico
		( 1, ("REG", "C")),
		( 2, ("COD_VER", "C")),
		( 3, ("TIPO_ESCRIT", "C")),
		( 4, ("IND_SIT_ESP", "C")),
		( 5, ("NUM_REC_ANTERIOR", "C")),
		( 6, ("DT_INI", "C")),
		( 7, ("DT_FIN", "C")),
		( 8, ("NOME", "C")),
		( 9, ("CNPJ", "C")),
		(10, ("UF", "C")),
		(11, ("COD_MUN", "C")),
		(12, ("SUFRAMA", "C")),
		(13, ("IND_NAT_PJ", "C")),
		(14, ("IND_ATIV", "C")),
	]);

	let registro_0001: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("1", "C")),
		( 1, ("REG", "C")),
		( 2, ("IND_MOV", "C")),
	]);

	let registro_0035: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("2", "C")),
		( 1, ("REG", "C")),
		( 2, ("COD_SCP", "C")),
		( 3, ("DESC_SCP", "C")),
		( 4, ("INF_COMP", "C")),
	]);

	let registro_0100: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("2", "C")),
		( 1, ("REG", "C")),
		( 2, ("NOME", "C")),
		( 3, ("CPF", "C")),
		( 4, ("CRC", "C")),
		( 5, ("CNPJ", "C")),
		( 6, ("CEP", "C")),
		( 7, ("END", "C")),
		( 8, ("NUM", "C")),
		( 9, ("COMPL", "C")),
		(10, ("BAIRRO", "C")),
		(11, ("FONE", "C")),
		(12, ("FAX", "C")),
		(13, ("EMAIL", "C")),
		(14, ("COD_MUN", "C")),
	]);

	let registro_0110: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("2", "C")),
		( 1, ("REG", "C")),
		( 2, ("COD_INC_TRIB", "C")),
		( 3, ("IND_APRO_CRED", "C")),
		( 4, ("COD_TIPO_CONT", "C")),
		( 5, ("IND_REG_CUM", "C")),
	]);

	let registro_0111: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("REC_BRU_NCUM_TRIB_MI", "Valor")),
		( 3, ("REC_BRU_NCUM_NT_MI", "Valor")),
		( 4, ("REC_BRU_NCUM_EXP", "Valor")),
		( 5, ("REC_BRU_CUM", "Valor")),
		( 6, ("REC_BRU_TOTAL", "Valor")),
	]);

	let registro_0120: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("2", "C")),
		( 1, ("REG", "C")),
		( 2, ("MES_REFER", "C")),
		( 3, ("INF_COMP", "C")),
	]);

	let registro_0140: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("2", "C")),
		( 1, ("REG", "C")),
		( 2, ("COD_EST", "C")),
		( 3, ("NOME", "C")),
		( 4, ("CNPJ", "C")),
		( 5, ("UF", "C")),
		( 6, ("IE", "C")),
		( 7, ("COD_MUN", "C")),
		( 8, ("IM", "C")),
		( 9, ("SUFRAMA", "C")),
	]);

	let registro_0145: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("COD_INC_TRIB", "C")),
		( 3, ("VL_REC_TOT", "Valor")),
		( 4, ("VL_REC_ATIV", "Valor")),
		( 5, ("VL_REC_DEMAIS_ATIV", "Valor")),
		( 6, ("INFO_COMPL", "C")),
	]);

	let registro_0150: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("COD_PART", "C")),
		( 3, ("NOME", "C")),
		( 4, ("COD_PAIS", "C")),
		( 5, ("CNPJ", "C")),
		( 6, ("CPF", "C")),
		( 7, ("IE", "C")),
		( 8, ("COD_MUN", "C")),
		( 9, ("SUFRAMA", "C")),
		(10, ("END", "C")),
		(11, ("NUM", "C")),
		(12, ("COMPL", "C")),
		(13, ("BAIRRO", "C")),
	]);

	let registro_0190: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("UNID", "C")),
		( 3, ("DESCR", "C")),
	]);

	let registro_0200: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("COD_ITEM", "C")),
		( 3, ("DESCR_ITEM", "C")),
		( 4, ("COD_BARRA", "C")),
		( 5, ("COD_ANT_ITEM", "C")),
		( 6, ("UNID_INV", "C")),
		( 7, ("TIPO_ITEM", "C")),
		( 8, ("COD_NCM", "C")),
		( 9, ("EX_IPI", "C")),
		(10, ("COD_GEN", "C")),
		(11, ("COD_LST", "C")),
		(12, ("ALIQ_ICMS", "Aliquota")),
	]);

	let registro_0205: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("DESCR_ANT_ITEM", "C")),
		( 3, ("DT_INI", "C")),
		( 4, ("DT_FIM", "C")),
		( 5, ("COD_ANT_ITEM", "C")),
	]);

	let registro_0206: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("COD_COMB", "C")),
	]);

	let registro_0208: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("COD_TAB", "C")),
		( 2, ("COD_GRU", "C")),
		( 2, ("MARCA_COM", "C")),
	]);

	let registro_0400: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("COD_NAT", "C")),
		( 3, ("DESCR_NAT", "C")),
	]);

	let registro_0450: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("COD_INF", "C")),
		( 3, ("TXT", "C")),
	]);

	let registro_0500: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("2", "C")),
		( 1, ("REG", "C")),
		( 2, ("DT_ALT", "C")),
		( 3, ("COD_NAT_CC", "C")),
		( 4, ("IND_CTA", "C")),
		( 5, ("NIVEL", "C")),
		( 6, ("COD_CTA", "C")),
		( 7, ("NOME_CTA", "C")),
		( 8, ("COD_CTA_REF", "C")),
		( 9, ("CNPJ_EST", "C")),
	]);

	let registro_0600: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("2", "C")),
		( 1, ("REG", "C")),
		( 2, ("DT_ALT", "C")),
		( 3, ("COD_CCUS", "C")),
		( 4, ("CCUS", "C")),
	]);

	let registro_0900: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("2", "C")),
		( 1, ("REG", "C")),
		( 2, ("REC_TOTAL_BLOCO_A", "Valor")),
		( 3, ("REC_NRB_BLOCO_A", "Valor")),
		( 4, ("REC_TOTAL_BLOCO_C", "Valor")),
		( 5, ("REC_NRB_BLOCO_C", "Valor")),
		( 6, ("REC_TOTAL_BLOCO_D", "Valor")),
		( 7, ("REC_NRB_BLOCO_D", "Valor")),
		( 8, ("REC_TOTAL_BLOCO_F", "Valor")),
		( 9, ("REC_NRB_BLOCO_F", "Valor")),
		(10, ("REC_TOTAL_BLOCO_I", "Valor")),
		(11, ("REC_NRB_BLOCO_I", "Valor")),
		(12, ("REC_TOTAL_BLOCO_1", "Valor")),
		(13, ("REC_NRB_BLOCO_1", "Valor")),
		(14, ("REC_TOTAL_PERIODO", "Valor")),
		(15, ("REC_TOTAL_NRB_PERIODO", "Valor")),
	]);

	let registro_0990: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("1", "C")),
		( 1, ("REG", "C")),
		( 2, ("QTD_LIN_0", "C")),
	]);

	let registro_a001: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("1", "C")),
		( 1, ("REG", "C")),
		( 2, ("IND_MOV", "C")),
	]);

	let registro_a010: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("2", "C")),
		( 1, ("REG", "C")),
		( 2, ("CNPJ", "C")),
	]);

	let registro_a100: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("IND_OPER", "C")),
		( 3, ("IND_EMIT", "C")),
		( 4, ("COD_PART", "C")),
		( 5, ("COD_SIT", "C")),
		( 6, ("SER", "C")),
		( 7, ("SUB", "C")),
		( 8, ("NUM_DOC", "C")),
		( 9, ("CHV_NFSE", "C")),
		(10, ("DT_DOC", "C")),
		(11, ("DT_EXE_SERV", "C")),
		(12, ("VL_DOC", "Valor")),
		(13, ("IND_PGTO", "C")),
		(14, ("VL_DESC", "Valor")),
		(15, ("VL_BC_PIS", "Valor")),
		(16, ("VL_PIS", "Valor")),
		(17, ("VL_BC_COFINS", "Valor")),
		(18, ("VL_COFINS", "Valor")),
		(19, ("VL_PIS_RET", "Valor")),
		(20, ("VL_COFINS_RET", "Valor")),
		(21, ("VL_ISS", "Valor")),
	]);

	let registro_a110: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("COD_INF", "C")),
		( 3, ("TXT_COMPL", "C")),
	]);

	let registro_a111: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("NUM_PROC", "C")),
		( 3, ("IND_PROC", "C")),
	]);

	let registro_a120: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("VL_TOT_SERV", "Valor")),
		( 3, ("VL_BC_PIS", "Valor")),
		( 4, ("VL_PIS_IMP", "Valor")),
		( 5, ("DT_PAG_PIS", "C")),
		( 6, ("VL_BC_COFINS", "Valor")),
		( 7, ("VL_COFINS_IMP", "Valor")),
		( 8, ("DT_PAG_COFINS", "C")),
		( 9, ("LOC_EXE_SERV", "C")),
	]);

	let registro_a170: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("NUM_ITEM", "C")),
		( 3, ("COD_ITEM", "C")),
		( 4, ("DESCR_COMPL", "C")),
		( 5, ("VL_ITEM", "Valor")),
		( 6, ("VL_DESC", "Valor")),
		( 7, ("NAT_BC_CRED", "C")),
		( 8, ("IND_ORIG_CRED", "C")),
		( 9, ("CST_PIS", "C")),
		(10, ("VL_BC_PIS", "Valor")),
		(11, ("ALIQ_PIS", "Aliquota")),
		(12, ("VL_PIS", "Valor")),
		(13, ("CST_COFINS", "C")),
		(14, ("VL_BC_COFINS", "Valor")),
		(15, ("ALIQ_COFINS", "Aliquota")),
		(16, ("VL_COFINS", "Valor")),
		(17, ("COD_CTA", "C")),
		(18, ("COD_CCUS", "C")),
	]);

	let registro_a990: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("1", "C")),
		( 1, ("REG", "C")),
		( 2, ("QTD_LIN_A", "C")),
	]);

	let registro_c001: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("1", "C")),
		( 1, ("REG", "C")),
		( 2, ("IND_MOV", "C")),
	]);

	let registro_c010: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("2", "C")),
		( 1, ("REG", "C")),
		( 2, ("CNPJ", "C")),
		( 3, ("IND_ESCRI", "C")),
	]);

	let registro_c100: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("IND_OPER", "C")),
		( 3, ("IND_EMIT", "C")),
		( 4, ("COD_PART", "C")),
		( 5, ("COD_MOD", "C")),
		( 6, ("COD_SIT", "C")),
		( 7, ("SER", "C")),
		( 8, ("NUM_DOC", "C")),
		( 9, ("CHV_NFE", "C")),
		(10, ("DT_DOC", "C")),
		(11, ("DT_E_S", "C")),
		(12, ("VL_DOC", "Valor")),
		(13, ("IND_PGTO", "C")),
		(14, ("VL_DESC", "Valor")),
		(15, ("VL_ABAT_NT", "Valor")),
		(16, ("VL_MERC", "Valor")),
		(17, ("IND_FRT", "C")),
		(18, ("VL_FRT", "Valor")),
		(19, ("VL_SEG", "Valor")),
		(20, ("VL_OUT_DA", "Valor")),
		(21, ("VL_BC_ICMS", "Valor")),
		(22, ("VL_ICMS", "Valor")),
		(23, ("VL_BC_ICMS_ST", "Valor")),
		(24, ("VL_ICMS_ST", "Valor")),
		(25, ("VL_IPI", "Valor")),
		(26, ("VL_PIS", "Valor")),
		(27, ("VL_COFINS", "Valor")),
		(28, ("VL_PIS_ST", "Valor")),
		(29, ("VL_COFINS_ST", "Valor")),
	]);

	let registro_c110: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("COD_INF", "C")),
		( 3, ("TXT_COMPL", "C")),
	]);

	let registro_c111: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("NUM_PROC", "C")),
		( 3, ("IND_PROC", "C")),
	]);

	let registro_c120: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("COD_DOC_IMP", "C")),
		( 3, ("NUM_DOC_IMP", "C")),
		( 4, ("VL_PIS_IMP", "Valor")),
		( 5, ("VL_COFINS_IMP", "Valor")),
		( 6, ("NUM_ACDRAW", "C")),
	]);

	let registro_c170: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("NUM_ITEM", "C")),
		( 3, ("COD_ITEM", "C")),
		( 4, ("DESCR_COMPL", "C")),
		( 5, ("QTD", "C")),
		( 6, ("UNID", "C")),
		( 7, ("VL_ITEM", "Valor")),
		( 8, ("VL_DESC", "Valor")),
		( 9, ("IND_MOV", "C")),
		(10, ("CST_ICMS", "C")),
		(11, ("CFOP", "C")),
		(12, ("COD_NAT", "C")),
		(13, ("VL_BC_ICMS", "Valor")),
		(14, ("ALIQ_ICMS", "Aliquota")),
		(15, ("VL_ICMS", "Valor")),
		(16, ("VL_BC_ICMS_ST", "Valor")),
		(17, ("ALIQ_ST", "Aliquota")),
		(18, ("VL_ICMS_ST", "Valor")),
		(19, ("IND_APUR", "C")),
		(20, ("CST_IPI", "C")),
		(21, ("COD_ENQ", "C")),
		(22, ("VL_BC_IPI", "Valor")),
		(23, ("ALIQ_IPI", "Aliquota")),
		(24, ("VL_IPI", "Valor")),
		(25, ("CST_PIS", "C")),
		(26, ("VL_BC_PIS", "Valor")),
		(27, ("ALIQ_PIS", "Aliquota")),
		(28, ("QUANT_BC_PIS", "C")),
		(29, ("ALIQ_PIS_QUANT", "Aliquota")),
		(30, ("VL_PIS", "Valor")),
		(31, ("CST_COFINS", "C")),
		(32, ("VL_BC_COFINS", "Valor")),
		(33, ("ALIQ_COFINS", "Aliquota")),
		(34, ("QUANT_BC_COFINS", "C")),
		(35, ("ALIQ_COFINS_QUANT", "Aliquota")),
		(36, ("VL_COFINS", "Valor")),
		(37, ("COD_CTA", "C")),
	]);

	let registro_c175: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("CFOP", "C")),
		( 3, ("VL_OPR", "Valor")),
		( 4, ("VL_DESC", "Valor")),
		( 5, ("CST_PIS", "C")),
		( 6, ("VL_BC_PIS", "Valor")),
		( 7, ("ALIQ_PIS", "Aliquota")),
		( 8, ("QUANT_BC_PIS", "C")),
		( 9, ("ALIQ_PIS_QUANT", "Aliquota")),
		(10, ("VL_PIS", "Valor")),
		(11, ("CST_COFINS", "C")),
		(12, ("VL_BC_COFINS", "Valor")),
		(13, ("ALIQ_COFINS", "Aliquota")),
		(14, ("QUANT_BC_COFINS", "C")),
		(15, ("ALIQ_COFINS_QUANT", "Aliquota")),
		(16, ("VL_COFINS", "Valor")),
		(17, ("COD_CTA", "C")),
		(18, ("INFO_COMPL", "C")),
	]);

	let registro_c180: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("COD_MOD", "C")),
		( 3, ("DT_DOC_INI", "C")),
		( 4, ("DT_DOC_FIN", "C")),
		( 5, ("COD_ITEM", "C")),
		( 6, ("COD_NCM", "C")),
		( 7, ("EX_IPI", "C")),
		( 8, ("VL_TOT_ITEM", "Valor")),
	]);

	let registro_c181: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("CST_PIS", "C")),
		( 3, ("CFOP", "C")),
		( 4, ("VL_ITEM", "Valor")),
		( 5, ("VL_DESC", "Valor")),
		( 6, ("VL_BC_PIS", "Valor")),
		( 7, ("ALIQ_PIS", "Aliquota")),
		( 8, ("QUANT_BC_PIS", "C")),
		( 9, ("ALIQ_PIS_QUANT", "Aliquota")),
		(10, ("VL_PIS", "Valor")),
		(11, ("COD_CTA", "C")),
	]);

	let registro_c185: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("CST_COFINS", "C")),
		( 3, ("CFOP", "C")),
		( 4, ("VL_ITEM", "Valor")),
		( 5, ("VL_DESC", "Valor")),
		( 6, ("VL_BC_COFINS", "Valor")),
		( 7, ("ALIQ_COFINS", "Aliquota")),
		( 8, ("QUANT_BC_COFINS", "C")),
		( 9, ("ALIQ_COFINS_QUANT", "Aliquota")),
		(10, ("VL_COFINS", "Valor")),
		(11, ("COD_CTA", "C")),
	]);

	let registro_c188: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("NUM_PROC", "C")),
		( 3, ("IND_PROC", "C")),
	]);

	let registro_c190: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("COD_MOD", "C")),
		( 3, ("DT_REF_INI", "C")),
		( 4, ("DT_REF_FIN", "C")),
		( 5, ("COD_ITEM", "C")),
		( 6, ("COD_NCM", "C")),
		( 7, ("EX_IPI", "C")),
		( 8, ("VL_TOT_ITEM", "Valor")),
	]);

	let registro_c191: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("CNPJ_CPF_PART", "C")),
		( 3, ("CST_PIS", "C")),
		( 4, ("CFOP", "C")),
		( 5, ("VL_ITEM", "Valor")),
		( 6, ("VL_DESC", "Valor")),
		( 7, ("VL_BC_PIS", "Valor")),
		( 8, ("ALIQ_PIS", "Aliquota")),
		( 9, ("QUANT_BC_PIS", "C")),
		(10, ("ALIQ_PIS_QUANT", "Aliquota")),
		(11, ("VL_PIS", "Valor")),
		(12, ("COD_CTA", "C")),
	]);

	let registro_c195: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("CNPJ_CPF_PART", "C")),
		( 3, ("CST_COFINS", "C")),
		( 4, ("CFOP", "C")),
		( 5, ("VL_ITEM", "Valor")),
		( 6, ("VL_DESC", "Valor")),
		( 7, ("VL_BC_COFINS", "Valor")),
		( 8, ("ALIQ_COFINS", "Aliquota")),
		( 9, ("QUANT_BC_COFINS", "C")),
		(10, ("ALIQ_COFINS_QUANT", "Aliquota")),
		(11, ("VL_COFINS", "Valor")),
		(12, ("COD_CTA", "C")),
	]);

	let registro_c198: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("NUM_PROC", "C")),
		( 3, ("IND_PROC", "C")),
	]);

	let registro_c199: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("COD_DOC_IMP", "C")),
		( 3, ("NUM_DOC_IMP", "C")),
		( 4, ("VL_PIS_IMP", "Valor")),
		( 5, ("VL_COFINS_IMP", "Valor")),
		( 6, ("NUM_ACDRAW", "C")),
	]);

	let registro_c380: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("COD_MOD", "C")),
		( 3, ("DT_DOC_INI", "C")),
		( 4, ("DT_DOC_FIN", "C")),
		( 5, ("NUM_DOC_INI", "C")),
		( 6, ("NUM_DOC_FIN", "C")),
		( 7, ("VL_DOC", "Valor")),
		( 8, ("VL_DOC_CANC", "Valor")),
	]);

	let registro_c381: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("CST_PIS", "C")),
		( 3, ("COD_ITEM", "C")),
		( 4, ("VL_ITEM", "Valor")),
		( 5, ("VL_BC_PIS", "Valor")),
		( 6, ("ALIQ_PIS", "Aliquota")),
		( 7, ("QUANT_BC_PIS", "C")),
		( 8, ("ALIQ_PIS_QUANT", "Aliquota")),
		( 9, ("VL_PIS", "Valor")),
		(10, ("COD_CTA", "C")),
	]);

	let registro_c385: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("CST_COFINS", "C")),
		( 3, ("COD_ITEM", "C")),
		( 4, ("VL_ITEM", "Valor")),
		( 5, ("VL_BC_COFINS", "Valor")),
		( 6, ("ALIQ_COFINS", "Aliquota")),
		( 7, ("QUANT_BC_COFINS", "C")),
		( 8, ("ALIQ_COFINS_QUANT", "Aliquota")),
		( 9, ("VL_COFINS", "Valor")),
		(10, ("COD_CTA", "C")),
	]);

	let registro_c395: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("COD_MOD", "C")),
		( 3, ("COD_PART", "C")),
		( 4, ("SER", "C")),
		( 5, ("SUB_SER", "C")),
		( 6, ("NUM_DOC", "C")),
		( 7, ("DT_DOC", "C")),
		( 8, ("VL_DOC", "Valor")),
	]);

	let registro_c396: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("COD_ITEM", "C")),
		( 3, ("VL_ITEM", "Valor")),
		( 4, ("VL_DESC", "Valor")),
		( 5, ("NAT_BC_CRED", "C")),
		( 6, ("CST_PIS", "C")),
		( 7, ("VL_BC_PIS", "Valor")),
		( 8, ("ALIQ_PIS", "Aliquota")),
		( 9, ("VL_PIS", "Valor")),
		(10, ("CST_COFINS", "C")),
		(11, ("VL_BC_COFINS", "Valor")),
		(12, ("ALIQ_COFINS", "Aliquota")),
		(13, ("VL_COFINS", "Valor")),
		(14, ("COD_CTA", "C")),
	]);

	let registro_c400: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("COD_MOD", "C")),
		( 3, ("ECF_MOD", "C")),
		( 4, ("ECF_FAB", "C")),
		( 5, ("ECF_CX", "C")),
	]);

	let registro_c405: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("DT_DOC", "C")),
		( 3, ("CRO", "C")),
		( 4, ("CRZ", "C")),
		( 5, ("NUM_COO_FIN", "C")),
		( 6, ("GT_FIN", "C")),
		( 7, ("VL_BRT", "Valor")),
	]);

	let registro_c481: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("5", "C")),
		( 1, ("REG", "C")),
		( 2, ("CST_PIS", "C")),
		( 3, ("VL_ITEM", "Valor")),
		( 4, ("VL_BC_PIS", "Valor")),
		( 5, ("ALIQ_PIS", "Aliquota")),
		( 6, ("QUANT_BC_PIS", "C")),
		( 7, ("ALIQ_PIS_QUANT", "Aliquota")),
		( 8, ("VL_PIS", "Valor")),
		( 9, ("COD_ITEM", "C")),
		(10, ("COD_CTA", "C")),
	]);

	let registro_c485: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("5", "C")),
		( 1, ("REG", "C")),
		( 2, ("CST_COFINS", "C")),
		( 3, ("VL_ITEM", "Valor")),
		( 4, ("VL_BC_COFINS", "Valor")),
		( 5, ("ALIQ_COFINS", "Aliquota")),
		( 6, ("QUANT_BC_COFINS", "C")),
		( 7, ("ALIQ_COFINS_QUANT", "Aliquota")),
		( 8, ("VL_COFINS", "Valor")),
		( 9, ("COD_ITEM", "C")),
		(10, ("COD_CTA", "C")),
	]);

	let registro_c489: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("NUM_PROC", "C")),
		( 3, ("IND_PROC", "C")),
	]);

	let registro_c490: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("DT_DOC_INI", "C")),
		( 3, ("DT_DOC_FIN", "C")),
		( 4, ("COD_MOD", "C")),
	]);

	let registro_c491: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("COD_ITEM", "C")),
		( 3, ("CST_PIS", "C")),
		( 4, ("CFOP", "C")),
		( 5, ("VL_ITEM", "Valor")),
		( 6, ("VL_BC_PIS", "Valor")),
		( 7, ("ALIQ_PIS", "Aliquota")),
		( 8, ("QUANT_BC_PIS", "C")),
		( 9, ("ALIQ_PIS_QUANT", "Aliquota")),
		(10, ("VL_PIS", "Valor")),
		(11, ("COD_CTA", "C")),
	]);

	let registro_c495: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("COD_ITEM", "C")),
		( 3, ("CST_COFINS", "C")),
		( 4, ("CFOP", "C")),
		( 5, ("VL_ITEM", "Valor")),
		( 6, ("VL_BC_COFINS", "Valor")),
		( 7, ("ALIQ_COFINS", "Aliquota")),
		( 8, ("QUANT_BC_COFINS", "C")),
		( 9, ("ALIQ_COFINS_QUANT", "Aliquota")),
		(10, ("VL_COFINS", "Valor")),
		(11, ("COD_CTA", "C")),
	]);

	let registro_c499: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("NUM_PROC", "C")),
		( 3, ("IND_PROC", "C")),
	]);

	let registro_c500: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("COD_PART", "C")),
		( 3, ("COD_MOD", "C")),
		( 4, ("COD_SIT", "C")),
		( 5, ("SER", "C")),
		( 6, ("SUB", "C")),
		( 7, ("NUM_DOC", "C")),
		( 8, ("DT_DOC", "C")),
		( 9, ("DT_ENT", "C")),
		(10, ("VL_DOC", "Valor")),
		(11, ("VL_ICMS", "Valor")),
		(12, ("COD_INF", "C")),
		(13, ("VL_PIS", "Valor")),
		(14, ("VL_COFINS", "Valor")),
		(15, ("CHV_DOCe", "C")),
	]);

	let registro_c501: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("CST_PIS", "C")),
		( 3, ("VL_ITEM", "Valor")),
		( 4, ("NAT_BC_CRED", "C")),
		( 5, ("VL_BC_PIS", "Valor")),
		( 6, ("ALIQ_PIS", "Aliquota")),
		( 7, ("VL_PIS", "Valor")),
		( 8, ("COD_CTA", "C")),
	]);

	let registro_c505: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("CST_COFINS", "C")),
		( 3, ("VL_ITEM", "Valor")),
		( 4, ("NAT_BC_CRED", "C")),
		( 5, ("VL_BC_COFINS", "Valor")),
		( 6, ("ALIQ_COFINS", "Aliquota")),
		( 7, ("VL_COFINS", "Valor")),
		( 8, ("COD_CTA", "C")),
	]);

	let registro_c509: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("NUM_PROC", "C")),
		( 3, ("IND_PROC", "C")),
	]);

	let registro_c600: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("COD_MOD", "C")),
		( 3, ("COD_MUN", "C")),
		( 4, ("SER", "C")),
		( 5, ("SUB", "C")),
		( 6, ("COD_CONS", "C")),
		( 7, ("QTD_CONS", "C")),
		( 8, ("QTD_CANC", "C")),
		( 9, ("DT_DOC", "C")),
		(10, ("VL_DOC", "Valor")),
		(11, ("VL_DESC", "Valor")),
		(12, ("CONS", "C")),
		(13, ("VL_FORN", "Valor")),
		(14, ("VL_SERV_NT", "Valor")),
		(15, ("VL_TERC", "Valor")),
		(16, ("VL_DA", "Valor")),
		(17, ("VL_BC_ICMS", "Valor")),
		(18, ("VL_ICMS", "Valor")),
		(19, ("VL_BC_ICMS_ST", "Valor")),
		(20, ("VL_ICMS_ST", "Valor")),
		(21, ("VL_PIS", "Valor")),
		(22, ("VL_COFINS", "Valor")),
	]);

	let registro_c601: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("CST_PIS", "C")),
		( 3, ("VL_ITEM", "Valor")),
		( 4, ("VL_BC_PIS", "Valor")),
		( 5, ("ALIQ_PIS", "Aliquota")),
		( 6, ("VL_PIS", "Valor")),
		( 7, ("COD_CTA", "C")),
	]);

	let registro_c605: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("CST_COFINS", "C")),
		( 3, ("VL_ITEM", "Valor")),
		( 4, ("VL_BC_COFINS", "Valor")),
		( 5, ("ALIQ_COFINS", "Aliquota")),
		( 6, ("VL_COFINS", "Valor")),
		( 7, ("COD_CTA", "C")),
	]);

	let registro_c609: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("NUM_PROC", "C")),
		( 3, ("IND_PROC", "C")),
	]);

	let registro_c800: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("COD_MOD", "C")),
		( 3, ("COD_SIT", "C")),
		( 4, ("NUM_CFE", "C")),
		( 5, ("DT_DOC", "C")),
		( 6, ("VL_CFE", "Valor")),
		( 7, ("VL_PIS", "Valor")),
		( 8, ("VL_COFINS", "Valor")),
		( 9, ("CNPJ_CPF", "C")),
		(10, ("NR_SAT", "C")),
		(11, ("CHV_CFE", "C")),
		(12, ("VL_DESC", "Valor")),
		(13, ("VL_MERC", "Valor")),
		(14, ("VL_OUT_DA", "Valor")),
		(15, ("VL_ICMS", "Valor")),
		(16, ("VL_PIS_ST", "Valor")),
		(17, ("VL_COFINS_ST", "Valor")),
	]);

	let registro_c810: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("CFOP", "C")),
		( 3, ("VL_ITEM", "Valor")),
		( 4, ("COD_ITEM", "C")),
		( 5, ("CST_PIS", "C")),
		( 6, ("VL_BC_PIS", "Valor")),
		( 7, ("ALIQ_PIS", "Aliquota")),
		( 8, ("VL_PIS", "Valor")),
		( 9, ("CST_COFINS", "C")),
		(10, ("VL_BC_COFINS", "Valor")),
		(11, ("ALIQ_COFINS", "Aliquota")),
		(12, ("VL_COFINS", "Valor")),
		(13, ("COD_CTA", "C")),
	]);

	let registro_c820: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("CFOP", "C")),
		( 3, ("VL_ITEM", "Valor")),
		( 4, ("COD_ITEM", "C")),
		( 5, ("CST_PIS", "C")),
		( 6, ("QUANT_BC_PIS", "C")),
		( 7, ("ALIQ_PIS_QUANT", "Aliquota")),
		( 8, ("VL_PIS", "Valor")),
		( 9, ("CST_COFINS", "C")),
		(10, ("QUANT_BC_COFINS", "C")),
		(11, ("ALIQ_COFINS_QUANT", "Aliquota")),
		(12, ("VL_COFINS", "Valor")),
		(13, ("COD_CTA", "C")),
	]);

	let registro_c830: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("NUM_PROC", "C")),
		( 3, ("IND_PROC", "C")),
	]);

	let registro_c860: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("COD_MOD", "C")),
		( 3, ("NR_SAT", "C")),
		( 4, ("DT_DOC", "C")),
		( 5, ("DOC_INI", "C")),
		( 6, ("DOC_FIM", "C")),
	]);

	let registro_c870: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("COD_ITEM", "C")),
		( 3, ("CFOP", "C")),
		( 4, ("VL_ITEM", "Valor")),
		( 5, ("VL_DESC", "Valor")),
		( 6, ("CST_PIS", "C")),
		( 7, ("VL_BC_PIS", "Valor")),
		( 8, ("ALIQ_PIS", "Aliquota")),
		( 9, ("VL_PIS", "Valor")),
		(10, ("CST_COFINS", "C")),
		(11, ("VL_BC_COFINS", "Valor")),
		(12, ("ALIQ_COFINS", "Aliquota")),
		(13, ("VL_COFINS", "Valor")),
		(14, ("COD_CTA", "C")),
	]);

	let registro_c880: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("COD_ITEM", "C")),
		( 3, ("CFOP", "C")),
		( 4, ("VL_ITEM", "Valor")),
		( 5, ("VL_DESC", "Valor")),
		( 6, ("CST_PIS", "C")),
		( 7, ("QUANT_BC_PIS", "C")),
		( 8, ("ALIQ_PIS_QUANT", "Aliquota")),
		( 9, ("VL_PIS", "Valor")),
		(10, ("CST_COFINS", "C")),
		(11, ("QUANT_BC_COFINS", "C")),
		(12, ("ALIQ_COFINS_QUANT", "Aliquota")),
		(13, ("VL_COFINS", "Valor")),
		(14, ("COD_CTA", "C")),
	]);

	let registro_c890: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("NUM_PROC", "C")),
		( 3, ("IND_PROC", "C")),
	]);

	let registro_c990: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("1", "C")),
		( 1, ("REG", "C")),
		( 2, ("QTD_LIN_C", "C")),
	]);

	let registro_d001: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("1", "C")),
		( 1, ("REG", "C")),
		( 2, ("IND_MOV", "C")),
	]);

	let registro_d010: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("2", "C")),
		( 1, ("REG", "C")),
		( 2, ("CNPJ", "C")),
	]);

	let registro_d100: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("IND_OPER", "C")),
		( 3, ("IND_EMIT", "C")),
		( 4, ("COD_PART", "C")),
		( 5, ("COD_MOD", "C")),
		( 6, ("COD_SIT", "C")),
		( 7, ("SER", "C")),
		( 8, ("SUB", "C")),
		( 9, ("NUM_DOC", "C")),
		(10, ("CHV_CTE", "C")),
		(11, ("DT_DOC", "C")),
		(12, ("DT_A_P", "C")),
		(13, ("TP_CT-e", "C")),
		(14, ("CHV_CTE_REF", "C")),
		(15, ("VL_DOC", "Valor")),
		(16, ("VL_DESC", "Valor")),
		(17, ("IND_FRT", "C")),
		(18, ("VL_SERV", "Valor")),
		(19, ("VL_BC_ICMS", "Valor")),
		(20, ("VL_ICMS", "Valor")),
		(21, ("VL_NT", "Valor")),
		(22, ("COD_INF", "C")),
		(23, ("COD_CTA", "C")),
	]);

	let registro_d101: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("IND_NAT_FRT", "C")),
		( 3, ("VL_ITEM", "Valor")),
		( 4, ("CST_PIS", "C")),
		( 5, ("NAT_BC_CRED", "C")),
		( 6, ("VL_BC_PIS", "Valor")),
		( 7, ("ALIQ_PIS", "Aliquota")),
		( 8, ("VL_PIS", "Valor")),
		( 9, ("COD_CTA", "C")),
	]);

	let registro_d105: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("IND_NAT_FRT", "C")),
		( 3, ("VL_ITEM", "Valor")),
		( 4, ("CST_COFINS", "C")),
		( 5, ("NAT_BC_CRED", "C")),
		( 6, ("VL_BC_COFINS", "Valor")),
		( 7, ("ALIQ_COFINS", "Aliquota")),
		( 8, ("VL_COFINS", "Valor")),
		( 9, ("COD_CTA", "C")),
	]);

	let registro_d111: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("NUM_PROC", "C")),
		( 3, ("IND_PROC", "C")),
	]);

	let registro_d200: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("COD_MOD", "C")),
		( 3, ("COD_SIT", "C")),
		( 4, ("SER", "C")),
		( 5, ("SUB", "C")),
		( 6, ("NUM_DOC_INI", "C")),
		( 7, ("NUM_DOC_FIN", "C")),
		( 8, ("CFOP", "C")),
		( 9, ("DT_REF", "C")),
		(10, ("VL_DOC", "Valor")),
		(11, ("VL_DESC", "Valor")),
	]);

	let registro_d201: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("CST_PIS", "C")),
		( 3, ("VL_ITEM", "Valor")),
		( 4, ("VL_BC_PIS", "Valor")),
		( 5, ("ALIQ_PIS", "Aliquota")),
		( 6, ("VL_PIS", "Valor")),
		( 7, ("COD_CTA", "C")),
	]);

	let registro_d205: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("CST_COFINS", "C")),
		( 3, ("VL_ITEM", "Valor")),
		( 4, ("VL_BC_COFINS", "Valor")),
		( 5, ("ALIQ_COFINS", "Aliquota")),
		( 6, ("VL_COFINS", "Valor")),
		( 7, ("COD_CTA", "C")),
	]);

	let registro_d209: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("NUM_PROC", "C")),
		( 3, ("IND_PROC", "C")),
	]);

	let registro_d300: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("COD_MOD", "C")),
		( 3, ("SER", "C")),
		( 4, ("SUB", "C")),
		( 5, ("NUM_DOC_INI", "C")),
		( 6, ("NUM_DOC_FIN", "C")),
		( 7, ("CFOP", "C")),
		( 8, ("DT_REF", "C")),
		( 9, ("VL_DOC", "Valor")),
		(10, ("VL_DESC", "Valor")),
		(11, ("CST_PIS", "C")),
		(12, ("VL_BC_PIS", "Valor")),
		(13, ("ALIQ_PIS", "Aliquota")),
		(14, ("VL_PIS", "Valor")),
		(15, ("CST_COFINS", "C")),
		(16, ("VL_BC_COFINS", "Valor")),
		(17, ("ALIQ_COFINS", "Aliquota")),
		(18, ("VL_COFINS", "Valor")),
		(19, ("COD_CTA", "C")),
	]);

	let registro_d309: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("NUM_PROC", "C")),
		( 3, ("IND_PROC", "C")),
	]);

	let registro_d350: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("COD_MOD", "C")),
		( 3, ("ECF_MOD", "C")),
		( 4, ("ECF_FAB", "C")),
		( 5, ("DT_DOC", "C")),
		( 6, ("CRO", "C")),
		( 7, ("CRZ", "C")),
		( 8, ("NUM_COO_FIN", "C")),
		( 9, ("GT_FIN", "C")),
		(10, ("VL_BRT", "Valor")),
		(11, ("CST_PIS", "C")),
		(12, ("VL_BC_PIS", "Valor")),
		(13, ("ALIQ_PIS", "Aliquota")),
		(14, ("QUANT_BC_PIS", "C")),
		(15, ("ALIQ_PIS_QUANT", "Aliquota")),
		(16, ("VL_PIS", "Valor")),
		(17, ("CST_COFINS", "C")),
		(18, ("VL_BC_COFINS", "Valor")),
		(19, ("ALIQ_COFINS", "Aliquota")),
		(20, ("QUANT_BC_COFINS", "C")),
		(21, ("ALIQ_COFINS_QUANT", "Aliquota")),
		(22, ("VL_COFINS", "Valor")),
		(23, ("COD_CTA", "C")),
	]);

	let registro_d359: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("NUM_PROC", "C")),
		( 3, ("IND_PROC", "C")),
	]);

	let registro_d500: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("IND_OPER", "C")),
		( 3, ("IND_EMIT", "C")),
		( 4, ("COD_PART", "C")),
		( 5, ("COD_MOD", "C")),
		( 6, ("COD_SIT", "C")),
		( 7, ("SER", "C")),
		( 8, ("SUB", "C")),
		( 9, ("NUM_DOC", "C")),
		(10, ("DT_DOC", "C")),
		(11, ("DT_A_P", "C")),
		(12, ("VL_DOC", "Valor")),
		(13, ("VL_DESC", "Valor")),
		(14, ("VL_SERV", "Valor")),
		(15, ("VL_SERV_NT", "Valor")),
		(16, ("VL_TERC", "Valor")),
		(17, ("VL_DA", "Valor")),
		(18, ("VL_BC_ICMS", "Valor")),
		(19, ("VL_ICMS", "Valor")),
		(20, ("COD_INF", "C")),
		(21, ("VL_PIS", "Valor")),
		(22, ("VL_COFINS", "Valor")),
	]);

	let registro_d501: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("CST_PIS", "C")),
		( 3, ("VL_ITEM", "Valor")),
		( 4, ("NAT_BC_CRED", "C")),
		( 5, ("VL_BC_PIS", "Valor")),
		( 6, ("ALIQ_PIS", "Aliquota")),
		( 7, ("VL_PIS", "Valor")),
		( 8, ("COD_CTA", "C")),
	]);

	let registro_d505: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("CST_COFINS", "C")),
		( 3, ("VL_ITEM", "Valor")),
		( 4, ("NAT_BC_CRED", "C")),
		( 5, ("VL_BC_COFINS", "Valor")),
		( 6, ("ALIQ_COFINS", "Aliquota")),
		( 7, ("VL_COFINS", "Valor")),
		( 8, ("COD_CTA", "C")),
	]);

	let registro_d509: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("NUM_PROC", "C")),
		( 3, ("IND_PROC", "C")),
	]);

	let registro_d600: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("COD_MOD", "C")),
		( 3, ("COD_MUN", "C")),
		( 4, ("SER", "C")),
		( 5, ("SUB", "C")),
		( 6, ("IND_REC", "C")),
		( 7, ("QTD_CONS", "C")),
		( 8, ("DT_DOC_INI", "C")),
		( 9, ("DT_DOC_FIN", "C")),
		(10, ("VL_DOC", "Valor")),
		(11, ("VL_DESC", "Valor")),
		(12, ("VL_SERV", "Valor")),
		(13, ("VL_SERV_NT", "Valor")),
		(14, ("VL_TERC", "Valor")),
		(15, ("VL_DA", "Valor")),
		(16, ("VL_BC_ICMS", "Valor")),
		(17, ("VL_ICMS", "Valor")),
		(18, ("VL_PIS", "Valor")),
		(19, ("VL_COFINS", "Valor")),
	]);

	let registro_d601: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("COD_CLASS", "C")),
		( 3, ("VL_ITEM", "Valor")),
		( 4, ("VL_DESC", "Valor")),
		( 5, ("CST_PIS", "C")),
		( 6, ("VL_BC_PIS", "Valor")),
		( 7, ("ALIQ_PIS", "Aliquota")),
		( 8, ("VL_PIS", "Valor")),
		( 9, ("COD_CTA", "C")),
	]);

	let registro_d605: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("COD_CLASS", "C")),
		( 3, ("VL_ITEM", "Valor")),
		( 4, ("VL_DESC", "Valor")),
		( 5, ("CST_COFINS", "C")),
		( 6, ("VL_BC_COFINS", "Valor")),
		( 7, ("ALIQ_COFINS", "Aliquota")),
		( 8, ("VL_COFINS", "Valor")),
		( 9, ("COD_CTA", "C")),
	]);

	let registro_d609: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("NUM_PROC", "C")),
		( 3, ("IND_PROC", "C")),
	]);

	let registro_d990: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("1", "C")),
		( 1, ("REG", "C")),
		( 2, ("QTD_LIN_D", "C")),
	]);

	let registro_f001: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("1", "C")),
		( 1, ("REG", "C")),
		( 2, ("IND_MOV", "C")),
	]);

	let registro_f010: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("2", "C")),
		( 1, ("REG", "C")),
		( 2, ("CNPJ", "C")),
	]);

	let registro_f100: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("IND_OPER", "C")),
		( 3, ("COD_PART", "C")),
		( 4, ("COD_ITEM", "C")),
		( 5, ("DT_OPER", "C")),
		( 6, ("VL_OPER", "Valor")),
		( 7, ("CST_PIS", "C")),
		( 8, ("VL_BC_PIS", "Valor")),
		( 9, ("ALIQ_PIS", "Aliquota")),
		(10, ("VL_PIS", "Valor")),
		(11, ("CST_COFINS", "C")),
		(12, ("VL_BC_COFINS", "Valor")),
		(13, ("ALIQ_COFINS", "Aliquota")),
		(14, ("VL_COFINS", "Valor")),
		(15, ("NAT_BC_CRED", "C")),
		(16, ("IND_ORIG_CRED", "C")),
		(17, ("COD_CTA", "C")),
		(18, ("COD_CCUS", "C")),
		(19, ("DESC_DOC_OPER", "C")),
	]);

	let registro_f111: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("NUM_PROC", "C")),
		( 3, ("IND_PROC", "C")),
	]);

	let registro_f120: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("NAT_BC_CRED", "C")),
		( 3, ("IDENT_BEM_IMOB", "C")),
		( 4, ("IND_ORIG_CRED", "C")),
		( 5, ("IND_UTIL_BEM_IMOB", "C")),
		( 6, ("VL_OPER_DEP", "Valor")),
		( 7, ("PARC_OPER_NAO_BC_CRED", "C")),
		( 8, ("CST_PIS", "C")),
		( 9, ("VL_BC_PIS", "Valor")),
		(10, ("ALIQ_PIS", "Aliquota")),
		(11, ("VL_PIS", "Valor")),
		(12, ("CST_COFINS", "C")),
		(13, ("VL_BC_COFINS", "Valor")),
		(14, ("ALIQ_COFINS", "Aliquota")),
		(15, ("VL_COFINS", "Valor")),
		(16, ("COD_CTA", "C")),
		(17, ("COD_CCUS", "C")),
		(18, ("DESC_BEM_IMOB", "C")),
	]);

	let registro_f129: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("NUM_PROC", "C")),
		( 3, ("IND_PROC", "C")),
	]);

	let registro_f130: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("NAT_BC_CRED", "C")),
		( 3, ("IDENT_BEM_IMOB", "C")),
		( 4, ("IND_ORIG_CRED", "C")),
		( 5, ("IND_UTIL_BEM_IMOB", "C")),
		( 6, ("MES_OPER_AQUIS", "C")),
		( 7, ("VL_OPER_AQUIS", "Valor")),
		( 8, ("PARC_OPER_NAO_BC_CRED", "C")),
		( 9, ("VL_BC_CRED", "Valor")),
		(10, ("IND_NR_PARC", "C")),
		(11, ("CST_PIS", "C")),
		(12, ("VL_BC_PIS", "Valor")),
		(13, ("ALIQ_PIS", "Aliquota")),
		(14, ("VL_PIS", "Valor")),
		(15, ("CST_COFINS", "C")),
		(16, ("VL_BC_COFINS", "Valor")),
		(17, ("ALIQ_COFINS", "Aliquota")),
		(18, ("VL_COFINS", "Valor")),
		(19, ("COD_CTA", "C")),
		(20, ("COD_CCUS", "C")),
		(21, ("DESC_BEM_IMOB", "C")),
	]);

	let registro_f139: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("NUM_PROC", "C")),
		( 3, ("IND_PROC", "C")),
	]);

	let registro_f150: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("NAT_BC_CRED", "C")),
		( 3, ("VL_TOT_EST", "Valor")),
		( 4, ("EST_IMP", "C")),
		( 5, ("VL_BC_EST", "Valor")),
		( 6, ("VL_BC_MEN_EST", "Valor")),
		( 7, ("CST_PIS", "C")),
		( 8, ("ALIQ_PIS", "Aliquota")),
		( 9, ("VL_CRED_PIS", "Valor")),
		(10, ("CST_COFINS", "C")),
		(11, ("ALIQ_COFINS", "Aliquota")),
		(12, ("VL_CRED_COFINS", "Valor")),
		(13, ("DESC_EST", "C")),
		(14, ("COD_CTA", "C")),
	]);

	let registro_f200: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("IND_OPER", "C")),
		( 3, ("UNID_IMOB", "C")),
		( 4, ("IDENT_EMP", "C")),
		( 5, ("DESC_UNID_IMOB", "C")),
		( 6, ("NUM_CONT", "C")),
		( 7, ("CPF_CNPJ_ADQU", "C")),
		( 8, ("DT_OPER", "C")),
		( 9, ("VL_TOT_VEND", "Valor")),
		(10, ("VL_REC_ACUM", "Valor")),
		(11, ("VL_TOT_REC", "Valor")),
		(12, ("CST_PIS", "C")),
		(13, ("VL_BC_PIS", "Valor")),
		(14, ("ALIQ_PIS", "Aliquota")),
		(15, ("VL_PIS", "Valor")),
		(16, ("CST_COFINS", "C")),
		(17, ("VL_BC_COFINS", "Valor")),
		(18, ("ALIQ_COFINS", "Aliquota")),
		(19, ("VL_COFINS", "Valor")),
		(20, ("PERC_REC_RECEB", "C")),
		(21, ("IND_NAT_EMP", "C")),
		(22, ("INF_COMP", "C")),
	]);

	let registro_f205: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("VL_CUS_INC_ACUM_ANT", "Valor")),
		( 3, ("VL_CUS_INC_PER_ESC", "Valor")),
		( 4, ("VL_CUS_INC_ACUM", "Valor")),
		( 5, ("VL_EXC_BC_CUS_INC_ACUM", "Valor")),
		( 6, ("VL_BC_CUS_INC", "Valor")),
		( 7, ("CST_PIS", "C")),
		( 8, ("ALIQ_PIS", "Aliquota")),
		( 9, ("VL_CRED_PIS_ACUM", "Valor")),
		(10, ("VL_CRED_PIS_DESC_ANT", "Valor")),
		(11, ("VL_CRED_PIS_DESC", "Valor")),
		(12, ("VL_CRED_PIS_DESC_FUT", "Valor")),
		(13, ("CST_COFINS", "C")),
		(14, ("ALIQ_COFINS", "Aliquota")),
		(15, ("VL_CRED_COFINS_ACUM", "Valor")),
		(16, ("VL_CRED_COFINS_DESC_ANT", "Valor")),
		(17, ("VL_CRED_COFINS_DESC", "Valor")),
		(18, ("VL_CRED_COFINS_DESC_FUT", "Valor")),
	]);

	let registro_f210: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("VL_CUS_ORC", "Valor")),
		( 3, ("VL_EXC", "Valor")),
		( 4, ("VL_CUS_ORC_AJU", "Valor")),
		( 5, ("VL_BC_CRED", "Valor")),
		( 6, ("CST_PIS", "C")),
		( 7, ("ALIQ_PIS", "Aliquota")),
		( 8, ("VL_CRED_PIS_UTIL", "Valor")),
		( 9, ("CST_COFINS", "C")),
		(10, ("ALIQ_COFINS", "Aliquota")),
		(11, ("VL_CRED_COFINS_UTIL", "Valor")),
	]);

	let registro_f211: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("NUM_PROC", "C")),
		( 3, ("IND_PROC", "C")),
	]);

	let registro_f500: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("VL_REC_CAIXA", "Valor")),
		( 3, ("CST_PIS", "C")),
		( 4, ("VL_DESC_PIS", "Valor")),
		( 5, ("VL_BC_PIS", "Valor")),
		( 6, ("ALIQ_PIS", "Aliquota")),
		( 7, ("VL_PIS", "Valor")),
		( 8, ("CST_COFINS", "C")),
		( 9, ("VL_DESC_COFINS", "Valor")),
		(10, ("VL_BC_COFINS", "Valor")),
		(11, ("ALIQ_COFINS", "Aliquota")),
		(12, ("VL_COFINS", "Valor")),
		(13, ("COD_MOD", "C")),
		(14, ("CFOP", "C")),
		(15, ("COD_CTA", "C")),
		(16, ("INFO_COMPL", "C")),
	]);

	let registro_f509: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("NUM_PROC", "C")),
		( 3, ("IND_PROC", "C")),
	]);

	let registro_f510: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("VL_REC_CAIXA", "Valor")),
		( 3, ("CST_PIS", "C")),
		( 4, ("VL_DESC_PIS", "Valor")),
		( 5, ("QUANT_BC_PIS", "C")),
		( 6, ("ALIQ_PIS_QUANT", "Aliquota")),
		( 7, ("VL_PIS", "Valor")),
		( 8, ("CST_COFINS", "C")),
		( 9, ("VL_DESC_COFINS", "Valor")),
		(10, ("QUANT_BC_COFINS", "C")),
		(11, ("ALIQ_COFINS_QUANT", "Aliquota")),
		(12, ("VL_COFINS", "Valor")),
		(13, ("COD_MOD", "C")),
		(14, ("CFOP", "C")),
		(15, ("COD_CTA", "C")),
		(16, ("INFO_COMPL", "C")),
	]);

	let registro_f519: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("NUM_PROC", "C")),
		( 3, ("IND_PROC", "C")),
	]);

	let registro_f525: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("VL_REC", "Valor")),
		( 3, ("IND_REC", "C")),
		( 4, ("CNPJ_CPF", "C")),
		( 5, ("NUM_DOC", "C")),
		( 6, ("COD_ITEM", "C")),
		( 7, ("VL_REC_DET", "Valor")),
		( 8, ("CST_PIS", "C")),
		( 9, ("CST_COFINS", "C")),
		(10, ("INFO_COMPL", "C")),
		(11, ("COD_CTA", "C")),
	]);

	let registro_f550: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("VL_REC_COMP", "Valor")),
		( 3, ("CST_PIS", "C")),
		( 4, ("VL_DESC_PIS", "Valor")),
		( 5, ("VL_BC_PIS", "Valor")),
		( 6, ("ALIQ_PIS", "Aliquota")),
		( 7, ("VL_PIS", "Valor")),
		( 8, ("CST_COFINS", "C")),
		( 9, ("VL_DESC_COFINS", "Valor")),
		(10, ("VL_BC_COFINS", "Valor")),
		(11, ("ALIQ_COFINS", "Aliquota")),
		(12, ("VL_COFINS", "Valor")),
		(13, ("COD_MOD", "C")),
		(14, ("CFOP", "C")),
		(15, ("COD_CTA", "C")),
		(16, ("INFO_COMPL", "C")),
	]);

	let registro_f559: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("NUM_PROC", "C")),
		( 3, ("IND_PROC", "C")),
	]);

	let registro_f560: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("VL_REC_COMP", "Valor")),
		( 3, ("CST_PIS", "C")),
		( 4, ("VL_DESC_PIS", "Valor")),
		( 5, ("QUANT_BC_PIS", "C")),
		( 6, ("ALIQ_PIS_QUANT", "Aliquota")),
		( 7, ("VL_PIS", "Valor")),
		( 8, ("CST_COFINS", "C")),
		( 9, ("VL_DESC_COFINS", "Valor")),
		(10, ("QUANT_BC_COFINS", "C")),
		(11, ("ALIQ_COFINS_QUANT", "Aliquota")),
		(12, ("VL_COFINS", "Valor")),
		(13, ("COD_MOD", "C")),
		(14, ("CFOP", "C")),
		(15, ("COD_CTA", "C")),
		(16, ("INFO_COMPL", "C")),
	]);

	let registro_f569: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("NUM_PROC", "C")),
		( 3, ("IND_PROC", "C")),
	]);

	let registro_f600: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("IND_NAT_RET", "C")),
		( 3, ("DT_RET", "C")),
		( 4, ("VL_BC_RET", "Valor")),
		( 5, ("VL_RET", "Valor")),
		( 6, ("COD_REC", "C")),
		( 7, ("IND_NAT_REC", "C")),
		( 8, ("CNPJ", "C")),
		( 9, ("VL_RET_PIS", "Valor")),
		(10, ("VL_RET_COFINS", "Valor")),
		(11, ("IND_DEC", "C")),
	]);

	let registro_f700: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("IND_ORI_DED", "C")),
		( 3, ("IND_NAT_DED", "C")),
		( 4, ("VL_DED_PIS", "Valor")),
		( 5, ("VL_DED_COFINS", "Valor")),
		( 6, ("VL_BC_OPER", "Valor")),
		( 7, ("CNPJ", "C")),
		( 8, ("INF_COMP", "C")),
	]);

	let registro_f800: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("IND_NAT_EVEN", "C")),
		( 3, ("DT_EVEN", "C")),
		( 4, ("CNPJ_SUCED", "C")),
		( 5, ("PA_CONT_CRED", "C")),
		( 6, ("COD_CRED", "C")),
		( 7, ("VL_CRED_PIS", "Valor")),
		( 8, ("VL_CRED_COFINS", "Valor")),
		( 9, ("PER_CRED_CIS", "C")),
	]);

	let registro_f990: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("1", "C")),
		( 1, ("REG", "C")),
		( 2, ("QTD_LIN_F", "C")),
	]);

	let registro_i001: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("1", "C")),
		( 1, ("REG", "C")),
		( 2, ("IND_MOV", "C")),
	]);

	let registro_i010: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("2", "C")),
		( 1, ("REG", "C")),
		( 2, ("CNPJ", "C")),
		( 3, ("IND_ATIV", "C")),
		( 4, ("INFO_COMPL", "C")),
	]);

	let registro_i100: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("VL_REC", "Valor")),
		( 3, ("CST_PIS_COFINS", "C")),
		( 4, ("VL_TOT_DED_GER", "Valor")),
		( 5, ("VL_TOT_DED_ESP", "Valor")),
		( 6, ("VL_BC_PIS", "Valor")),
		( 7, ("ALIQ_PIS", "Aliquota")),
		( 8, ("VL_PIS", "Valor")),
		( 9, ("VL_BC_COFINS", "Valor")),
		(10, ("ALIQ_COFINS", "Aliquota")),
		(11, ("VL_COFINS", "Valor")),
		(12, ("INFO_COMPL", "C")),
	]);

	let registro_i199: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("NUM_PROC", "C")),
		( 3, ("IND_PROC", "C")),
	]);

	let registro_i200: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("NUM_CAMPO", "C")),
		( 3, ("COD_DET", "C")),
		( 4, ("DET_VALOR", "C")),
		( 5, ("COD_CTA", "C")),
		( 6, ("INFO_COMPL", "C")),
	]);

	let registro_i299: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("5", "C")),
		( 1, ("REG", "C")),
		( 2, ("NUM_PROC", "C")),
		( 3, ("IND_PROC", "C")),
	]);

	let registro_i300: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("5", "C")),
		( 1, ("REG", "C")),
		( 2, ("COD_COMP", "C")),
		( 3, ("DET_VALOR", "C")),
		( 4, ("COD_CTA", "C")),
		( 5, ("INFO_COMPL", "C")),
	]);

	let registro_i399: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("6", "C")),
		( 1, ("REG", "C")),
		( 2, ("NUM_PROC", "C")),
		( 3, ("IND_PROC", "C")),
	]);

	let registro_i990: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("1", "C")),
		( 1, ("REG", "C")),
		( 2, ("QTD_LIN_I", "C")),
	]);

	let registro_m001: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("1", "C")),
		( 1, ("REG", "C")),
		( 2, ("IND_MOV", "C")),
	]);

	let registro_m100: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("2", "C")),
		( 1, ("REG", "C")),
		( 2, ("COD_CRED", "C")),
		( 3, ("IND_CRED_ORI", "C")),
		( 4, ("VL_BC_PIS", "Valor")),
		( 5, ("ALIQ_PIS", "Aliquota")),
		( 6, ("QUANT_BC_PIS", "C")),
		( 7, ("ALIQ_PIS_QUANT", "Aliquota")),
		( 8, ("VL_CRED", "Valor")),
		( 9, ("VL_AJUS_ACRES", "Valor")),
		(10, ("VL_AJUS_REDUC", "Valor")),
		(11, ("VL_CRED_DIF", "Valor")),
		(12, ("VL_CRED_DISP", "Valor")),
		(13, ("IND_DESC_CRED", "C")),
		(14, ("VL_CRED_DESC", "Valor")),
		(15, ("SLD_CRED", "Valor")),
	]);

	let registro_m105: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("NAT_BC_CRED", "C")),
		( 3, ("CST_PIS", "C")),
		( 4, ("VL_BC_PIS_TOT", "Valor")),
		( 5, ("VL_BC_PIS_CUM", "Valor")),
		( 6, ("VL_BC_PIS_NC", "Valor")),
		( 7, ("VL_BC_PIS", "Valor")),
		( 8, ("QUANT_BC_PIS_TOT", "C")),
		( 9, ("QUANT_BC_PIS", "C")),
		(10, ("DESC_CRED", "C")),
	]);

	let registro_m110: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("IND_AJ", "C")),
		( 3, ("VL_AJ", "Valor")),
		( 4, ("COD_AJ", "C")),
		( 5, ("NUM_DOC", "C")),
		( 6, ("DESCR_AJ", "C")),
		( 7, ("DT_REF", "C")),
	]);

	let registro_m115: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("DET_VALOR_AJ", "C")),
		( 3, ("CST_PIS", "C")),
		( 4, ("DET_BC_CRED", "C")),
		( 5, ("DET_ALIQ", "C")),
		( 6, ("DT_OPER_AJ", "C")),
		( 7, ("DESC_AJ", "C")),
		( 8, ("COD_CTA", "C")),
		( 9, ("INFO_COMPL", "C")),
	]);

	let registro_m200: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("2", "C")),
		( 1, ("REG", "C")),
		( 2, ("VL_TOT_CONT_NC_PER", "Valor")),
		( 3, ("VL_TOT_CRED_DESC", "Valor")),
		( 4, ("VL_TOT_CRED_DESC_ANT", "Valor")),
		( 5, ("VL_TOT_CONT_NC_DEV", "Valor")),
		( 6, ("VL_RET_NC", "Valor")),
		( 7, ("VL_OUT_DED_NC", "Valor")),
		( 8, ("VL_CONT_NC_REC", "Valor")),
		( 9, ("VL_TOT_CONT_CUM_PER", "Valor")),
		(10, ("VL_RET_CUM", "Valor")),
		(11, ("VL_OUT_DED_CUM", "Valor")),
		(12, ("VL_CONT_CUM_REC", "Valor")),
		(13, ("VL_TOT_CONT_REC", "Valor")),
	]);

	let registro_m205: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("NUM_CAMPO", "C")),
		( 3, ("COD_REC", "C")),
		( 4, ("VL_DEBITO", "Valor")),
	]);

	let registro_m210: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("COD_CONT", "C")),
		( 3, ("VL_REC_BRT", "Valor")),
		( 4, ("VL_BC_CONT", "Valor")),
		( 5, ("VL_AJUS_ACRES_BC_PIS", "Valor")),
		( 6, ("VL_AJUS_REDUC_BC_PIS", "Valor")),
		( 7, ("VL_BC_CONT_AJUS", "Valor")),
		( 8, ("ALIQ_PIS", "Aliquota")),
		( 9, ("QUANT_BC_PIS", "C")),
		(10, ("ALIQ_PIS_QUANT", "Aliquota")),
		(11, ("VL_CONT_APUR", "Valor")),
		(12, ("VL_AJUS_ACRES", "Valor")),
		(13, ("VL_AJUS_REDUC", "Valor")),
		(14, ("VL_CONT_DIFER", "Valor")),
		(15, ("VL_CONT_DIFER_ANT", "Valor")),
		(16, ("VL_CONT_PER", "Valor")),
	]);

	let registro_m210_antigo: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("COD_CONT", "C")),
		( 3, ("VL_REC_BRT", "Valor")),
		( 4, ("VL_BC_CONT", "Valor")),
		( 5, ("ALIQ_PIS", "Aliquota")),
		( 6, ("QUANT_BC_PIS", "C")),
		( 7, ("ALIQ_PIS_QUANT", "Aliquota")),
		( 8, ("VL_CONT_APUR", "Valor")),
		( 9, ("VL_AJUS_ACRES", "Valor")),
		(10, ("VL_AJUS_REDUC", "Valor")),
		(11, ("VL_CONT_DIFER", "Valor")),
		(12, ("VL_CONT_DIFER_ANT", "Valor")),
		(13, ("VL_CONT_PER", "Valor")),
	]);

	let registro_m211: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("IND_TIP_COOP", "C")),
		( 3, ("VL_BC_CONT_ANT_EXC_COOP", "Valor")),
		( 4, ("VL_EXC_COOP_GER", "Valor")),
		( 5, ("VL_EXC_ESP_COOP", "Valor")),
		( 6, ("VL_BC_CONT", "Valor")),
	]);

	let registro_m215: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("IND_AJ_BC", "C")),
		( 3, ("VL_AJ_BC", "Valor")),
		( 4, ("COD_AJ_BC", "C")),
		( 5, ("NUM_DOC", "C")),
		( 6, ("DESCR_AJ_BC", "C")),
		( 7, ("DT_REF", "C")),
		( 8, ("COD_CTA", "C")),
		( 9, ("CNPJ", "C")),
		(10, ("INFO_COMPL", "C")),
	]);

	let registro_m220: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("IND_AJ", "C")),
		( 3, ("VL_AJ", "Valor")),
		( 4, ("COD_AJ", "C")),
		( 5, ("NUM_DOC", "C")),
		( 6, ("DESCR_AJ", "C")),
		( 7, ("DT_REF", "C")),
	]);

	let registro_m225: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("5", "C")),
		( 1, ("REG", "C")),
		( 2, ("DET_VALOR_AJ", "C")),
		( 3, ("CST_PIS", "C")),
		( 4, ("DET_BC_CRED", "C")),
		( 5, ("DET_ALIQ", "C")),
		( 6, ("DT_OPER_AJ", "C")),
		( 7, ("DESC_AJ", "C")),
		( 8, ("COD_CTA", "C")),
		( 9, ("INFO_COMPL", "C")),
	]);

	let registro_m230: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("CNPJ", "C")),
		( 3, ("VL_VEND", "Valor")),
		( 4, ("VL_NAO_RECEB", "Valor")),
		( 5, ("VL_CONT_DIF", "Valor")),
		( 6, ("VL_CRED_DIF", "Valor")),
		( 7, ("COD_CRED", "C")),
	]);

	let registro_m300: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("2", "C")),
		( 1, ("REG", "C")),
		( 2, ("COD_CONT", "C")),
		( 3, ("VL_CONT_APUR_DIFER", "Valor")),
		( 4, ("NAT_CRED_DESC", "C")),
		( 5, ("VL_CRED_DESC_DIFER", "Valor")),
		( 6, ("VL_CONT_DIFER_ANT", "Valor")),
		( 7, ("PER_APUR", "C")),
		( 8, ("DT_RECEB", "C")),
	]);

	let registro_m350: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("2", "C")),
		( 1, ("REG", "C")),
		( 2, ("VL_TOT_FOL", "Valor")),
		( 3, ("VL_EXC_BC", "Valor")),
		( 4, ("VL_TOT_BC", "Valor")),
		( 5, ("ALIQ_PIS_FOL", "Aliquota")),
		( 6, ("VL_TOT_CONT_FOL", "Valor")),
	]);

	let registro_m400: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("2", "C")),
		( 1, ("REG", "C")),
		( 2, ("CST_PIS", "C")),
		( 3, ("VL_TOT_REC", "Valor")),
		( 4, ("COD_CTA", "C")),
		( 5, ("DESC_COMPL", "C")),
	]);

	let registro_m410: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("NAT_REC", "C")),
		( 3, ("VL_REC", "Valor")),
		( 4, ("COD_CTA", "C")),
		( 5, ("DESC_COMPL", "C")),
	]);

	let registro_m500: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("2", "C")),
		( 1, ("REG", "C")),
		( 2, ("COD_CRED", "C")),
		( 3, ("IND_CRED_ORI", "C")),
		( 4, ("VL_BC_COFINS", "Valor")),
		( 5, ("ALIQ_COFINS", "Aliquota")),
		( 6, ("QUANT_BC_COFINS", "C")),
		( 7, ("ALIQ_COFINS_QUANT", "Aliquota")),
		( 8, ("VL_CRED", "Valor")),
		( 9, ("VL_AJUS_ACRES", "Valor")),
		(10, ("VL_AJUS_REDUC", "Valor")),
		(11, ("VL_CRED_DIFER", "Valor")),
		(12, ("VL_CRED_DISP", "Valor")),
		(13, ("IND_DESC_CRED", "C")),
		(14, ("VL_CRED_DESC", "Valor")),
		(15, ("SLD_CRED", "Valor")),
	]);

	let registro_m505: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("NAT_BC_CRED", "C")),
		( 3, ("CST_COFINS", "C")),
		( 4, ("VL_BC_COFINS_TOT", "Valor")),
		( 5, ("VL_BC_COFINS_CUM", "Valor")),
		( 6, ("VL_BC_COFINS_NC", "Valor")),
		( 7, ("VL_BC_COFINS", "Valor")),
		( 8, ("QUANT_BC_COFINS_TOT", "C")),
		( 9, ("QUANT_BC_COFINS", "C")),
		(10, ("DESC_CRED", "C")),
	]);

	let registro_m510: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("IND_AJ", "C")),
		( 3, ("VL_AJ", "Valor")),
		( 4, ("COD_AJ", "C")),
		( 5, ("NUM_DOC", "C")),
		( 6, ("DESCR_AJ", "C")),
		( 7, ("DT_REF", "C")),
	]);

	let registro_m515: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("DET_VALOR_AJ", "C")),
		( 3, ("CST_COFINS", "C")),
		( 4, ("DET_BC_CRED", "C")),
		( 5, ("DET_ALIQ", "C")),
		( 6, ("DT_OPER_AJ", "C")),
		( 7, ("DESC_AJ", "C")),
		( 8, ("COD_CTA", "C")),
		( 9, ("INFO_COMPL", "C")),
	]);

	let registro_m600: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("2", "C")),
		( 1, ("REG", "C")),
		( 2, ("VL_TOT_CONT_NC_PER", "Valor")),
		( 3, ("VL_TOT_CRED_DESC", "Valor")),
		( 4, ("VL_TOT_CRED_DESC_ANT", "Valor")),
		( 5, ("VL_TOT_CONT_NC_DEV", "Valor")),
		( 6, ("VL_RET_NC", "Valor")),
		( 7, ("VL_OUT_DED_NC", "Valor")),
		( 8, ("VL_CONT_NC_REC", "Valor")),
		( 9, ("VL_TOT_CONT_CUM_PER", "Valor")),
		(10, ("VL_RET_CUM", "Valor")),
		(11, ("VL_OUT_DED_CUM", "Valor")),
		(12, ("VL_CONT_CUM_REC", "Valor")),
		(13, ("VL_TOT_CONT_REC", "Valor")),
	]);

	let registro_m605: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("NUM_CAMPO", "C")),
		( 3, ("COD_REC", "C")),
		( 4, ("VL_DEBITO", "Valor")),
	]);

	let registro_m610: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("COD_CONT", "C")),
		( 3, ("VL_REC_BRT", "Valor")),
		( 4, ("VL_BC_CONT", "Valor")),
		( 5, ("VL_AJUS_ACRES_BC_COFINS", "Valor")),
		( 6, ("VL_AJUS_REDUC_BC_COFINS", "Valor")),
		( 7, ("VL_BC_CONT_AJUS", "Valor")),
		( 8, ("ALIQ_COFINS", "Aliquota")),
		( 9, ("QUANT_BC_COFINS", "C")),
		(10, ("ALIQ_COFINS_QUANT", "Aliquota")),
		(11, ("VL_CONT_APUR", "Valor")),
		(12, ("VL_AJUS_ACRES", "Valor")),
		(13, ("VL_AJUS_REDUC", "Valor")),
		(14, ("VL_CONT_DIFER", "Valor")),
		(15, ("VL_CONT_DIFER_ANT", "Valor")),
		(16, ("VL_CONT_PER", "Valor")),
	]);

	let registro_m610_antigo: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("COD_CONT", "C")),
		( 3, ("VL_REC_BRT", "Valor")),
		( 4, ("VL_BC_CONT", "Valor")),
		( 5, ("ALIQ_COFINS", "Aliquota")),
		( 6, ("QUANT_BC_COFINS", "C")),
		( 7, ("ALIQ_COFINS_QUANT", "Aliquota")),
		( 8, ("VL_CONT_APUR", "Valor")),
		( 9, ("VL_AJUS_ACRES", "Valor")),
		(10, ("VL_AJUS_REDUC", "Valor")),
		(11, ("VL_CONT_DIFER", "Valor")),
		(12, ("VL_CONT_DIFER_ANT", "Valor")),
		(13, ("VL_CONT_PER", "Valor")),
	]);

	let registro_m611: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("IND_TIP_COOP", "C")),
		( 3, ("VL_BC_CONT_ANT_EXC_COOP", "Valor")),
		( 4, ("VL_EXC_COOP_GER", "Valor")),
		( 5, ("VL_EXC_ESP_COOP", "Valor")),
		( 6, ("VL_BC_CONT", "Valor")),
	]);

	let registro_m615: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("IND_AJ_BC", "C")),
		( 3, ("VL_AJ_BC", "Valor")),
		( 4, ("COD_AJ_BC", "C")),
		( 5, ("NUM_DOC", "C")),
		( 6, ("DESCR_AJ_BC", "C")),
		( 7, ("DT_REF", "C")),
		( 8, ("COD_CTA", "C")),
		( 9, ("CNPJ", "C")),
		(10, ("INFO_COMPL", "C")),
	]);

	let registro_m620: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("IND_AJ", "C")),
		( 3, ("VL_AJ", "Valor")),
		( 4, ("COD_AJ", "C")),
		( 5, ("NUM_DOC", "C")),
		( 6, ("DESCR_AJ", "C")),
		( 7, ("DT_REF", "C")),
	]);

	let registro_m625: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("5", "C")),
		( 1, ("REG", "C")),
		( 2, ("DET_VALOR_AJ", "C")),
		( 3, ("CST_COFINS", "C")),
		( 4, ("DET_BC_CRED", "C")),
		( 5, ("DET_ALIQ", "C")),
		( 6, ("DT_OPER_AJ", "C")),
		( 7, ("DESC_AJ", "C")),
		( 8, ("COD_CTA", "C")),
		( 9, ("INFO_COMPL", "C")),
	]);

	let registro_m630: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("CNPJ", "C")),
		( 3, ("VL_VEND", "Valor")),
		( 4, ("VL_NAO_RECEB", "Valor")),
		( 5, ("VL_CONT_DIF", "Valor")),
		( 6, ("VL_CRED_DIF", "Valor")),
		( 7, ("COD_CRED", "C")),
	]);

	let registro_m700: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("2", "C")),
		( 1, ("REG", "C")),
		( 2, ("COD_CONT", "C")),
		( 3, ("VL_CONT_APUR_DIFER", "Valor")),
		( 4, ("NAT_CRED_DESC", "C")),
		( 5, ("VL_CRED_DESC_DIFER", "Valor")),
		( 6, ("VL_CONT_DIFER_ANT", "Valor")),
		( 7, ("PER_APUR", "C")),
		( 8, ("DT_RECEB", "C")),
	]);

	let registro_m800: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("2", "C")),
		( 1, ("REG", "C")),
		( 2, ("CST_COFINS", "C")),
		( 3, ("VL_TOT_REC", "Valor")),
		( 4, ("COD_CTA", "C")),
		( 5, ("DESC_COMPL", "C")),
	]);

	let registro_m810: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("NAT_REC", "C")),
		( 3, ("VL_REC", "Valor")),
		( 4, ("COD_CTA", "C")),
		( 5, ("DESC_COMPL", "C")),
	]);

	let registro_m990: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("1", "C")),
		( 1, ("REG", "C")),
		( 2, ("QTD_LIN_M", "C")),
	]);

	let registro_p001: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("1", "C")),
		( 1, ("REG", "C")),
		( 2, ("IND_MOV", "C")),
	]);

	let registro_p010: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("2", "C")),
		( 1, ("REG", "C")),
		( 2, ("CNPJ", "C")),
	]);

	let registro_p100: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("DT_INI", "C")),
		( 3, ("DT_FIN", "C")),
		( 4, ("VL_REC_TOT_EST", "Valor")),
		( 5, ("COD_ATIV_ECON", "C")),
		( 6, ("VL_REC_ATIV_ESTAB", "Valor")),
		( 7, ("VL_EXC", "Valor")),
		( 8, ("VL_BC_CONT", "Valor")),
		( 9, ("ALIQ_CONT", "Aliquota")),
		(10, ("VL_CONT_APU", "Valor")),
		(11, ("COD_CTA", "C")),
		(12, ("INFO_COMPL", "C")),
	]);

	let registro_p110: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("NUM_CAMPO", "C")),
		( 3, ("COD_DET", "C")),
		( 4, ("DET_VALOR", "C")),
		( 5, ("INF_COMPL", "C")),
	]);

	let registro_p199: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("NUM_PROC", "C")),
		( 3, ("IND_PROC", "C")),
	]);

	let registro_p200: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("2", "C")),
		( 1, ("REG", "C")),
		( 2, ("PER_REF", "C")),
		( 3, ("VL_TOT_CONT_APU", "Valor")),
		( 4, ("VL_TOT_AJ_REDUC", "Valor")),
		( 5, ("VL_TOT_AJ_ACRES", "Valor")),
		( 6, ("VL_TOT_CONT_DEV", "Valor")),
		( 7, ("COD_REC", "C")),
	]);

	let registro_p210: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("IND_AJ", "C")),
		( 3, ("VL_AJ", "Valor")),
		( 4, ("COD_AJ", "C")),
		( 5, ("NUM_DOC", "C")),
		( 6, ("DESCR_AJ", "C")),
		( 7, ("DT_REF", "C")),
	]);

	let registro_p990: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("1", "C")),
		( 1, ("REG", "C")),
		( 2, ("QTD_LIN_P", "C")),
	]);

	let registro_1001: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("1", "C")),
		( 1, ("REG", "C")),
		( 2, ("IND_MOV", "C")),
	]);

	let registro_1010: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("2", "C")),
		( 1, ("REG", "C")),
		( 2, ("NUM_PROC", "C")),
		( 3, ("ID_SEC_JUD", "C")),
		( 4, ("ID_VARA", "C")),
		( 5, ("IND_NAT_ACAO", "C")),
		( 6, ("DESC_DEC_JUD", "C")),
		( 7, ("DT_SENT_JUD", "C")),
	]);

	let registro_1011: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("REG_REF", "C")),
		( 3, ("CHAVE_DOC", "C")),
		( 4, ("COD_PART", "C")),
		( 5, ("COD_ITEM", "C")),
		( 6, ("DT_OPER", "C")),
		( 7, ("VL_OPER", "Valor")),
		( 8, ("CST_PIS", "C")),
		( 9, ("VL_BC_PIS", "Valor")),
		(10, ("ALIQ_PIS", "Aliquota")),
		(11, ("VL_PIS", "Valor")),
		(12, ("CST_COFINS", "C")),
		(13, ("VL_BC_COFINS", "Valor")),
		(14, ("ALIQ_COFINS", "Aliquota")),
		(15, ("VL_COFINS", "Valor")),
		(16, ("CST_PIS_SUSP", "C")),
		(17, ("VL_BC_PIS_SUSP", "Valor")),
		(18, ("ALIQ_PIS_SUSP", "Aliquota")),
		(19, ("VL_PIS_SUSP", "Valor")),
		(20, ("CST_COFINS_SUSP", "C")),
		(21, ("VL_BC_COFINS_SUSP", "Valor")),
		(22, ("ALIQ_COFINS_SUSP", "Aliquota")),
		(23, ("VL_COFINS_SUSP", "Valor")),
		(24, ("COD_CTA", "C")),
		(25, ("COD_CCUS", "C")),
		(26, ("DESC_DOC_OPER", "C")),
	]);

	let registro_1020: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("2", "C")),
		( 1, ("REG", "C")),
		( 2, ("NUM_PROC", "C")),
		( 3, ("IND_NAT_ACAO", "C")),
		( 4, ("DT_DEC_ADM", "C")),
	]);

	let registro_1050: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("2", "C")),
		( 1, ("REG", "C")),
		( 2, ("DT_REF", "C")),
		( 3, ("IND_AJ_BC", "C")),
		( 4, ("CNPJ", "C")),
		( 5, ("VL_AJ_TOT", "Valor")),
		( 6, ("VL_AJ_CST01", "Valor")),
		( 7, ("VL_AJ_CST02", "Valor")),
		( 8, ("VL_AJ_CST03", "Valor")),
		( 9, ("VL_AJ_CST04", "Valor")),
		(10, ("VL_AJ_CST05", "Valor")),
		(11, ("VL_AJ_CST06", "Valor")),
		(12, ("VL_AJ_CST07", "Valor")),
		(13, ("VL_AJ_CST08", "Valor")),
		(14, ("VL_AJ_CST09", "Valor")),
		(15, ("VL_AJ_CST49", "Valor")),
		(16, ("VL_AJ_CST99", "Valor")),
		(17, ("IND_APROP", "C")),
		(18, ("NUM_REC", "C")),
		(19, ("INFO_COMPL", "C")),
	]);

	let registro_1100: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("2", "C")),
		( 1, ("REG", "C")),
		( 2, ("PER_APU_CRED", "C")),
		( 3, ("ORIG_CRED", "C")),
		( 4, ("CNPJ_SUC", "C")),
		( 5, ("COD_CRED", "C")),
		( 6, ("VL_CRED_APU", "Valor")),
		( 7, ("VL_CRED_EXT_APU", "Valor")),
		( 8, ("VL_TOT_CRED_APU", "Valor")),
		( 9, ("VL_CRED_DESC_PA_ANT", "Valor")),
		(10, ("VL_CRED_PER_PA_ANT", "Valor")),
		(11, ("VL_CRED_DCOMP_PA_ANT", "Valor")),
		(12, ("SD_CRED_DISP_EFD", "C")),
		(13, ("VL_CRED_DESC_EFD", "Valor")),
		(14, ("VL_CRED_PER_EFD", "Valor")),
		(15, ("VL_CRED_DCOMP_EFD", "Valor")),
		(16, ("VL_CRED_TRANS", "Valor")),
		(17, ("VL_CRED_OUT", "Valor")),
		(18, ("SLD_CRED_FIM", "Valor")),
	]);

	let registro_1101: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("COD_PART", "C")),
		( 3, ("COD_ITEM", "C")),
		( 4, ("COD_MOD", "C")),
		( 5, ("SER", "C")),
		( 6, ("SUB_SER", "C")),
		( 7, ("NUM_DOC", "C")),
		( 8, ("DT_OPER", "C")),
		( 9, ("CHV_NFE", "C")),
		(10, ("VL_OPER", "Valor")),
		(11, ("CFOP", "C")),
		(12, ("NAT_BC_CRED", "C")),
		(13, ("IND_ORIG_CRED", "C")),
		(14, ("CST_PIS", "C")),
		(15, ("VL_BC_PIS", "Valor")),
		(16, ("ALIQ_PIS", "Aliquota")),
		(17, ("VL_PIS", "Valor")),
		(18, ("COD_CTA", "C")),
		(19, ("COD_CCUS", "C")),
		(20, ("DESC_COMPL", "C")),
		(21, ("PER_ESCRIT", "C")),
		(22, ("CNPJ", "C")),
	]);

	let registro_1102: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("VL_CRED_PIS_TRIB_MI", "Valor")),
		( 3, ("VL_CRED_PIS_NT_MI", "Valor")),
		( 4, ("VL_CRED_PIS_EXP", "Valor")),
	]);

	let registro_1200: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("2", "C")),
		( 1, ("REG", "C")),
		( 2, ("PER_APUR_ANT", "C")),
		( 3, ("NAT_CONT_REC", "C")),
		( 4, ("VL_CONT_APUR", "Valor")),
		( 5, ("VL_CRED_PIS_DESC", "Valor")),
		( 6, ("VL_CONT_DEV", "Valor")),
		( 7, ("VL_OUT_DED", "Valor")),
		( 8, ("VL_CONT_EXT", "Valor")),
		( 9, ("VL_MUL", "Valor")),
		(10, ("VL_JUR", "Valor")),
		(11, ("DT_RECOL", "C")),
	]);

	let registro_1210: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("CNPJ", "C")),
		( 3, ("CST_PIS", "C")),
		( 4, ("COD_PART", "C")),
		( 5, ("DT_OPER", "C")),
		( 6, ("VL_OPER", "Valor")),
		( 7, ("VL_BC_PIS", "Valor")),
		( 8, ("ALIQ_PIS", "Aliquota")),
		( 9, ("VL_PIS", "Valor")),
		(10, ("COD_CTA", "C")),
		(11, ("DESC_COMPL", "C")),
	]);

	let registro_1220: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("PER_APU_CRED", "C")),
		( 3, ("ORIG_CRED", "C")),
		( 4, ("COD_CRED", "C")),
		( 5, ("VL_CRED", "Valor")),
	]);

	let registro_1300: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("2", "C")),
		( 1, ("REG", "C")),
		( 2, ("IND_NAT_RET", "C")),
		( 3, ("PR_REC_RET", "C")),
		( 4, ("VL_RET_APU", "Valor")),
		( 5, ("VL_RET_DED", "Valor")),
		( 6, ("VL_RET_PER", "Valor")),
		( 7, ("VL_RET_DCOMP", "Valor")),
		( 8, ("SLD_RET", "Valor")),
	]);

	let registro_1500: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("2", "C")),
		( 1, ("REG", "C")),
		( 2, ("PER_APU_CRED", "C")),
		( 3, ("ORIG_CRED", "C")),
		( 4, ("CNPJ_SUC", "C")),
		( 5, ("COD_CRED", "C")),
		( 6, ("VL_CRED_APU", "Valor")),
		( 7, ("VL_CRED_EXT_APU", "Valor")),
		( 8, ("VL_TOT_CRED_APU", "Valor")),
		( 9, ("VL_CRED_DESC_PA_ANT", "Valor")),
		(10, ("VL_CRED_PER_PA_ANT", "Valor")),
		(11, ("VL_CRED_DCOMP_PA_ANT", "Valor")),
		(12, ("SD_CRED_DISP_EFD", "C")),
		(13, ("VL_CRED_DESC_EFD", "Valor")),
		(14, ("VL_CRED_PER_EFD", "Valor")),
		(15, ("VL_CRED_DCOMP_EFD", "Valor")),
		(16, ("VL_CRED_TRANS", "Valor")),
		(17, ("VL_CRED_OUT", "Valor")),
		(18, ("SLD_CRED_FIM", "Valor")),
	]);

	let registro_1501: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("COD_PART", "C")),
		( 3, ("COD_ITEM", "C")),
		( 4, ("COD_MOD", "C")),
		( 5, ("SER", "C")),
		( 6, ("SUB_SER", "C")),
		( 7, ("NUM_DOC", "C")),
		( 8, ("DT_OPER", "C")),
		( 9, ("CHV_NFE", "C")),
		(10, ("VL_OPER", "Valor")),
		(11, ("CFOP", "C")),
		(12, ("NAT_BC_CRED", "C")),
		(13, ("IND_ORIG_CRED", "C")),
		(14, ("CST_COFINS", "C")),
		(15, ("VL_BC_COFINS", "Valor")),
		(16, ("ALIQ_COFINS", "Aliquota")),
		(17, ("VL_COFINS", "Valor")),
		(18, ("COD_CTA", "C")),
		(19, ("COD_CCUS", "C")),
		(20, ("DESC_COMPL", "C")),
		(21, ("PER_ESCRIT", "C")),
		(22, ("CNPJ", "C")),
	]);

	let registro_1502: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("4", "C")),
		( 1, ("REG", "C")),
		( 2, ("VL_CRED_COFINS_TRIB_MI", "Valor")),
		( 3, ("VL_CRED_COFINS_NT_MI", "Valor")),
		( 4, ("VL_CRED_COFINS_EXP", "Valor")),
	]);

	let registro_1600: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("2", "C")),
		( 1, ("REG", "C")),
		( 2, ("PER_APUR_ANT", "C")),
		( 3, ("NAT_CONT_REC", "C")),
		( 4, ("VL_CONT_APUR", "Valor")),
		( 5, ("VL_CRED_COFINS_DESC", "Valor")),
		( 6, ("VL_CONT_DEV", "Valor")),
		( 7, ("VL_OUT_DED", "Valor")),
		( 8, ("VL_CONT_EXT", "Valor")),
		( 9, ("VL_MUL", "Valor")),
		(10, ("VL_JUR", "Valor")),
		(11, ("DT_RECOL", "C")),
	]);

	let registro_1610: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("CNPJ", "C")),
		( 3, ("CST_COFINS", "C")),
		( 4, ("COD_PART", "C")),
		( 5, ("DT_OPER", "C")),
		( 6, ("VL_OPER", "Valor")),
		( 7, ("VL_BC_COFINS", "Valor")),
		( 8, ("ALIQ_COFINS", "Aliquota")),
		( 9, ("VL_COFINS", "Valor")),
		(10, ("COD_CTA", "C")),
		(11, ("DESC_COMPL", "C")),
	]);

	let registro_1620: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("PER_APU_CRED", "C")),
		( 3, ("ORIG_CRED", "C")),
		( 4, ("COD_CRED", "C")),
		( 5, ("VL_CRED", "Valor")),
	]);

	let registro_1700: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("2", "C")),
		( 1, ("REG", "C")),
		( 2, ("IND_NAT_RET", "C")),
		( 3, ("PR_REC_RET", "C")),
		( 4, ("VL_RET_APU", "Valor")),
		( 5, ("VL_RET_DED", "Valor")),
		( 6, ("VL_RET_PER", "Valor")),
		( 7, ("VL_RET_DCOMP", "Valor")),
		( 8, ("SLD_RET", "Valor")),
	]);

	let registro_1800: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("2", "C")),
		( 1, ("REG", "C")),
		( 2, ("INC_IMOB", "C")),
		( 3, ("REC_RECEB_RET", "Valor")),
		( 4, ("REC_FIN_RET", "Valor")),
		( 5, ("BC_RET", "C")),
		( 6, ("ALIQ_RET", "Aliquota")),
		( 7, ("VL_REC_UNI", "Valor")),
		( 8, ("DT_REC_UNI", "C")),
		( 9, ("COD_REC", "C")),
	]);

	let registro_1809: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("3", "C")),
		( 1, ("REG", "C")),
		( 2, ("NUM_PROC", "C")),
		( 3, ("IND_PROC", "C")),
	]);

	let registro_1900: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("2", "C")),
		( 1, ("REG", "C")),
		( 2, ("CNPJ", "C")),
		( 3, ("COD_MOD", "C")),
		( 4, ("SER", "C")),
		( 5, ("SUB_SER", "C")),
		( 6, ("COD_SIT", "C")),
		( 7, ("VL_TOT_REC", "Valor")),
		( 8, ("QUANT_DOC", "C")),
		( 9, ("CST_PIS", "C")),
		(10, ("CST_COFINS", "C")),
		(11, ("CFOP", "C")),
		(12, ("INF_COMPL", "C")),
		(13, ("COD_CTA", "C")),
	]);

	let registro_1990: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("1", "C")),
		( 1, ("REG", "C")),
		( 2, ("QTD_LIN_1", "C")),
	]);

	let registro_9001: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("1", "C")),
		( 1, ("REG", "C")),
		( 2, ("IND_MOV", "C")),
	]);

	let registro_9900: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("2", "C")),
		( 1, ("REG", "C")),
		( 2, ("REG_BLC", "C")),
		( 3, ("QTD_REG_BLC", "C")),
	]);

	let registro_9990: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("1", "C")),
		( 1, ("REG", "C")),
		( 2, ("QTD_LIN_9", "C")),
	]);

	let registro_9999: HashMap<u16, (&str, &str)> = HashMap::from([
		( 0, ("0", "C")),
		( 1, ("REG", "C")),
		( 2, ("QTD_LIN", "C")),
	]);

	// Adicionar todos os registros em efd_blocos:
	let efd_blocos: HashMap::<&'static str, HashMap<u16, (&'static str, &'static str)>> = HashMap::from([

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
		let mut index    = 5;
		let (reg_0000_05, _tipo) = registros_efd[registro][&index];

		registro = "C170";
		index    = 37;
		let (reg_c170_37, _tipo) = registros_efd[registro][&index];

		registro = "1610";
		index    = 9;
		let (reg_1610_09, _tipo) = registros_efd[registro][&index];

		println!("reg_0000_05: {reg_0000_05}");
		println!("reg_C170_37: {reg_c170_37}");
		println!("reg_1610_09: {reg_1610_09}");

		assert_eq!(reg_0000_05, "NUM_REC_ANTERIOR");
		assert_eq!(reg_c170_37, "COD_CTA");
		assert_eq!(reg_1610_09, "VL_COFINS");
	}
}
