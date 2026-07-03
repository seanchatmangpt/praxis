#!/usr/bin/env bash
#
# trustless_replay.sh — re-verify praxis-synthesis receipts with NO cargo and
# NO crate source, in a bare directory whose PATH contains nothing but python3
# and b3sum.
#
# Subcommands:
#   package              (re)generate receipts/trustless/ (requires cargo)
#   verify [dir]         verify artifacts (default: receipts/trustless) —
#                        this is the default subcommand
#
# Exit: 0 = both verifiers passed; 1 = a MISMATCH (the verifier's own line is
# the last output); 2 = missing prerequisite or artifact (named).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

PACKAGE_CMD="cargo test -p praxis-synthesis --test trustless_artifacts -- --ignored"

package() {
    command -v cargo >/dev/null 2>&1 || {
        echo "trustless_replay: cargo not on PATH; 'package' needs it" >&2
        exit 2
    }
    cd "${REPO_ROOT}"
    ${PACKAGE_CMD}
    echo "==> artifacts in ${REPO_ROOT}/receipts/trustless"
}

verify() {
    local artifact_dir="${1:-${REPO_ROOT}/receipts/trustless}"

    # -- Preflight: the only two binaries this recipe is allowed to need.
    local py b3
    py="$(command -v python3 || true)"
    b3="$(command -v b3sum || true)"
    [[ -n "${py}" ]] || { echo "trustless_replay: python3 not on PATH" >&2; exit 2; }
    [[ -n "${b3}" ]] || { echo "trustless_replay: b3sum not on PATH" >&2; exit 2; }

    local missing=0
    for f in "${SCRIPT_DIR}/foreign_verify.py" "${SCRIPT_DIR}/foreign_verify_graph.py"; do
        if [[ ! -f "${f}" ]]; then
            echo "trustless_replay: missing verifier script: ${f}" >&2
            missing=1
        fi
    done
    for f in cell.json groups.json workflow.ttl workflow_receipt.json; do
        if [[ ! -f "${artifact_dir}/${f}" ]]; then
            echo "trustless_replay: missing artifact: ${artifact_dir}/${f}" >&2
            missing=1
        fi
    done
    if [[ "${missing}" -ne 0 ]]; then
        echo "trustless_replay: regenerate artifacts with:" >&2
        echo "    ${PACKAGE_CMD}" >&2
        exit 2
    fi

    # -- Bare room: only the six files, only the two binaries.
    local tmp
    tmp="$(mktemp -d)"
    # shellcheck disable=SC2064 -- expand now: `tmp` is local and gone at EXIT.
    trap "rm -rf '${tmp}'" EXIT

    cp "${SCRIPT_DIR}/foreign_verify.py" \
       "${SCRIPT_DIR}/foreign_verify_graph.py" \
       "${artifact_dir}/cell.json" \
       "${artifact_dir}/groups.json" \
       "${artifact_dir}/workflow.ttl" \
       "${artifact_dir}/workflow_receipt.json" \
       "${tmp}/"

    mkdir "${tmp}/bin"
    ln -s "${py}" "${tmp}/bin/python3"
    ln -s "${b3}" "${tmp}/bin/b3sum"

    echo "==> verifying in bare directory ${tmp} (PATH = python3 + b3sum only)"
    (
        cd "${tmp}"
        env -i PATH="${tmp}/bin" HOME="${tmp}" \
            python3 ./foreign_verify.py cell cell.json groups.json
        env -i PATH="${tmp}/bin" HOME="${tmp}" \
            python3 ./foreign_verify_graph.py graph workflow.ttl workflow_receipt.json
    )

    cat <<'EOF'

-- What this run proved --
The cell receipt and the workflow receipt re-verified from their JSON alone,
by a second implementation in a second language (python3) using a second
BLAKE3 binary (b3sum), inside a directory containing no crate source, with a
PATH containing nothing but python3 and b3sum.

-- What this run did NOT prove --
* The ir/plan/topology/geometry stage hashes in the workflow chain were
  refolded exactly as claimed in the receipt, not re-derived; re-derivation
  requires replay_workflow in the Rust crate.
* Nothing binds these artifacts to any git commit.
EOF
    if ! command -v docker >/dev/null 2>&1; then
        cat <<'EOF'
* docker is not installed on this host: no container or namespace isolation
  is claimed. The guarantee is directory + PATH hygiene only, and the
  python3/b3sum used are the host's own binaries.
EOF
    else
        cat <<'EOF'
* This recipe did not use docker even though it is installed: no container or
  namespace isolation is claimed. The guarantee is directory + PATH hygiene
  only, and the python3/b3sum used are the host's own binaries.
EOF
    fi
}

cmd="${1:-verify}"
case "${cmd}" in
    package) package ;;
    verify)  shift || true; verify "${@}" ;;
    *)
        echo "usage: trustless_replay.sh [package|verify [artifact-dir]]" >&2
        exit 2
        ;;
esac
