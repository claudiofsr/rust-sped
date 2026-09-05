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
