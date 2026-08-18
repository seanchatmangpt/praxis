use praxis_reconciler::{
    execute_prepared, expected_receipt_digest, prepare_reconciliation, ActuationReceipt,
    AdmittedObservation, AuthorityGrant, ConstructedIntent, Observation, ReceiptedActuator,
    ReconcileEnvironment, Refusal, RefusalCode, RepairOperator, ReplayVerdict, ResidualVector,
};
use std::collections::{BTreeMap, BTreeSet};

struct Fixture {
    residual: u64,
    calls: u64,
}

impl Fixture {
    fn new(residual: u64) -> Self {
        Self { residual, calls: 0 }
    }

    fn operator() -> RepairOperator {
        RepairOperator {
            id: "repair-build".to_string(),
            targets: BTreeSet::from(["build".to_string()]),
            authority_scope: "praxis:repair/build".to_string(),
            reversible: true,
            estimated_cost: 1,
            expected_reduction: BTreeMap::from([("build".to_string(), 1)]),
        }
    }
}

impl ReceiptedActuator for Fixture {
    fn actuate_receipted(
        &mut self,
        intent: &ConstructedIntent,
        authority: &AuthorityGrant,
    ) -> Result<ActuationReceipt, Refusal> {
        self.calls += 1;
        let before = format!("state:{}", self.residual);
        self.residual = self.residual.saturating_sub(1);
        let after = format!("state:{}", self.residual);
        let mut receipt = ActuationReceipt {
            subject: intent.subject.clone(),
            construct_digest: intent.construct_digest.clone(),
            authority_grant_id: authority.grant_id.clone(),
            operator_id: intent.operator_id.clone(),
            before_identity: before,
            after_identity: after,
            changed: true,
            replay_key: format!("replay:{}", intent.construct_digest),
            receipt_digest: String::new(),
        };
        receipt.receipt_digest = expected_receipt_digest(&receipt)?;
        Ok(receipt)
    }

    fn replay(&self, receipt: &ActuationReceipt) -> Result<ReplayVerdict, Refusal> {
        Ok(ReplayVerdict {
            after_identity: receipt.after_identity.clone(),
            matched: true,
        })
    }
}

impl ReconcileEnvironment for Fixture {
    fn observe(&mut self) -> Result<Observation, Refusal> {
        Ok(Observation {
            subject: "repo:seanchatmangpt/praxis".to_string(),
            identity: format!("state:{}", self.residual),
            logical_time: self.calls,
            residuals: ResidualVector {
                dimensions: BTreeMap::from([("build".to_string(), self.residual)]),
            },
        })
    }

    fn available_operators(
        &self,
        _admitted: &AdmittedObservation,
    ) -> Result<Vec<RepairOperator>, Refusal> {
        Ok(vec![Self::operator()])
    }
}

fn must<T>(result: Result<T, Refusal>, context: &str) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("{context}: {error}"),
    }
}

fn authority_for(intent: &ConstructedIntent) -> AuthorityGrant {
    AuthorityGrant {
        grant_id: "grant:1".to_string(),
        subject: intent.subject.clone(),
        scopes: BTreeSet::from([intent.authority_scope.clone()]),
        construct_digest: intent.construct_digest.clone(),
    }
}

#[test]
fn no_authority_means_zero_actuation() {
    let mut fixture = Fixture::new(2);
    let prepared = must(
        prepare_reconciliation(&mut fixture),
        "fixture must prepare without DO",
    );
    let error = match execute_prepared(&mut fixture, prepared, None) {
        Ok(_) => panic!("missing authority must refuse"),
        Err(error) => error,
    };
    assert_eq!(error.code, RefusalCode::NoAuthority);
    assert_eq!(fixture.calls, 0);
    assert_eq!(fixture.residual, 2);
}

#[test]
fn exact_construct_authority_produces_receipt_and_replay() {
    let mut fixture = Fixture::new(2);
    let prepared = must(
        prepare_reconciliation(&mut fixture),
        "fixture must prepare without DO",
    );
    let authority = authority_for(&prepared.intent);

    let checkpoint = must(
        execute_prepared(&mut fixture, prepared, Some(&authority)),
        "exact grant must permit one receipted checkpoint",
    );
    assert_eq!(fixture.calls, 1);
    assert_eq!(fixture.residual, 1);
    assert!(checkpoint.receipt.changed);
    assert!(checkpoint.replay.matched);
    assert!(checkpoint.evidence.executed);
    assert!(checkpoint.evidence.verified);
}

#[test]
fn authority_bound_to_other_construct_refuses_before_do() {
    let mut fixture = Fixture::new(2);
    let prepared = must(
        prepare_reconciliation(&mut fixture),
        "fixture must prepare without DO",
    );
    let authority = AuthorityGrant {
        grant_id: "grant:wrong".to_string(),
        subject: prepared.intent.subject.clone(),
        scopes: BTreeSet::from([prepared.intent.authority_scope.clone()]),
        construct_digest: "not-the-current-construct".to_string(),
    };

    let error = match execute_prepared(&mut fixture, prepared, Some(&authority)) {
        Ok(_) => panic!("mismatched construct digest must refuse"),
        Err(error) => error,
    };
    assert_eq!(error.code, RefusalCode::ConstructMismatch);
    assert_eq!(fixture.calls, 0);
}

#[test]
fn stale_observation_refuses_before_do() {
    let mut fixture = Fixture::new(2);
    let prepared = must(
        prepare_reconciliation(&mut fixture),
        "fixture must prepare without DO",
    );
    let authority = authority_for(&prepared.intent);
    fixture.residual = 3;

    let error = match execute_prepared(&mut fixture, prepared, Some(&authority)) {
        Ok(_) => panic!("stale O* must refuse"),
        Err(error) => error,
    };
    assert_eq!(error.code, RefusalCode::StaleObservation);
    assert_eq!(fixture.calls, 0);
}


#[test]
fn prepared_tamper_refuses_before_do() {
    let mut fixture = Fixture::new(2);
    let mut prepared = must(
        prepare_reconciliation(&mut fixture),
        "fixture must prepare without DO",
    );
    let authority = authority_for(&prepared.intent);
    prepared.intent.operator_id.push_str(":tampered");

    let error = match execute_prepared(&mut fixture, prepared, Some(&authority)) {
        Ok(_) => panic!("tampered prepared object must refuse"),
        Err(error) => error,
    };
    assert_eq!(error.code, RefusalCode::PreparedMismatch);
    assert_eq!(fixture.calls, 0);
}

#[test]
fn receipt_tamper_is_detected() {
    let mut fixture = Fixture::new(2);
    let prepared = must(
        prepare_reconciliation(&mut fixture),
        "fixture must prepare without DO",
    );
    let authority = authority_for(&prepared.intent);
    let mut receipt = must(
        fixture.actuate_receipted(&prepared.intent, &authority),
        "fixture emits receipt",
    );
    receipt.after_identity.push_str(":tampered");

    let error = match praxis_reconciler::verify_receipt(&prepared.intent, &authority, &receipt) {
        Ok(()) => panic!("tamper must break the digest"),
        Err(error) => error,
    };
    assert_eq!(error.code, RefusalCode::ReceiptMismatch);
}
