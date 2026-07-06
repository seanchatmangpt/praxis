use crate::term::{Triple, VarOrTerm};
use crate::Encoder;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct BodyLiteral {
    pub negated: bool,
    pub pattern: Triple,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AggregateFunction {
    Count,
    Sum,
    Min,
    Max,
    Avg,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Aggregate {
    pub function: AggregateFunction,
    pub source_var: String,
    pub target_var: String,
    pub group_vars: Vec<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Rule {
    pub body: Vec<BodyLiteral>,
    pub head: Triple,
}

/// Sentinel head predicate marking a denial/consistency-check rule (`{ body
/// } => false.`, e.g. SKOS's disjointness constraints) -- a real N3/RIF
/// idiom for integrity constraints rather than fact derivation, distinct
/// from ordinary rule assertion. Reusing `Rule.head: Triple` with a
/// dedicated marker predicate (rather than adding a new field/enum to
/// `Rule` itself) avoids changing every one of the ~18 existing call sites
/// across the workspace that construct a `Rule` literal for the ordinary
/// assertion case -- the same predicate-string-dispatch technique this
/// engine already uses for `log:implies`/`log:notIncludes`/
/// `log:collectAllIn` (see `reasoner.rs`'s `find_log_*_literal` helpers).
pub const DENIAL_HEAD_MARKER: &str = "<http://www.w3.org/2000/10/swap/log#__denial__>";

impl Rule {
    /// Construct a denial rule's `Rule` value for the parser: `body` with a
    /// head that's always the reserved `DENIAL_HEAD_MARKER` sentinel triple
    /// (its s/o carry no meaning and are never inspected).
    pub fn new_denial(body: Vec<BodyLiteral>) -> Rule {
        let marker = VarOrTerm::new_term(DENIAL_HEAD_MARKER.to_string());
        Rule {
            body,
            head: Triple { s: marker.clone(), p: marker.clone(), o: marker, g: None },
        }
    }

    /// Whether this rule is a denial/consistency-check rule (`=> false.`)
    /// rather than an ordinary fact-asserting rule.
    pub fn is_denial(&self) -> bool {
        self.head.p.is_term()
            && Encoder::decode(&self.head.p.to_encoded()).as_deref() == Some(DENIAL_HEAD_MARKER)
    }
}
