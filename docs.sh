#!/bin/bash
# Simple script to push the docs.

REPO="git@github.com:SBSTP/rust-igd.git"
TMPDIR="/tmp/rust-docs-$$"

echo "REPO: $REPO"
echo "TMPDIR: $TMPDIR"

mkdir $TMPDIR
cd $TMPDIR

git clone $REPO repo
mkdir docs

cd repo
if cargo doc ; then
    cp -R target/doc/* ../docs
    if [ -z $(git branch -a | grep gh-pages) ] ; then
        git checkout --orphan gh-pages
        echo "Creating new branch"
    else
        git checkout gh-pages
        echo "Using existing branch"
    fi
    rm -rf *
    cp -R ../docs/* ./
    git add -A
    git commit -m 'Update docs' > /dev/null
    git push origin gh-pages
fi

rm -rf $TMPDIR
