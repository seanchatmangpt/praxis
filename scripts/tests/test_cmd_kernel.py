#!/usr/bin/env python3
from __future__ import annotations

import ast
import copy
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SCRIPTS = ROOT / "scripts"
if str(SCRIPTS) not in sys.path:
    sys.path.insert(0, str(SCRIPTS))

import cmd_kernel as kernel


def fixture_dimensions() -> list[kernel.Dimension]:
    return [
        kernel.Dimension("runtime", "Internal", 10, 1, "Pairwise", "execution", 4),
        kernel.Dimension("storage", "Internal", 20, 1, "Pairwise", "state", 4),
        kernel.Dimension("provider", "External", 30, 1, "RiskWeightedExhaustive", "provider", 4),
        kernel.Dimension("consent", "External", 40, 1, "RiskWeightedExhaustive", "consent", 4),
    ]


def fixture_options() -> list[kernel.Option]:
    return [
        kernel.Option("rust", "runtime", "Rust", 1, "none", "REVERSIBLE", provides_capabilities=("plan",)),
        kernel.Option("wasm", "runtime", "WASM", 2, "host", "REVERSIBLE", incompatible_with=("disk",)),
        kernel.Option("memory", "storage", "memory", 1, "none", "REVERSIBLE"),
        kernel.Option("disk", "storage", "disk", 2, "grant", "REVERSIBLE_WITH_SNAPSHOT"),
        kernel.Option("github", "provider", "GitHub", 1, "broker", "COMPENSATABLE", requires_options=("explicit",)),
        kernel.Option("local", "provider", "local", 1, "local", "REVERSIBLE"),
        kernel.Option("explicit", "consent", "evidence", 1, "consent", "REVERSIBLE"),
        kernel.Option("none", "consent", "none", 0, "read-only", "REVERSIBLE", incompatible_with=("github",)),
    ]


class KernelTests(unittest.TestCase):
    def test_repeated_candidate_and_plan_identity(self) -> None:
        dimensions = fixture_dimensions()
        options = fixture_options()
        selected = {"runtime": "rust", "storage": "memory"}
        first = kernel.validate_candidate("Internal", selected, dimensions, options, verified=True)
        second = kernel.validate_candidate("Internal", dict(reversed(list(selected.items()))), dimensions, options, verified=True)
        self.assertEqual(first, second)

        kwargs = dict(
            source_revisions={"repo": "abc"},
            parameters={"z": 1, "a": {"b": True}},
            policy_digest="blake3:policy",
            observed_tree_digest="blake3:tree",
            ownership_digest="blake3:owners",
            consequence_digest="blake3:consequence",
            compiler_identity="kernel/v1",
            provided_capabilities={"plan"},
        )
        plan1 = kernel.build_plan(first, **kwargs)
        plan2 = kernel.build_plan(second, **copy.deepcopy(kwargs))
        self.assertEqual(plan1.plan_digest, plan2.plan_digest)

    def test_changed_tree_changes_plan_digest(self) -> None:
        candidate = kernel.validate_candidate(
            "Internal",
            {"runtime": "rust", "storage": "memory"},
            fixture_dimensions(),
            fixture_options(),
            verified=True,
        )
        base = dict(
            source_revisions={"repo": "abc"},
            parameters={},
            policy_digest="blake3:policy",
            ownership_digest="blake3:owners",
            consequence_digest="blake3:consequence",
            compiler_identity="kernel/v1",
            provided_capabilities={"plan"},
        )
        one = kernel.build_plan(candidate, observed_tree_digest="blake3:tree-one", **base)
        two = kernel.build_plan(candidate, observed_tree_digest="blake3:tree-two", **base)
        self.assertNotEqual(one.plan_digest, two.plan_digest)

    def test_missing_dimension_refuses(self) -> None:
        with self.assertRaisesRegex(kernel.Refusal, "CMD-G3-MISSING-DIMENSION"):
            kernel.validate_candidate(
                "Internal",
                {"runtime": "rust"},
                fixture_dimensions(),
                fixture_options(),
            )

    def test_incompatible_options_refuse(self) -> None:
        with self.assertRaisesRegex(kernel.Refusal, "CMD-G3-INCOMPATIBLE"):
            kernel.validate_candidate(
                "Internal",
                {"runtime": "wasm", "storage": "disk"},
                fixture_dimensions(),
                fixture_options(),
            )

    def test_premature_actuation_refuses(self) -> None:
        with self.assertRaisesRegex(kernel.Refusal, "CMD-G3-PREMATURE-ACTUATION"):
            kernel.validate_candidate(
                "Internal",
                {"runtime": "rust", "storage": "memory"},
                fixture_dimensions(),
                fixture_options(),
                actuated=True,
            )

    def test_duplicate_candidate_and_count_refuse(self) -> None:
        candidate = kernel.validate_candidate(
            "Internal",
            {"runtime": "rust", "storage": "memory"},
            fixture_dimensions(),
            fixture_options(),
        )
        with self.assertRaisesRegex(kernel.Refusal, "CMD-G3-DUPLICATE-CANDIDATE"):
            kernel.verify_candidate_set([candidate, candidate], declared_count=2)
        with self.assertRaisesRegex(kernel.Refusal, "CMD-G3-COUNT-MISMATCH"):
            kernel.verify_candidate_set([candidate], declared_count=2)

    def test_pairwise_selection_covers_independent_universe(self) -> None:
        candidates = [
            kernel.validate_candidate(
                "Internal",
                {"runtime": runtime, "storage": storage},
                fixture_dimensions(),
                fixture_options(),
            )
            for runtime in ("rust", "wasm")
            for storage in ("memory", "disk")
            if not (runtime == "wasm" and storage == "disk")
        ]
        report = kernel.verify_candidate_set(candidates, declared_count=len(candidates))
        self.assertEqual(report["candidate_count"], 3)
        self.assertGreaterEqual(report["witness_count"], 1)

    def test_dependency_cycle_and_unknown_refuse(self) -> None:
        with self.assertRaisesRegex(kernel.Refusal, "CMD-G6-CYCLE"):
            kernel.dependency_closure(["a"], {"a": ["b"], "b": ["a"]})
        with self.assertRaisesRegex(kernel.Refusal, "CMD-G6-UNKNOWN-CAPABILITY"):
            kernel.dependency_closure(["a"], {"a": ["missing"]})

    def test_ownership_refusals(self) -> None:
        with self.assertRaisesRegex(kernel.Refusal, "CMD-G1-OWNER-MISSING"):
            kernel.verify_ownership({"out": []}, {})
        with self.assertRaisesRegex(kernel.Refusal, "CMD-G1-DUPLICATE-AUTHORITY"):
            kernel.verify_ownership({"out": ["a", "b"]}, {})
        kernel.verify_ownership({"out": ["a", "b"]}, {"out": "stable-sort-merge"})

    def test_external_authority_positive_and_falsifiers(self) -> None:
        external = kernel.validate_candidate(
            "External",
            {"provider": "github", "consent": "explicit"},
            fixture_dimensions(),
            fixture_options(),
            verified=True,
        )
        intent = kernel.manufacture_intent(
            candidate=external,
            operation="observe",
            subject="repo",
            subject_digest="blake3:subject",
            resource_scope="metadata",
            jurisdiction="US",
            expected_postcondition="head-observed",
            resource_budget=1,
            expiry_epoch=200,
            idempotency_key="key-1",
            expected_evidence=["git-object"],
        )
        consent = {
            "subject": "repo",
            "operation": "observe",
            "resource_scope": "metadata and source",
            "revoked": False,
        }
        grant = {
            "intent_identity": intent["identity"],
            "subject_digest": "blake3:subject",
            "expiry_epoch": 200,
            "resource_ceiling": 1,
        }
        kernel.verify_external_authority(
            intent=intent,
            grant=grant,
            consent=consent,
            observed_subject_digest="blake3:subject",
            observed_postcondition="head-observed",
            required_trust="VERIFIED_PUBLISHER",
            actual_trust="INDEPENDENTLY_VERIFIED",
            allowed_jurisdictions=["US"],
            now_epoch=100,
        )

        with self.assertRaisesRegex(kernel.Refusal, "CMD-G4-CONSENT-MISSING"):
            kernel.verify_external_authority(
                intent=intent,
                grant=grant,
                consent=None,
                observed_subject_digest="blake3:subject",
                observed_postcondition=None,
                required_trust="LOCALLY_ADMITTED",
                actual_trust="LOCALLY_ADMITTED",
                allowed_jurisdictions=["US"],
                now_epoch=100,
            )
        with self.assertRaisesRegex(kernel.Refusal, "CMD-G4-TRUST"):
            kernel.verify_external_authority(
                intent=intent,
                grant=grant,
                consent=consent,
                observed_subject_digest="blake3:subject",
                observed_postcondition=None,
                required_trust="VERIFIED_PUBLISHER",
                actual_trust="UNTRUSTED",
                allowed_jurisdictions=["US"],
                now_epoch=100,
            )
        with self.assertRaisesRegex(kernel.Refusal, "CMD-G4-JURISDICTION"):
            kernel.verify_external_authority(
                intent=intent,
                grant=grant,
                consent=consent,
                observed_subject_digest="blake3:subject",
                observed_postcondition=None,
                required_trust="LOCALLY_ADMITTED",
                actual_trust="LOCALLY_ADMITTED",
                allowed_jurisdictions=["EU"],
                now_epoch=100,
            )
        with self.assertRaisesRegex(kernel.Refusal, "CMD-G8-GRANT-EXPIRED"):
            kernel.verify_external_authority(
                intent=intent,
                grant={**grant, "expiry_epoch": 99},
                consent=consent,
                observed_subject_digest="blake3:subject",
                observed_postcondition=None,
                required_trust="LOCALLY_ADMITTED",
                actual_trust="LOCALLY_ADMITTED",
                allowed_jurisdictions=["US"],
                now_epoch=100,
            )
        with self.assertRaisesRegex(kernel.Refusal, "CMD-G8-POSTCONDITION"):
            kernel.verify_external_authority(
                intent=intent,
                grant=grant,
                consent=consent,
                observed_subject_digest="blake3:subject",
                observed_postcondition="wrong",
                required_trust="LOCALLY_ADMITTED",
                actual_trust="LOCALLY_ADMITTED",
                allowed_jurisdictions=["US"],
                now_epoch=100,
            )

    def test_pure_kernel_has_no_actuator_imports(self) -> None:
        source = (SCRIPTS / "cmd_kernel.py").read_text(encoding="utf-8")
        tree = ast.parse(source)
        forbidden = {
            "os", "pathlib", "subprocess", "socket", "requests", "urllib",
            "http", "boto3", "kubernetes", "pulumi", "terraform", "git",
            "shutil", "tempfile",
        }
        observed: set[str] = set()
        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                observed.update(alias.name.split(".")[0] for alias in node.names)
            elif isinstance(node, ast.ImportFrom) and node.module:
                observed.add(node.module.split(".")[0])
        self.assertEqual(set(), observed & forbidden)

    def test_small_property_matrix_is_total_and_deterministic(self) -> None:
        dimensions = fixture_dimensions()
        options = fixture_options()
        valid: list[kernel.Candidate] = []
        for runtime in ("rust", "wasm"):
            for storage in ("memory", "disk"):
                try:
                    candidate = kernel.validate_candidate(
                        "Internal",
                        {"runtime": runtime, "storage": storage},
                        dimensions,
                        options,
                    )
                except kernel.Refusal as refusal:
                    self.assertEqual("CMD-G3-INCOMPATIBLE", refusal.code)
                    continue
                duplicate = kernel.validate_candidate(
                    "Internal",
                    {"storage": storage, "runtime": runtime},
                    dimensions,
                    options,
                )
                self.assertEqual(candidate.signature, duplicate.signature)
                valid.append(candidate)
        self.assertEqual(3, len(valid))


if __name__ == "__main__":
    unittest.main()
