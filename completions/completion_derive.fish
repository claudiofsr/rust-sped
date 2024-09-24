complete -c efd_contribuicoes -s a -l all-files -r -F
complete -c efd_contribuicoes -s g -l generate -d 'If provided, outputs the completion file for given shell' -r -f -a "{bash	,elvish	,fish	,powershell	,zsh	}"
complete -c efd_contribuicoes -s r -l range -d 'Select SPED EFD files to analyze by specifying the range.' -r
complete -c efd_contribuicoes -s t -l threads -d 'Number of additional threads used to generate worksheets.' -r
complete -c efd_contribuicoes -s c -l clear_terminal -d 'Clear the terminal screen before listing the duplicate files'
complete -c efd_contribuicoes -s f -l find -d 'Find SPED EFD files'
complete -c efd_contribuicoes -s p -l print-csv -d 'Print CSV (Comma Separated Values) file.'
complete -c efd_contribuicoes -s h -l help -d 'Print help (see more with \'--help\')'
complete -c efd_contribuicoes -s V -l version -d 'Print version'
