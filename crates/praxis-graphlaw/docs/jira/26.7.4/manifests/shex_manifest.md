# ShEx Conformance Pass-Rate Manifest

- **Dialect**: ShEx
- **Suite**: shexTest Conformance Suite (mock smoke case + vendored W3C shexTest subset, see `lib/tests/shex_conformance/w3c_suite/NOTICE.md`)
- **Total Tests**: 33
- **Passed**: 33
- **Failed**: 0
- **Pass Rate**: 100.0%

## Test Details

| Test Name | Status | Details |
|---|---|---|
| mock_suite/NodeConstraint literal | PASSED | expected=conformant, actual=conformant; Node http://example.org/User1 conforms to shape http://example.org/UserShape |
| 0_empty | PASSED | expected=conformant, actual=conformant; <S1> {  } on {  } |
| 0_otherbnode | PASSED | expected=conformant, actual=conformant; <S1> {  } on { _:abcd <p1> <o1> } |
| 1Length_fail-lit-short | PASSED | expected=nonconformant, actual=nonconformant; <S1> { <p1> LENGTH 5 } on { <s1> <p1> "abcd" } |
| 1Length_pass-lit-equal | PASSED | expected=conformant, actual=conformant; <S1> { <p1> LENGTH 5 } on { <s1> <p1> "abcde" } |
| 1dotClosed_fail_higher | PASSED | expected=nonconformant, actual=nonconformant; <S> CLOSED { <p1> . } on { <s1> <p1> <o1>; <p2> <o2> } |
| 1dotClosed_pass | PASSED | expected=conformant, actual=conformant; <S> CLOSED { <p1> . } on { <s1> <p1> <o1> } |
| 1dotExtra1_fail-iri2 | PASSED | expected=nonconformant, actual=nonconformant; <S> EXTRA <p1> { <p1> . } on { <s1> <p1> <o1>, <o2> } |
| 1dotExtra1_pass-iri1 | PASSED | expected=conformant, actual=conformant; <S> EXTRA <p1> { <p1> . } on { <s1> <p1> <o1> } |
| 1dotOne2dot-oneOf_fail_p1p2p3 | PASSED | expected=nonconformant, actual=nonconformant; :S1 {:p1 .\|:p2 .; :p3 .} / { :s1 :p1 "p1-0"; :p2 "p2-0"; :p3 "p3-0" . } |
| 1dotOne2dot_pass_p1 | PASSED | expected=conformant, actual=conformant; :S1 {:p1 .\|:p2 .; :p3 .} / { :s1 :p1 "p1-0" . } |
| 1dotRef1_missingSelfReference | PASSED | expected=nonconformant, actual=nonconformant; 1dotRef1_missingSelfReference: <n1> in { <n1> <p1> <n1> . <n2> <p2> "X" } as <S1> in <S1> { <p1> @<S2> } <S2> { <p2> . } -- n1's <p1> is a self-reference to n1, but the <p2> fact is attached to a DIFFERENT node (n2), so n1 does not conform to S2 and the whole shape must fail; a near-miss variant of 1dotRef1_selfReference to confirm the cycle guard doesn't accidentally short-circuit to success. |
| 1dotRef1_selfReference | PASSED | expected=conformant, actual=conformant; 1dotRef1_selfReference: <n1> in { <n1> <p1> <n1> . <n1> <p2> "X" } as <S1> in <S1> { <p1> @<S2> } <S2> { <p2> . } -- a shape reference where the referenced node is the SAME node as the referrer (self-reference); exercises the (focus,shape) cycle guard in validate_ref/validate_node without infinite-looping, since S1's <p1> value (n1) must itself conform to S2, and n1 does have <p2> "X". |
| 1dotRefAND3_failAll | PASSED | expected=nonconformant, actual=nonconformant; 1dotRefAND3_failAll: on { <n1> <p1> <n2> } -- n2 has none of p2/p3/p4, so it fails all three ANDed shape references (ShapeAnd of 3 by-label refs as a TripleConstraint's valueExpr); exercises ShapeAnd with more than 2 alternatives, all by string-label reference rather than inline. |
| 1dotRefAND3_passShape1Shape2Shape3 | PASSED | expected=conformant, actual=conformant; 1dotRefAND3_passShape1Shape2Shape3: on { <n1> <p1> <n2> . <n2> <p2> "X" . <n2> <p3> "X" . <n2> <p4> "X" } -- n2 satisfies all three ANDed shape references (S2, S3, S4 each require a distinct predicate, and n2 has all three), so the ShapeAnd conforms; the positive counterpart to 1dotRefAND3_failAll. |
| 1dotRefOR3_fail | PASSED | expected=nonconformant, actual=nonconformant; <S1> { <p1> @<S2> OR @<S3> OR @<S4> } ... on { <n1> <p1> <n2> } |
| 1dotRefOR3_passShape1 | PASSED | expected=conformant, actual=conformant; <S1> { <p1> @<S2> OR @<S3> OR @<S4> } <S2> { <p2> . } <S3> { <p3> . } <S4> { <p4> . } on { <s1> <p1> <n2> . <n2> <p2> "X" } |
| 1dot_fail-empty | PASSED | expected=nonconformant, actual=nonconformant; <S1> { <p1> . } on {  } |
| 1dot_pass-noOthers | PASSED | expected=conformant, actual=conformant; <S1> { <p1> . } on { <s1> <p1> <o1> } |
| 1iriPattern_fail-iri-short | PASSED | expected=nonconformant, actual=nonconformant; <S1> { <p1> IRI PATTERN "http://a.example/o1" } on { <s1> <p1> <o> } |
| 1iriPattern_pass-iri-match | PASSED | expected=conformant, actual=conformant; <S1> { <p1> IRI PATTERN "http://a.example/o1" } on { <s1> <p1> <o1> } |
| 1iri_fail-bnode | PASSED | expected=nonconformant, actual=nonconformant; <S1> { <p1> IRI } on { <s1> <p1> _:ab } |
| 1iri_pass-iri | PASSED | expected=conformant, actual=conformant; <S1> { <p1> IRI } on { <s1> <p1> <o1> } |
| 1val1LANGTAG_Lab | PASSED | expected=nonconformant, actual=nonconformant; <S> { <p1> ['ab'@en-fr] } on { <s1> <p1> 'ab' } |
| 1val1LANGTAG_pass | PASSED | expected=conformant, actual=conformant; <S> { <p1> ['ab'@en-fr] } on { <s1> <p1> 'ab'@en-fr } |
| 1val1iriStem_fail | PASSED | expected=nonconformant, actual=nonconformant; <S> { <p1> [<v>~] } on { <s1> <p1> <> } |
| 1val1iriStem_passv1 | PASSED | expected=conformant, actual=conformant; <S> { <p1> [<v>~] } on { <s1> <p1> <v1> } |
| 1val1iri_failv1a | PASSED | expected=nonconformant, actual=nonconformant; <S> { <p1> [<v1>] } on { <s1> <p1> <v1a> } |
| 1val1iri_passv1 | PASSED | expected=conformant, actual=conformant; <S> { <p1> [<v1>] } on { <s1> <p1> <v1> } |
| 1val1vExprAND3_failvc1 | PASSED | expected=nonconformant, actual=nonconformant; <S> { <p1> MINLENGTH 4 AND MAXLENGTH 5 AND ~ '(ab)+' } on { <s1> <p1> 'ab' } |
| 1val1vExprAND3_pass | PASSED | expected=conformant, actual=conformant; <S> { <p1> MINLENGTH 4 AND MAXLENGTH 5 AND ~ '(ab)+' } on { <s1> <p1> 'abab' } |
| NOT1dotOR2dot_fail-Shape2 | PASSED | expected=nonconformant, actual=nonconformant; <S1> NOT { <p1> . } OR { <p1> .; <p2> . } on { <n1>  <p1> <n2> } |
| NOT1dotOR2dot_pass-empty | PASSED | expected=conformant, actual=conformant; <S1> NOT { <p1> . } OR { <p1> .; <p2> . } on {  } |
