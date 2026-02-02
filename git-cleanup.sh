#!/usr/bin/env bash

# https://stackoverflow.com/questions/15617673/git-how-to-delete-all-commits-except-last-one

# rm -rf .git
# git init
# git add .
# git commit -m "Initial commit"
# git branch -M master
# git remote add origin git@github.com:claudiofsr/randy.git
# git push -u --force origin master
# git status

# https://stackoverflow.com/questions/4114095/how-do-i-revert-a-git-repository-to-a-previous-commit
# Using Git — how to go back to a previous commit
# git log --oneline
# Take a note of the ID of the commit you want to revert to
# 7a0fc45 (HEAD -> master, origin/master) cargo update v2
# d3d5f25 cargo update
# 0ebf6a9 cargo update v2
# b3e01cb cargo update
# ...
# ea812fa Initial commit
# Example, ID: ea812fa
# Use git checkout & the ID to go back:
# git checkout ID
# git checkout b4f4835 -> rateio incorreto
# git checkout 53d360f -> rateio correto
# To return from 'detached HEAD' state
# git switch -
# git checkout main (or master or -)

# git log --oneline
# git reset --hard <commitId> && git clean -f
# git push --force

clear

FILE=".git/config"

if [ -f "$FILE" ]
then
    printf "$FILE:\n\n"
else
    printf "File $FILE is not there, aborting!\n"
    exit
fi

config=$(<$FILE)
printf "$config\n\n"

# url = git@github.com:claudiofsr/rust-sped.git
my_url=$(cat .git/config | sed '/url/!d' | sed -r 's/.*=\s*(.*)/\1/')

printf "my_url = '$my_url'\n"

# [branch "master"]
my_branch=$(cat .git/config | sed '/branch/!d' | sed -r 's/.*branch\s*"(.*)".*/\1/')

printf "my_branch = '$my_branch'\n"

# Define your function here
new_git () {
    rm -rf .git
    git init
    # edit .gitignore
    git add .
    git commit -m "Initial commit"
    git branch -M $my_branch
    git remote add origin $my_url
    git push -u --force origin $my_branch
}

while true; do
    printf "\n"
    printf "\t Delete .git directory\n"
    printf "\t Continue? (Y/N):"
    read yn
    case $yn in
        [Yy]* ) new_git; break;;
        [Nn]* ) exit;;
        * ) printf "\n\t Please answer Y or N.\n";;
    esac
done

