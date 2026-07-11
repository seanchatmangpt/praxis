//! Mermaid `stateDiagram-v2` renderer for the 16-state cross-engine
//! `DispatchState` machine (`dispatch::DispatchState`, PROJ-720/PROJ-720
//! doc comment in `dispatch.rs`).
//!
//! This is a pure projection, not a second copy of the law: every state and
//! every edge is derived by enumerating `DispatchState::ALL` against
//! `DispatchState::lawful_to` — the SAME method the drift test
//! `sixteen_state_transition_law_is_exact` (`dispatch_test.rs`) enumerates
//! against to verify the table matches its doc comment. There is no
//! hand-copied transition list in this file to drift out of sync with
//! `dispatch.rs`; if `lawful_to` changes, this renderer's output changes
//! with it on the next build.
//!
//! Determinism: state enumeration is the fixed-order `DispatchState::ALL`
//! array; edges are collected into a `BTreeSet` (lexicographic order, never
//! `HashMap`/`HashSet` iteration order); no wall clock, no randomness. Two
//! calls to [`render_mermaid`] in the same process — or in two separate
//! processes on the same binary — produce byte-identical strings.

use std::collections::BTreeSet;

use super::dispatch::DispatchState;

/// One lawful `(from, to)` edge, keyed by the states' `as_str()` names so
/// ordering is lexicographic on the rendered vocabulary, not on enum
/// declaration order.
type Edge = (&'static str, &'static str);

/// Enumerates every lawful edge in `DispatchState::lawful_to`'s table by
/// checking all 16×16 ordered pairs of `DispatchState::ALL` — the exact
/// method `sixteen_state_transition_law_is_exact` (`dispatch_test.rs`) uses
/// to verify the table — so this function and that test can never disagree
/// about what the lawful set is.
///
/// # Complexity
/// O(|states|^2) = 256 fixed `lawful_to` checks; `BTreeSet` insert is
/// O(log E) per edge, E ≤ 256, so total O(|states|^2 log |states|^2) —
/// trivial at this fixed state count.
fn lawful_edges() -> BTreeSet<Edge> {
    let mut edges = BTreeSet::new();
    for from in DispatchState::ALL {
        for to in DispatchState::ALL {
            if from.lawful_to(to) {
                edges.insert((from.as_str(), to.as_str()));
            }
        }
    }
    edges
}

/// Whether `state` is terminal in the lawful table: no `DispatchState` it
/// can lawfully advance to. O(|states|) fixed checks (16).
fn is_terminal(state: DispatchState) -> bool {
    DispatchState::ALL
        .into_iter()
        .all(|to| !state.lawful_to(to))
}

/// Renders the 16-state `DispatchState` machine as a Mermaid
/// `stateDiagram-v2` string:
///
/// 1. `[*] --> MANUFACTURED` — the sole entry state (declaration order
///    invariant: `DispatchState::ALL[0]`).
/// 2. One `FROM --> TO` line per lawful edge, sorted lexicographically
///    (`BTreeSet` order).
/// 3. One `STATE --> [*]` line per terminal state that some lawful edge
///    actually reaches (`COMPLETED`, `BLOCKED`) — a terminal state with no
///    inbound lawful edge is not drawn as reachable exit.
/// 4. An isolated declaration for any state that appears in neither (1),
///    (2), nor (3) — today exactly `UNKNOWN`, the declared-but-unreached
///    16th `disp:DispatchState` individual (see the `Unknown` variant's
///    doc comment in `dispatch.rs`): no lawful edge ever names it, so it
///    would otherwise be silently absent from the diagram despite being
///    part of the 16-state vocabulary.
///
/// # Complexity
/// O(|states|^2) edge derivation (see [`lawful_edges`]) + O(E) terminal /
/// isolated-state classification + O(|output|) string building. E ≤ 256,
/// |states| = 16 — all fixed, not input-dependent.
pub fn render_mermaid() -> String {
    let edges = lawful_edges();

    let mut mentioned: BTreeSet<&'static str> = BTreeSet::new();
    let entry = DispatchState::Manufactured.as_str();
    mentioned.insert(entry);
    for (from, to) in &edges {
        mentioned.insert(from);
        mentioned.insert(to);
    }

    let mut out = String::new();
    out.push_str("stateDiagram-v2\n");
    out.push_str(&format!("    [*] --> {entry}\n"));
    for (from, to) in &edges {
        out.push_str(&format!("    {from} --> {to}\n"));
    }

    // Terminal states reached by at least one lawful edge get a `--> [*]`
    // exit arrow, sorted for determinism (BTreeSet, not enum order).
    let mut reachable_terminals: BTreeSet<&'static str> = BTreeSet::new();
    for state in DispatchState::ALL {
        let name = state.as_str();
        if is_terminal(state) && mentioned.contains(name) {
            reachable_terminals.insert(name);
        }
    }
    for name in &reachable_terminals {
        out.push_str(&format!("    {name} --> [*]\n"));
    }

    // Any state neither entered, transitioned through, nor exited above is
    // an isolated vocabulary member (today: UNKNOWN) — declared so the
    // diagram still shows all 16 states, annotated so a reader does not
    // mistake the isolation for an omission.
    for state in DispatchState::ALL {
        let name = state.as_str();
        if !mentioned.contains(name) && !reachable_terminals.contains(name) {
            out.push_str(&format!(
                "    %% {name}: declared disp:DispatchState individual, no lawful edge ever names it (dispatch.rs DispatchState doc comment)\n"
            ));
            out.push_str(&format!("    {name}\n"));
        }
    }

    out
}

#[cfg(test)]
#[path = "dispatch_diagram_test.rs"]
mod dispatch_diagram_test;
