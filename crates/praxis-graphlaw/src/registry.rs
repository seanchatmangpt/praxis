//! Process-wide singleton side tables backing the synthetic list/formula
//! terms defined in `crate::term`. These are global `static`s (not fields
//! on `TripleStore`), so their state is shared across *every* `TripleStore`
//! instance in the process, not scoped to one store. This is known, pre-
//! existing coupling (see `crate::term`'s module doc for the design
//! rationale behind representing lists/formulas as synthetic blank nodes
//! keyed into these tables) and is not being fixed in this pass -- this
//! module only relocates the statics verbatim from their prior home in
//! `triples.rs`.

use crate::term::{Triple, VarOrTerm};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::Mutex;

pub(crate) static LIST_REGISTRY: Lazy<Mutex<HashMap<usize, Vec<VarOrTerm>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
// Reverse index for `new_list`: structurally-identical member sequences
// must always resolve to the SAME list id, not a fresh one each call --
// otherwise a rule that reconstructs a list term in its head (e.g.
// `?X :is (:good ?Y)` as a consequent) mints a brand-new, never-equal-to-
// anything-previous id on every fixpoint iteration, and `materialize()`
// never converges (each iteration "derives" a structurally-identical but
// id-different fact forever). Found the hard way: a real infinite loop
// while closing the EYE `good_cobbler` corpus case's list-substitution gap.
pub(crate) static LIST_INTERN: Lazy<Mutex<HashMap<Vec<VarOrTerm>, usize>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
pub(crate) static FORMULA_REGISTRY: Lazy<Mutex<HashMap<usize, Vec<Triple>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
pub(crate) static SYNTHETIC_COUNTER: AtomicUsize = AtomicUsize::new(0);
