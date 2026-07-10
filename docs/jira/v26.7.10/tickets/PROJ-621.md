# PROJ-621 — Arazzo dialect (external API orchestration surface)

Status: DONE (session-verified via this session's green `just cng-test-bench` — 31 lib tests
+ integration suites passing, recorded in session logs; `RELEASE_CONTROL.md` Sec. 8)

## Summary

Clean-room Arazzo dialect (requirements mined from `/Users/sac/dev/wkflo-module` schemas and
the Arazzo 1.0.0 spec text; no code ports, per repo explore/exploit discipline): admit an
Arazzo workflow description as RDF (TTL vocabulary for workflow/step/criterion/
success-actions), project admitted Arazzo steps into PROJ-618 dispatch contracts executed
through the broker loopback adapter. Arazzo is the orchestration surface; POWL remains the
canonical workflow model. Unsupported spec features are refused by name (`CNG_R18`, 80/20
profile doctrine per `docs/standing/SEMANTIC_PROFILE_DOCTRINE.md`). One benchmark category
(`software-delivery` or new `api-orchestration`) exercises the Arazzo path end to end.

## Acceptance criteria

1. Arazzo descriptions admitted as RDF through a closed TTL vocabulary; unknown/unsupported
   features refuse by name with `CNG_R18` (negative test).
2. Admitted steps project into complete 20-field dispatch contracts (PROJ-618) and route
   through the broker only.
3. Arazzo registered in the dialect registry (PROJ-613) with all 8 invariant fields; it does
   not replace POWL anywhere.
4. One benchmark category exercises the Arazzo path end to end, receipted and replayable.

## Verification

`just cng-test-bench` after the wave lands: Arazzo admission, projection, and refusal-by-name
tests green; the Arazzo category present in the same-seed byte-identical bundle (PROJ-616).

## Links

- `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` Sec. 5, 7
- `docs/releases/v26.7.10/RELEASE_CONTROL.md` Sec. 8
