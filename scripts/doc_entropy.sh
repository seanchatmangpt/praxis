#!/usr/bin/env bash
# Compression-ratio proxy for documentation information density: for every .md under the
# given directories, report (raw bytes, gzip bytes, ratio). Low ratio = mostly restating
# structure/boilerplate; high ratio = denser novel content that doesn't compress away.
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
    echo "usage: doc_entropy.sh <dir> [<dir> ...] [--format json|text]" >&2
    exit 2
fi

files=$(find "${dirs[@]}" -name '*.md' | sort)

if [ "$format" = "json" ]; then
    printf '['
    first=1
    for f in $files; do
        raw=$(wc -c < "$f" | tr -d ' ')
        gz=$(gzip -c "$f" | wc -c | tr -d ' ')
        ratio=$(awk "BEGIN{ if ($raw>0) printf \"%.4f\", $gz/$raw; else print 0 }")
        [ "$first" -eq 0 ] && printf ','
        first=0
        printf '{"file":"%s","raw_bytes":%s,"gzip_bytes":%s,"ratio":%s}' "$f" "$raw" "$gz" "$ratio"
    done
    printf ']\n'
else
    for f in $files; do
        raw=$(wc -c < "$f" | tr -d ' ')
        gz=$(gzip -c "$f" | wc -c | tr -d ' ')
        ratio=$(awk "BEGIN{ if ($raw>0) printf \"%.4f\", $gz/$raw; else print 0 }")
        echo "$f: raw=${raw}B gzip=${gz}B ratio=${ratio}"
    done
fi
