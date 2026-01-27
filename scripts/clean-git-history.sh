#!/bin/bash

# Script to clean .vscode directory from git history
# WARNING: This rewrites git history. Only run if you haven't pushed to a public remote yet.

set -e

echo "⚠️  WARNING: This script will rewrite git history!"
echo "This should only be run if you haven't pushed to a public remote repository."
echo ""
read -p "Continue? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]
then
    echo "Aborted."
    exit 1
fi

echo "Checking for uncommitted changes..."
if [[ -n $(git status -s) ]]; then
    echo "❌ You have uncommitted changes. Please commit or stash them first."
    exit 1
fi

echo "Creating backup branch..."
git branch backup-before-history-clean

echo "Removing .vscode from git history using filter-branch..."
git filter-branch --force --index-filter \
  'git rm -r --cached --ignore-unmatch .vscode' \
  --prune-empty --tag-name-filter cat -- --all

echo "Cleaning up refs..."
git for-each-ref --format='delete %(refname)' refs/original | git update-ref --stdin
git reflog expire --expire=now --all
git gc --prune=now --aggressive

echo ""
echo "✅ .vscode has been removed from git history."
echo ""
echo "⚠️  Important next steps:"
echo "1. Verify your repository still works: git log, git status"
echo "2. If everything looks good, force push: git push origin --force --all"
echo "3. If something went wrong, restore: git reset --hard backup-before-history-clean"
echo ""
echo "Backup branch created: backup-before-history-clean"
