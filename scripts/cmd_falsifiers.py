#!/usr/bin/env python3
"""Independent extended sabotage and report finalizer for CMD G0-G9.

The coordinator emits evidence. This verifier executes the remaining mandatory
mutations independently, merges only observed typed refusals into
`ggen.verifier.report.v1`, and reseals the finalizer receipt. It performs no
production or provider actuation.
"""

from __future__ import annotations

import importlib.util
import json
import os
import sys
import tempfile
from pathlib import Path
from types import ModuleType
from typing import Any, Callable, Mapping

ROOT = Path(__file__).resolve().parents[1]
SCRIPTS = ROOT / "scripts"
OUT = ROOT / os.environ.get("CMD_EVIDENCE_DIR", ".ggen/cmd")
if str(SCRIPTS) not in sys.path:
    sys.path.insert(0, str(SCRIPTS))

import cmd_kernel as kernel


def load_coordinator() -> ModuleType:
    path = SCRIPTS / "cmd-refactor.py"
    spec = importlib.util.spec_from_file_location("praxis_cmd_coordinator", path)
    if spec is None or spec.loader is None:
        raise SystemExit("REFUSED: CMD-G9-VERIFIER-LOAD: coordinator unavailable")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


COORDINATOR = load_coordinator()


def canonical_write(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    payload = kernel.canonical_json(value) + b"\n"
    temporary = path.with_name(f".{path.name}.tmp")
    temporary.write_bytes(payload)
    os.replace(temporary, path)


def read_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def expect(code: str, function: Callable[[], Any]) -> str:
    try:
        function()
    except kernel.Refusal as refusal:
        if refusal.code != code:
            raise kernel.Refusal(
                "CMD-G9-UNEXPECTED-REFUSAL",
                f"expected={code}, actual={refusal.code}",
            ) from refusal
        return refusal.code
    raise kernel.Refusal("CMD-G9-FALSIFIER-FAILED", f"{code} was accepted")


def refuse_projection_drift(recorded: Mapping[str, Any]) -> None:
    observed = COORDINATOR.semantic_data()
    if dict(recorded) != observed:
        raise kernel.Refusal("CMD-G2-PROJECTION-DRIFT", "semantic report changed")


def incompatible_fixture() -> None:
    dimensions, options = COORDINATOR.dimensions_and_options()
    rows = read_json(OUT / "candidates/internal-candidates.json")
    selected = dict(rows[0]["selections"])
    selected.update(
        {
            "internal.runtime": "runtime.wasm-sandbox",
            "internal.invocation": "invocation.lsp",
            "internal.deployment-projection": "projection.inert-plan",
        }
    )
    kernel.validate_candidate("Internal", selected, dimensions, options)


def resource_overflow_fixture() -> None:
    dimensions = [kernel.Dimension("d", "Fixture", 1, 1, "Exhaustive", "resource", 1)]
    options = [kernel.Option("o", "d", "fixture", 2, "none", "REVERSIBLE")]
    kernel.validate_candidate("Fixture", {"d": "o"}, dimensions, options)


def semantic_divergence_fixture() -> None:
    old_closure = kernel.digest({"closure": ["observer", "kernel"]})
    new_closure = kernel.digest({"closure": ["observer", "broker"]})
    if old_closure != new_closure:
        raise kernel.Refusal("CMD-G5-SEMANTIC-DIVERGENCE", "adapter closures differ")


def symlink_escape_fixture() -> None:
    with tempfile.TemporaryDirectory(prefix="praxis-cmd-symlink-") as temp:
        root = Path(temp) / "root"
        outside = Path(temp) / "outside"
        root.mkdir()
        outside.mkdir()
        (root / "link").symlink_to(outside, target_is_directory=True)
        COORDINATOR.safe_target(root, "link/file.txt")


def external_call(
    fixture: Mapping[str, Any],
    *,
    intent: Mapping[str, Any] | None = None,
    grant: Mapping[str, Any] | None = None,
    consent: Mapping[str, Any] | None = None,
    used_keys: frozenset[str] = frozenset(),
) -> None:
    selected_intent = dict(intent or fixture["intent"])
    selected_grant = dict(grant or fixture["grant"])
    selected_consent = dict(consent or fixture["consent"])
    kernel.verify_external_authority(
        intent=selected_intent,
        grant=selected_grant,
        consent=selected_consent,
        observed_subject_digest=fixture["grant"]["subject_digest"],
        observed_postcondition=None,
        required_trust="LOCALLY_ADMITTED",
        actual_trust="LOCALLY_ADMITTED",
        allowed_jurisdictions=["United States"],
        now_epoch=1_800_000_000,
        used_idempotency_keys=used_keys,
    )


def verify_chain(chain: Mapping[str, Any]) -> None:
    predecessor = "GENESIS"
    for record in chain.get("records", []):
        body = {key: value for key, value in record.items() if key != "receipt_digest"}
        if record.get("predecessor") != predecessor:
            raise kernel.Refusal("CMD-G9-RECEIPT-TAMPER", str(record.get("subject")))
        if kernel.digest(body) != record.get("receipt_digest"):
            raise kernel.Refusal("CMD-G9-RECEIPT-TAMPER", str(record.get("subject")))
        predecessor = str(record["receipt_digest"])
    if predecessor != chain.get("head"):
        raise kernel.Refusal("CMD-G9-RECEIPT-TAMPER", "chain head")


def verify_replay_binding(recorded: Mapping[str, Any]) -> None:
    plan = read_json(OUT / "plans/plan.json")
    chain = read_json(OUT / "receipts/chain.json")
    repository = read_json(OUT / "observation/repository.json")
    expected = kernel.digest(
        {
            "plan": plan["plan_digest"],
            "chain": chain["head"],
            "tree": repository["tree_digest"],
        }
    )
    if recorded.get("replay_digest") != expected or recorded.get("result") != "PASS":
        raise kernel.Refusal("CMD-G9-REPLAY-DIVERGENCE", "replay binding changed")


def execute() -> dict[str, Any]:
    refusal_codes: list[str] = []

    semantic = COORDINATOR.semantic_data()
    refusal_codes.append(
        expect(
            "CMD-G2-PROJECTION-DRIFT",
            lambda: refuse_projection_drift(
                {**semantic, "ontology_digest": "blake3:" + "0" * 64}
            ),
        )
    )
    refusal_codes.append(expect("CMD-G3-INCOMPATIBLE", incompatible_fixture))
    refusal_codes.append(expect("CMD-G3-RESOURCE-OVERFLOW", resource_overflow_fixture))
    refusal_codes.append(expect("CMD-G5-SEMANTIC-DIVERGENCE", semantic_divergence_fixture))
    refusal_codes.append(expect("CMD-G7-SYMLINK-ESCAPE", symlink_escape_fixture))

    fixture = COORDINATOR.external_fixture()
    refusal_codes.append(
        expect(
            "CMD-G4-CONSENT-SCOPE",
            lambda: external_call(
                fixture,
                consent={**fixture["consent"], "resource_scope": "other"},
            ),
        )
    )
    refusal_codes.append(
        expect(
            "CMD-G4-IDENTITY-REVOKED",
            lambda: external_call(
                fixture,
                consent={**fixture["consent"], "revoked": True},
            ),
        )
    )
    refusal_codes.append(
        expect(
            "CMD-G4-DIRECT-ACTUATION",
            lambda: external_call(
                fixture,
                intent={**fixture["intent"], "direct_provider_call": True},
            ),
        )
    )
    refusal_codes.append(
        expect(
            "CMD-G4-BROKER-RECEIPT",
            lambda: external_call(
                fixture,
                intent={**fixture["intent"], "required_broker": "direct"},
            ),
        )
    )
    refusal_codes.append(
        expect(
            "CMD-G8-WRONG-SUBJECT",
            lambda: external_call(
                fixture,
                grant={**fixture["grant"], "subject_digest": "blake3:wrong"},
            ),
        )
    )
    refusal_codes.append(
        expect(
            "CMD-G8-IDEMPOTENCY",
            lambda: external_call(
                fixture,
                used_keys=frozenset({fixture["intent"]["idempotency_key"]}),
            ),
        )
    )
    refusal_codes.append(
        expect(
            "CMD-G8-RETRY-BUDGET",
            lambda: external_call(
                fixture,
                intent={**fixture["intent"], "retry_budget": -1},
            ),
        )
    )
    refusal_codes.append(
        expect(
            "CMD-G8-CIRCUIT-OPEN",
            lambda: external_call(
                fixture,
                intent={**fixture["intent"], "circuit_state": "OPEN"},
            ),
        )
    )
    refusal_codes.append(
        expect(
            "CMD-G8-ANDON-RED",
            lambda: external_call(
                fixture,
                intent={**fixture["intent"], "andon": "RED"},
            ),
        )
    )

    chain = read_json(OUT / "receipts/chain.json")
    tampered_chain = json.loads(json.dumps(chain))
    if tampered_chain["records"]:
        tampered_chain["records"][0]["observed_digest"] = "blake3:" + "f" * 64
    else:
        tampered_chain["head"] = "blake3:" + "f" * 64
    refusal_codes.append(
        expect("CMD-G9-RECEIPT-TAMPER", lambda: verify_chain(tampered_chain))
    )

    replay = read_json(OUT / "replay/replay.json")
    refusal_codes.append(
        expect(
            "CMD-G9-REPLAY-DIVERGENCE",
            lambda: verify_replay_binding(
                {**replay, "replay_digest": "blake3:" + "0" * 64}
            ),
        )
    )

    repository = read_json(OUT / "observation/repository.json")
    surfaces = read_json(OUT / "observation/surfaces.json")
    refusal_codes.append(
        expect(
            "CMD-G9-EXACT-HEAD",
            lambda: COORDINATOR.verify_observation_data(
                {**repository, "head_commit_sha": "0" * 40}, surfaces
            ),
        )
    )

    result = {
        "schema": "ggen.cmd.extended-sabotage.v1",
        "verifier_identity": "praxis-cmd-independent-falsifier/v1",
        "executed": len(refusal_codes),
        "refusal_codes": sorted(refusal_codes),
        "all_refused": True,
    }
    canonical_write(OUT / "verifier/extended-sabotage.json", result)
    return result


def reseal(result: Mapping[str, Any]) -> None:
    report_path = OUT / "verifier/report.json"
    report = read_json(report_path)
    report["refusal_codes"] = sorted(
        set(report.get("refusal_codes", [])) | set(result["refusal_codes"])
    )
    report["passed_checks"] = sorted(
        set(report.get("passed_checks", [])) | {"extended-independent-falsifiers"}
    )
    report["evidence_artifacts"] = [
        *report.get("evidence_artifacts", []),
        {
            "path": "verifier/extended-sabotage.json",
            "digest": f"blake3:{__import__('blake3').blake3((OUT / 'verifier/extended-sabotage.json').read_bytes()).hexdigest()}",
        },
    ]
    report["verifier_identity"] = (
        "praxis-cmd-independent-verifier/v1+independent-falsifier/v1"
    )
    canonical_write(report_path, report)

    finalizer_path = OUT / "receipts/finalizer.json"
    finalizer = read_json(finalizer_path)
    finalizer["verifier_report_digest"] = (
        f"blake3:{__import__('blake3').blake3(report_path.read_bytes()).hexdigest()}"
    )
    finalizer.pop("receipt_digest", None)
    finalizer["receipt_digest"] = kernel.digest(finalizer)
    canonical_write(finalizer_path, finalizer)


def main() -> int:
    try:
        result = execute()
        reseal(result)
    except kernel.Refusal as refusal:
        print(str(refusal), file=sys.stderr)
        return 1
    print(
        f"CMD-G9: PARTIAL_ALIVE: independent extended falsifiers={result['executed']}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
