#![cfg(test)]

//! Tests for the Mermaid `stateDiagram-v2` renderer over `DispatchState`
//! (`dispatch_diagram.rs`). The edge-completeness check cross-derives the
//! expected lawful set straight from `DispatchState::ALL` +
//! `DispatchState::lawful_to` — the same authority the
//! `sixteen_state_transition_law_is_exact` drift test in `dispatch_test.rs`
//! checks against — rather than duplicating a hand-copied literal edge
//! list, so this test cannot silently drift from the renderer's own source
//! of truth.

use std::collections::BTreeSet;

use chicago_tdd_tools::prelude::*;

use super::render_mermaid;
use crate::bench::dispatch::DispatchState;

test!(renders_state_diagram_v2_header, {
    // Arrange / Act
    let mermaid = render_mermaid();

    // Assert
    assert!(mermaid.starts_with("stateDiagram-v2\n"));
});

test!(renders_every_lawful_edge_and_nothing_else, {
    // Arrange: the authoritative lawful edge set, derived the same way the
    // dispatch_test.rs drift test derives it — NOT a copy-pasted literal.
    let mut expected: BTreeSet<(&str, &str)> = BTreeSet::new();
    for from in DispatchState::ALL {
        for to in DispatchState::ALL {
            if from.lawful_to(to) {
                expected.insert((from.as_str(), to.as_str()));
            }
        }
    }

    // Act
    let mermaid = render_mermaid();
    let rendered_edges: BTreeSet<(&str, &str)> = mermaid
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.starts_with("%%") || !line.contains("-->") {
                return None;
            }
            let (from, to) = line.split_once("-->")?;
            let (from, to) = (from.trim(), to.trim());
            if from == "[*]" || to == "[*]" {
                return None;
            }
            Some((from, to))
        })
        .collect();

    // Assert: exactly the lawful set, no extra, no missing.
    assert_eq!(rendered_edges, expected);
    assert_eq!(expected.len(), 22);
});

test!(renders_key_states, {
    // Arrange / Act
    let mermaid = render_mermaid();

    // Assert: the states named in the task's acceptance criteria are
    // present, plus the entry/terminal-exit wiring around them.
    for state in [
        "DISPATCH_READY",
        "BLOCKED",
        "COMPLETED",
        "MANUFACTURED",
        "UNKNOWN",
    ] {
        assert!(mermaid.contains(state), "missing state {state}");
    }
    assert!(mermaid.contains("[*] --> MANUFACTURED"));
    assert!(mermaid.contains("BLOCKED --> [*]"));
    assert!(mermaid.contains("COMPLETED --> [*]"));
});

test!(unknown_state_is_declared_but_has_no_transition_edges, {
    // Arrange / Act
    let mermaid = render_mermaid();

    // Assert: UNKNOWN appears (isolated declaration, part of the 16-state
    // vocabulary) but never as either side of a `-->` transition line — it
    // is declared-but-unreached per the `DispatchState::Unknown` doc
    // comment in dispatch.rs; no lawful edge ever names it.
    assert!(mermaid.contains("UNKNOWN"));
    for line in mermaid.lines() {
        if line.contains("-->") {
            assert!(
                !line.contains("UNKNOWN"),
                "UNKNOWN must not appear on a transition line: {line}"
            );
        }
    }
});

test!(all_sixteen_states_appear_in_the_diagram, {
    // Arrange / Act
    let mermaid = render_mermaid();

    // Assert: every declared DispatchState individual is rendered
    // somewhere (entry line, a transition edge, a terminal exit, or an
    // isolated declaration) — the diagram never silently drops a state.
    assert_eq!(DispatchState::ALL.len(), 16);
    for state in DispatchState::ALL {
        let name = state.as_str();
        assert!(mermaid.contains(name), "missing state {name} entirely");
    }
});

test!(two_consecutive_renders_are_byte_identical, {
    // Arrange / Act
    let first = render_mermaid();
    let second = render_mermaid();

    // Assert: determinism — no HashMap/HashSet iteration order, no wall
    // clock, no randomness anywhere in the render path, so two renders in
    // the same process (and across processes on the same binary) match
    // byte for byte.
    assert_eq!(first, second);
    assert_eq!(first.as_bytes(), second.as_bytes());
});
