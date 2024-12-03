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
'*-a+[]:ALL_FILES:_files' \
'*--all-files=[]:ALL_FILES:_files' \
'-g+[If provided, outputs the completion file for given shell]:GENERATOR:(bash elvish fish powershell zsh)' \
'--generate=[If provided, outputs the completion file for given shell]:GENERATOR:(bash elvish fish powershell zsh)' \
'*-r+[Select SPED EFD files to analyze by specifying the range.]:RANGE:_default' \
'*--range=[Select SPED EFD files to analyze by specifying the range.]:RANGE:_default' \
'-c[Clear the terminal screen before presenting the analysis of EFD files]' \
'--clear_terminal[Clear the terminal screen before presenting the analysis of EFD files]' \
'-e[Delete output operations items from Excel and CSV files.]' \
'--excluir-saidas[Delete output operations items from Excel and CSV files.]' \
'-f[Find SPED EFD files]' \
'--find[Find SPED EFD files]' \
'-o[Retain only credit entries (CST 50 .. 66)]' \
'--operacoes-de-creditos[Retain only credit entries (CST 50 .. 66)]' \
'-p[Print CSV (Comma Separated Values) file.]' \
'--print-csv[Print CSV (Comma Separated Values) file.]' \
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
