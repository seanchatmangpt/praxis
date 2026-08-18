#!/usr/bin/env python3
"""Pure combinatorial-maximalism kernel.

This module contains no filesystem, process, network, credential, registry, or
deployment imports. Adapters may perform observation and serialization, but all
identity, closure, constraint, ownership, resource, authority, and plan logic
must converge here.
"""

from __future__ import annotations

import json
from dataclasses import dataclass
from itertools import combinations
from typing import Any, Iterable, Mapping, Sequence

import blake3


class Refusal(ValueError):
    """Closed typed refusal emitted by the pure kernel."""

    def __init__(self, code: str, message: str) -> None:
        super().__init__(f"REFUSED: {code}: {message}")
        self.code = code
        self.message = message


@dataclass(frozen=True)
class Dimension:
    identity: str
    domain: str
    order: int
    cardinality: int
    coverage_mode: str
    risk_class: str
    resource_ceiling: int


@dataclass(frozen=True)
class Option:
    identity: str
    dimension: str
    implementation: str
    resource_cost: int
    authority_requirement: str
    reversal_class: str
    requires_options: tuple[str, ...] = ()
    incompatible_with: tuple[str, ...] = ()
    provides_capabilities: tuple[str, ...] = ()
    requires_capabilities: tuple[str, ...] = ()


@dataclass(frozen=True)
class Candidate:
    domain: str
    selections: tuple[tuple[str, str], ...]
    signature: str
    resource_cost: int
    verified: bool = False
    authorized: bool = False
    actuated: bool = False


@dataclass(frozen=True)
class Plan:
    schema: str
    candidate_signature: str
    selected_options: tuple[str, ...]
    resolved_capabilities: tuple[str, ...]
    source_revisions: tuple[tuple[str, str], ...]
    parameters: tuple[tuple[str, Any], ...]
    policy_digest: str
    observed_tree_digest: str
    ownership_digest: str
    consequence_digest: str
    compiler_identity: str
    plan_digest: str


TRUST_ORDER = {
    "UNTRUSTED": 0,
    "LOCALLY_ADMITTED": 1,
    "SIGNED": 2,
    "VERIFIED_PUBLISHER": 3,
    "INDEPENDENTLY_VERIFIED": 4,
    "PRODUCTION_APPROVED": 5,
    "REVOKED": -1,
}


def canonical_json(value: Any) -> bytes:
    """Return stable UTF-8 JSON bytes."""
    return json.dumps(
        value,
        sort_keys=True,
        separators=(",", ":"),
        ensure_ascii=False,
        allow_nan=False,
    ).encode("utf-8")


def digest(value: Any) -> str:
    """Return a BLAKE3 identity over canonical semantic bytes."""
    return f"blake3:{blake3.blake3(canonical_json(value)).hexdigest()}"


def normalize_mapping(value: Mapping[str, Any]) -> tuple[tuple[str, Any], ...]:
    """Normalize a manifest-like mapping without dropping unknown fields."""
    normalized: list[tuple[str, Any]] = []
    for key in sorted(value):
        item = value[key]
        if isinstance(item, Mapping):
            item = normalize_mapping(item)
        elif isinstance(item, (list, tuple)):
            item = tuple(_normalize_value(part) for part in item)
        else:
            item = _normalize_value(item)
        normalized.append((str(key), item))
    return tuple(normalized)


def _normalize_value(value: Any) -> Any:
    if isinstance(value, Mapping):
        return normalize_mapping(value)
    if isinstance(value, (list, tuple)):
        return tuple(_normalize_value(item) for item in value)
    if value is None or isinstance(value, (str, int, bool)):
        return value
    raise Refusal("CMD-G6-MANIFEST-TYPE", f"unsupported value type: {type(value).__name__}")


def dimension_map(dimensions: Sequence[Dimension]) -> dict[str, Dimension]:
    result: dict[str, Dimension] = {}
    orders: set[int] = set()
    for dimension in dimensions:
        if not dimension.identity:
            raise Refusal("CMD-G3-DIMENSION-IDENTITY", "dimension identity is empty")
        if dimension.identity in result:
            raise Refusal("CMD-G3-DUPLICATE-DIMENSION", dimension.identity)
        if dimension.order in orders:
            raise Refusal("CMD-G3-DIMENSION-ORDER", str(dimension.order))
        if dimension.cardinality != 1:
            raise Refusal(
                "CMD-G3-CARDINALITY",
                f"{dimension.identity} requires unsupported cardinality {dimension.cardinality}",
            )
        if dimension.resource_ceiling < 1:
            raise Refusal("CMD-G3-RESOURCE-CEILING", dimension.identity)
        result[dimension.identity] = dimension
        orders.add(dimension.order)
    return result


def option_map(options: Sequence[Option], dimensions: Mapping[str, Dimension]) -> dict[str, Option]:
    result: dict[str, Option] = {}
    by_dimension: dict[str, int] = {identity: 0 for identity in dimensions}
    for option in options:
        if option.identity in result:
            raise Refusal("CMD-G3-DUPLICATE-OPTION", option.identity)
        if option.dimension not in dimensions:
            raise Refusal(
                "CMD-G3-UNKNOWN-DIMENSION",
                f"{option.identity} -> {option.dimension}",
            )
        if option.resource_cost < 0:
            raise Refusal("CMD-G3-RESOURCE-COST", option.identity)
        if not option.reversal_class:
            raise Refusal("CMD-G3-REVERSAL-MISSING", option.identity)
        result[option.identity] = option
        by_dimension[option.dimension] += 1
    missing = sorted(identity for identity, count in by_dimension.items() if count == 0)
    if missing:
        raise Refusal("CMD-G3-OPTION-CLOSURE", ",".join(missing))
    return result


def validate_candidate(
    domain: str,
    selected: Mapping[str, str],
    dimensions: Sequence[Dimension],
    options: Sequence[Option],
    *,
    verified: bool = False,
    authorized: bool = False,
    actuated: bool = False,
) -> Candidate:
    dims = dimension_map([item for item in dimensions if item.domain == domain])
    opts = option_map(
        [item for item in options if item.dimension in dims],
        dims,
    )
    if set(selected) != set(dims):
        missing = sorted(set(dims) - set(selected))
        extra = sorted(set(selected) - set(dims))
        raise Refusal(
            "CMD-G3-MISSING-DIMENSION",
            f"missing={missing}, extra={extra}",
        )

    selected_options: list[Option] = []
    for dimension in sorted(dims.values(), key=lambda item: (item.order, item.identity)):
        option_id = selected[dimension.identity]
        option = opts.get(option_id)
        if option is None or option.dimension != dimension.identity:
            raise Refusal(
                "CMD-G3-OPTION-DIMENSION",
                f"{dimension.identity} -> {option_id}",
            )
        selected_options.append(option)

    selected_ids = {item.identity for item in selected_options}
    for option in selected_options:
        missing_requirements = sorted(set(option.requires_options) - selected_ids)
        if missing_requirements:
            raise Refusal(
                "CMD-G3-DEPENDENCY-CLOSURE",
                f"{option.identity} requires {missing_requirements}",
            )
        conflicts = sorted(set(option.incompatible_with) & selected_ids)
        if conflicts:
            raise Refusal(
                "CMD-G3-INCOMPATIBLE",
                f"{option.identity} conflicts with {conflicts}",
            )

    provided = {
        capability
        for option in selected_options
        for capability in option.provides_capabilities
    }
    for option in selected_options:
        missing_capabilities = sorted(set(option.requires_capabilities) - provided)
        if missing_capabilities:
            raise Refusal(
                "CMD-G6-UNKNOWN-CAPABILITY",
                f"{option.identity} requires {missing_capabilities}",
            )

    resource_cost = sum(item.resource_cost for item in selected_options)
    resource_ceiling = sum(item.resource_ceiling for item in dims.values())
    if resource_cost > resource_ceiling:
        raise Refusal(
            "CMD-G3-RESOURCE-OVERFLOW",
            f"cost={resource_cost}, ceiling={resource_ceiling}",
        )
    if actuated and not authorized:
        raise Refusal("CMD-G3-PREMATURE-ACTUATION", "actuated candidate is unauthorized")
    if authorized and not verified:
        raise Refusal("CMD-G3-PREMATURE-AUTHORIZATION", "authorized candidate is unverified")

    selections = tuple(
        (dimension.identity, selected[dimension.identity])
        for dimension in sorted(dims.values(), key=lambda item: (item.order, item.identity))
    )
    signature = digest({"domain": domain, "selections": selections})
    return Candidate(
        domain=domain,
        selections=selections,
        signature=signature,
        resource_cost=resource_cost,
        verified=verified,
        authorized=authorized,
        actuated=actuated,
    )


def candidate_pairs(candidate: Candidate) -> frozenset[tuple[str, str]]:
    """Return unordered selected-option pairs for independent coverage checks."""
    option_ids = [option for _, option in candidate.selections]
    return frozenset(tuple(sorted(pair)) for pair in combinations(option_ids, 2))


def select_pairwise(candidates: Sequence[Candidate]) -> tuple[Candidate, ...]:
    """Deterministically select a greedy pairwise witness set."""
    if not candidates:
        return ()
    universe = set().union(*(candidate_pairs(candidate) for candidate in candidates))
    uncovered = set(universe)
    selected: list[Candidate] = []
    remaining = sorted(candidates, key=lambda item: item.signature)
    while uncovered:
        ranked = sorted(
            remaining,
            key=lambda item: (
                -len(candidate_pairs(item) & uncovered),
                item.resource_cost,
                item.signature,
            ),
        )
        best = ranked[0]
        gain = candidate_pairs(best) & uncovered
        if not gain:
            raise Refusal("CMD-G3-COVERAGE-STALLED", f"uncovered={len(uncovered)}")
        selected.append(best)
        uncovered -= gain
        remaining = [item for item in remaining if item.signature != best.signature]
    return tuple(selected)


def verify_candidate_set(
    candidates: Sequence[Candidate],
    *,
    declared_count: int,
    require_unique: bool = True,
) -> dict[str, Any]:
    signatures = [item.signature for item in candidates]
    if require_unique and len(signatures) != len(set(signatures)):
        raise Refusal("CMD-G3-DUPLICATE-CANDIDATE", "duplicate candidate signature")
    if declared_count != len(candidates):
        raise Refusal(
            "CMD-G3-COUNT-MISMATCH",
            f"declared={declared_count}, observed={len(candidates)}",
        )
    expected_pairs = set().union(*(candidate_pairs(item) for item in candidates)) if candidates else set()
    witnesses = select_pairwise(candidates)
    witnessed_pairs = set().union(*(candidate_pairs(item) for item in witnesses)) if witnesses else set()
    missing_pairs = sorted(expected_pairs - witnessed_pairs)
    if missing_pairs:
        raise Refusal("CMD-G3-COVERAGE-MISSING", repr(missing_pairs[:5]))
    return {
        "candidate_count": len(candidates),
        "pair_count": len(expected_pairs),
        "witness_count": len(witnesses),
        "witness_signatures": [item.signature for item in witnesses],
    }


def dependency_closure(
    roots: Iterable[str],
    dependencies: Mapping[str, Sequence[str]],
) -> tuple[str, ...]:
    """Return deterministic dependency closure or refuse cycles/unknown nodes."""
    visiting: set[str] = set()
    visited: set[str] = set()
    ordered: list[str] = []

    def visit(node: str, stack: tuple[str, ...]) -> None:
        if node in visiting:
            raise Refusal("CMD-G6-CYCLE", " -> ".join((*stack, node)))
        if node in visited:
            return
        if node not in dependencies:
            raise Refusal("CMD-G6-UNKNOWN-CAPABILITY", node)
        visiting.add(node)
        for dependency in sorted(dependencies[node]):
            visit(dependency, (*stack, node))
        visiting.remove(node)
        visited.add(node)
        ordered.append(node)

    for root in sorted(set(roots)):
        visit(root, ())
    return tuple(ordered)


def verify_ownership(outputs: Mapping[str, Sequence[str]], merge_laws: Mapping[str, str]) -> None:
    for output in sorted(outputs):
        owners = tuple(sorted(set(outputs[output])))
        if not owners:
            raise Refusal("CMD-G1-OWNER-MISSING", output)
        if len(owners) > 1 and not merge_laws.get(output):
            raise Refusal(
                "CMD-G1-DUPLICATE-AUTHORITY",
                f"{output}: {owners}",
            )


def build_plan(
    candidate: Candidate,
    *,
    source_revisions: Mapping[str, str],
    parameters: Mapping[str, Any],
    policy_digest: str,
    observed_tree_digest: str,
    ownership_digest: str,
    consequence_digest: str,
    compiler_identity: str,
    provided_capabilities: Iterable[str],
) -> Plan:
    if not candidate.verified:
        raise Refusal("CMD-G6-UNVERIFIED-CANDIDATE", candidate.signature)
    normalized_sources = tuple(sorted((str(k), str(v)) for k, v in source_revisions.items()))
    normalized_parameters = normalize_mapping(parameters)
    selected_options = tuple(option for _, option in candidate.selections)
    capabilities = tuple(sorted(set(provided_capabilities)))
    body = {
        "schema": "ggen.cmd.plan.v1",
        "candidate_signature": candidate.signature,
        "selected_options": selected_options,
        "resolved_capabilities": capabilities,
        "source_revisions": normalized_sources,
        "parameters": normalized_parameters,
        "policy_digest": policy_digest,
        "observed_tree_digest": observed_tree_digest,
        "ownership_digest": ownership_digest,
        "consequence_digest": consequence_digest,
        "compiler_identity": compiler_identity,
    }
    return Plan(
        schema=body["schema"],
        candidate_signature=candidate.signature,
        selected_options=selected_options,
        resolved_capabilities=capabilities,
        source_revisions=normalized_sources,
        parameters=normalized_parameters,
        policy_digest=policy_digest,
        observed_tree_digest=observed_tree_digest,
        ownership_digest=ownership_digest,
        consequence_digest=consequence_digest,
        compiler_identity=compiler_identity,
        plan_digest=digest(body),
    )


def verify_external_authority(
    *,
    intent: Mapping[str, Any],
    grant: Mapping[str, Any] | None,
    consent: Mapping[str, Any] | None,
    observed_subject_digest: str,
    observed_postcondition: str | None,
    required_trust: str,
    actual_trust: str,
    allowed_jurisdictions: Sequence[str],
    now_epoch: int,
    used_idempotency_keys: frozenset[str] = frozenset(),
) -> None:
    if intent.get("direct_provider_call"):
        raise Refusal("CMD-G4-DIRECT-ACTUATION", "provider call bypasses broker")
    if intent.get("required_broker") != "BRCE":
        raise Refusal("CMD-G4-BROKER-RECEIPT", "required broker is not BRCE")
    if consent is None:
        raise Refusal("CMD-G4-CONSENT-MISSING", "no consent evidence")
    if consent.get("revoked"):
        raise Refusal("CMD-G4-IDENTITY-REVOKED", "consent or identity is revoked")
    if consent.get("subject") != intent.get("subject"):
        raise Refusal("CMD-G4-CONSENT-SCOPE", "consent subject mismatch")
    if consent.get("operation") != intent.get("operation"):
        raise Refusal("CMD-G4-CONSENT-SCOPE", "consent operation mismatch")
    if intent.get("resource_scope") not in str(consent.get("resource_scope", "")):
        raise Refusal("CMD-G4-CONSENT-SCOPE", "resource is outside consent")
    jurisdiction = intent.get("jurisdiction")
    if jurisdiction not in allowed_jurisdictions:
        raise Refusal("CMD-G4-JURISDICTION", str(jurisdiction))
    if actual_trust == "REVOKED":
        raise Refusal("CMD-G4-IDENTITY-REVOKED", "trust state is REVOKED")
    if TRUST_ORDER.get(actual_trust, -1) < TRUST_ORDER.get(required_trust, 10):
        raise Refusal(
            "CMD-G4-TRUST",
            f"required={required_trust}, actual={actual_trust}",
        )
    if grant is None:
        raise Refusal("CMD-G8-GRANT-MISSING", "no authority grant")
    if grant.get("intent_identity") != intent.get("identity"):
        raise Refusal("CMD-G8-WRONG-SUBJECT", "grant does not bind intent")
    if grant.get("subject_digest") != observed_subject_digest:
        raise Refusal("CMD-G8-WRONG-SUBJECT", "subject digest changed")
    if int(grant.get("expiry_epoch", -1)) < now_epoch:
        raise Refusal("CMD-G8-GRANT-EXPIRED", str(grant.get("expiry_epoch")))
    if int(grant.get("resource_ceiling", -1)) < int(intent.get("resource_budget", 0)):
        raise Refusal("CMD-G3-RESOURCE-OVERFLOW", "intent exceeds grant")
    key = str(intent.get("idempotency_key", ""))
    if not key or key in used_idempotency_keys:
        raise Refusal("CMD-G8-IDEMPOTENCY", key or "missing")
    if int(intent.get("retry_budget", 0)) < 0:
        raise Refusal("CMD-G8-RETRY-BUDGET", str(intent.get("retry_budget")))
    if intent.get("circuit_state") == "OPEN":
        raise Refusal("CMD-G8-CIRCUIT-OPEN", "circuit is OPEN")
    if intent.get("andon") == "RED":
        raise Refusal("CMD-G8-ANDON-RED", "Andon is RED")
    expected = intent.get("expected_postcondition")
    if observed_postcondition is not None and observed_postcondition != expected:
        raise Refusal(
            "CMD-G8-POSTCONDITION",
            f"expected={expected!r}, observed={observed_postcondition!r}",
        )


def manufacture_intent(
    *,
    candidate: Candidate,
    operation: str,
    subject: str,
    subject_digest: str,
    resource_scope: str,
    jurisdiction: str,
    expected_postcondition: str,
    resource_budget: int,
    expiry_epoch: int,
    idempotency_key: str,
    expected_evidence: Sequence[str],
) -> dict[str, Any]:
    if not candidate.verified:
        raise Refusal("CMD-G4-CANDIDATE-UNVERIFIED", candidate.signature)
    if candidate.authorized or candidate.actuated:
        raise Refusal("CMD-G4-CANDIDATE-AUTHORITY", "intent source must remain inert")
    body = {
        "schema": "ggen.cmd.intent.v1",
        "candidate_identity": candidate.signature,
        "operation": operation,
        "subject": subject,
        "subject_digest": subject_digest,
        "resource_scope": resource_scope,
        "jurisdiction": jurisdiction,
        "expected_postcondition": expected_postcondition,
        "required_authority": "exact-grant",
        "required_broker": "BRCE",
        "resource_budget": resource_budget,
        "expiry_epoch": expiry_epoch,
        "idempotency_key": idempotency_key,
        "retry_budget": 0,
        "circuit_state": "CLOSED",
        "andon": "GREEN",
        "expected_evidence": tuple(sorted(set(expected_evidence))),
        "direct_provider_call": False,
    }
    return {"identity": digest(body), **body}
