# rust-sped
Descrição: Este programa analisa informações contidas em arquivos de SPED EFD Contribuições

Para clonar/copiar o projeto:

	git clone https://github.com/claudiofsr/rust-sped

Cloning into 'rust-sped'...

Para compilar e gerar o executável:

	cd rust-sped

	cargo build --release

Caso não tenha feito antes, não se esqueça de incluir rust_bin em PATH.
No Linux, adicione no arquivo ~/.bashrc ou ~/.zshrc:

	rust_bin="$HOME/.cargo/bin"

	if [ -d $rust_bin ] ; then
		export PATH="$rust_bin:$PATH"
	fi

Para instalar o programa em "$HOME/.cargo/bin":

	cargo install --path=.

Em seguida, execute o programa em um diretório contendo arquivos de SPED EFD Contribuições:

	efd_contribuicoes -f
	efd_contribuicoes -r 1 -p

Note que -p é opcional para imprimir o arquivo de formato .csv.

Observe o resustado no diretório novo:

	ls novo

<!-- fix.py:stream -->
## Modo streaming (milhares de arquivos grandes)

```console
# 4 arquivos por vez, CSV por arquivo (-p), CSV final único (--concat-csv)
efd_contribuicoes -s -r 1 50000 --max-ram-gb 60 -j 4 -p --concat-csv --spill-dir /dados/spill
```

Cada arquivo é analisado, gera um checkpoint pequeno (`.agg`) e, com `-p`, um CSV
de detalhe; em seguida é descartado da memória. No final, os checkpoints são
fundidos **por CNPJ base** (a consolidação são somas por chave) e geram as tabelas
e o Excel de cada CNPJ em `novo/<cnpj>/`.

* **Retomável:** rode o mesmo comando de novo; arquivos já processados são pulados.
  `--fresh` descarta os checkpoints.
* **Spill em disco:** use `--spill-dir` em disco com espaço (não `/tmp`, que costuma
  ser tmpfs, isto é, RAM).
* **Detalhe:** os CSV por arquivo podem ser lidos juntos por DuckDB/polars
  (`read_csv('spill/*.csv')`). O Excel traz só as abas de consolidação.
* **CSV final (`--concat-csv`, com `-p`):** gravado em `novo/` com o mesmo nome do
  modo global e **idêntico byte a byte** a ele (mesma ordem, mesmo formato e
  numeração global da coluna `Linhas`). Os CSV por arquivo no spill mantêm a
  numeração local. <!-- fix.py:csv-identico -->
* **Diferenças em relação ao modo global:** sem `--global-excel`, a consolidação é
  por CNPJ (as somas trimestrais dependem do período de cada CNPJ); as tabelas
  por arquivo não são impressas; o Excel traz só as abas de consolidação.

<!-- fix.py:ram-auto -->
### RAM automática

Sem `--max-ram-gb`, o orçamento de RAM é a **RAM disponível** detectada no início
(via `sysinfo`: Linux, macOS, Windows, FreeBSD...; em contêiner Linux respeita o
limite do cgroup). Se a detecção falhar, usa 16 GiB e avisa. Com `--max-ram-gb`,
vale o valor informado (aviso se exceder a RAM total). Veja o orçamento usado com
`RUST_LOG=info`. O `--mem-factor` continua sendo uma estimativa: calibre com
`/usr/bin/time -v`.

<!-- fix.py:global-excel -->
### Excel global (`--global-excel`)

```console
efd_contribuicoes -s --global-excel -r 1 50000 --max-ram-gb 60 --spill-dir /dados/spill
```

Funde **todos** os checkpoints e finaliza uma vez só, gerando um único Excel em
`novo/` com o mesmo resultado do modo global original. Antes de carregar, o
programa estima a RAM pelo tamanho dos `.agg` (heurística: 4x) e **aborta** se
exceder `--max-ram-gb`; os checkpoints são mantidos, então basta rodar sem
`--global-excel` (um Excel por CNPJ) ou aumentar o orçamento. O Excel traz só as
abas de consolidação.
