#!/usr/bin/env bash

shells="bash elvish fish powershell zsh"

mkdir completions

for shell in $shells; do
 efd_contribuicoes --generate=$shell > completions/completion_derive.$shell
done
