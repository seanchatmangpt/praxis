# PROJ-816: Bootstrap the autonomous iteration loop

**Status**: OPEN
**Dependencies**: PROJ-811

## Scope

Implement the loop architecture designed this session for continuing this milestone's backlog
(and future ones) autonomously:

- **Unit of work**: one backlog item → one commit, sized S/M/L. L items are pre-split before
  entering the queue (never handed to the loop raw).
- **Gate, tiered**:
  | Tier | When | Recipe |
  |---|---|---|
  | Commit gate | every tick | `just fmt-check` (or `fmt-check-pkg <crate>`) + crate-scoped `just check` |
  | Advance gate | every tick, before marking done | `just test-changed` |
  | Checkpoint gate | loop start/end + shared-surface items | `just verify-all` |
- **Mechanism**: `/loop` in self-paced mode, each tick invoking a `Workflow` script following
  this repo's existing `.claude/workflows/*.js` pattern (implement → gate → independent-audit
  phase — a second agent re-verifies the builder's own claims from scratch, per
  `no-overclaiming.md`/`verification-before-completion`). `ScheduleWakeup` is a supporting
  primitive only (polling a backgrounded `verify-all`), not the outer driver.
- **Task queue**: `TaskList` is the live pointer (seeded from this milestone's tickets with
  `addBlockedBy`/`addBlocks` edges matching the Dependencies column in `index.md`);
  `docs/jira/v26.8.16/tickets/PROJ-NNN.md` files are the durable record each commit also
  updates (Status field).
- **Stop conditions**: a gate failure the item's own scope can't absorb (mark `blocked` in
  TaskList with the failing command+output, per PROJ-811's own current state); an
  escalation-reserved item reached (none currently queued in this milestone — PROJ-811 itself is
  escalation-reserved and sits at the front of the queue, which is why the loop cannot start
  until a human resolves it); backlog exhaustion.

## Bootstrap steps

1. `TaskCreate` one task per PROJ-811..816, each `metadata: {size, gate_tier}` populated from
   this ticket's own descriptions.
2. Wire `addBlockedBy` edges matching the Dependencies column in `index.md` (PROJ-812/813/814/816
   → blocked by PROJ-811; PROJ-815 unblocked).
3. Confirm PROJ-811 is resolved (human decision made, gate passes) before the loop's first real
   tick — the loop itself should not attempt to resolve PROJ-811 autonomously, since that ticket
   is explicitly reserved for sign-off.
4. Once PROJ-811 clears, `/loop` can proceed through PROJ-815 (no dependency, can run any time)
   and then PROJ-812/813/814 in any order (all three are mutually independent once unblocked).

## Verification plan

The loop's own gate gates each of its ticks — no separate verification needed for this
bootstrap ticket beyond confirming `TaskList` reflects the dependency graph above correctly
before the first tick fires.
