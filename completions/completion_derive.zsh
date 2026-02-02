#compdef efd_contribuicoes

autoload -U is-at-least

_efd_contribuicoes() {
    typeset -A opt_args
    typeset -a _arguments_options
    local ret=1

    if is-at-least 5.2; then
        _arguments_options=(-s -S -C)
    else
        _arguments_options=(-s -C)
    fi

    local context curcontext="$curcontext" state line
    _arguments "${_arguments_options[@]}" : \
'-g+[Gera o arquivo de auto-complete para o shell especificado (bash, zsh, fish, etc)]:GENERATOR:(bash elvish fish powershell zsh)' \
'--generate=[Gera o arquivo de auto-complete para o shell especificado (bash, zsh, fish, etc)]:GENERATOR:(bash elvish fish powershell zsh)' \
'*-r+[Selecione arquivos SPED EFD para analisar especificando o intervalo]:RANGE:_default' \
'*--range=[Selecione arquivos SPED EFD para analisar especificando o intervalo]:RANGE:_default' \
'-c[Limpa a tela do terminal antes de apresentar a análise]' \
'--clear_terminal[Limpa a tela do terminal antes de apresentar a análise]' \
'-d[Ativar mensagens de debug (ex\: detalhes de correlações do Bloco M)]' \
'--debug[Ativar mensagens de debug (ex\: detalhes de correlações do Bloco M)]' \
'-e[Exclui itens de operações de SAÍDA dos arquivos finais (Excel/CSV).]' \
'--excluir-saidas[Exclui itens de operações de SAÍDA dos arquivos finais (Excel/CSV).]' \
'-t[Excluir CST 49 do Rateio da Receita Bruta.]' \
'--excluir-cst-49[Excluir CST 49 do Rateio da Receita Bruta.]' \
'-f[Listar arquivos SPED EFD encontrados no diretório atual.]' \
'--find[Listar arquivos SPED EFD encontrados no diretório atual.]' \
'-o[Retém apenas itens que geram crédito (50 <= CST <= 66).]' \
'--operacoes-de-creditos[Retém apenas itens que geram crédito (50 <= CST <= 66).]' \
'-p[Gerar arquivo CSV.]' \
'--csv[Gerar arquivo CSV.]' \
'-n[Desativar a geração do arquivo Excel (.xlsx).]' \
'--no-excel[Desativar a geração do arquivo Excel (.xlsx).]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
'-V[Print version]' \
'--version[Print version]' \
&& ret=0
}

(( $+functions[_efd_contribuicoes_commands] )) ||
_efd_contribuicoes_commands() {
    local commands; commands=()
    _describe -t commands 'efd_contribuicoes commands' commands "$@"
}

if [ "$funcstack[1]" = "_efd_contribuicoes" ]; then
    _efd_contribuicoes "$@"
else
    compdef _efd_contribuicoes efd_contribuicoes
fi
