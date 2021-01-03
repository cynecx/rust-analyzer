#!/bin/bash

set -xe

if [[ -z "$GITHUB_TOKEN" ]]; then
	echo "Set the GITHUB_TOKEN env variable."
	exit 1
fi

git config --global user.email "runner@gha.local"
git config --global user.name "GitHub Actions"

git remote add upstream "https://github.com/rust-analyzer/rust-analyzer.git"
git fetch upstream

git pull --no-edit --stat --rebase upstream master
git push origin

