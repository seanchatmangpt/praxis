#!/usr/bin/env bash
# Extract every fenced ```bash / ```sh block from the given markdown directories and replay
# them against a real scratch project using the actual ggen binary — the markdown analog of
# `cargo test --doc`. All RUN blocks within one markdown file share ONE scratch directory,
# executed in document order, because tutorials are sequential narratives (a later block
# assumes an earlier block's `cd myproj` already happened) — unlike independent rustdoc
# doctests, which are each self-contained. A block preceded immediately by
# "<!-- doc-example: ignore -->" is skipped (its output is not produced, so later blocks that
# depend on it will legitimately fail too — same as skipping a step in a real walkthrough).
set -euo pipefail

if [ "$#" -eq 0 ]; then
    echo "usage: doc_examples_run.sh <dir> [<dir> ...]" >&2
    exit 2
fi

repo_root=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
cd "$repo_root"

echo "doc_examples_run: building ggen binary once"
cargo build --quiet --bin ggen -p ggen 2>&1 | tail -20
export PATH="$repo_root/target/debug:$PATH"

fail_count=0
run_count=0
skip_count=0

extract_blocks() {
    # Prints each fenced ```bash/```sh block on its own, blocks separated by a line
    # containing only "===BLOCK-BOUNDARY===", prefixed with "SKIP" if the preceding
    # non-blank line was the ignore marker.
    awk '
        /<!-- doc-example: ignore -->/ { ignore_next = 1; next }
        /^```(bash|sh)[ \t]*$/ {
            in_block = 1
            skip = ignore_next
            ignore_next = 0
            block = ""
            next
        }
        /^```[ \t]*$/ {
            if (in_block) {
                in_block = 0
                print (skip ? "SKIP" : "RUN")
                print block
                print "===BLOCK-BOUNDARY==="
            }
            next
        }
        {
            if (in_block) {
                block = block $0 "\n"
            } else {
                ignore_next = 0
            }
        }
    ' "$1"
}

md_files=$(find "$@" -name '*.md' | sort)

for md in $md_files; do
    blocks=$(extract_blocks "$md")
    [ -z "$blocks" ] && continue

    scratch=$(mktemp -d)
    combined_script="$scratch/.combined.sh"
    {
        echo "set -euo pipefail"
        echo "cd '$scratch'"
    } > "$combined_script"

    block_num=0
    current_mode=""
    current_body=""
    file_has_run_block=0
    while IFS= read -r line; do
        if [ "$line" = "RUN" ] || [ "$line" = "SKIP" ]; then
            current_mode="$line"
            current_body=""
            continue
        fi
        if [ "$line" = "===BLOCK-BOUNDARY===" ]; then
            block_num=$((block_num + 1))
            if [ "$current_mode" = "SKIP" ]; then
                echo "SKIP: $md block #$block_num (marked doc-example: ignore)"
                skip_count=$((skip_count + 1))
                echo "echo '--- doc_examples_run: skipped block #$block_num, later blocks may legitimately fail ---'" >> "$combined_script"
            else
                {
                    echo "echo '--- doc_examples_run: $md block #$block_num ---'"
                    printf '%s' "$current_body"
                } >> "$combined_script"
                file_has_run_block=1
            fi
            continue
        fi
        current_body="$current_body$line
"
    done <<< "$blocks"

    if [ "$file_has_run_block" -eq 1 ]; then
        run_count=$((run_count + 1))
        if bash "$combined_script" > "$scratch/.output.log" 2>&1; then
            echo "OK: $md ($((block_num)) blocks, sequential)"
        else
            echo "FAIL: $md (sequential run broke partway through $((block_num)) blocks)"
            sed 's/^/    | /' "$scratch/.output.log"
            fail_count=$((fail_count + 1))
        fi
    fi
    rm -rf "$scratch"
done

echo "doc_examples_run: ran $run_count files, skipped $skip_count blocks, failed $fail_count files"

if [ "$fail_count" -gt 0 ]; then
    exit 1
fi
