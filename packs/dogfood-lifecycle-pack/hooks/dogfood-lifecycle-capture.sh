#!/usr/bin/env bash
# dogfood-lifecycle-capture.sh — Operation Dogfood (v26.7.13) PostToolUse hook.
#
# Appends one dfl:ToolEvent Turtle node per tool event to
# .cargo-cicd/lifecycle/session-<id>.ttl, conforming to
# packs/dogfood-lifecycle-pack/{ontology,shapes}.ttl. This is how Multifractal
# Workflow governs its own Claude Code lifecycle: every tool call becomes an
# admitted RDF observation the session-end recipe can validate + receipt.
#
# OBSERVATION ONLY. This hook records what a tool event did; it never asserts
# authority, permission, or actuation. Outcome is the TOOL-level result (Ok
# unless the tool itself errored/was blocked); the underlying command's exit
# lives inside the content-addressed prov:generated payload, not here.
#
# BEST-EFFORT: any internal failure must never disrupt the session. Every step
# is guarded and the hook always exits 0.

payload=$(cat 2>/dev/null || true)
{
  command -v jq >/dev/null 2>&1 || exit 0
  command -v b3sum >/dev/null 2>&1 || exit 0

  sid=$(printf '%s' "$payload" | jq -r '.session_id // "unknown"' 2>/dev/null || echo unknown)
  tool=$(printf '%s' "$payload" | jq -r '.tool_name // "unknown"' 2>/dev/null || echo unknown)
  # Only the closed tool-name set is admitted by dfl:ToolNameScheme / the shape.
  case "$tool" in
    Bash|Edit|Write|Read|Grep|Glob|Task|WebFetch|WebSearch) : ;;
    *) exit 0 ;;
  esac

  # Content-address the input and the result with real blake3 digests.
  in_hash=$(printf '%s' "$payload" | jq -c '.tool_input // {}' 2>/dev/null | b3sum --no-names 2>/dev/null | cut -c1-64)
  out_hash=$(printf '%s' "$payload" | jq -c '.tool_response // {}' 2>/dev/null | b3sum --no-names 2>/dev/null | cut -c1-64)
  [ -n "$in_hash" ] && [ -n "$out_hash" ] || exit 0

  # Tool-level outcome: Error only if the tool itself signalled an error.
  outcome=$(printf '%s' "$payload" | jq -r '
    if (.tool_response.is_error // .is_error // false) == true then "Error" else "Ok" end
  ' 2>/dev/null || echo Ok)
  case "$outcome" in Ok|Error|Blocked) : ;; *) outcome=Ok ;; esac

  ts=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

  dir="/Users/sac/praxis/.cargo-cicd/lifecycle"
  mkdir -p "$dir" 2>/dev/null || exit 0
  f="$dir/session-${sid}.ttl"

  if [ ! -f "$f" ]; then
    # First event: emit prefixes + the session node + its acting agent.
    {
      printf '@prefix dfl:     <http://seanchatmangpt.github.io/packs/dogfood-lifecycle#> .\n'
      printf '@prefix prov:    <http://www.w3.org/ns/prov#> .\n'
      printf '@prefix dcterms: <http://purl.org/dc/terms/> .\n'
      printf '@prefix skos:    <http://www.w3.org/2004/02/skos/core#> .\n'
      printf '@prefix time:    <http://www.w3.org/2006/time#> .\n'
      printf '@prefix xsd:     <http://www.w3.org/2001/XMLSchema#> .\n\n'
      printf '<urn:dfl:agent:%s> a prov:SoftwareAgent , prov:Agent .\n\n' "$sid"
      printf '<urn:dfl:session:%s> a dfl:Session , prov:Activity ;\n' "$sid"
      printf '    dcterms:identifier "%s" ;\n' "$sid"
      printf '    prov:wasAssociatedWith <urn:dfl:agent:%s> .\n' "$sid"
    } >> "$f" 2>/dev/null || exit 0
    seq=0
  else
    seq=$(grep -c 'a dfl:ToolEvent' "$f" 2>/dev/null || echo 0)
  fi

  # Append the tool-event node (all shape-required properties present).
  {
    printf '\n<urn:dfl:event:%s:%s> a dfl:ToolEvent , prov:Activity ;\n' "$sid" "$seq"
    printf '    dcterms:isPartOf <urn:dfl:session:%s> ;\n' "$sid"
    printf '    prov:wasAssociatedWith <urn:dfl:agent:%s> ;\n' "$sid"
    printf '    skos:notation "%s" ;\n' "$tool"
    printf '    dfl:sequenceIndex "%s"^^xsd:integer ;\n' "$seq"
    printf '    time:inXSDDateTimeStamp "%s"^^xsd:dateTimeStamp ;\n' "$ts"
    printf '    prov:used <urn:blake3:%s> ;\n' "$in_hash"
    printf '    prov:generated <urn:blake3:%s> ;\n' "$out_hash"
    printf '    dfl:outcome dfl:%s .\n' "$outcome"
  } >> "$f" 2>/dev/null || exit 0
} 2>/dev/null || true

exit 0
