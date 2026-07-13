#!/usr/bin/env bash
# dogfood-lifecycle-session-end.sh — Operation Dogfood (v26.7.13) session-end
# validator + receipt.
#
# Validates every captured session lifecycle log
# (.cargo-cicd/lifecycle/session-*.ttl, produced by the PostToolUse
# dogfood-lifecycle-capture.sh hook) with `ggen graph validate --files` and
# appends a content-addressed (blake3) validation receipt per log to
# receipts.jsonl. Closes the loop: capture -> admit/validate -> receipt.
#
# Invoke manually (`bash .claude/hooks/dogfood-lifecycle-session-end.sh`) or
# wire as a SessionEnd/Stop hook.
#
# SCOPE NOTE: `ggen graph validate --files` performs Turtle PARSE validation
# today, not SHACL. The shapes.ttl constraints bite once the `--files X
# --shapes Y` SHACL layer lands (a named follow-up). The receipt here is a
# content-addressed digest binding, not yet the chained praxis-core
# ReceiptStore envelope (a named follow-up).

set -uo pipefail
dir="/Users/sac/praxis/.cargo-cicd/lifecycle"
shopt -s nullglob
files=("$dir"/session-*.ttl)
if [ ${#files[@]} -eq 0 ]; then
  echo "dogfood: no session logs in $dir"
  exit 0
fi

args=()
for f in "${files[@]}"; do args+=(--files "$f"); done

echo "dogfood: validating ${#files[@]} session log(s) via ggen graph validate --files ..."
if ggen graph validate "${args[@]}" >/dev/null 2>&1; then
  echo "dogfood: VALID — all ${#files[@]} session log(s) parse"
  rc=0
else
  echo "dogfood: INVALID — at least one session log failed parse validation:"
  ggen graph validate "${args[@]}" 2>&1 | tail -5
  rc=1
fi

recs="$dir/receipts.jsonl"
for f in "${files[@]}"; do
  h=$(b3sum --no-names "$f" 2>/dev/null | cut -c1-64)
  n=$(grep -c 'a dfl:ToolEvent' "$f" 2>/dev/null || echo 0)
  printf '{"session_log":"%s","blake3":"%s","tool_events":%s,"parse_valid":%s}\n' \
    "$(basename "$f")" "$h" "$n" "$([ "$rc" -eq 0 ] && echo true || echo false)" >> "$recs"
done
echo "dogfood: appended ${#files[@]} content-addressed receipt(s) -> $recs"
exit "$rc"
