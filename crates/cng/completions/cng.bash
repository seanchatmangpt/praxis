#!/usr/bin/env bash
# STATIC, HAND-WRITTEN bash completion script for `cng` (crates/cng/src/main.rs).
#
# WHY STATIC: clap-noun-verb's `run()` — the function cng's main() actually calls
# (`pub use cli::run;` in /Users/sac/clap-noun-verb/src/cli/mod.rs) is a fixed
# linkme-auto-discovery entrypoint with no builder callback; it never exposes the
# underlying clap::Command. clap-noun-verb DOES ship a completions feature
# (`clap_ext::completions::CompletionGenerator` + `.with_completions_subcommand()`,
# see clap-noun-verb/tests/completions_subcommand.rs), but that lives on the separate
# `MainCliBuilder`/`OpinionatedCliBuilder` construction path, requires manually
# populating the command/option list (it does not introspect the `#[verb(...)]`
# linkme registry either), and would require restructuring cng's main() — out of
# scope for a completions-only change. So: this script, not a fragile integration.
#
# REGENERATE MANUALLY when the verb surface changes (grep '#\[verb(' crates/cng/src/
# main.rs is the source of truth — 17 verbs across 5 nouns as of v26.9.10). This file
# is not wired into any build step; nothing regenerates it automatically.
#
# Install (bash): `source crates/cng/completions/cng.bash`, or copy into your bash
# completion directory (e.g. /usr/local/etc/bash_completion.d/ on macOS+Homebrew).

_cng_nouns() {
    echo "plan workflow benchmark engine evidence"
}

_cng_verbs_for_noun() {
    case "$1" in
        plan) echo "import admit generate decompose" ;;
        workflow) echo "project export inspect doctor validate evidence" ;;
        benchmark) echo "generate run workday verify" ;;
        engine) echo "serve resume" ;;
        evidence) echo "replay" ;;
        *) echo "" ;;
    esac
}

# Long flags for one noun+verb pair. `bench`-feature-only verbs are marked below;
# their flags still complete here regardless of which binary feature set you built —
# this script has no way to know your build's active features.
_cng_flags_for_verb() {
    case "$1 $2" in
        "plan import") echo "--dir" ;;
        "plan admit") echo "--dir" ;;
        "plan generate") echo "--dir" ;;
        "plan decompose") echo "--domain --problem --out --base-iri" ;;                 # bench
        "workflow project") echo "--dir --base-iri --derived-from" ;;
        "workflow export") echo "--dir --out --base-iri --derived-from" ;;
        "workflow inspect") echo "--file" ;;
        "workflow doctor") echo "" ;;
        "workflow validate") echo "--dir" ;;                                            # runner
        "workflow evidence") echo "--dir --out --base-iri --derived-from --seed" ;;     # runner
        "benchmark generate") echo "--out --workers --sets --depth --seed --refusal-per-mille" ;; # bench
        "benchmark run") echo "--dir --threads --replay-per-mille --queries-dir" ;;     # bench
        "benchmark workday") echo "--out --seed --ticks --refusal-per-mille" ;;         # bench
        "benchmark verify") echo "--dir --sample-every --threads" ;;                    # bench
        "engine serve") echo "--root --engine-id --seed --max-polls --poll-wait-ms" ;;  # bench
        "engine resume") echo "--root --engine-id --seed --max-polls --poll-wait-ms" ;; # bench
        "evidence replay") echo "--bundle" ;;                                           # bench
        *) echo "" ;;
    esac
}

# Global flags accepted by every verb (clap-noun-verb framework flags, not
# per-verb parameters — see crates/cng/CHEATSHEET.md "Global flags").
_cng_global_flags() {
    echo "--format --select --introspect --structured-errors --autonomic --help --version"
}

_cng_completions() {
    local cur prev words cword
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"

    local noun="" verb=""
    if [ "${#COMP_WORDS[@]}" -ge 2 ]; then
        noun="${COMP_WORDS[1]}"
    fi
    if [ "${#COMP_WORDS[@]}" -ge 3 ]; then
        verb="${COMP_WORDS[2]}"
    fi

    case "$COMP_CWORD" in
        1)
            COMPREPLY=( $(compgen -W "$(_cng_nouns) --help --version" -- "$cur") )
            ;;
        2)
            COMPREPLY=( $(compgen -W "$(_cng_verbs_for_noun "$noun")" -- "$cur") )
            ;;
        *)
            local flags
            flags="$(_cng_flags_for_verb "$noun" "$verb") $(_cng_global_flags)"
            COMPREPLY=( $(compgen -W "$flags" -- "$cur") )
            ;;
    esac
}

complete -o bashdefault -o default -F _cng_completions cng
