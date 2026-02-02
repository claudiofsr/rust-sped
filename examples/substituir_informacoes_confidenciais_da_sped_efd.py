#!/usr/bin/env python
# -*- coding: utf-8 -*-

# cargo test -- --show-output test_executar_programa

# Substituir informações de CPF/CNPJ e Chave de 44 digitos por números aleatórios
from random import randint, choice
import re

def random_with_N_digits(n):
    range_start = 10 ** (n - 1)
    range_end = (10**n) - 1
    return randint(range_start, range_end)

def CNPJ_corrige_digito_verificador(valor):
    # valor = '12.345.678/9012-30'
    # remover os caracteres não dígitos (\D)
    valor = re.sub(r"\D", "", str(valor))

    if not re.search(r"^\d{14}$", str(valor)):
        return valor

    multiplicadores = [6, 5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2]

    cnpj = [int(c) for c in valor]

    soma1 = sum([cnpj[i] * multiplicadores[i + 1] for i in range(12)])

    digito1 = 11 - (soma1 % 11)
    if digito1 >= 10:
        digito1 = 0

    cnpj[12] = digito1

    soma2 = sum([cnpj[i] * multiplicadores[i] for i in range(13)])

    digito2 = 11 - (soma2 % 11)
    if digito2 >= 10:
        digito2 = 0

    cnpj[13] = digito2

    cnpj = [str(c) for c in cnpj]

    valor = "".join(cnpj)

    return valor

def Substituir_CNPJ(cnpj_ficticio, cnpj_origi, cnpj_atual):
    # remover os caracteres não dígitos (\D)
    cnpj_origi = re.sub(r"\D", "", str(cnpj_origi))
    cnpj_atual = re.sub(r"\D", "", str(cnpj_atual))

    if not re.search(r"^\d{14}$", str(cnpj_origi)) or \
       not re.search(r"^\d{14}$", str(cnpj_atual)):
        return cnpj_ficticio

    # Get the first N digits of a number:
    cnpj_origi_first8 = int(str(cnpj_origi)[:8])
    cnpj_atual_first8 = int(str(cnpj_atual)[:8])

    if cnpj_origi_first8 == cnpj_atual_first8:
        return cnpj_ficticio
    else:
        return CNPJ_corrige_digito_verificador(random_with_N_digits(14))

def CPF_corrige_digito_verificador(valor):
    # valor = '333.333.333-33'
    # remover os caracteres não dígitos (\D)
    valor = re.sub(r"\D", "", str(valor))

    if not re.search(r"^\d{11}$", str(valor)):
        return valor

    multiplicadores = [11, 10, 9, 8, 7, 6, 5, 4, 3, 2]

    cpf = [int(c) for c in valor]

    soma1 = sum([cpf[i] * multiplicadores[i + 1] for i in range(9)])

    digito1 = 11 - (soma1 % 11)
    if digito1 >= 10:
        digito1 = 0

    cpf[9] = digito1

    soma2 = sum([cpf[i] * multiplicadores[i] for i in range(10)])

    digito2 = 11 - (soma2 % 11)
    if digito2 >= 10:
        digito2 = 0

    cpf[10] = digito2

    cpf = [str(c) for c in cpf]

    valor = "".join(cpf)

    return valor

def NFe_corrige_digito_verificador(valor, verbose=False):
    # remover os caracteres não dígitos (\D)
    valor = re.sub(r"\D", "", str(valor))

    if not re.search(r"^\d{44}$", str(valor)):
        return valor

    # dentro da chave eletrônica há o CNPJ do emitente
    # que também será verificado
    cnpj_original = str(valor)[6:20]
    cnpj_corrigido = CNPJ_corrige_digito_verificador(cnpj_original)
    valor_corrigido = valor[:6] + cnpj_corrigido + valor[20:]

    if verbose:
        print(f"valor original : {valor}")
        print(f"cnpj  original : {cnpj_original:>20}")
        print(f"cnpj  corrigido: {cnpj_corrigido:>20}")
        print(f"valor corrigido: {valor_corrigido}\n")

    chave = [int(digito) for digito in valor_corrigido]
    multiplicadores = [4, 3, 2] + [9, 8, 7, 6, 5, 4, 3, 2] * 5 + [0]

    soma = sum([chave[i] * multiplicadores[i] for i in range(44)])

    resto_da_divisao = soma % 11
    digito_verificador = 11 - resto_da_divisao

    if digito_verificador >= 10:
        digito_verificador = 0

    chave[-1] = digito_verificador

    chave = [str(c) for c in chave]

    valor = "".join(chave)

    return valor

if __name__ == "__main__":
    # Read a file line by line in Python
    arquivo_de_entrada = "efd_data_random"
    encode_info = ["WINDOWS_1252", "latin-1", "UTF-8"]
    count = 0
    verbose = False

    # Writing to a file
    arquivo_de_saida = open(
        "efd_data_new",
        mode="w",
        encoding=encode_info[0],
        errors="ignore",
    )

    cnpj_ficticio = '12345678901230' # código fictício qualquer

    # D101 e D105 estão correlacionados.
    # Assim, os campos devem ser compatíveis
    # Iniciar variáveis
    natureza_d101 = randint(1, 18)
    valor_item_d101 = random_with_N_digits(5)
    valor_bc_contrib_d101 = random_with_N_digits(4)
    aliquotas_d101 = choice([(0.8250, 3.8000), (1.65, 7.60), (2.10, 9.65)])

    # Ler linha a linha do arquivo original arquivo_de_entrada.
    # Em seguida, realizar substituições com origem em números aleatórios.
    with open(arquivo_de_entrada, mode="r", encoding=encode_info[0], errors="ignore") as filename:
        for line in filename:
            campos = line.strip().split("|")

            if len(campos) < 3:
                continue

            campos_novos = []

            for campo in campos:
                campo.strip()
                match_NFe = re.search(r"^\s*\d{44}\s*$", campo)
                if match_NFe:
                    digit = str(random_with_N_digits(44))
                    campo = NFe_corrige_digito_verificador(digit)
                campos_novos.append(campo)

            registro = campos[1]

            if registro == "0000":
                # Alterar dados originais por dados aleatórios
                # Nome
                campos_novos[8] = f"Empresa Fictícia {random_with_N_digits(5)} SA"
                # CNPJ
                cnpj_original = campos_novos[9]
                campos_novos[9] = cnpj_ficticio
                # Código do Município
                campos_novos[11] = str(random_with_N_digits(7))
                # Inscrição SUFRAMA
                campos_novos[12] = str(random_with_N_digits(9))

            if registro == "0100":
                # Alterar dados originais por dados aleatórios
                # Nome do Contabilista
                campos_novos[2] = f"Contador {random_with_N_digits(5)}"
                # CPF
                campos_novos[3] = CPF_corrige_digito_verificador(random_with_N_digits(11))
                # CRC
                campos_novos[4] = str(random_with_N_digits(8))
                # CNPJ
                campos_novos[5] = Substituir_CNPJ(cnpj_ficticio, cnpj_original, campos[5])
                # CEP
                campos_novos[6] = str(random_with_N_digits(8))
                # Endereço Rua
                campos_novos[7] = f"Rua Fictícia {random_with_N_digits(5)}"
                # Endereço Complemento
                campos_novos[9] = f"Andar {random_with_N_digits(3)}"
                # Endereço Bairro
                campos_novos[10] = f"Bairro Fictício {random_with_N_digits(5)}"
                # FONE
                campos_novos[11] = str(random_with_N_digits(11))
                # FAX
                campos_novos[12] = str(random_with_N_digits(11))
                # Email
                campos_novos[13] = f"email{random_with_N_digits(5)}@testXXX.org.br"
                # Código do Município
                campos_novos[14] = str(random_with_N_digits(7))

            if registro == "0111":
                # Alterar dados originais por dados aleatórios
                # REC_BRU_NCUM_TRIB_MI
                campos_novos[2] = str(random_with_N_digits(5))
                # REC_BRU_NCUM_NT_MI
                campos_novos[3] = str(random_with_N_digits(5))
                # REC_BRU_NCUM_EXP
                campos_novos[4] = str(random_with_N_digits(5))
                # REC_BRU_CUM
                campos_novos[5] = str(random_with_N_digits(5))
                # REC_BRU_TOTAL
                rec_bru_ncum  = int(campos_novos[2]) + int(campos_novos[3]) + int(campos_novos[4])
                rec_bru_total = rec_bru_ncum + int(campos_novos[5])
                campos_novos[6] = str(rec_bru_total)

            if registro == "0140":
                # Alterar dados originais por dados aleatórios
                # Código do Estabelecimento
                campos_novos[2] = str(random_with_N_digits(10))
                # Nome
                campos_novos[3] = f"Empresa Fictícia {random_with_N_digits(5)} SA"
                # CNPJ
                campos_novos[4] = Substituir_CNPJ(cnpj_ficticio, cnpj_original, campos[4])
                # Inscrição Estadual
                campos_novos[6] = str(random_with_N_digits(6))
                # Código do Município
                campos_novos[7] = str(random_with_N_digits(7))
                # IM
                campos_novos[8] = str(random_with_N_digits(8))
                # Inscrição SUFRAMA
                campos_novos[9] = str(random_with_N_digits(9))

            if registro == "0150":
                # Alterar dados originais por dados aleatórios
                # Nome
                campos_novos[3] = f"Empresa Fictícia {random_with_N_digits(5)} LTDA"
                # CNPJ
                campos_novos[5] = Substituir_CNPJ(cnpj_ficticio, cnpj_original, campos[5])
                # CPF
                campos_novos[6] = CPF_corrige_digito_verificador(random_with_N_digits(11))
                # Inscrição Estadual
                campos_novos[7] = str(random_with_N_digits(6))
                # Código do Município
                campos_novos[8] = str(random_with_N_digits(7))
                # Inscrição SUFRAMA
                campos_novos[9] = str(random_with_N_digits(9))
                # Endereço Rua
                campos_novos[10] = f"Rua Fictícia {random_with_N_digits(5)}"
                # Endereço Número
                campos_novos[11] = f"nº {random_with_N_digits(4)}"
                # Endereço Complemento
                campos_novos[12] = f"Andar {random_with_N_digits(3)}"
                # Endereço Bairro
                campos_novos[13] = f"Bairro Fictício {random_with_N_digits(5)}"

            if registro == "0200":
                # Alterar dados originais por dados aleatórios
                # DESCR_ITEM
                campos_novos[3] = f"Descrição {random_with_N_digits(6)}"
                # COD_BARRA
                campos_novos[4] = f"Código aleatório {random_with_N_digits(5)}"
                # TIPO_ITEM
                campos_novos[7] = f"0{random_with_N_digits(1)}"
                # COD_NCM
                campos_novos[8] = str(random_with_N_digits(8))

            if registro == "0400":
                # Alterar dados originais por dados aleatórios
                # Descrição da natureza da operação/prestação
                campos_novos[3] = f"Operação {random_with_N_digits(5)}"

            if registro == "0450":
                # Alterar dados originais por dados aleatórios
                # Texto livre da informação complementar existente
                campos_novos[3] = f"Produto XXX {random_with_N_digits(5)} Aleatório"

            if registro == "0500":
                # Alterar dados originais por dados aleatórios
                # Nome da conta analítica/grupo de contas.
                campos_novos[7] = f"Conta ABC {random_with_N_digits(6)}"

            if registro == "0600":
                # Alterar dados originais por dados aleatórios
                # Nome do centro de custos.
                campos_novos[4] = f"Centro de custos {random_with_N_digits(6)}"

            if registro == "A010" or registro == "C010" or \
               registro == "D010" or registro == "F010" or \
               registro == "I010":
                # Alterar dados originais por dados aleatórios
                # CNPJ
                campos_novos[2] = Substituir_CNPJ(cnpj_ficticio, cnpj_original, campos[2])

            if registro == "A100":
                # Alterar dados originais por dados aleatórios
                # NUM_DOC
                campos_novos[8] = str(random_with_N_digits(6))
                # VL_DOC
                campos_novos[12] = str(random_with_N_digits(5))
                valor_bc_contrib = random_with_N_digits(4)
                # VL_BC_PIS
                campos_novos[15] = str(valor_bc_contrib)
                # VL_PIS
                campos_novos[16] = str(int(valor_bc_contrib * 1.65/100)).replace('.', ',')
                # VL_BC_COFINS
                campos_novos[17] = str(valor_bc_contrib)
                # VL_COFINS
                campos_novos[18] = str(int(valor_bc_contrib * 7.60/100)).replace('.', ',')

            if registro == "A170":
                # Alterar dados originais por dados aleatórios
                # VL_ITEM
                campos_novos[5] = str(random_with_N_digits(5))
                # NAT_BC_CRED
                natureza = randint(1, 18)
                campos_novos[7] = f"{natureza:02}"
                valor_bc_contrib = random_with_N_digits(4)
                # VL_BC_PIS
                campos_novos[10] = str(valor_bc_contrib)
                aliquotas = choice([(0.8250, 3.8000), (1.65, 7.60), (2.10, 9.65)])
                # ALIQ_PIS
                aliq_pis = aliquotas[0]
                campos_novos[11] = str(aliq_pis).replace('.', ',')
                # VL_PIS
                campos_novos[12] = str(int(valor_bc_contrib * aliq_pis/100)).replace('.', ',')
                # ALIQ_COFINS
                aliq_cofins = aliquotas[1]
                campos_novos[15] = str(aliq_cofins).replace('.', ',')
                # VL_BC_COFINS
                campos_novos[14] = str(valor_bc_contrib)
                # VL_COFINS
                campos_novos[16] = str(int(valor_bc_contrib * aliq_cofins/100)).replace('.', ',')

            if registro == "C100":
                # Alterar dados originais por dados aleatórios
                # NUM_DOC
                campos_novos[8] = str(random_with_N_digits(6))

            if registro == "C170":
                # Alterar dados originais por dados aleatórios
                # DESCR_COMPL
                campos_novos[4] = f"Descrição do item {random_with_N_digits(4)}"
                # VL_ITEM
                campos_novos[7] = str(random_with_N_digits(5))
                valor_bc_contrib = random_with_N_digits(4)
                # VL_BC_PIS
                campos_novos[26] = str(valor_bc_contrib)
                aliquotas = choice([(0.8250, 3.8000), (1.65, 7.60), (2.10, 9.65)])
                # ALIQ_PIS
                aliq_pis = aliquotas[0]
                campos_novos[27] = str(aliq_pis).replace('.', ',')
                # VL_PIS
                campos_novos[30] = str(int(valor_bc_contrib * aliq_pis/100)).replace('.', ',')
                # VL_BC_COFINS
                campos_novos[32] = str(valor_bc_contrib)
                # ALIQ_COFINS
                aliq_cofins = aliquotas[1]
                campos_novos[33] = str(aliq_cofins).replace('.', ',')
                # VL_COFINS
                campos_novos[36] = str(int(valor_bc_contrib * aliq_cofins/100)).replace('.', ',')

            if registro == "C180" or registro == "C190":
                # Alterar dados originais por dados aleatórios
                # COD_NCM
                campos_novos[6] = str(random_with_N_digits(8))

            if registro == "D101":
                # Alterar dados originais por dados aleatórios
                # VL_ITEM
                valor_item_d101 = random_with_N_digits(5)
                campos_novos[3] = str(valor_item_d101)
                # NAT_BC_CRED
                natureza_d101 = randint(1, 18)
                campos_novos[5] = f"{natureza_d101:02}"
                valor_bc_contrib_d101 = random_with_N_digits(4)
                # VL_BC_PIS
                campos_novos[6] = str(valor_bc_contrib_d101)
                aliquotas_d101 = choice([(0.8250, 3.8000), (1.65, 7.60), (2.10, 9.65)])
                # ALIQ_PIS
                aliq_pis = aliquotas_d101[0]
                campos_novos[7] = str(aliq_pis).replace('.', ',')
                # VL_PIS
                campos_novos[8] = str(int(valor_bc_contrib * aliq_pis/100)).replace('.', ',')

            if registro == "D105":
                # Alterar dados originais por dados aleatórios
                # VL_ITEM
                campos_novos[3] = str(valor_item_d101)
                # NAT_BC_CRED
                campos_novos[5] = f"{natureza_d101:02}"
                # VL_BC_COFINS
                campos_novos[6] = str(valor_bc_contrib_d101)
                # ALIQ_COFINS
                aliq_cofins = aliquotas_d101[1]
                campos_novos[7] = str(aliq_cofins).replace('.', ',')
                # VL_COFINS
                campos_novos[8] = str(int(valor_bc_contrib * aliq_cofins/100)).replace('.', ',')

            if registro == "F100":
                # Alterar dados originais por dados aleatórios
                # VL_OPER
                campos_novos[6] = str(random_with_N_digits(5))
                valor_bc_contrib = random_with_N_digits(4)
                # VL_BC_PIS
                campos_novos[8] = str(valor_bc_contrib)
                aliquotas = choice([(0.8250, 3.8000), (1.65, 7.60), (2.10, 9.65)])
                # ALIQ_PIS
                aliq_pis = aliquotas[0]
                campos_novos[9] = str(aliq_pis).replace('.', ',')
                # VL_PIS
                campos_novos[10] = str(int(valor_bc_contrib * aliq_pis/100)).replace('.', ',')
                # VL_BC_COFINS
                campos_novos[12] = str(valor_bc_contrib)
                # ALIQ_COFINS
                aliq_cofins = aliquotas[1]
                campos_novos[13] = str(aliq_cofins).replace('.', ',')
                # VL_COFINS
                campos_novos[14] = str(int(valor_bc_contrib * aliq_cofins/100)).replace('.', ',')
                # NAT_BC_CRED
                natureza = randint(1, 18)
                campos_novos[15] = f"{natureza:02}"
                # DESC_DOC_OPER
                campos_novos[19] = f"Descrição do documento {random_with_N_digits(4)}"

            linha_alterada = "|".join(campos_novos)
            arquivo_de_saida.writelines(f"{linha_alterada}\n")
            if verbose:
                print(f"count {count:>4} ; 1 campos: {campos}")
                print(f"count {count:>4} ; 2 campos: {campos_novos}\n")
                if count > 20:
                    break
            count += 1

    arquivo_de_saida.close()
