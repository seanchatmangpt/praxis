# AB-10 — Acceptance Traceability (Fixtures to Auditor Verdict)

| Facet | Value |
|---|---|
| Invariant | Error paths tested as rigorously as happy paths — every Refusal variant traces to a fixture and a gate |
| Information-Loss Risk | An untraced test is unauditable evidence; the chain fixture -> harness -> variant -> gate is explicit |
| TPS Purpose | Genchi genbutsu: the auditor's verdict is grounded in re-runnable fixtures, not summaries |
| DfLSS CTQ | Every gate cites the exact command run this session; no verdict from hearsay |
| CENG Boundary | The auditor consumes only gate outputs and the standing index — never prior-agent claims |

```mermaid
flowchart TD
    F[Fixture directories] --> F1[happy-path envelopes]
    F --> F2[refusal fixtures per variant]
    F --> F3[replay fixtures with tampered fields]

    F1 --> H1[Pipeline harness S1-S6]
    F2 --> H2[Negative-test harness]
    F3 --> H3[Replay harness AB-07]

    H1 --> V1[AdmittedTransition + receipt_root]
    H2 --> V2[Typed Refusal variants ~29]
    H3 --> V3[Per-field ReplayMismatch]

    V1 --> G1[Gate: determinism 5x byte-identical]
    V2 --> G2[Gate: every Refusal variant covered]
    V3 --> G3[Gate: fail-fast field naming]

    G1 --> S[Standing index refresh]
    G2 --> S
    G3 --> S
    S --> A[Auditor verdict — scoped readiness claim]
```
