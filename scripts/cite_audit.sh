#!/usr/bin/env bash
# Verify every "path/to/file.ext:NN" or "path/to/file.ext:NN-MM" citation found in the
# given doc trees names a file that actually exists (relative to repo root) and a line
# (or line range) within that file's actual length. Exits non-zero on any miss, printing
# every failing citation with its source doc — exhaustive, not sampled.
set -euo pipefail

if [ "$#" -eq 0 ]; then
    echo "usage: cite_audit.sh <dir> [<dir> ...]" >&2
    exit 2
fi

repo_root=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
cd "$repo_root"

fail_count=0
checked_count=0

# Extract every "some/path.ext:NN" or "some/path.ext:NN-MM" token from all .md files
# under the given directories. Extensions restricted to source/config/doc types that
# this project actually cites (rs, toml, ttl, md).
citations=$(grep -RnoE '[A-Za-z0-9_./-]+\.(rs|toml|ttl|md):[0-9]+(-[0-9]+)?' "$@" 2>/dev/null || true)

if [ -z "$citations" ]; then
    echo "cite_audit: no citations found under: $*"
    exit 0
fi

while IFS= read -r line; do
    [ -z "$line" ] && continue
    # grep -Rno format: "source_doc:source_lineno:cited_path:cited_linespec"
    IFS=':' read -r source_doc _source_lineno path linespec <<< "$line"

    checked_count=$((checked_count + 1))

    if [ ! -f "$path" ]; then
        echo "MISS (file not found): $source_doc cites '$path:$linespec'"
        fail_count=$((fail_count + 1))
        continue
    fi

    actual_lines=$(wc -l < "$path" | tr -d ' ')

    if [[ "$linespec" == *-* ]]; then
        start="${linespec%-*}"
        end="${linespec#*-}"
    else
        start="$linespec"
        end="$linespec"
    fi

    if [ "$start" -lt 1 ] || [ "$end" -gt "$actual_lines" ] || [ "$start" -gt "$end" ]; then
        echo "MISS (out of range): $source_doc cites '$path:$linespec' but file has $actual_lines lines"
        fail_count=$((fail_count + 1))
    fi
done <<< "$citations"

echo "cite_audit: checked $checked_count citations, $fail_count failed"

if [ "$fail_count" -gt 0 ]; then
    exit 1
fi
