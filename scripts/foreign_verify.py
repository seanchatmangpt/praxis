#!/usr/bin/env python3
"""Step 3 of the cell roadmap — the foreign verifier.

A SECOND IMPLEMENTATION, in a different language, using a different BLAKE3
binary (`b3sum`), that re-verifies praxis-synthesis receipts from their JSON
alone. If this script and the Rust crate agree on a hash, the receipt is not
self-attested.

Usage:
  foreign_verify.py dag  <dag_receipt.json>    # verify chain refold + root
  foreign_verify.py cell <cell.json> <groups.json>  # verify cell hash from roll-ups

Exit 0 = verified; exit 1 = MISMATCH (printed); exit 2 = usage/IO error.

Chain semantics (chatman-common/provenance.rs, replicated independently):
  genesis(domain) = blake3(domain_bytes)                      (lowercase hex)
  fold(prev_hex, payload) = blake3(prev_hex_ascii || payload)
Frame canonicalization: JSON object, keys sorted, compact separators —
matching serde_json's BTreeMap serialization.
"""
import json
import subprocess
import sys

DAG_DOMAIN = "praxis-synthesis/dag/v1"
CELL_DOMAIN = "praxis-synthesis/cell/v1"


def b3(data: bytes) -> str:
    """BLAKE3 via the b3sum binary — deliberately not the Rust crate."""
    out = subprocess.run(
        ["b3sum", "--no-names"], input=data, capture_output=True, check=True
    )
    return out.stdout.decode().strip()


def genesis(domain: str) -> str:
    return b3(domain.encode())


def fold(prev_hex: str, payload: bytes) -> str:
    return b3(prev_hex.encode() + payload)


def canonical_frame(nr: dict) -> bytes:
    frame = {
        "node_id": nr["node_id"],
        "action_hash": nr["action_hash"],
        "input_hashes": nr["input_hashes"],
        "output_hash": nr["output_hash"],
    }
    return json.dumps(frame, sort_keys=True, separators=(",", ":")).encode()


def verify_dag(path: str) -> int:
    receipt = json.load(open(path))
    # 1. Refold the chain over node frames in recorded order.
    chain = genesis(DAG_DOMAIN)
    for nr in receipt["node_receipts"]:
        chain = fold(chain, canonical_frame(nr))
        if chain != nr["chain"]:
            print(f"MISMATCH: node {nr['node_id'][:12]} chain "
                  f"recomputed {chain[:16]} != recorded {nr['chain'][:16]}")
            return 1
    # 2. Recompute the order-independent root.
    pairs = sorted(f"{nr['node_id']}:{nr['output_hash']}"
                   for nr in receipt["node_receipts"])
    root = b3("\n".join(pairs).encode())
    if root != receipt["root_hash"]:
        print(f"MISMATCH: root recomputed {root[:16]} != "
              f"recorded {receipt['root_hash'][:16]}")
        return 1
    print(f"VERIFIED dag: {len(receipt['node_receipts'])} frames, "
          f"chain {chain[:16]}…, root {root[:16]}…")
    return 0


def verify_cell(cell_path: str, groups_path: str) -> int:
    cell = json.load(open(cell_path))
    groups = json.load(open(groups_path))
    if len(groups) != cell["g"]:
        print(f"MISMATCH: {len(groups)} groups != declared {cell['g']}")
        return 1
    # Cell hash from group replay roots ALONE — no member data touched.
    h = genesis(CELL_DOMAIN)
    for gr in groups:
        h = fold(h, gr["replay_root"].encode())
    if h != cell["cell_hash"]:
        print(f"MISMATCH: cell hash recomputed {h[:16]} != "
              f"recorded {cell['cell_hash'][:16]}")
        return 1
    admitted = sum(gr["admitted"] for gr in groups)
    refused = sum(gr["refused"] for gr in groups)
    if admitted != cell["admitted"] or refused != cell["refused"]:
        print(f"MISMATCH: counts {admitted}/{refused} != "
              f"declared {cell['admitted']}/{cell['refused']}")
        return 1
    print(f"VERIFIED cell: n={cell['n']} g={cell['g']} "
          f"admitted={admitted} refused={refused}, "
          f"cell_hash {h[:16]}… — from {len(groups)} roll-ups, "
          f"zero interiors read")
    return 0


def main() -> int:
    if len(sys.argv) < 3:
        print(__doc__)
        return 2
    mode = sys.argv[1]
    if mode == "dag":
        return verify_dag(sys.argv[2])
    if mode == "cell" and len(sys.argv) >= 4:
        return verify_cell(sys.argv[2], sys.argv[3])
    print(__doc__)
    return 2


if __name__ == "__main__":
    sys.exit(main())
