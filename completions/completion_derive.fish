complete -c efd_contribuicoes -s a -l all-files -r -F
complete -c efd_contribuicoes -s g -l generate -d 'If provided, outputs the completion file for given shell' -r -f -a "bash\t''
elvish\t''
fish\t''
powershell\t''
zsh\t''"
complete -c efd_contribuicoes -s r -l range -d 'Select SPED EFD files to analyze by specifying the range.' -r
complete -c efd_contribuicoes -s c -l clear_terminal -d 'Clear the terminal screen before presenting the analysis of EFD files'
complete -c efd_contribuicoes -s e -l excluir-saidas -d 'Delete output operations items from Excel and CSV files.'
complete -c efd_contribuicoes -s t -l excluir-cst-49 -d 'Excluir CST 49 do Rateio da Receita Bruta.'
complete -c efd_contribuicoes -s f -l find -d 'Find SPED EFD files'
complete -c efd_contribuicoes -s o -l operacoes-de-creditos -d 'Retain only credit entries (50 <= CST <= 66)'
complete -c efd_contribuicoes -s p -l print-csv -d 'Print CSV (Comma Separated Values) file.'
complete -c efd_contribuicoes -s h -l help -d 'Print help (see more with \'--help\')'
complete -c efd_contribuicoes -s V -l version -d 'Print version'
