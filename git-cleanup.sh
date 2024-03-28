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
#	4497e26 (HEAD -> main, origin/main, origin/HEAD) worker_threads(4)
#	0b87d60 egui version "0.25"
#	b5abeef cargo update
#	87cc1a8 test float format
#	29b0fd0 aligning according to data_col.data_type()
#	90a9de1 Update README.md
# Example, ID: 87cc1a8
# Use git checkout & the ID to go back:
# git checkout ID
# git checkout 87cc1a8
# git checkout 4497e2
# To return from 'detached HEAD' state
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

