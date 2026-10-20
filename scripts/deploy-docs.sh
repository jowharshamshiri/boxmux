#!/bin/sh
# Builds the documentation site in docs/ and publishes it to the gh-pages branch, which
# GitHub Pages serves at boxmux.com.
#
# Usage: scripts/deploy-docs.sh [--dry-run]
#   --dry-run  build and commit to a local gh-pages worktree, but do not push.
set -eu

dry_run=false
case "${1:-}" in
    --dry-run) dry_run=true ;;
    "") ;;
    *) echo "usage: $0 [--dry-run]" >&2; exit 2 ;;
esac

cd "$(dirname "$0")/.."
[ -z "$(git status --porcelain docs)" ] || {
    echo "deploy-docs: docs/ has uncommitted changes" >&2
    exit 1
}
source_commit=$(git rev-parse --short HEAD)

(cd docs && npm ci && npm run build)

site=$(mktemp -d)
trap 'git worktree remove --force "$site" 2>/dev/null; rm -rf "$site"' EXIT
git fetch --quiet origin gh-pages
git worktree add --force "$site" origin/gh-pages
(
    cd "$site"
    git checkout --quiet -B gh-pages origin/gh-pages
    # Replace the published files with the new build, keeping the repository metadata.
    find . -mindepth 1 -maxdepth 1 ! -name .git -exec rm -rf {} +
    cp -R "$OLDPWD/docs/build/." .
    touch .nojekyll
    git add -A
    if git diff --cached --quiet; then
        echo "deploy-docs: the site is unchanged"
        exit 0
    fi
    git commit --quiet -m "Docs from $source_commit"
    if $dry_run; then
        echo "dry run: committed to the local gh-pages branch; not pushed"
    else
        git push origin gh-pages
    fi
)
