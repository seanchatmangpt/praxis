//! Failure-Process Trajectory Analysis -- Wire-phase-0, standalone.
//!
//! **Not part of the 30-family v26.7.12 architecture atlas** documented in this crate's
//! `lib.rs`. This module carries no `V12-0XX` ticket, is not a crown-witness edge, and does
//! not read or write `docs/jira/v26.7.12/CROWN_STATUS.md` -- it only *cites* that file's
//! existing text in doc comments and test data, as a real example of the phenomenon this
//! module analyzes. No caller wires this module into the crown-witness drivers
//! (`crown_local.rs` / `crown_external.rs`) as of this writing; that is future, undone work,
//! disclosed here rather than hidden.
//!
//! # What this implements
//!
//! Zhao, Li, Li, Zhao, Barr & Sarro & Ye, "Failure as a Process: An Anatomy of CLI Coding
//! Agent Trajectories" (arXiv:2607.09510), frame every failing agent run around three
//! timestamps -- t_err (the decisive/root-cause error), t_lock (the point after which no
//! correct recovery is observed), t_obs (the first observable failure signal) -- plus two
//! derived intervals: fix window = t_lock - t_err (recovery was still possible) and
//! observability lag = t_obs - t_lock (the failure was real but invisible). The paper's own
//! headline finding (F2/F11) is that a fix window usually exists and usually goes unused,
//! and that the gap between successful and failed trajectories is whether the agent *acts*
//! on an observed error signal, not whether one occurs.
//!
//! This module re-grounds that framework in data that already exists in this repository
//! today, rather than in a live agent trace (no such trace exists for this repo's own
//! engineering process, and inventing one would be new instrumentation, which this pass
//! was explicitly asked not to build). A **trajectory** here is the ordered slice of this
//! repo's own `git log` history for one **claim**: a commit asserted something (e.g. "this
//! call graph edge is a `REAL_EDGE`"), zero or more later commits built on that assertion
//! without re-checking it, and eventually an independent verdict -- a fix commit, or an
//! external audit that has not yet produced one -- either confirmed or falsified it.
//!
//! - **t_err** = the commit whose own claim a later, independent verdict falsifies.
//! - **t_lock** = the last commit, among those the caller identifies as having built on the
//!   claim without re-verifying it, before the verdict landed. If the caller supplies no
//!   such dependent commits, t_lock = t_err itself -- a real, reportable zero-length fix
//!   window, not a missing value.
//! - **t_obs** = the verdict event: either a fix commit's own author timestamp, or (when no
//!   fix commit exists yet -- which is this module's own worked example's actual state) an
//!   externally-supplied observation instant and description.
//!
//! # Why session-git-history + refusal-chain, not OTel spans or task-output JSON
//!
//! Four independent design proposals were produced for this milestone. Two of them (spans
//! over an OTel/OCEL pipeline; parsing `tasks/*.output` Workflow-harness JSON) require data
//! that either needs new query/instrumentation wiring or lives in an ephemeral,
//! session-scoped scratch directory not guaranteed to exist across sessions. The other two
//! -- treating `git log` as the trajectory, and treating "does a downstream call actually
//! read the upstream struct it `?`-gated on" as the structural symptom worth flagging --
//! are directly exercisable against data that is real, immutable, and already sitting in
//! this repository's own object store, with zero new instrumentation. This module grafts
//! both: [`CommitRecord`]/[`FailureTrajectory`] carry the git-history skeleton, and
//! [`StageDataThreading`] carries the "control-sequenced but not data-threaded" structural
//! check as one source of `evidence` for a [`ClaimVerdict`] (see the worked example at the
//! bottom of this file, which uses it against the module's own real dogfood case:
//! `crown_local.rs`'s `F18 -> F19` edge).
//!
//! # No wall clock, anywhere in this module
//!
//! Every timestamp this module touches is caller-supplied: either `git log`'s own
//! `%aI`/`%at` author-date fields (an already-committed, content-addressed object's
//! historical record -- read once as input data, the same way this workspace's
//! `chatman-common::git_runtime` module reads git state via `std::process::Command` without
//! ever treating the *output* as anything but data), or an externally-recorded observation
//! instant the caller supplies as a literal. This module calls neither `SystemTime::now()`
//! nor `Instant::now()` anywhere -- grep confirms zero occurrences in this file. This
//! mirrors, in substance, this repo's "no wall clock in hash/receipt paths, time only from
//! graph-asserted literals" invariant, applied here to git-asserted literals instead of
//! RDF/OWL-Time literals, since this module has no RDF graph to draw from.
//!
//! # Worked example: this repo's own F18 -> F19 dogfood case
//!
//! `crates/multifractal-workflow/src/crown_local.rs:572-580` calls
//! `resolve_hook_for_action(&run.hook_pack_turtle, &ground_action, &mut hook_ledger)`
//! (see `f19_hooks.rs:443-447` for its real signature -- it takes no `BrokerReceipt`-typed
//! or `BrokerReceipt`-derived argument at all) immediately after
//! `dispatch_local_execution_via_broker` succeeds and binds `broker_receipt`
//! (`crown_local.rs:554-564`). That is real control sequencing (the `?` on
//! `dispatch_local_execution_via_broker` gates whether `resolve_hook_for_action` runs at
//! all) with zero data threading (none of `BrokerReceipt`'s seven public fields, listed at
//! `f18_broker_law.rs:374-382`, is read anywhere between the two calls). Commit `eeca952a`
//! (2026-07-12T15:44:25-07:00) wired this and, two minutes later via commit `77da318b`,
//! `docs/jira/v26.7.12/CROWN_STATUS.md:73` classified the edge `REAL_EDGE`. Eight further
//! commits (`77da318b` through `66cb59b1`, spanning 2026-07-12T15:46:34-07:00 through
//! 2026-07-12T17:08:50-07:00) built the rest of the LOCAL crown-witness chain on top of that
//! claim without re-checking it. No commit touching `crown_local.rs` (the file the disputed
//! call site lives in) landed after `66cb59b1` through this module's own commit-range cutoff
//! (`b69f9959`, 2026-07-12T21:07:44-07:00) -- the remaining commits in that window are
//! `CROWN_STATUS.md`-only documentation of the separately-wired EXTERNAL witness tail. The
//! `f18_f19_case_study` test at the bottom of this file runs this module's own
//! [`FailureTrajectory::compute_failure_window`] over the real, `git log`-sourced commit
//! records for exactly this case and asserts the exact computed fix-window and
//! observability-lag values. (This crate sets `doctest = false` in its `Cargo.toml`, so a
//! `#[test]` is the honest choice here, not a doc-example a `cargo test --doc` run would
//! silently skip.)
//!
//! # Limitations (disclosed, not hidden)
//!
//! - **Verdicts are never inferred.** `git log` has no ground truth of correctness -- only
//!   what was claimed. Whether a claim was an `Overclaim` must come from an external audit
//!   (a human, or another tool); this module will never itself discover that a claim was
//!   wrong. Auto-inferring a verdict from commit text alone would risk becoming exactly the
//!   kind of fabricated-confidence failure (paper F12) this framework exists to catch.
//! - **Dependencies are never inferred.** "Commit X built on commit W's unverified claim" is
//!   a source-level data-flow fact (established here by directly reading `crown_local.rs`'s
//!   real argument lists, not by a commit-message or same-file heuristic). A same-file/
//!   same-keyword auto-linker was deliberately not built: it would itself be an unverified
//!   claim about which commits "count," laundering exactly the kind of overclaim this module
//!   is meant to surface. [`AssumedDependency`] is caller-supplied for this reason.
//! - **Timestamps are self-reported and rewritable.** `git commit --amend` / `rebase`
//!   changes author dates; this is weaker evidence than this repo's BLAKE3 receipt
//!   invariant and must never be presented as tamper-proof.
//! - **t_lock selection is a judgment call**, not a formula, exactly as the paper's own
//!   t_lock is defined via retrospective human-annotator agreement rather than a mechanical
//!   rule. This module computes t_lock mechanically *given* the caller's own
//!   [`AssumedDependency`] list (max timestamp among them), but which commits belong on that
//!   list is a human/audit judgment -- in the worked example, "the LOCAL crown-witness chain
//!   stopped building on the disputed edge and moved to documenting the separately-wired
//!   EXTERNAL tail" is disclosed as the selection rationale, not proven optimal.
//! - **[`StageDataThreading`]'s field lists are curated by direct source reading, not
//!   extracted by an AST parser.** No `syn`-based (or other) static extractor exists in this
//!   crate. A future Wire-phase could replace manual curation with real static extraction;
//!   this module does not claim that capability exists today.
//! - **Granularity mismatch with the paper.** The paper's "step" is one tool call inside a
//!   single continuous agent turn (minutes, median t_err = step 7). This module's unit is a
//!   commit, and this repo's own worked example spans hours across dozens of commits.
//!   Commit counts and the paper's step counts are analogous in shape (early decisive error,
//!   short-then-closed fix window, late observability -- F1/F2/F3 analogs), never
//!   numerically comparable.
//! - **No proactive/real-time detection.** This module can only compute a failure window
//!   for a claim someone already re-audited and disagreed with. An overclaim nobody ever
//!   re-checks is invisible to this design by construction -- the same blind spot the
//!   paper's own prefix-only monitor has (F4: 3.7-8.7% real-time recall without deeper
//!   context). This module does not attempt to beat that ceiling; it does not attempt
//!   real-time detection at all.
//! - **n = 1.** This module's only worked, fully-real example is the F18->F19 case. No
//!   median, precision, or recall claim is made anywhere in this file -- doing so from a
//!   single case would itself be an overclaim of the kind this module exists to catch.

use std::collections::{BTreeMap, BTreeSet};

// ---------------------------------------------------------------------------------------
// Structural evidence: "control-sequenced but not data-threaded" (grafted from the
// refusal-chain / unread-consequence design angle).
// ---------------------------------------------------------------------------------------

/// One field-level data-threading check for a documented "upstream stage produces struct
/// `X`, downstream stage function `?`-gates on `X` existing" edge: which of `X`'s fields
/// the downstream call site actually reads before producing its own result.
///
/// Field lists here are curated by direct source reading (cite the exact file:line in the
/// owning [`ClaimVerdict::evidence`]), not extracted by an AST parser -- see this module's
/// top-level Limitations. The predicate itself ([`Self::is_unread_consequence`]) is real,
/// reusable, and independently testable regardless of how the field lists were obtained.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageDataThreading {
    /// Name of the upstream struct type the downstream call `?`-gates on existing.
    pub upstream_type: &'static str,
    /// Name of the downstream function whose call site is being checked.
    pub downstream_fn: &'static str,
    /// The upstream struct's real, public fields (its full field list, or the subset the
    /// auditor confirmed exists -- see the curating [`ClaimVerdict::evidence`]).
    pub upstream_fields_available: Vec<&'static str>,
    /// Which of `upstream_fields_available` are actually referenced at or before the
    /// downstream call site, per direct source reading.
    pub upstream_fields_read: Vec<&'static str>,
}

impl StageDataThreading {
    /// True when the upstream struct's fields were available to the downstream call
    /// (`upstream_fields_available` non-empty) but none were read
    /// (`upstream_fields_read` empty) before the downstream call produced its own result --
    /// real control sequencing (a `?`-gate on the struct's existence) without data
    /// threading. One structural proxy for the paper's "ignored signal" / "premature
    /// action" epistemic-error categories (F5), not a general classifier for the rest of
    /// them.
    ///
    /// # Complexity
    /// O(1): two `Vec::is_empty` checks.
    pub fn is_unread_consequence(&self) -> bool {
        !self.upstream_fields_available.is_empty() && self.upstream_fields_read.is_empty()
    }
}

// ---------------------------------------------------------------------------------------
// Git-history skeleton.
// ---------------------------------------------------------------------------------------

/// One commit, as `git log` itself reports it -- never generated or guessed by this
/// module. `authored_at_unix`/`authored_at_iso` are `git`'s own record of an
/// already-committed, content-addressed object's author date (`%at`/`%aI`), read once as
/// historical input data. `touched_paths` is caller-supplied (e.g. from a separate
/// `git show --stat <sha>` per commit); [`parse_git_log_plumbing`] always leaves it empty
/// (see that function's own doc for why) -- use [`FailureTrajectory::attach_touched_paths`]
/// to fill it in afterward.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitRecord {
    /// Full 40-hex commit SHA.
    pub sha: String,
    /// Author date, Unix seconds (`git log --format=%at`).
    pub authored_at_unix: i64,
    /// Author date, ISO-8601 (`git log --format=%aI`) -- kept alongside the Unix form
    /// purely for human-readable display; all comparisons in this module use
    /// `authored_at_unix`.
    pub authored_at_iso: String,
    /// Commit subject line (`git log --format=%s`).
    pub subject: String,
    /// Paths this commit's diff touched. Empty unless the caller populated it (see struct
    /// doc above).
    pub touched_paths: Vec<String>,
}

/// Parse the newline-delimited, tab-separated plumbing output of:
///
/// ```text
/// git log --format='%H%x09%aI%x09%at%x09%s' --reverse -- <path> [<path>...]
/// ```
///
/// `--reverse` is required so lines arrive oldest-first, matching
/// [`FailureTrajectory::validate`]'s chronological-order requirement. Each line's four
/// tab-separated fields are: full 40-hex SHA (`%H`), author date as ISO-8601 (`%aI`),
/// author date as Unix seconds (`%at`), and the commit subject (`%s`). This function does
/// not itself invoke `git` -- `text` is caller-supplied, exactly as this module's top-level
/// doc explains for every timestamp it touches.
///
/// [`CommitRecord::touched_paths`] is left empty for every parsed record: this single-line
/// plumbing format carries no file list, and combining it with `--name-only`'s
/// multi-line-per-commit grouping was judged more parsing-logic risk than this Wire-phase-0
/// pass should take on unverified (no `cargo test` run backs this file yet -- see the crate
/// root's build-hygiene note). A caller that needs `touched_paths` populated should attach
/// them separately, e.g. via [`FailureTrajectory::attach_touched_paths`] fed from a
/// per-commit `git show --stat`. Disclosed as a real, present scope limitation, not hidden
/// behind a placeholder that looks like a real answer.
///
/// # Complexity
/// O(L) where L = the number of newline-terminated lines in `text`. One pass, no nested
/// scan; blank lines are skipped.
///
/// # Errors
/// - [`TrajectoryRefusal::MalformedGitLogLine`] if a non-empty line does not split into
///   exactly 4 tab-separated fields.
/// - [`TrajectoryRefusal::MalformedUnixTimestamp`] if the third field does not parse as
///   `i64`.
pub fn parse_git_log_plumbing(text: &str) -> Result<Vec<CommitRecord>, TrajectoryRefusal> {
    let mut out = Vec::new();
    for (idx, line) in text.lines().enumerate() {
        if line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() != 4 {
            return Err(TrajectoryRefusal::MalformedGitLogLine {
                line_number: idx + 1,
                field_count: fields.len(),
            });
        }
        let authored_at_unix =
            fields[2]
                .parse::<i64>()
                .map_err(|e| TrajectoryRefusal::MalformedUnixTimestamp {
                    line_number: idx + 1,
                    value: fields[2].to_string(),
                    reason: e.to_string(),
                })?;
        out.push(CommitRecord {
            sha: fields[0].to_string(),
            authored_at_iso: fields[1].to_string(),
            authored_at_unix,
            subject: fields[3].to_string(),
            touched_paths: Vec::new(),
        });
    }
    Ok(out)
}

// ---------------------------------------------------------------------------------------
// Claims and verdicts (caller-supplied; never inferred -- see module Limitations).
// ---------------------------------------------------------------------------------------

/// A claim made by one commit: `commit_sha` asserted `claim_text` under the identifier
/// `claim_id` (e.g. `"F18->F19:REAL_EDGE"`, matching a `docs/jira/v26.7.12/CROWN_STATUS.md`
/// row's own edge naming, cited for orientation only -- this module never parses that file).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Claim {
    /// Full 40-hex SHA of the commit that made this claim.
    pub commit_sha: String,
    /// Stable identifier for the thing being claimed about (a call-graph edge, a ticket, a
    /// receipt property -- whatever the caller's domain names it).
    pub claim_id: String,
    /// Human-readable summary of what was claimed, for [`FailureWindow`] display. A
    /// paraphrase with a file:line citation is preferred over a long verbatim quote from
    /// project docs (this module's own no-overclaiming-rust convention: cite, don't
    /// reproduce at length).
    pub claim_text: String,
}

/// Where a claim's verdict came from: a real fix commit (this repo already produces this
/// shape for other, unrelated overclaims -- e.g. commit `62e2e0b6`), or an external
/// observation for a claim that has not yet produced one -- which is this module's own
/// worked example's actual, current state (`git status` on `crown_local.rs` and
/// `CROWN_STATUS.md` is clean as of this module's own authoring session; no fix commit
/// exists for the F18->F19 overclaim yet).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerdictSource {
    /// SHA of the commit that fixed/corrected the claim.
    Commit(String),
    /// A verdict recorded outside `git` -- a human or agent audit's own finding, with its
    /// own recorded instant. `observed_at_unix`/`observed_at_iso` are caller-supplied data
    /// (e.g. a workflow harness's own recorded completion timestamp for the audit run),
    /// never a live clock read performed by this module.
    External {
        observed_at_unix: i64,
        observed_at_iso: String,
        description: String,
    },
}

/// Whether an independent re-check agreed with the original claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// The claim held up under re-check.
    Confirmed,
    /// The claim did not hold up -- this is the case [`FailureTrajectory::compute_failure_window`]
    /// computes a [`FailureWindow`] for.
    Overclaim,
}

/// One claim plus the verdict an independent re-check reached about it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimVerdict {
    pub claim: Claim,
    pub verdict: Verdict,
    pub verdict_source: VerdictSource,
    /// Free-text justification for `verdict`, citing exact file:line where possible (this
    /// repo's own no-overclaiming convention: "BLOCKED (cite file:line)").
    pub evidence: String,
    /// Optional structural backing for `evidence`, when the overclaim has the specific
    /// "control-sequenced but not data-threaded" shape [`StageDataThreading`] checks for.
    /// `None` for claims this shape does not apply to -- most overclaims are not this
    /// specific structural symptom (see module Limitations on epistemic-error coverage).
    pub structural_evidence: Option<StageDataThreading>,
}

/// A caller-asserted fact: `dependent_sha` built on or reaffirmed `claim_id`'s claim
/// without re-verifying it. Never inferred by this module (see module Limitations) --
/// establishing this requires reading `dependent_sha`'s actual diff/source, which is an
/// audit-level, source-level judgment this module deliberately does not attempt to
/// automate from commit metadata alone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssumedDependency {
    pub claim_id: String,
    pub dependent_sha: String,
}

// ---------------------------------------------------------------------------------------
// The trajectory and its computed failure window.
// ---------------------------------------------------------------------------------------

/// A trajectory: a chronologically-ordered slice of this repo's own commit history, plus
/// the claims made within it and the caller's own record of which later commits built on
/// which claims. All three inputs are independent (claims and dependencies are never
/// inferred from `commits` -- see module Limitations), mirroring the paper's own
/// distinction between what a prefix-only monitor can see from trace text alone (F4) and
/// what only an external check can supply.
#[derive(Debug)]
pub struct FailureTrajectory {
    /// Commit history, ascending by `authored_at_unix` (oldest first). Typically produced
    /// by [`parse_git_log_plumbing`] against a `git log --reverse` invocation, but any
    /// caller-constructed `Vec<CommitRecord>` in ascending order is accepted.
    pub commits: Vec<CommitRecord>,
    pub claims: Vec<ClaimVerdict>,
    pub dependencies: Vec<AssumedDependency>,
}

impl FailureTrajectory {
    /// Returns `self` with [`CommitRecord::touched_paths`] filled in for every commit whose
    /// `sha` is a key in `paths_by_sha`. Commits absent from `paths_by_sha` are left
    /// unchanged (typically still empty, per [`parse_git_log_plumbing`]'s disclosed gap).
    ///
    /// # Complexity
    /// O(C) where C = `self.commits.len()`: one `BTreeMap` lookup per commit.
    pub fn attach_touched_paths(mut self, paths_by_sha: &BTreeMap<String, Vec<String>>) -> Self {
        for commit in &mut self.commits {
            if let Some(paths) = paths_by_sha.get(&commit.sha) {
                commit.touched_paths = paths.clone();
            }
        }
        self
    }

    /// O(C): linear scan for the commit with this SHA. `commits` is a small
    /// (single-digit-to-low-hundreds), caller-scoped range in every use this module
    /// anticipates (a git-log slice for one file or one claim's neighborhood, not the whole
    /// repo's history); a `BTreeMap<sha, &CommitRecord>` index was judged unnecessary
    /// complexity for this Wire-phase-0 pass -- a disclosed trade-off, not an oversight.
    fn find_commit(&self, sha: &str) -> Option<&CommitRecord> {
        self.commits.iter().find(|c| c.sha == sha)
    }

    /// Structural validation shared by every public computation on this trajectory.
    ///
    /// # Complexity
    /// O(C) for the chronological-order pass, plus O((K + D) * C) for the claim/dependency
    /// existence checks (K = `claims.len()`, D = `dependencies.len()`), since each check is
    /// an O(C) [`Self::find_commit`] lookup -- see that method's own complexity note on why
    /// no index is built.
    fn validate(&self) -> Result<(), TrajectoryRefusal> {
        if self.commits.is_empty() {
            return Err(TrajectoryRefusal::EmptyCommitRange);
        }
        for window in self.commits.windows(2) {
            if window[1].authored_at_unix < window[0].authored_at_unix {
                return Err(TrajectoryRefusal::CommitRangeNotChronological {
                    sha: window[1].sha.clone(),
                });
            }
        }
        let mut seen_claim_ids: BTreeSet<&str> = BTreeSet::new();
        for cv in &self.claims {
            if !seen_claim_ids.insert(cv.claim.claim_id.as_str()) {
                return Err(TrajectoryRefusal::DuplicateClaimId {
                    claim_id: cv.claim.claim_id.clone(),
                });
            }
            if self.find_commit(&cv.claim.commit_sha).is_none() {
                return Err(TrajectoryRefusal::ClaimCommitNotInRange {
                    claim_id: cv.claim.claim_id.clone(),
                    commit_sha: cv.claim.commit_sha.clone(),
                });
            }
            if let VerdictSource::Commit(sha) = &cv.verdict_source {
                if self.find_commit(sha).is_none() {
                    return Err(TrajectoryRefusal::VerdictCommitNotInRange {
                        claim_id: cv.claim.claim_id.clone(),
                        commit_sha: sha.clone(),
                    });
                }
            }
        }
        let known_claim_ids: BTreeSet<&str> = seen_claim_ids;
        for dep in &self.dependencies {
            if !known_claim_ids.contains(dep.claim_id.as_str()) {
                return Err(TrajectoryRefusal::DependencyReferencesUnknownClaim {
                    claim_id: dep.claim_id.clone(),
                    dependent_sha: dep.dependent_sha.clone(),
                });
            }
            if self.find_commit(&dep.dependent_sha).is_none() {
                return Err(TrajectoryRefusal::DependencyCommitNotInRange {
                    claim_id: dep.claim_id.clone(),
                    dependent_sha: dep.dependent_sha.clone(),
                });
            }
        }
        Ok(())
    }

    /// Compute the paper's t_err/t_lock/t_obs (plus derived fix window / observability lag
    /// / recovery-opportunity count) for the [`ClaimVerdict`] identified by `claim_id`.
    ///
    /// Only defined for claims whose verdict is [`Verdict::Overclaim`] -- a `Confirmed`
    /// claim has no failure to anatomize, and this function refuses rather than returning a
    /// hollow all-`None`-shaped success (this module's own no-silent-defaults discipline:
    /// "nothing to report" is a typed refusal, not a quietly-empty struct dressed up as a
    /// real answer).
    ///
    /// # Complexity
    /// O(D * C + C) where D = dependencies for this `claim_id`, C = `self.commits.len()`:
    /// [`Self::validate`]'s own cost, plus one O(C) `find_commit` per matching dependency,
    /// plus one O(C) pass each for the observability-lag commit count and the
    /// recovery-opportunity scan.
    ///
    /// # Errors
    /// See [`TrajectoryRefusal`]'s variants; every one that this trajectory's shape can
    /// trigger is exercised by name in this file's own test module.
    pub fn compute_failure_window(
        &self,
        claim_id: &str,
    ) -> Result<FailureWindow, TrajectoryRefusal> {
        self.validate()?;

        let claim_verdict = self
            .claims
            .iter()
            .find(|cv| cv.claim.claim_id == claim_id)
            .ok_or_else(|| TrajectoryRefusal::UnknownClaimId {
                claim_id: claim_id.to_string(),
            })?;
        if claim_verdict.verdict != Verdict::Overclaim {
            return Err(TrajectoryRefusal::ClaimNotOverclaimed {
                claim_id: claim_id.to_string(),
            });
        }

        // Presence already checked by `validate`; `find_commit` cannot return `None` here,
        // but a typed refusal via `ok_or_else` is used anyway rather than `.unwrap()` --
        // this module's own invariant applies to itself, not just to the paths it analyzes.
        let t_err_commit = self
            .find_commit(&claim_verdict.claim.commit_sha)
            .ok_or_else(|| TrajectoryRefusal::ClaimCommitNotInRange {
                claim_id: claim_id.to_string(),
                commit_sha: claim_verdict.claim.commit_sha.clone(),
            })?;

        let (t_obs_at_unix, t_obs_at_iso, t_obs_description) = match &claim_verdict.verdict_source {
            VerdictSource::Commit(sha) => {
                let c = self.find_commit(sha).ok_or_else(|| {
                    TrajectoryRefusal::VerdictCommitNotInRange {
                        claim_id: claim_id.to_string(),
                        commit_sha: sha.clone(),
                    }
                })?;
                (
                    c.authored_at_unix,
                    c.authored_at_iso.clone(),
                    format!("fix commit {}: {}", c.sha, c.subject),
                )
            }
            VerdictSource::External {
                observed_at_unix,
                observed_at_iso,
                description,
            } => (
                *observed_at_unix,
                observed_at_iso.clone(),
                description.clone(),
            ),
        };
        if t_obs_at_unix < t_err_commit.authored_at_unix {
            return Err(TrajectoryRefusal::VerdictPrecedesClaim {
                claim_id: claim_id.to_string(),
                claim_at_unix: t_err_commit.authored_at_unix,
                verdict_at_unix: t_obs_at_unix,
            });
        }

        // Dependent commits: caller-asserted build-on-the-claim commits, restricted to
        // [t_err, t_obs) and excluding the claim's own commit (a dependency naming the
        // claim's own commit would not be a downstream build-on).
        let mut dependent_commits: Vec<&CommitRecord> = Vec::new();
        for dep in self.dependencies.iter().filter(|d| d.claim_id == claim_id) {
            let c = self.find_commit(&dep.dependent_sha).ok_or_else(|| {
                TrajectoryRefusal::DependencyCommitNotInRange {
                    claim_id: claim_id.to_string(),
                    dependent_sha: dep.dependent_sha.clone(),
                }
            })?;
            if c.sha == t_err_commit.sha {
                continue;
            }
            if c.authored_at_unix >= t_err_commit.authored_at_unix
                && c.authored_at_unix < t_obs_at_unix
            {
                dependent_commits.push(c);
            }
        }

        // t_lock = latest dependent commit; degenerate case (no dependencies known) is
        // t_lock = t_err itself -- a real zero-length fix window, not a missing value.
        let t_lock_commit: &CommitRecord = dependent_commits
            .iter()
            .copied()
            .max_by_key(|c| c.authored_at_unix)
            .unwrap_or(t_err_commit);

        let fix_window_commit_count = dependent_commits.len() as u32;
        let fix_window_seconds = t_lock_commit.authored_at_unix - t_err_commit.authored_at_unix;
        let observability_lag_seconds = t_obs_at_unix - t_lock_commit.authored_at_unix;

        // Observability-lag commit count: every commit in this trajectory's own commit
        // range strictly between t_lock and t_obs, regardless of which paths it touched --
        // matching the paper's own framing (F3) that nothing during this window corrected
        // the failure, not that nothing happened during it.
        let observability_lag_commit_count = self
            .commits
            .iter()
            .filter(|c| {
                c.authored_at_unix > t_lock_commit.authored_at_unix
                    && c.authored_at_unix < t_obs_at_unix
            })
            .count() as u32;

        // Recovery opportunities: commits strictly after t_err and before t_obs that touch
        // >=1 of the same paths t_err's own commit touched, are not already counted as a
        // dependency, and are not t_lock itself -- real "could have re-checked this, chose
        // to build elsewhere instead" commits (paper F2's "recovery steps that went
        // unused," narrowed here to same-file edits specifically; see module Limitations on
        // why this narrows rather than matches the paper's broader "any action" framing).
        // Yields 0, honestly, when `t_err_commit.touched_paths` is empty (no path data
        // supplied) rather than silently counting everything.
        let dependent_shas: BTreeSet<&str> =
            dependent_commits.iter().map(|c| c.sha.as_str()).collect();
        let t_err_paths: BTreeSet<&str> = t_err_commit
            .touched_paths
            .iter()
            .map(|s| s.as_str())
            .collect();
        let mut recovery_opportunities: u32 = 0;
        if !t_err_paths.is_empty() {
            for c in &self.commits {
                if c.sha == t_err_commit.sha {
                    continue;
                }
                if c.authored_at_unix <= t_err_commit.authored_at_unix {
                    continue;
                }
                if c.authored_at_unix >= t_obs_at_unix {
                    continue;
                }
                if dependent_shas.contains(c.sha.as_str()) {
                    continue;
                }
                if c.touched_paths
                    .iter()
                    .any(|p| t_err_paths.contains(p.as_str()))
                {
                    recovery_opportunities += 1;
                }
            }
        }

        Ok(FailureWindow {
            claim_id: claim_id.to_string(),
            t_err: CommitPoint {
                sha: t_err_commit.sha.clone(),
                authored_at_unix: t_err_commit.authored_at_unix,
                authored_at_iso: t_err_commit.authored_at_iso.clone(),
                description: claim_verdict.claim.claim_text.clone(),
            },
            t_lock: CommitPoint {
                sha: t_lock_commit.sha.clone(),
                authored_at_unix: t_lock_commit.authored_at_unix,
                authored_at_iso: t_lock_commit.authored_at_iso.clone(),
                description: t_lock_commit.subject.clone(),
            },
            t_obs: ObservationPoint {
                source: claim_verdict.verdict_source.clone(),
                at_unix: t_obs_at_unix,
                at_iso: t_obs_at_iso,
                description: t_obs_description,
            },
            fix_window_commit_count,
            fix_window_seconds,
            observability_lag_commit_count,
            observability_lag_seconds,
            recovery_opportunities,
        })
    }
}

/// One endpoint of a [`FailureWindow`] that is always a real commit (t_err, t_lock).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitPoint {
    pub sha: String,
    pub authored_at_unix: i64,
    pub authored_at_iso: String,
    pub description: String,
}

/// The t_obs endpoint, which may or may not correspond to a commit (see [`VerdictSource`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservationPoint {
    pub source: VerdictSource,
    pub at_unix: i64,
    pub at_iso: String,
    pub description: String,
}

/// The paper's t_err/t_lock/t_obs plus derived fix window, observability lag, and recovery
/// opportunity count, computed for one [`Claim`] within one [`FailureTrajectory`]. See
/// [`FailureTrajectory::compute_failure_window`] for how each field is derived and this
/// module's top-level doc for the honest limitations of every number here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailureWindow {
    pub claim_id: String,
    pub t_err: CommitPoint,
    pub t_lock: CommitPoint,
    pub t_obs: ObservationPoint,
    /// Number of caller-supplied [`AssumedDependency`] commits between t_err and t_obs
    /// (paper F2's "recovery steps available" analog).
    pub fix_window_commit_count: u32,
    /// `t_lock.authored_at_unix - t_err.authored_at_unix`. Always >= 0 by construction.
    pub fix_window_seconds: i64,
    /// Number of commits in the trajectory's own commit range strictly between t_lock and
    /// t_obs (paper F3's observability-lag analog, at commit granularity).
    pub observability_lag_commit_count: u32,
    /// `t_obs.at_unix - t_lock.authored_at_unix`. Always >= 0 by construction (enforced by
    /// [`TrajectoryRefusal::VerdictPrecedesClaim`] and the `< t_obs_at_unix` filter on
    /// dependent commits).
    pub observability_lag_seconds: i64,
    /// Commits between t_err and t_obs that touched the same paths as t_err's own commit
    /// but were not already counted in `fix_window_commit_count` -- real, unused chances to
    /// re-check the claim (paper F2's "recovery steps that went unused," narrowed to
    /// same-file edits; see module Limitations).
    pub recovery_opportunities: u32,
}

// ---------------------------------------------------------------------------------------
// Typed refusal.
// ---------------------------------------------------------------------------------------

/// Typed refusal for this module's own analysis pipeline. Every variant is raised from
/// exactly one call site in [`FailureTrajectory::validate`] or
/// [`FailureTrajectory::compute_failure_window`] (or [`parse_git_log_plumbing`]) and has a
/// dedicated test in this file's `tests` module -- an untested `Refusal` variant is a
/// guess, not a contract, per this repo's own discipline.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TrajectoryRefusal {
    /// [`FailureTrajectory::commits`] was empty -- there is no history to analyze.
    #[error("trajectory_failure_process: commit range is empty")]
    EmptyCommitRange,
    /// `commits` was not sorted ascending by `authored_at_unix`.
    #[error(
        "trajectory_failure_process: commit range is not chronologically sorted ascending \
         by authored_at_unix; first out-of-order sha = {sha}"
    )]
    CommitRangeNotChronological { sha: String },
    /// Two [`ClaimVerdict`]s in the same trajectory share a `claim_id`.
    #[error("trajectory_failure_process: duplicate claim_id '{claim_id}' in this trajectory")]
    DuplicateClaimId { claim_id: String },
    /// No [`ClaimVerdict`] in this trajectory has the requested `claim_id`.
    #[error(
        "trajectory_failure_process: unknown claim_id '{claim_id}' -- no ClaimVerdict in \
         this trajectory has this claim_id"
    )]
    UnknownClaimId { claim_id: String },
    /// A [`Claim::commit_sha`] is not present in the supplied commit range.
    #[error(
        "trajectory_failure_process: claim '{claim_id}' commit_sha '{commit_sha}' is not \
         present in the supplied commit range"
    )]
    ClaimCommitNotInRange {
        claim_id: String,
        commit_sha: String,
    },
    /// A [`VerdictSource::Commit`] SHA is not present in the supplied commit range.
    #[error(
        "trajectory_failure_process: claim '{claim_id}' verdict_source commit '{commit_sha}' \
         is not present in the supplied commit range"
    )]
    VerdictCommitNotInRange {
        claim_id: String,
        commit_sha: String,
    },
    /// An [`AssumedDependency::claim_id`] does not match any known claim.
    #[error(
        "trajectory_failure_process: dependency references unknown claim_id '{claim_id}' \
         (dependent commit {dependent_sha})"
    )]
    DependencyReferencesUnknownClaim {
        claim_id: String,
        dependent_sha: String,
    },
    /// An [`AssumedDependency::dependent_sha`] is not present in the supplied commit range.
    #[error(
        "trajectory_failure_process: dependency commit '{dependent_sha}' for claim \
         '{claim_id}' is not present in the supplied commit range"
    )]
    DependencyCommitNotInRange {
        claim_id: String,
        dependent_sha: String,
    },
    /// [`FailureTrajectory::compute_failure_window`] was asked to compute a window for a
    /// claim whose verdict is [`Verdict::Confirmed`] -- correctly, there is nothing to
    /// compute.
    #[error(
        "trajectory_failure_process: claim '{claim_id}' verdict is Confirmed, not Overclaim \
         -- no failure window exists to compute (this is the correct, quiet outcome for a \
         claim that held up under re-check)"
    )]
    ClaimNotOverclaimed { claim_id: String },
    /// A verdict's own timestamp is earlier than the claim commit it judges -- impossible
    /// for a real audit, so refused rather than silently producing a negative-duration
    /// window.
    #[error(
        "trajectory_failure_process: claim '{claim_id}' verdict timestamp \
         ({verdict_at_unix}) precedes its own claim commit timestamp ({claim_at_unix}) -- \
         a verdict cannot predate the claim it judges"
    )]
    VerdictPrecedesClaim {
        claim_id: String,
        claim_at_unix: i64,
        verdict_at_unix: i64,
    },
    /// [`parse_git_log_plumbing`]: a non-empty line did not split into exactly 4
    /// tab-separated fields.
    #[error(
        "trajectory_failure_process: failed to parse git-log plumbing line {line_number}: \
         expected 4 tab-separated fields (sha, ISO-8601 author date, unix author date, \
         subject), got {field_count}"
    )]
    MalformedGitLogLine {
        line_number: usize,
        field_count: usize,
    },
    /// [`parse_git_log_plumbing`]: the third field did not parse as `i64`.
    #[error(
        "trajectory_failure_process: failed to parse unix timestamp '{value}' on git-log \
         plumbing line {line_number}: {reason}"
    )]
    MalformedUnixTimestamp {
        line_number: usize,
        value: String,
        reason: String,
    },
}

// ---------------------------------------------------------------------------------------
// Tests: one per Refusal variant (repo discipline), plus the real F18->F19 case study.
// ---------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn commit(sha: &str, unix: i64, subject: &str) -> CommitRecord {
        CommitRecord {
            sha: sha.to_string(),
            authored_at_unix: unix,
            authored_at_iso: format!("iso({unix})"),
            subject: subject.to_string(),
            touched_paths: Vec::new(),
        }
    }

    // -- parse_git_log_plumbing -----------------------------------------------------

    #[test]
    fn parse_git_log_plumbing_parses_real_git_output() {
        // Real output of:
        //   git log --format='%H%x09%aI%x09%at%x09%s' --reverse -- \
        //     crates/multifractal-workflow/src/crown_local.rs \
        //     docs/jira/v26.7.12/CROWN_STATUS.md
        // captured verbatim this session (see this file's module doc for the citation).
        let raw = "3322bf2d5db417bade62ea4a867b3aa13b8f5a81\t2026-07-12T13:55:23-07:00\t1783889723\tfeat(multifractal-workflow): compose the entire shared crown prefix F02->F03->F08->F09->F10 as one real production edge\n\
eeca952a8e93b5a0d61d4db3b8af9e95665d03cd\t2026-07-12T15:44:25-07:00\t1783896265\tfeat(multifractal-workflow): wire F18->F19 into crown_local (LOCAL witness real path now 7 edges)\n";
        let commits = parse_git_log_plumbing(raw).expect("valid plumbing text must parse");
        assert_eq!(commits.len(), 2);
        assert_eq!(commits[0].sha, "3322bf2d5db417bade62ea4a867b3aa13b8f5a81");
        assert_eq!(commits[0].authored_at_unix, 1783889723);
        assert_eq!(commits[1].sha, "eeca952a8e93b5a0d61d4db3b8af9e95665d03cd");
        assert_eq!(commits[1].authored_at_iso, "2026-07-12T15:44:25-07:00");
        assert!(commits[1]
            .subject
            .starts_with("feat(multifractal-workflow): wire F18->F19"));
        assert!(commits[1].touched_paths.is_empty());
    }

    #[test]
    fn parse_git_log_plumbing_rejects_malformed_line() {
        let raw = "only-two\tfields";
        let err = parse_git_log_plumbing(raw).unwrap_err();
        assert_eq!(
            err,
            TrajectoryRefusal::MalformedGitLogLine {
                line_number: 1,
                field_count: 2,
            }
        );
    }

    #[test]
    fn parse_git_log_plumbing_rejects_bad_unix_timestamp() {
        let raw = "deadbeef\t2026-07-12T00:00:00-07:00\tnot-a-number\tsubject";
        let err = parse_git_log_plumbing(raw).unwrap_err();
        match err {
            TrajectoryRefusal::MalformedUnixTimestamp {
                line_number, value, ..
            } => {
                assert_eq!(line_number, 1);
                assert_eq!(value, "not-a-number");
            }
            other => panic!("expected MalformedUnixTimestamp, got {other:?}"),
        }
    }

    // -- StageDataThreading -----------------------------------------------------------

    #[test]
    fn is_unread_consequence_true_for_f18_f19() {
        // Real field list of BrokerReceipt (f18_broker_law.rs:374-382) vs. the real,
        // empty read set at the F18->F19 call site (crown_local.rs:572-580 /
        // f19_hooks.rs:443-447 -- resolve_hook_for_action takes no BrokerReceipt-derived
        // argument at all).
        let edge = StageDataThreading {
            upstream_type: "BrokerReceipt",
            downstream_fn: "resolve_hook_for_action",
            upstream_fields_available: vec![
                "workflow_id",
                "step_id",
                "idempotency_key",
                "correlation_id",
                "authority_token_hex",
                "consequence_hash_hex",
                "receipt_hash_hex",
            ],
            upstream_fields_read: vec![],
        };
        assert!(edge.is_unread_consequence());
    }

    #[test]
    fn is_unread_consequence_false_for_f19_f02_readmit() {
        // Contrast case: the very next edge (crown_local.rs:588-591) DOES read
        // broker_receipt.receipt_hash_hex to build actuation_subject_iri.
        let edge = StageDataThreading {
            upstream_type: "BrokerReceipt",
            downstream_fn: "build_actuation_payload (via actuation_subject_iri)",
            upstream_fields_available: vec!["receipt_hash_hex"],
            upstream_fields_read: vec!["receipt_hash_hex"],
        };
        assert!(!edge.is_unread_consequence());
    }

    #[test]
    fn is_unread_consequence_false_when_nothing_was_available() {
        // A stage that legitimately produces no fields the next stage could read is not
        // an unread consequence -- there was nothing to ignore.
        let edge = StageDataThreading {
            upstream_type: "Unit",
            downstream_fn: "next_stage",
            upstream_fields_available: vec![],
            upstream_fields_read: vec![],
        };
        assert!(!edge.is_unread_consequence());
    }

    // -- validate() refusal variants ---------------------------------------------------

    #[test]
    fn validate_rejects_empty_commit_range() {
        let traj = FailureTrajectory {
            commits: vec![],
            claims: vec![],
            dependencies: vec![],
        };
        assert_eq!(
            traj.compute_failure_window("anything").unwrap_err(),
            TrajectoryRefusal::EmptyCommitRange
        );
    }

    #[test]
    fn validate_rejects_non_chronological_commits() {
        let traj = FailureTrajectory {
            commits: vec![commit("b", 200, "second"), commit("a", 100, "first")],
            claims: vec![],
            dependencies: vec![],
        };
        assert_eq!(
            traj.compute_failure_window("anything").unwrap_err(),
            TrajectoryRefusal::CommitRangeNotChronological {
                sha: "a".to_string()
            }
        );
    }

    #[test]
    fn validate_rejects_duplicate_claim_id() {
        let mk_cv = |sha: &str| ClaimVerdict {
            claim: Claim {
                commit_sha: sha.to_string(),
                claim_id: "dup".to_string(),
                claim_text: "text".to_string(),
            },
            verdict: Verdict::Confirmed,
            verdict_source: VerdictSource::Commit(sha.to_string()),
            evidence: "e".to_string(),
            structural_evidence: None,
        };
        let traj = FailureTrajectory {
            commits: vec![commit("a", 100, "first")],
            claims: vec![mk_cv("a"), mk_cv("a")],
            dependencies: vec![],
        };
        assert_eq!(
            traj.compute_failure_window("dup").unwrap_err(),
            TrajectoryRefusal::DuplicateClaimId {
                claim_id: "dup".to_string()
            }
        );
    }

    #[test]
    fn validate_rejects_claim_commit_not_in_range() {
        let traj = FailureTrajectory {
            commits: vec![commit("a", 100, "first")],
            claims: vec![ClaimVerdict {
                claim: Claim {
                    commit_sha: "missing".to_string(),
                    claim_id: "c1".to_string(),
                    claim_text: "text".to_string(),
                },
                verdict: Verdict::Overclaim,
                verdict_source: VerdictSource::Commit("a".to_string()),
                evidence: "e".to_string(),
                structural_evidence: None,
            }],
            dependencies: vec![],
        };
        assert_eq!(
            traj.compute_failure_window("c1").unwrap_err(),
            TrajectoryRefusal::ClaimCommitNotInRange {
                claim_id: "c1".to_string(),
                commit_sha: "missing".to_string(),
            }
        );
    }

    #[test]
    fn validate_rejects_verdict_commit_not_in_range() {
        let traj = FailureTrajectory {
            commits: vec![commit("a", 100, "first")],
            claims: vec![ClaimVerdict {
                claim: Claim {
                    commit_sha: "a".to_string(),
                    claim_id: "c1".to_string(),
                    claim_text: "text".to_string(),
                },
                verdict: Verdict::Overclaim,
                verdict_source: VerdictSource::Commit("missing-fix".to_string()),
                evidence: "e".to_string(),
                structural_evidence: None,
            }],
            dependencies: vec![],
        };
        assert_eq!(
            traj.compute_failure_window("c1").unwrap_err(),
            TrajectoryRefusal::VerdictCommitNotInRange {
                claim_id: "c1".to_string(),
                commit_sha: "missing-fix".to_string(),
            }
        );
    }

    #[test]
    fn validate_rejects_dependency_unknown_claim() {
        let traj = FailureTrajectory {
            commits: vec![commit("a", 100, "first")],
            claims: vec![],
            dependencies: vec![AssumedDependency {
                claim_id: "no-such-claim".to_string(),
                dependent_sha: "a".to_string(),
            }],
        };
        assert_eq!(
            traj.compute_failure_window("no-such-claim").unwrap_err(),
            TrajectoryRefusal::DependencyReferencesUnknownClaim {
                claim_id: "no-such-claim".to_string(),
                dependent_sha: "a".to_string(),
            }
        );
    }

    #[test]
    fn validate_rejects_dependency_commit_not_in_range() {
        let traj = FailureTrajectory {
            commits: vec![commit("a", 100, "first")],
            claims: vec![ClaimVerdict {
                claim: Claim {
                    commit_sha: "a".to_string(),
                    claim_id: "c1".to_string(),
                    claim_text: "text".to_string(),
                },
                verdict: Verdict::Overclaim,
                verdict_source: VerdictSource::Commit("a".to_string()),
                evidence: "e".to_string(),
                structural_evidence: None,
            }],
            dependencies: vec![AssumedDependency {
                claim_id: "c1".to_string(),
                dependent_sha: "missing".to_string(),
            }],
        };
        assert_eq!(
            traj.compute_failure_window("c1").unwrap_err(),
            TrajectoryRefusal::DependencyCommitNotInRange {
                claim_id: "c1".to_string(),
                dependent_sha: "missing".to_string(),
            }
        );
    }

    // -- compute_failure_window() refusal variants -------------------------------------

    #[test]
    fn compute_failure_window_rejects_unknown_claim_id() {
        let traj = FailureTrajectory {
            commits: vec![commit("a", 100, "first")],
            claims: vec![],
            dependencies: vec![],
        };
        assert_eq!(
            traj.compute_failure_window("does-not-exist").unwrap_err(),
            TrajectoryRefusal::UnknownClaimId {
                claim_id: "does-not-exist".to_string()
            }
        );
    }

    #[test]
    fn compute_failure_window_rejects_confirmed_claim() {
        let traj = FailureTrajectory {
            commits: vec![commit("a", 100, "first")],
            claims: vec![ClaimVerdict {
                claim: Claim {
                    commit_sha: "a".to_string(),
                    claim_id: "c1".to_string(),
                    claim_text: "text".to_string(),
                },
                verdict: Verdict::Confirmed,
                verdict_source: VerdictSource::Commit("a".to_string()),
                evidence: "held up under re-check".to_string(),
                structural_evidence: None,
            }],
            dependencies: vec![],
        };
        assert_eq!(
            traj.compute_failure_window("c1").unwrap_err(),
            TrajectoryRefusal::ClaimNotOverclaimed {
                claim_id: "c1".to_string()
            }
        );
    }

    #[test]
    fn compute_failure_window_rejects_verdict_preceding_claim() {
        // Commits must stay chronological (see `validate_rejects_non_chronological_commits`
        // for that separate check) so this test isolates VerdictPrecedesClaim on its own:
        // the claim commit "a" is chronologically later than the fix commit "b", but "b" is
        // still the one whose verdict_source names it as the (impossible) verdict.
        let traj = FailureTrajectory {
            commits: vec![commit("b", 100, "earlier-fix"), commit("a", 200, "claim")],
            claims: vec![ClaimVerdict {
                claim: Claim {
                    commit_sha: "a".to_string(),
                    claim_id: "c1".to_string(),
                    claim_text: "text".to_string(),
                },
                verdict: Verdict::Overclaim,
                verdict_source: VerdictSource::Commit("b".to_string()),
                evidence: "e".to_string(),
                structural_evidence: None,
            }],
            dependencies: vec![],
        };
        assert_eq!(
            traj.compute_failure_window("c1").unwrap_err(),
            TrajectoryRefusal::VerdictPrecedesClaim {
                claim_id: "c1".to_string(),
                claim_at_unix: 200,
                verdict_at_unix: 100,
            }
        );
    }

    // -- happy path: clean recovery, no lock-in ------------------------------------------

    /// A trajectory where a bad claim is caught and fixed before anything else builds on
    /// it, even though unrelated work continues in the same commit range. Contrasts
    /// directly with `f18_f19_case_study` below: there, 8 commits built on the bad claim
    /// (`fix_window_commit_count == 8`) and 11 more landed before anyone noticed
    /// (`observability_lag_commit_count == 11`). Here, zero commits build on the claim
    /// (`AssumedDependency` list is empty) and the fix lands via a same-session commit, so
    /// `t_lock == t_err` (no lock-in occurred) and both counts are small. This is the
    /// paper's own F9/F10 shape: 71% of *successful* trajectories still hit an error, but
    /// the fix window closes immediately instead of being built on.
    #[test]
    fn happy_path_clean_recovery_no_lock_in() {
        let traj = FailureTrajectory {
            commits: vec![
                commit("a", 1_000, "claim: wire F-x -> F-y as REAL_EDGE"),
                commit(
                    "b",
                    1_010,
                    "unrelated: docs typo fix, does not touch the claim",
                ),
                commit(
                    "c",
                    1_020,
                    "fix: F-x -> F-y was not actually wired; correct claim",
                ),
            ],
            claims: vec![ClaimVerdict {
                claim: Claim {
                    commit_sha: "a".to_string(),
                    claim_id: "Fx->Fy:REAL_EDGE".to_string(),
                    claim_text: "claimed REAL_EDGE, self-corrected before anything else \
                        built on it"
                        .to_string(),
                },
                verdict: Verdict::Overclaim,
                verdict_source: VerdictSource::Commit("c".to_string()),
                evidence: "same-session self-review caught the overclaim before any \
                    dependent commit landed"
                    .to_string(),
                structural_evidence: None,
            }],
            // No AssumedDependency entries at all: nothing built on the bad claim before
            // it was fixed -- this is what "no lock-in" means mechanically in this
            // module's model (see `compute_failure_window`'s t_lock-degenerate-to-t_err
            // fallback).
            dependencies: vec![],
        };

        let window = traj
            .compute_failure_window("Fx->Fy:REAL_EDGE")
            .expect("clean-recovery trajectory must compute a failure window");

        // No lock-in: t_lock falls back to t_err itself because zero dependencies were
        // asserted.
        assert_eq!(window.t_lock.sha, window.t_err.sha);
        assert_eq!(window.t_err.sha, "a");
        assert_eq!(window.fix_window_commit_count, 0);
        assert_eq!(window.fix_window_seconds, 0);

        // Observability lag is measured from t_lock (== t_err, commit "a" @ 1000) to the
        // fix commit "c" @ 1020: 20 seconds, and exactly one commit ("b") falls strictly
        // between them -- unrelated parallel work, not a missed recovery chance.
        assert_eq!(window.observability_lag_seconds, 20);
        assert_eq!(window.observability_lag_commit_count, 1);

        // No touched_paths were supplied, so recovery_opportunities is honestly 0 rather
        // than silently scanning commit "b" in as a false-positive recovery chance.
        assert_eq!(window.recovery_opportunities, 0);
    }

    // -- degenerate case: zero supplied dependencies ------------------------------------

    #[test]
    fn compute_failure_window_degenerate_zero_dependencies() {
        let traj = FailureTrajectory {
            commits: vec![
                commit("a", 1000, "claim"),
                commit("b", 5000, "external-audit-anchor"),
            ],
            claims: vec![ClaimVerdict {
                claim: Claim {
                    commit_sha: "a".to_string(),
                    claim_id: "c1".to_string(),
                    claim_text: "claimed REAL_EDGE with no downstream build-on ever supplied"
                        .to_string(),
                },
                verdict: Verdict::Overclaim,
                verdict_source: VerdictSource::External {
                    observed_at_unix: 5000,
                    observed_at_iso: "iso(5000)".to_string(),
                    description: "external re-audit".to_string(),
                },
                evidence: "e".to_string(),
                structural_evidence: None,
            }],
            dependencies: vec![],
        };
        let window = traj.compute_failure_window("c1").expect("must compute");
        assert_eq!(
            window.t_lock.sha, window.t_err.sha,
            "t_lock = t_err with zero dependencies"
        );
        assert_eq!(window.fix_window_commit_count, 0);
        assert_eq!(window.fix_window_seconds, 0);
        assert_eq!(window.observability_lag_seconds, 4000);
    }

    // -- the real F18->F19 dogfood case study -------------------------------------------

    /// Real, `git log`-sourced case study for this module's own dogfood incident (see the
    /// module-level doc for the full narrative and file:line citations). This is the
    /// executable stand-in for a doctest -- this crate sets `doctest = false` in its
    /// `Cargo.toml`, so a doc-example here would never actually run under `cargo test`.
    #[test]
    fn f18_f19_case_study() {
        // Real output of:
        //   git log --format='%H%x09%aI%x09%at%x09%s' --reverse -- \
        //     crates/multifractal-workflow/src/crown_local.rs \
        //     docs/jira/v26.7.12/CROWN_STATUS.md
        // captured verbatim this session.
        let raw_git_log = "3322bf2d5db417bade62ea4a867b3aa13b8f5a81\t2026-07-12T13:55:23-07:00\t1783889723\tfeat(multifractal-workflow): compose the entire shared crown prefix F02->F03->F08->F09->F10 as one real production edge\n\
4224d8580a04b3ac99b655c8686bd4039e8e323b\t2026-07-12T14:44:35-07:00\t1783892675\tdocs(v26.7.12): add adversarially-verified crown-frontier status\n\
d60f2036b0c9cc0877d2e5df7cba4716e9352217\t2026-07-12T14:53:17-07:00\t1783893197\tfeat(multifractal-workflow): extend crown_local through F10->F11->F18 (LOCAL witness real path now 6 edges)\n\
eeca952a8e93b5a0d61d4db3b8af9e95665d03cd\t2026-07-12T15:44:25-07:00\t1783896265\tfeat(multifractal-workflow): wire F18->F19 into crown_local (LOCAL witness real path now 7 edges)\n\
77da318bc9e7d56611d421c1d6c30f0a6c569828\t2026-07-12T15:46:34-07:00\t1783896394\tdocs(v26.7.12): bring CROWN_STATUS.md current with commits d60f2036, eeca952a\n\
66d8732eead2179fd51c536fe1151fe67e02fdd0\t2026-07-12T16:12:14-07:00\t1783897934\tfeat(multifractal-workflow): wire F19->F02(re-admit) into crown_local (LOCAL witness real path now 8 edges)\n\
a2c52732bb8a241feb95d3c63c6d1ea9eea4fc37\t2026-07-12T16:14:56-07:00\t1783898096\tdocs(v26.7.12): bring CROWN_STATUS.md current with commit 66d8732e; flag REMAINING_WORK.md staleness\n\
0815680a0c3a334c66b735c12410dadb7ba6c10d\t2026-07-12T16:33:52-07:00\t1783899232\tfeat(multifractal-workflow): wire F02(re-admit)->F24 into crown_local (LOCAL witness real path now 9 edges)\n\
80432701851ed5e46f826f32632c0c3dec54c549\t2026-07-12T16:35:54-07:00\t1783899354\tdocs(v26.7.12): bring CROWN_STATUS.md current with commit 0815680a\n\
217dc37d971d77cb233d89ef841ed8d05543b065\t2026-07-12T16:54:17-07:00\t1783900457\tfeat(multifractal-workflow): wire F24->F21 into crown_local (LOCAL witness real path now 10 edges)\n\
393a50ba6c7abb1779644e6466ca5b92b95a87c5\t2026-07-12T16:56:29-07:00\t1783900589\tdocs(v26.7.12): bring CROWN_STATUS.md current with commit 217dc37d\n\
66cb59b13845c525e57ff2775785ff9567acdd0e\t2026-07-12T17:08:50-07:00\t1783901330\tfeat(multifractal-workflow): wire F21->F25 into crown_local -- LOCAL crown witness fully closed (11/11 edges)\n\
78c31135e036517b54f5d23c882c74681d0128d6\t2026-07-12T17:11:16-07:00\t1783901476\tdocs(v26.7.12): CROWN_STATUS.md -- LOCAL_OBSERVATION_TO_REPLAY_CONTIGUOUS_PATH = true\n\
cd081a9383a23a2d12d0b0338a5dc3c32f09b524\t2026-07-12T17:20:27-07:00\t1783902027\tdocs(v26.7.12): record this cycle's investigation of F10->F12 and F20->F02(re-admit) as ruled out\n\
d84cf6fb36d5a8889965777707cbafcd2d26e2fa\t2026-07-12T18:01:45-07:00\t1783904505\tdocs(v26.7.12): CROWN_STATUS.md -- F20->F02(re-admit) closed to REAL_EDGE (commit b4d743f7)\n\
4505b61a63b4103ed62219082f233810df1876e7\t2026-07-12T18:14:30-07:00\t1783905270\tdocs(v26.7.12): CROWN_STATUS.md -- F02(re-admit)->F15(AIR transition) closed to REAL_EDGE (38048b27)\n\
70d80c675f56c275b026cd0493760468db6db839\t2026-07-12T18:27:16-07:00\t1783906036\tdocs(v26.7.12): CROWN_STATUS.md -- F15(AIR transition)->F21 closed to REAL_EDGE (a139d477)\n\
54532a91e38be954f9d6f5cee6394ca0eddef432\t2026-07-12T18:48:48-07:00\t1783907328\tdocs(v26.7.12): CROWN_STATUS.md -- F21->F24 closed to REAL_EDGE (8c2675be)\n\
5982027306a16d35617910ed5abf2f50430b8b72\t2026-07-12T19:05:54-07:00\t1783908354\tdocs(v26.7.12): CROWN_STATUS.md -- entire EXTERNAL loop-back tail complete (F24->F25, 11dcee0e)\n\
0ed375b0659a74a380a61e67a566058ef6100842\t2026-07-12T19:21:12-07:00\t1783909272\tdocs(v26.7.12): re-confirm F15->F16 wiring deferral with code-level regression evidence\n\
9774e58388cab414e11438ff93ef009d41c7bb4d\t2026-07-12T20:29:40-07:00\t1783913380\tdocs(v26.7.12): CROWN_STATUS.md -- F15->F16 closed to REAL_EDGE (1d3b9fb2)\n\
9f51cea870f17a36439dc9faf7e7bf1332c91845\t2026-07-12T20:45:49-07:00\t1783914349\tdocs(v26.7.12): CROWN_STATUS.md -- F16->F18 closed to REAL_EDGE (4ce20102)\n\
b69f995960b76732de96d3bac871dac3ad9f69f9\t2026-07-12T21:07:44-07:00\t1783915664\tdocs(v26.7.12): CROWN_STATUS.md -- F18->F20 closed, MISSING_EDGE_COUNT=0 (1e1ce976)\n";

        let commits = parse_git_log_plumbing(raw_git_log).expect("real git output must parse");
        assert_eq!(commits.len(), 23);

        // touched_paths, from `git show --stat <sha>` run for each commit this session.
        let cl = "crates/multifractal-workflow/src/crown_local.rs".to_string();
        let cl_test = "crates/multifractal-workflow/src/crown_local_test.rs".to_string();
        let status = "docs/jira/v26.7.12/CROWN_STATUS.md".to_string();
        let remaining = "docs/jira/v26.7.12/REMAINING_WORK.md".to_string();
        let mut paths_by_sha: BTreeMap<String, Vec<String>> = BTreeMap::new();
        paths_by_sha.insert(
            "eeca952a8e93b5a0d61d4db3b8af9e95665d03cd".to_string(),
            vec![cl.clone(), cl_test.clone()],
        );
        for sha in [
            "77da318bc9e7d56611d421c1d6c30f0a6c569828",
            "80432701851ed5e46f826f32632c0c3dec54c549",
            "393a50ba6c7abb1779644e6466ca5b92b95a87c5",
            "78c31135e036517b54f5d23c882c74681d0128d6",
            "cd081a9383a23a2d12d0b0338a5dc3c32f09b524",
            "d84cf6fb36d5a8889965777707cbafcd2d26e2fa",
            "4505b61a63b4103ed62219082f233810df1876e7",
            "70d80c675f56c275b026cd0493760468db6db839",
            "54532a91e38be954f9d6f5cee6394ca0eddef432",
            "5982027306a16d35617910ed5abf2f50430b8b72",
            "0ed375b0659a74a380a61e67a566058ef6100842",
            "9774e58388cab414e11438ff93ef009d41c7bb4d",
            "9f51cea870f17a36439dc9faf7e7bf1332c91845",
            "b69f995960b76732de96d3bac871dac3ad9f69f9",
        ] {
            paths_by_sha.insert(sha.to_string(), vec![status.clone()]);
        }
        paths_by_sha.insert(
            "a2c52732bb8a241feb95d3c63c6d1ea9eea4fc37".to_string(),
            vec![status.clone(), remaining.clone()],
        );
        for sha in [
            "66d8732eead2179fd51c536fe1151fe67e02fdd0",
            "0815680a0c3a334c66b735c12410dadb7ba6c10d",
            "217dc37d971d77cb233d89ef841ed8d05543b065",
            "66cb59b13845c525e57ff2775785ff9567acdd0e",
        ] {
            paths_by_sha.insert(sha.to_string(), vec![cl.clone(), cl_test.clone()]);
        }

        // The claim: CROWN_STATUS.md:73 (commit 77da318b, two minutes after eeca952a) --
        // paraphrased and cited, not quoted at length -- classified F18->F19 REAL_EDGE.
        // t_err is anchored to eeca952a (the commit that actually wired the unread-
        // consequence call site), not 77da318b (the doc-sync commit two minutes later):
        // the code was already wrong before the doc caught up to it.
        let claim = Claim {
            commit_sha: "eeca952a8e93b5a0d61d4db3b8af9e95665d03cd".to_string(),
            claim_id: "F18->F19:REAL_EDGE".to_string(),
            claim_text: "CROWN_STATUS.md:73 classifies F18->F19 as REAL_EDGE: \
                resolve_hook_for_action called ?-gated on a real broker_receipt"
                .to_string(),
        };

        let structural_evidence = StageDataThreading {
            upstream_type: "BrokerReceipt",
            downstream_fn: "resolve_hook_for_action",
            upstream_fields_available: vec![
                "workflow_id",
                "step_id",
                "idempotency_key",
                "correlation_id",
                "authority_token_hex",
                "consequence_hash_hex",
                "receipt_hash_hex",
            ],
            upstream_fields_read: vec![],
        };
        assert!(structural_evidence.is_unread_consequence());

        // t_obs: External, sourced from this repo's own real re-audit workflow (task
        // wqv5aaz7u, agent "mid", agentId a00af5c847378602b) -- no fix commit exists for
        // this claim as of this module's own authoring session (`git status --porcelain`
        // on crown_local.rs and CROWN_STATUS.md was clean), so VerdictSource::External is
        // the only honest choice, not a stand-in for a commit that should exist yet
        // doesn't.
        let claim_verdict = ClaimVerdict {
            claim,
            verdict: Verdict::Overclaim,
            verdict_source: VerdictSource::External {
                observed_at_unix: 1_783_925_118,
                observed_at_iso: "2026-07-12T23:45:18-07:00".to_string(),
                description: "independent re-audit workflow (task wqv5aaz7u, agent 'mid', \
                    agentId a00af5c847378602b) read crown_local.rs:572-580 and \
                    f19_hooks.rs:443-458 directly and found resolve_hook_for_action takes \
                    no BrokerReceipt-derived argument and reads none of broker_receipt's \
                    fields before returning; verdict timestamp is that workflow's own \
                    recorded last_progress_at (harness-recorded completion instant, not a \
                    live clock read by this module)"
                    .to_string(),
            },
            evidence: "crown_local.rs:572-580 calls resolve_hook_for_action(\
                &run.hook_pack_turtle, &ground_action, &mut hook_ledger) -- no \
                BrokerReceipt-derived argument. f19_hooks.rs:443-447's real signature takes \
                (hook_pack_turtle, action, ledger), never a BrokerReceipt. Contrast \
                crown_local.rs:588-591 (F19->F02 re-admit), which does read \
                broker_receipt.receipt_hash_hex."
                .to_string(),
            structural_evidence: Some(structural_evidence),
        };

        // 8 commits (77da318b through 66cb59b1) built the rest of the LOCAL crown-witness
        // chain on top of the F18->F19 claim without re-checking it -- caller-asserted,
        // established by directly reading each commit's diff this session, not inferred.
        let dependencies: Vec<AssumedDependency> = [
            "77da318bc9e7d56611d421c1d6c30f0a6c569828",
            "66d8732eead2179fd51c536fe1151fe67e02fdd0",
            "a2c52732bb8a241feb95d3c63c6d1ea9eea4fc37",
            "0815680a0c3a334c66b735c12410dadb7ba6c10d",
            "80432701851ed5e46f826f32632c0c3dec54c549",
            "217dc37d971d77cb233d89ef841ed8d05543b065",
            "393a50ba6c7abb1779644e6466ca5b92b95a87c5",
            "66cb59b13845c525e57ff2775785ff9567acdd0e",
        ]
        .iter()
        .map(|sha| AssumedDependency {
            claim_id: "F18->F19:REAL_EDGE".to_string(),
            dependent_sha: sha.to_string(),
        })
        .collect();

        let trajectory = FailureTrajectory {
            commits,
            claims: vec![claim_verdict],
            dependencies,
        }
        .attach_touched_paths(&paths_by_sha);

        let window = trajectory
            .compute_failure_window("F18->F19:REAL_EDGE")
            .expect("real F18->F19 case study must compute a failure window");

        assert_eq!(window.t_err.sha, "eeca952a8e93b5a0d61d4db3b8af9e95665d03cd");
        assert_eq!(window.t_err.authored_at_unix, 1_783_896_265);
        assert_eq!(
            window.t_lock.sha,
            "66cb59b13845c525e57ff2775785ff9567acdd0e"
        );
        assert_eq!(window.t_lock.authored_at_unix, 1_783_901_330);
        assert_eq!(window.t_obs.at_unix, 1_783_925_118);

        // Fix window: 1h24m25s (5065s), 8 commits -- matches this repo's own real
        // dogfood-case narrative exactly.
        assert_eq!(window.fix_window_commit_count, 8);
        assert_eq!(window.fix_window_seconds, 5065);

        // Observability lag: 6h36m28s (23788s) from t_lock to this session's re-audit;
        // 11 commits in this trajectory's own (two-file-scoped) commit range fall in that
        // window, all of them CROWN_STATUS.md-only documentation of the separately-wired
        // EXTERNAL witness tail -- none re-touches crown_local.rs.
        assert_eq!(window.observability_lag_seconds, 23_788);
        assert_eq!(window.observability_lag_commit_count, 11);

        // Recovery opportunities, under this module's strict "touched the same files as
        // t_err's own commit" definition: 0. This is a real, honest finding, not an
        // under-count bug -- no commit after 66cb59b1 touches crown_local.rs again before
        // this audit (confirmed via `git show --stat` on every intervening commit this
        // session); every later commit in this window only ever re-documents the
        // separately-wired EXTERNAL tail. A broader "any action, not just same-file edits"
        // definition (closer to the paper's own F2 framing) would count differently; see
        // module Limitations on why this module chose the narrower, more mechanically
        // verifiable definition instead.
        assert_eq!(window.recovery_opportunities, 0);
    }
}
