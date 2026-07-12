-module(arazzo_runner_atomvm_differential_test).
-include_lib("eunit/include/eunit.hrl").
-include("arazzo_runner.hrl").

%% PROJ-761 (docs/jira/v26.7.11/tickets/index.md; PRD v26.7.11 7.9 lines 431-436,
%% requirement 24 "OTP/AtomVM differential conformance corpus", DoD line 1096
%% "AtomVM passes the shared semantic conformance corpus").
%%
%% > For identical AIR and identical ordered admitted event corpus, OTP and
%% > AtomVM SHALL produce equivalent: state digest; result digest; refusal
%% > class; command sequence.
%%
%% This is the real differential harness PROJ-760's own PROOF_OF_EQUIVALENCE.md
%% names as "the actual evidence surface" (that document's prose "proof by
%% structural induction" is explicitly retired as evidence of record -- see its
%% own UNVERIFIED banner). apps/arazzo_atomvm/test/arazzo_atomvm_SUITE.erl was
%% left as a breadcrumb pointing here; see that file for the pointer.
%%
%% ## Why this file lives under apps/arazzo_runner/test/, not apps/arazzo_atomvm
%%
%% This harness needs #runner_state{}/#workflow_identity{} (arazzo_runner.hrl)
%% to read the OTP path's core context back out of arazzo_runner_workflow's ETS
%% state. Erlang record definitions require textual -include, and this app's
%% own test/ directory is the only place that resolves for free, matching every
%% sibling test in this directory (arazzo_runner_workflow_test.erl,
%% arazzo_runner_broker_test.erl). No such friction exists in the other
%% direction: the AtomVM side (atomvm_runner, arazzo_atomvm_workflow) is called
%% here purely through its exported functions and air_core's own opaque
%% context() accessors (ready_steps/1, get_env/1, get_history/1) -- exactly the
%% same opacity discipline arazzo_atomvm_workflow.erl's own comments already
%% describe ("air_core:context() -- opaque outside air_core.erl") -- so no
%% cross-app include is ever needed for that half. Calling another app's
%% exported functions across apps/ requires no include at all; only records and
%% macros do.
%%
%% ## Shared corpus
%%
%% corpus_workflow/0 and corpus_events/0 below are a fresh copy of the exact
%% shape PROJ-756's apps/air_core/test/air_core_corpus_test.erl established
%% (same step graph, same event order): a linear segment (merge -> finalize,
%% and the audit leaf), a real AND-join (merge requires BOTH gather_a and
%% gather_b), and one genuine failure (audit times out mid-sequence). Kept as
%% an independent copy rather than calling into that other test module's
%% private (unexported) functions -- this is a new, self-contained corpus that
%% happens to share PROJ-756's already-established shape/conventions, per this
%% ticket's own instruction to model on "the existing working pattern".
%%
%% ## Method: driving two structurally different wrappers with one corpus
%%
%% arazzo_runner_workflow speaks a "reaction event" vocabulary (`result`,
%% `timeout`, ...) that it translates internally into air_core's own
%% `{step_completed,_,_}` / `{step_failed,_,_}` events (see
%% arazzo_runner_workflow:handle_reaction/3). arazzo_atomvm_workflow has no
%% such layer -- it forwards the air_core event straight through. otp_event/1
%% below is the one-line translation from this corpus's air_core-shaped events
%% into the OTP side's reaction vocabulary; the AtomVM side takes corpus_events()
%% unmodified. Both ultimately reach the identical call
%% `air_core:transition(Event, Context)` with the identical Event and an
%% equivalent Context -- that shared call is what this harness actually
%% observes (see "Command sequence capture" below), so the difference in outer
%% vocabulary does not weaken the comparison.
%%
%% otp_event/1 only handles the two reaction classes this corpus actually
%% needs (`result`, `timeout`); anything else is a deliberate function_clause
%% (fail loud, not a silent default) -- this harness does not claim to cover
%% retry_due/acknowledgment/child_complete/child_refused/admission_result.
%%
%% ## Command sequence capture (the one dimension neither wrapper exposes
%% symmetrically)
%%
%% state/result/refusal-class are all recoverable from air_core's own exported
%% accessors on whatever Context each wrapper hands back (get_runner_state/1's
%% #runner_state.core for OTP; atomvm_runner:get_state/1 for AtomVM). Command
%% sequence (air_core's C, the dispatch_step commands a transition computes) is
%% NOT symmetric: the OTP side routes every command through
%% arazzo_runner_broker:dispatch/4 and keeps an audit trail
%% (#runner_state.broker_dispatches); the AtomVM side computes the identical C
%% and then -- by design, see arazzo_atomvm_workflow.erl's loop/2 comment,
%% "Commands is currently unconsumed: PROJ-758 ... is not yet built [for
%% AtomVM]" -- discards it. There is no exported AtomVM-side accessor for it at
%% all. This is a genuine, disclosed asymmetry (see the report this ticket
%% produces), not a gap this harness papers over: to get a fair, symmetric
%% observation for BOTH sides, this harness uses the same technique for both --
%% Erlang call tracing (`erlang:trace/3` + `erlang:trace_pattern/2`) on the
%% wrapper's own process, capturing the actual return value of every
%% `air_core:transition/2` call that process makes, in call order. This adds no
%% code to either wrapper (apps/arazzo_runner/src, apps/arazzo_atomvm/src,
%% apps/atomvm_runner/src are untouched by this ticket), is a standard,
%% built-in BEAM facility (not a NIF, not a new dependency), and is exactly as
%% real for the OTP path as for the AtomVM path -- both are observed the same
%% way, so neither side's result is more or less trustworthy than the other's.
%% As a cross-check, test_otp_broker_dispatch_trail_matches_traced_commands/0
%% below confirms the OTP side's own natively-exposed broker_dispatches trail
%% agrees with what tracing captured, so the tracing technique is not the only
%% evidence for that half.
%%
%% Because each wrapper process handles one message at a time (a plain
%% `receive` loop on both sides) and this harness waits for one full
%% call+return_from trace pair before sending the next corpus event, trace
%% messages arrive in the same order the corpus was fed in -- no reordering
%% risk, no polling/sleep-based synchronization needed anywhere in this file.

%% ---------------------------------------------------------------------
%% Fixture
%% ---------------------------------------------------------------------

arazzo_runner_atomvm_differential_test_() ->
    {setup,
     fun setup/0,
     fun cleanup/1,
     fun(_) ->
         [
             {"OTP and AtomVM, driven by the identical ordered admitted-event "
              "corpus (linear segment + AND-join + one failure), produce "
              "identical state digest, result digest, refusal class, and "
              "command sequence",
              {timeout, 30, fun test_differential_four_dimensions_match/0}},
             {"the OTP path's own natively-exposed broker_dispatches trail "
              "agrees with what call-tracing captured -- the tracing "
              "technique is cross-checked, not the sole witness, for the "
              "one side that has a native accessor",
              {timeout, 30, fun test_otp_broker_dispatch_trail_matches_traced_commands/0}},
             {"repeating the full differential comparison 3 independent "
              "times from scratch produces byte-identical digests and "
              "command trails every time -- not asserted, actually run and "
              "compared",
              {timeout, 90, fun test_differential_deterministic_across_repeated_runs/0}},
             {"PROJ-762: an intentional divergence (one extra event sent "
              "to only the AtomVM side) makes compare_four_dimensions/2 "
              "return an OTP_ATOMVM_SEMANTIC_DRIFT refusal naming the "
              "correct (root-cause) dimension, not merely ok or a wrong one",
              {timeout, 30, fun test_semantic_drift_refusal_fires_on_correct_dimension/0}}
         ]
     end}.

setup() ->
    Dir = filename:join(
        "/tmp",
        "arazzo_diff_eunit_" ++ integer_to_list(erlang:unique_integer([positive]))
    ),
    ok = filelib:ensure_dir(filename:join(Dir, "x")),
    ok = application:set_env(arazzo_runner, state_dir, Dir),
    %% Distinct DETS table name from every other eunit module in this app (same
    %% rationale as arazzo_runner_broker_test.erl's setup/0: dets:open_file/2
    %% raises incompatible_arguments if the same table name is reopened
    %% pointing at a different file while a prior reference is still live).
    TableName = list_to_atom(
        "arazzo_diff_eunit_state_" ++ integer_to_list(erlang:unique_integer([positive]))),
    ok = application:set_env(arazzo_runner, dets_table, TableName),
    {ok, _Started} = application:ensure_all_started(arazzo_runner),
    %% Force real infra bootstrap (pg scope, io-worker pool, broker ETS ledger
    %% tables) before any test below runs -- same warm-up arazzo_runner_broker_
    %% test.erl's setup/0 does, for the same reason: these tests must not
    %% depend on which other eunit module happened to run first in this VM.
    BootstrapId = <<"wf-diff-infra-bootstrap">>,
    {ok, _BootstrapPid} = arazzo_runner_workflow:start_link(bootstrap_start_spec(BootstrapId)),
    Dir.

cleanup(Dir) ->
    catch application:stop(arazzo_runner),
    catch arazzo_runner_identity:close_table(),
    catch os:cmd("rm -rf " ++ Dir),
    ok = application:unset_env(arazzo_runner, state_dir),
    ok = application:unset_env(arazzo_runner, dets_table),
    ok.

bootstrap_start_spec(WorkflowId) ->
    maps:merge(sample_identity(WorkflowId), #{
        workflow_def => #{steps => #{}},
        active_steps => [],
        env => #{},
        history => []
    }).

%% ---------------------------------------------------------------------
%% Shared corpus (see module doc "Shared corpus" above)
%% ---------------------------------------------------------------------

%% Workflow shape (identical to PROJ-756's air_core_corpus_test.erl):
%%   init -> [gather_a, gather_b, audit]
%%   gather_a -> [merge]           (predecessor of merge)
%%   gather_b -> [merge]           (predecessor of merge; AND/join)
%%   audit -> []                   (fails; leaf, no downstream effect)
%%   merge -> [finalize]           (linear: ready only once BOTH gathers done)
%%   finalize -> []
%%
%% ## Why every step except `init` binds outputs from a typed `__result__` op
%%
%% This ticket's own broker fix (arazzo_runner_broker.erl's admit_return/3
%% wired into do_dispatch_actuate/6) means every dispatch_step command
%% apply_transition/4 produces now genuinely round-trips through
%% arazzo_runner_broker:dispatch/4 -> the echo io-worker
%% (execute_io_request/1) -> admit_return/3, all INLINE and SYNCHRONOUS
%% inside the very air_core:transition/2 call this harness traces. A step
%% whose outputs never reference `__result__` under a type-coercing op (like
%% `init`'s own {literal, true} below -- `init` is never dispatched via the
%% broker at all, since it is in the OTP/AtomVM start spec's active_steps,
%% not a transition-produced command, so this does not apply to it) derives
%% an EMPTY required_result_types set, which the echo worker's
%% `{processed, StepDef}` tuple vacuously satisfies -- so the step
%% auto-admits and self-completes THE INSTANT it becomes ready, via a second,
%% harness-uninitiated `air_core:transition/2` call the SAME process makes
%% to itself.
%%
%% That is fatal to this harness's core technique (see module doc "Method"
%% and "Command sequence capture" above): drive_one/2 sends exactly one
%% corpus event and waits for exactly one air_core:transition/2 return_from
%% trace message per call. An auto-admitted gather_a/gather_b/audit/merge/
%% finalize would each synthesize an EXTRA, harness-uninitiated
%% air_core:transition/2 call (and trace message) interleaved with -- but not
%% requested by -- the harness's own corpus_events() sequence, permanently
%% desynchronizing which traced Commands belong to which corpus event, and
%% would (for `audit` specifically) force-complete it as a SUCCESS before
%% corpus_events()'s own {step_failed, <<"audit">>, timeout} ever arrives --
%% corrupting the one genuine failure/AND-join case this corpus exists to
%% exercise.
%%
%% gather_a/gather_b/audit/merge/finalize's outputs below therefore each
%% require the eventually-admitted result to be a real boolean
%% (`{op, 'and', {var, '__result__'}, {literal, true}}`) -- required_result_
%% types/1 derives `[boolean]` from that op, and the echo worker's tuple
%% response never satisfies `is_boolean/1`, so admit_return/3's structure
%% stage (RETURN_STRUCTURE_REFUSED, PROJ-785) deterministically refuses
%% every one of the broker's own auto-dispatch attempts for these 5 steps,
%% every run. Only this file's own corpus_events() (Result = true for every
%% genuine completion) ever admits them -- restoring the one-event-in/
%% one-transition-out invariant this harness depends on. See
%% test_otp_broker_dispatch_trail_matches_traced_commands/0 for the
%% cross-check that every one of these 5 broker dispatches really is
%% actuated-then-refused, not skipped.
corpus_workflow() ->
    #{
        steps => #{
            <<"init">> => #{
                outputs => [{bind, <<"init_done">>, {literal, true}}],
                next => [<<"gather_a">>, <<"gather_b">>, <<"audit">>]
            },
            <<"gather_a">> => #{
                outputs => [{bind, <<"a_done">>, {op, 'and', {var, '__result__'}, {literal, true}}}],
                next => [<<"merge">>]
            },
            <<"gather_b">> => #{
                outputs => [{bind, <<"b_done">>, {op, 'and', {var, '__result__'}, {literal, true}}}],
                next => [<<"merge">>]
            },
            <<"audit">> => #{
                outputs => [{bind, <<"audit_note">>, {op, 'and', {var, '__result__'}, {literal, true}}}],
                next => []
            },
            <<"merge">> => #{
                outputs => [{bind, <<"merged">>, {op, 'and', {var, '__result__'}, {literal, true}}}],
                next => [<<"finalize">>]
            },
            <<"finalize">> => #{
                outputs => [{bind, <<"status">>, {op, 'and', {var, '__result__'}, {literal, true}}}],
                next => []
            }
        }
    }.

%% Fixed, ordered event sequence (air_core-shaped): gather_a completes, then
%% audit *times out* (the one failure/refusal case -- proving it does not
%% disturb the pending AND/join on merge), then gather_b completes -- only
%% then are ALL of merge's predecessors satisfied.
%%
%% Result is `true` (a real boolean) for every {step_completed, ...} event
%% except `init`'s own -- see corpus_workflow/0's doc comment: gather_a/
%% gather_b/merge/finalize's outputs now do real bool-typed arithmetic on
%% `__result__`, so their genuine completion (via exactly these events, the
%% only route left that can admit them) must supply a value that op can
%% decode without badarg-ing inside air_core's real eval_expr_nif. `init`'s
%% own outputs stay a bare {literal, true} (untouched), so its Result
%% (`ok`, unchanged) is never evaluated against `__result__` at all.
%% `audit` is never a {step_completed, ...} event in this corpus (only
%% {step_failed, ...} below) -- handle_step_failed/3 never calls
%% bind_outputs/3, so audit's own typed outputs are never evaluated for
%% real; they exist solely to make its broker auto-dispatch attempt refuse
%% (see corpus_workflow/0).
corpus_events() ->
    [
        {step_completed, <<"init">>, ok},
        {step_completed, <<"gather_a">>, true},
        {step_failed, <<"audit">>, timeout},
        {step_completed, <<"gather_b">>, true},
        {step_completed, <<"merge">>, true},
        {step_completed, <<"finalize">>, true}
    ].

%% Translates a corpus (air_core-shaped) event into arazzo_runner_workflow's
%% reaction-event vocabulary. Deliberately partial (function_clause on
%% anything else, fail loud) -- see module doc.
otp_event({step_completed, StepId, Result}) -> {result, StepId, Result};
otp_event({step_failed, StepId, timeout}) -> {timeout, StepId}.

%% ---------------------------------------------------------------------
%% Identity + workflow-def fixtures for the OTP side
%% ---------------------------------------------------------------------

%% Real, non-empty correlation_id and receipt_head -- so arazzo_runner_broker
%% actuates every command instead of refusing CORRELATION_MISSING /
%% BROKER_RECEIPT_PRECONDITION_MISSING. Those two refusal codes are PROJ-758's
%% own already-tested scope (arazzo_runner_broker_test.erl); an
%% under-configured identity here would inject a broker-layer refusal that has
%% nothing to do with AIR transition semantics, contaminating the very
%% comparison this ticket exists to make. "Identical AIR and identical event
%% corpus" (the PRD's own precondition) implies both runners are configured to
%% actually execute, not one of them deliberately misconfigured.
sample_identity(WorkflowId) ->
    #{
        workflow_id => WorkflowId,
        parent_workflow_id => undefined,
        arazzo_workflow_id => <<"arazzo-wf-diff">>,
        source_powl_region_id => <<"powl-region-diff">>,
        dispatch_id => <<"dispatch-diff-1">>,
        correlation_id => <<"corr-diff-1">>,
        source_digest => <<"src-digest-diff">>,
        projection_digest => <<"proj-digest-diff">>,
        receipt_head => <<"receipt-head-diff">>,
        replay_id => <<"replay-diff-1">>
    }.

otp_start_spec(WorkflowId) ->
    maps:merge(sample_identity(WorkflowId), #{
        workflow_def => corpus_workflow(),
        active_steps => [<<"init">>],
        env => #{},
        history => []
    }).

%% ---------------------------------------------------------------------
%% Driving each path through the shared corpus (see module doc "Method" and
%% "Command sequence capture" above)
%% ---------------------------------------------------------------------

%% # Complexity
%% O(|Events|): one air_core:transition/2 call (and its trace round trip) per
%% corpus event; each call itself bounded by air_core's own documented
%% O(|next(StepId)|). No traversal beyond the corpus length.
-spec run_otp(binary(), [tuple()]) -> {term(), [[binary()]]}.
run_otp(WorkflowId, Events) ->
    {ok, Pid} = arazzo_runner_workflow:start_link(otp_start_spec(WorkflowId)),
    1 = erlang:trace(Pid, true, [call]),
    erlang:trace_pattern({air_core, transition, 2}, [{'_', [], [{return_trace}]}]),
    CommandTrail = [drive_one(Pid, fun() ->
                        ok = arazzo_runner_workflow:dispatch_event(Pid, otp_event(Event))
                    end) || Event <- Events],
    erlang:trace(Pid, false, [call]),
    {ok, RS} = arazzo_runner_workflow:get_runner_state(WorkflowId),
    {RS#runner_state.core, CommandTrail}.

-spec run_atomvm(binary(), [tuple()]) -> {term(), [[binary()]]}.
run_atomvm(WorkflowId, Events) ->
    {ok, Pid} = atomvm_runner:start(WorkflowId, #{
        workflow => corpus_workflow(),
        active_steps => [<<"init">>],
        env => #{},
        history => []
    }),
    1 = erlang:trace(Pid, true, [call]),
    erlang:trace_pattern({air_core, transition, 2}, [{'_', [], [{return_trace}]}]),
    CommandTrail = [drive_one(Pid, fun() ->
                        ok = atomvm_runner:dispatch_event(Pid, Event)
                    end) || Event <- Events],
    erlang:trace(Pid, false, [call]),
    {ok, Core} = atomvm_runner:get_state(Pid),
    ok = atomvm_runner:stop(Pid),
    {Core, CommandTrail}.

%% Sends exactly one event via SendFun, waits for the one resulting
%% air_core:transition/2 return_from trace message on Pid, and returns the
%% sorted dispatch_step StepIds it produced. Sorting within a single
%% transition's command set mirrors PROJ-756's air_core_corpus_test.erl
%% convention (`lists:sort/1` over CommandIds) -- which step becomes ready
%% first within one event is a bitmask-iteration/`next`-list-order
%% implementation detail, not part of either corpus's canonical content; the
%% ORDER OF EVENTS (this list's own position) is what is semantically
%% meaningful and is preserved untouched.
drive_one(Pid, SendFun) ->
    ok = SendFun(),
    {_NewCore, Commands} = wait_for_transition_return(Pid),
    lists:sort([StepId || {dispatch_step, StepId, _StepDef} <- Commands]).

%% Splits List into successive chunks of the given Lengths, in order.
%% length(List) must equal lists:sum(Lengths) -- any mismatch is a real bug
%% in the caller's accounting and fails loud via lists:split/2's own badarg,
%% not a silent truncation.
chunk_by_lengths([], []) -> [];
chunk_by_lengths(List, [Len | Rest]) ->
    {Chunk, Remainder} = lists:split(Len, List),
    [Chunk | chunk_by_lengths(Remainder, Rest)].

wait_for_transition_return(Pid) ->
    receive
        {trace, Pid, call, {air_core, transition, _Args}} ->
            wait_for_transition_return(Pid);
        {trace, Pid, return_from, {air_core, transition, 2}, Result} ->
            Result
    after 5000 ->
        error({timeout_waiting_for_air_core_transition, Pid})
    end.

%% ---------------------------------------------------------------------
%% The four PRD 7.9 comparison dimensions
%% ---------------------------------------------------------------------

%% State digest: the full per-event command trail plus final ready-set, env,
%% and history -- each explicitly sorted/ordered rather than relying on
%% map/bitmask iteration order (same discipline as PROJ-756's canonical_bytes/1).
%% get_history/1 is most-recent-first internally (each transition prepends);
%% reversed here to chronological order before hashing.
state_bytes(Core, CommandTrail) ->
    Canonical = {
        commands, CommandTrail,
        final_ready, lists:sort(air_core:ready_steps(Core)),
        final_env, lists:sort(maps:to_list(air_core:get_env(Core))),
        final_history, lists:reverse(air_core:get_history(Core))
    },
    term_to_binary(Canonical).

%% Result digest: distinct from state digest -- just the final bound output
%% values (the workflow's "result"), not the full transition/command trail.
result_bytes(Core) ->
    term_to_binary({result_env, lists:sort(maps:to_list(air_core:get_env(Core)))}).

%% Refusal class: derived from each path's OWN observed history (not
%% hardcoded from the input corpus), so this genuinely checks that both paths
%% recorded the same failures for the same reasons, not merely that they were
%% fed the same input.
refusal_class(Core) ->
    lists:sort([{StepId, Reason} || {step_failed, StepId, Reason} <- air_core:get_history(Core)]).

%% Shells out to the real `b3sum` BLAKE3 reference implementation over a temp
%% file -- same technique and rationale as PROJ-756's air_core_corpus_test.erl
%% blake3_hex/1 (see that module's doc comment for why: extending the
%% air_core_nif crate for one more digest would mean a new NIF export, a new
%% Cargo dependency, and a fresh native build outside the `just`-only
%% crates/ discipline; b3sum is the reference implementation, confirmed
%% present on this machine, invoked as a subprocess -- real BLAKE3, no new
%% native build surface). Copied fresh into this module rather than calling
%% air_core_corpus_test's (unexported, private-to-that-module) version.
blake3_hex(Bytes) ->
    B3sum = case os:find_executable("b3sum") of
        false -> erlang:error(b3sum_not_found);
        Path -> Path
    end,
    TmpPath = tmp_file_path(),
    ok = file:write_file(TmpPath, Bytes),
    Output = try
        Port = erlang:open_port(
            {spawn_executable, B3sum},
            [{args, ["--no-names", TmpPath]}, binary, exit_status, use_stdio, stderr_to_stdout]
        ),
        collect_port_output(Port, <<>>)
    after
        file:delete(TmpPath)
    end,
    string:trim(Output).

collect_port_output(Port, Acc) ->
    receive
        {Port, {data, Data}} ->
            collect_port_output(Port, <<Acc/binary, Data/binary>>);
        {Port, {exit_status, 0}} ->
            Acc;
        {Port, {exit_status, Status}} ->
            erlang:error({b3sum_failed, Status, Acc})
    after 10000 ->
        erlang:error(b3sum_timeout)
    end.

tmp_file_path() ->
    Dir = case os:getenv("TMPDIR") of
        false -> "/tmp";
        D -> D
    end,
    Unique = erlang:unique_integer([positive]),
    filename:join(Dir, "arazzo_diff_corpus_" ++ integer_to_list(Unique) ++ ".bin").

%% ---------------------------------------------------------------------
%% PROJ-762 (Semantic Drift Refusal, docs/jira/v26.7.11/tickets/PROJ-762.md):
%% one real comparison function over the four PRD 7.9 dimensions.
%% ---------------------------------------------------------------------

%% Bundles one runner's observed values for all four PRD 7.9 dimensions
%% (command sequence, state digest, result digest, refusal class) into one
%% map, so `compare_four_dimensions/2` below takes exactly one argument per
%% side instead of four positional pairs.
-spec observe([[binary()]], term()) -> #{
    commands := [[binary()]],
    state := binary(),
    result := binary(),
    refusal := [{binary(), term()}]
}.
observe(CommandTrail, Core) ->
    #{
        commands => CommandTrail,
        state => blake3_hex(state_bytes(Core, CommandTrail)),
        result => blake3_hex(result_bytes(Core)),
        refusal => refusal_class(Core)
    }.

%% The single real comparison function this ticket adds: the four
%% dimension-by-dimension `?assertEqual` checks that used to live inline in
%% `test_differential_four_dimensions_match/0` (command trail, state digest,
%% result digest, refusal class -- same order that test already checked
%% them in), now one function returning `ok` when all four agree or a
%% refusal naming the first dimension that does not.
%%
%% Matches the `{refused, Code, Details}` idiom `arazzo_runner_broker.erl`
%% already established (e.g. `{refused, 'CORRELATION_MISSING', #{...}}`):
%% `Code` is the fixed atom `'OTP_ATOMVM_SEMANTIC_DRIFT'` (every call to
%% this function that fails means the same thing -- the two engines
%% disagreed on an identical AIR + identical event corpus), and `Details`
%% names which dimension diverged plus both observed values, so a caller
%% does not need to re-derive which of the four checks failed.
%%
%% Checks command trail before state digest deliberately, even though
%% state_bytes/2's own canonical form embeds the command trail (see its doc
%% comment): a command-trail divergence would otherwise also be reported as
%% a state-digest divergence, which is technically true but not the root
%% cause a caller should act on.
-spec compare_four_dimensions(map(), map()) ->
    ok | {refused, 'OTP_ATOMVM_SEMANTIC_DRIFT', {atom(), term(), term()}}.
compare_four_dimensions(ObsOtp, ObsAtom) ->
    first_mismatch([
        {command_trail, maps:get(commands, ObsOtp), maps:get(commands, ObsAtom)},
        {state_digest, maps:get(state, ObsOtp), maps:get(state, ObsAtom)},
        {result_digest, maps:get(result, ObsOtp), maps:get(result, ObsAtom)},
        {refusal_class, maps:get(refusal, ObsOtp), maps:get(refusal, ObsAtom)}
    ]).

%% Returns `ok` if every `{Dimension, ValueOtp, ValueAtom}` triple has
%% `ValueOtp =:= ValueAtom` (checked by matching the same variable name
%% twice in one pattern -- exact-term equality, the same strictness
%% `?assertEqual` already relied on for these same four checks), or the
%% first triple that does not, wrapped as an `'OTP_ATOMVM_SEMANTIC_DRIFT'`
%% refusal. Order of `Dimensions` is the priority order for which dimension
%% gets reported when more than one has diverged.
-spec first_mismatch([{atom(), term(), term()}]) ->
    ok | {refused, 'OTP_ATOMVM_SEMANTIC_DRIFT', {atom(), term(), term()}}.
first_mismatch([]) ->
    ok;
first_mismatch([{_Dimension, Value, Value} | Rest]) ->
    first_mismatch(Rest);
first_mismatch([{Dimension, ValueOtp, ValueAtom} | _Rest]) ->
    {refused, 'OTP_ATOMVM_SEMANTIC_DRIFT', {Dimension, ValueOtp, ValueAtom}}.

%% ---------------------------------------------------------------------
%% Golden command trail (pinned, human-legible -- the direct proof this
%% corpus exercises the AND-join and the failure case, independent of the
%% digest checks below). "audit" appears alongside gather_a/gather_b in the
%% first entry (all 3 become ready the instant init completes); "merge" does
%% NOT appear until BOTH gathers have completed.
%% ---------------------------------------------------------------------
-define(EXPECTED_COMMAND_TRAIL, [
    [<<"audit">>, <<"gather_a">>, <<"gather_b">>],
    [],
    [],
    [<<"merge">>],
    [<<"finalize">>],
    []
]).

%% Golden digests: pinned expected values for this exact corpus + this exact
%% canonicalization, computed once and checked in here so a future change to
%% either the corpus or the readiness/env logic it exercises must consciously
%% update these constants rather than silently drifting (same discipline as
%% PROJ-756's ?GOLDEN_DIGEST).
%%
%% Recomputed (this ticket, the broker fix that wired admit_return/3 into
%% do_dispatch_actuate/6): corpus_workflow/0's outputs for gather_a/gather_b/
%% merge/finalize changed from bare {literal, true} to a real bool-typed op
%% on `__result__` (see that function's own doc comment for why), and
%% corpus_events() correspondingly changed those 4 steps' completion Result
%% from `ok` to `true` -- both feed state_bytes/2's `final_env` and
%% `final_history` components, so the digest changes even though
%% ?EXPECTED_COMMAND_TRAIL, refusal_class, and the OTP/AtomVM agreement
%% (compare_four_dimensions/2 = ok) are all unchanged. Values below are the
%% real output of this exact test's own blake3_hex/1 (b3sum) computation
%% against the corpus as it stands now, not hand-derived.
-define(GOLDEN_STATE_DIGEST, <<"8fe1d48f66792dfbb606fdfe7a2a1bbc73f6c066ca7732122fe9da16b2d95c52">>).
-define(GOLDEN_RESULT_DIGEST, <<"fb71b91bc8b7705ae73656181f51d8c22e117fadf555cd2a37ba030bf452146a">>).

%% ---------------------------------------------------------------------
%% Tests
%% ---------------------------------------------------------------------

test_differential_four_dimensions_match() ->
    {CoreOtp, CmdOtp} = run_otp(<<"wf-diff-otp-1">>, corpus_events()),
    {CoreAtom, CmdAtom} = run_atomvm(<<"wf-diff-atomvm-1">>, corpus_events()),

    %% Command sequence matches the human-legible expected trail on both
    %% paths, ahead of the general four-dimension comparison below.
    ?assertEqual(?EXPECTED_COMMAND_TRAIL, CmdOtp),
    ?assertEqual(?EXPECTED_COMMAND_TRAIL, CmdAtom),

    ObsOtp = observe(CmdOtp, CoreOtp),
    ObsAtom = observe(CmdAtom, CoreAtom),

    %% PROJ-762: the single real comparison function. `ok` here means all
    %% four PRD 7.9 dimensions (command trail, state digest, result digest,
    %% refusal class) agreed between OTP and AtomVM.
    ?assertEqual(ok, compare_four_dimensions(ObsOtp, ObsAtom)),

    %% Refusal class: exactly the one failure/refusal case this corpus
    %% carries (audit timing out) -- confirmed on the OTP side; the AtomVM
    %% side agreeing is already covered by the `ok` assertion above.
    ?assertEqual([{<<"audit">>, timeout}], maps:get(refusal, ObsOtp)),

    ?assertEqual(?GOLDEN_STATE_DIGEST, maps:get(state, ObsOtp)),
    ?assertEqual(?GOLDEN_RESULT_DIGEST, maps:get(result, ObsOtp)),
    ok.

%% PROJ-762's negative fixture: "one extra event sent to only one side"
%% (the pattern this ticket names as an acceptable seed for a real
%% divergence). AtomVM receives one extra, well-formed event beyond the
%% shared corpus -- `finalize` completing a second time. air_core:transition/2
%% handles this without erroring: handle_step_completed/3's bitmask/map
%% operations are no-ops on an already-completed step's bit (see
%% air_core.erl), and `finalize`'s `next => []` means no new commands are
%% dispatched either. What it DOES do is append a second
%% `{step_completed, <<"finalize">>, true}` history entry and one extra
%% (empty) command-trail chunk that the OTP run -- fed the unmodified
%% corpus -- never produces. This is a genuine divergence the harness
%% itself produces by actually running one extra transition, not a
%% hand-faked value standing in for one. Result is `true` (not `ok`) for
%% the same reason every other {step_completed, ...} event in this corpus
%% now is (see corpus_workflow/0's doc comment): finalize's outputs do real
%% bool-typed arithmetic on `__result__`, and this extra event drives a
%% genuine air_core:transition/2 call (on the AtomVM side, which never
%% routes through the broker at all -- see module doc "Method") that would
%% otherwise badarg inside eval_expr_nif on a non-boolean Result.
%%
%% Because state_bytes/2's canonical form embeds the command trail (see its
%% doc comment), this divergence is technically visible in both
%% `command_trail` and `state_digest`; `compare_four_dimensions/2` checks
%% `command_trail` first (see its doc comment), so `command_trail` is the
%% dimension this test expects to see reported -- confirming the refusal
%% fires on the correct (root-cause) dimension, not merely *a* dimension.
test_semantic_drift_refusal_fires_on_correct_dimension() ->
    {CoreOtp, CmdOtp} = run_otp(<<"wf-diff-drift-otp-1">>, corpus_events()),
    DriftEvents = corpus_events() ++ [{step_completed, <<"finalize">>, true}],
    {CoreAtom, CmdAtom} = run_atomvm(<<"wf-diff-drift-atomvm-1">>, DriftEvents),

    %% Confirm the seeded divergence is real before trusting it as the
    %% negative fixture: the extra event really did produce a longer,
    %% differently-shaped command trail on the AtomVM side.
    ?assertEqual(7, length(CmdAtom)),
    ?assertEqual(6, length(CmdOtp)),
    ?assertNotEqual(CmdOtp, CmdAtom),

    ObsOtp = observe(CmdOtp, CoreOtp),
    ObsAtom = observe(CmdAtom, CoreAtom),
    ?assertEqual(
        {refused, 'OTP_ATOMVM_SEMANTIC_DRIFT', {command_trail, CmdOtp, CmdAtom}},
        compare_four_dimensions(ObsOtp, ObsAtom)
    ),
    ok.

%% Cross-check: the OTP side has its own native, non-traced record of which
%% dispatch_step commands it routed (#runner_state.broker_dispatches, one
%% entry per command, in the order apply_transition/4's foldl processed
%% them). This proves the call-tracing technique used above is not the only
%% evidence for the OTP half of the comparison.
test_otp_broker_dispatch_trail_matches_traced_commands() ->
    WorkflowId = <<"wf-diff-otp-crosscheck-1">>,
    {Core, CmdOtp} = run_otp(WorkflowId, corpus_events()),
    {ok, RS} = arazzo_runner_workflow:get_runner_state(WorkflowId),
    ?assertEqual(Core, RS#runner_state.core),
    %% broker_dispatches is one flat, chronological list (no event-boundary
    %% markers of its own) in the UNSORTED order apply_transition/4's foldl
    %% over air_core's Commands produced it -- i.e. the corpus workflow's own
    %% `next` list order, not sorted. CmdOtp (from drive_one/2) is already
    %% sorted WITHIN each event (see drive_one/2's doc comment: within-event
    %% order is a `next`-list/bitmask implementation detail, not canonical
    %% content). To compare fairly, re-partition the flat native trail back
    %% into the same per-event chunk sizes CmdOtp has, then sort each chunk
    %% the same way -- this checks the real invariant (same STEP SET
    %% dispatched per event, same event order) without asserting an
    %% ordering neither side actually promises within one event.
    NativeStepIds = [StepId || {StepId, _BrokerResult} <- lists:reverse(RS#runner_state.broker_dispatches)],
    ChunkLengths = [length(Chunk) || Chunk <- CmdOtp],
    NativeChunkedSorted = [lists:sort(Chunk) || Chunk <- chunk_by_lengths(NativeStepIds, ChunkLengths)],
    ?assertEqual(CmdOtp, NativeChunkedSorted),
    %% And every one of them was genuinely ACTUATED (the identity fixture
    %% really does carry a working correlation_id + receipt_head, not just
    %% well-typed placeholders -- dispatch/4 got past every pre-actuation
    %% gate and the echo io-worker really round-tripped) but deterministically
    %% REFUSED at the return-admission structure stage (RETURN_STRUCTURE_
    %% REFUSED, PROJ-785): corpus_workflow/0's outputs for every step except
    %% `init` require a real boolean `__result__`, which the echo worker's
    %% `{processed, StepDef}` tuple never supplies (see corpus_workflow/0's
    %% own doc comment for why this is deliberate, not a bug). That is what
    %% keeps this harness's one-event-in/one-transition-out synchronization
    %% sound: no corpus step is ever completed by the broker's own closed
    %% loop behind this harness's back, only by corpus_events() itself.
    ?assertEqual(
        lists:duplicate(length(NativeStepIds), 'RETURN_STRUCTURE_REFUSED'),
        [case Result of {refused, Code, _Ctx} -> Code; Other -> Other end
         || {_StepId, Result} <- lists:reverse(RS#runner_state.broker_dispatches)]
    ),
    ok.

test_differential_deterministic_across_repeated_runs() ->
    Run = fun(N) ->
        Suffix = integer_to_list(N) ++ "-" ++ integer_to_list(erlang:unique_integer([positive])),
        {CoreOtp, CmdOtp} = run_otp(list_to_binary("wf-diff-det-otp-" ++ Suffix), corpus_events()),
        {CoreAtom, CmdAtom} = run_atomvm(list_to_binary("wf-diff-det-atomvm-" ++ Suffix), corpus_events()),
        {
            blake3_hex(state_bytes(CoreOtp, CmdOtp)),
            blake3_hex(state_bytes(CoreAtom, CmdAtom)),
            blake3_hex(result_bytes(CoreOtp)),
            blake3_hex(result_bytes(CoreAtom)),
            refusal_class(CoreOtp),
            refusal_class(CoreAtom),
            CmdOtp,
            CmdAtom
        }
    end,
    R1 = Run(1),
    R2 = Run(2),
    R3 = Run(3),
    ?assertEqual(R1, R2),
    ?assertEqual(R2, R3),
    ok.
