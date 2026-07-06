#!/usr/bin/env bash
# Citation density: file:line citations per 100 words, per doc — reusing the same
# extraction regex as cite_audit.sh. Measures how tightly a document is coupled to
# verifiable ground truth vs. free-floating prose with nothing to check it against.
# Descriptive only — not pass/fail. Supports "--format json" for doc-metrics-report.
set -euo pipefail

format="text"
dirs=()
for arg in "$@"; do
    case "$arg" in
        --format) want_format=1 ;;
        json|text) if [ "${want_format:-0}" = "1" ]; then format="$arg"; want_format=0; else dirs+=("$arg"); fi ;;
        *) dirs+=("$arg") ;;
    esac
done

if [ "${#dirs[@]}" -eq 0 ]; then
    echo "usage: doc_density.sh <dir> [<dir> ...] [--format json|text]" >&2
    exit 2
fi

files=$(find "${dirs[@]}" -name '*.md' | sort)

emit_row() {
    local f="$1" fmt="$2"
    local words citations density
    words=$(wc -w < "$f" | tr -d ' ')
    citations=$(grep -ocE '[A-Za-z0-9_./-]+\.(rs|toml|ttl|md):[0-9]+(-[0-9]+)?' "$f" 2>/dev/null || true)
    citations=${citations:-0}
    density=$(awk "BEGIN{ if ($words>0) printf \"%.4f\", ($citations/$words)*100; else print 0 }")
    if [ "$fmt" = "json" ]; then
        printf '{"file":"%s","words":%s,"citations":%s,"citations_per_100_words":%s}' "$f" "$words" "$citations" "$density"
    else
        echo "$f: words=$words citations=$citations per_100_words=$density"
    fi
}

if [ "$format" = "json" ]; then
    printf '['
    first=1
    for f in $files; do
        [ "$first" -eq 0 ] && printf ','
        first=0
        emit_row "$f" json
    done
    printf ']\n'
else
    for f in $files; do
        emit_row "$f" text
    done
fi
