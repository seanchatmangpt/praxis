-module(air_core_corpus_test).
-include_lib("eunit/include/eunit.hrl").

%% PROJ-756: a fixed, checked-in event corpus run through air_core:transition/2,
%% content-hashed with BLAKE3 (this repo's hashing convention -- see
%% docs/CORE_TEAM_DISCIPLINE.md and crates/praxis-graphlaw's receipt paths)
%% and asserted byte-identical across repeated runs. This is the Erlang-side
%% equivalent of the Rust determinism checks used elsewhere in this repo
%% this session: run N times, diff/compare the digests, don't just claim
%% determinism.
%%
%% BLAKE3 tooling reachable from Erlang here: the air_core_nif crate
%% (apps/air_core/native/air_core_nif) does NOT wire up blake3 today (it
%% only exports eval_expr_nif, and its Cargo.toml has no blake3 dependency)
%% -- extending it would mean adding a new NIF export, a new Cargo
%% dependency, and a fresh `cargo build` of a cdylib outside the `just`-only
%% crates/ discipline this repo otherwise enforces. Instead this uses the
%% `b3sum` CLI (the reference BLAKE3 implementation, confirmed present on
%% this machine at /opt/homebrew/bin/b3sum), invoked as a subprocess over a
%% temp file. That is real BLAKE3 (not "a different hash"), with no new
%% native build surface.
%%
%% The corpus below deliberately includes the AND/join fix from PROJ-756
%% (Bug 1, see air_core.erl's pred_mask_map/completed_mask): "merge" has
%% two real predecessors (gather_a, gather_b) and must not be dispatched
%% until both have completed. It also includes a step_failed event
%% (audit), not just step_completed events on a linear chain.

%% Workflow shape:
%%   init -> [gather_a, gather_b, audit]
%%   gather_a -> [merge]           (predecessor of merge)
%%   gather_b -> [merge]           (predecessor of merge; AND/join)
%%   audit -> []                   (fails; leaf, no downstream effect)
%%   merge -> [finalize]           (ready only once BOTH gather_a & gather_b done)
%%   finalize -> []
corpus_workflow() ->
    #{
        steps => #{
            <<"init">> => #{
                outputs => [{bind, <<"init_done">>, {literal, true}}],
                next => [<<"gather_a">>, <<"gather_b">>, <<"audit">>]
            },
            <<"gather_a">> => #{
                outputs => [{bind, <<"a_done">>, {literal, true}}],
                next => [<<"merge">>]
            },
            <<"gather_b">> => #{
                outputs => [{bind, <<"b_done">>, {literal, true}}],
                next => [<<"merge">>]
            },
            <<"audit">> => #{
                outputs => [{bind, <<"audit_note">>, {literal, <<"pending">>}}],
                next => []
            },
            <<"merge">> => #{
                outputs => [{bind, <<"merged">>, {literal, true}}],
                next => [<<"finalize">>]
            },
            <<"finalize">> => #{
                outputs => [{bind, <<"status">>, {literal, <<"DONE">>}}],
                next => []
            }
        }
    }.

%% Fixed event sequence. Order matters and is part of the corpus: gather_a
%% completes, then audit *fails* (exercising step_failed mid-sequence, and
%% proving it does not disturb the pending AND/join on merge), then
%% gather_b completes -- only at that point are ALL of merge's
%% predecessors satisfied and merge becomes ready.
corpus_events() ->
    [
        {step_completed, <<"init">>, ok},
        {step_completed, <<"gather_a">>, ok},
        {step_failed, <<"audit">>, timeout},
        {step_completed, <<"gather_b">>, ok},
        {step_completed, <<"merge">>, ok},
        {step_completed, <<"finalize">>, ok}
    ].

event_key({step_completed, StepId, _Result}) -> {step_completed, StepId};
event_key({step_failed, StepId, _Reason}) -> {step_failed, StepId}.

%% Runs the fixed corpus from a fresh context and returns {FinalContext,
%% Trail}, where Trail is the ordered, per-event record of exactly which
%% steps were newly dispatched (sorted, so trail content does not depend on
%% bitmask iteration order -- that is an implementation detail, not part of
%% the corpus's canonical content).
run_corpus() ->
    Context0 = air_core:new(#{
        workflow => corpus_workflow(),
        active_steps => [<<"init">>],
        env => #{},
        history => []
    }),
    lists:foldl(
        fun(Event, {Ctx, TrailAcc}) ->
            {NewCtx, Commands} = air_core:transition(Event, Ctx),
            CommandIds = lists:sort([StepId || {dispatch_step, StepId, _StepDef} <- Commands]),
            {EventTag, StepId} = event_key(Event),
            {NewCtx, [{EventTag, StepId, CommandIds} | TrailAcc]}
        end,
        {Context0, []},
        corpus_events()
    ).

%% Canonicalizes a {Context, ReverseTrail} pair (as produced by run_corpus/0)
%% into a deterministic byte sequence: the trail in chronological order,
%% plus the final ready-step set, env, and history, each explicitly sorted
%% or put in a fixed order rather than relying on map/bitmask iteration
%% order. term_to_binary/1 on this fully-normalized tuple (no raw maps left
%% in it) is what actually gets BLAKE3-hashed.
canonical_bytes({FinalContext, ReverseTrail}) ->
    Trail = lists:reverse(ReverseTrail),
    Canonical = {
        trail, Trail,
        final_ready, lists:sort(air_core:ready_steps(FinalContext)),
        final_env, lists:sort(maps:to_list(air_core:get_env(FinalContext))),
        final_history, lists:reverse(air_core:get_history(FinalContext))
    },
    term_to_binary(Canonical).

%% Shells out to the real `b3sum` BLAKE3 reference implementation over a
%% temp file (see module doc for why this, not a NIF extension). Returns
%% the lowercase hex digest as a binary, with no trailing newline/name
%% suffix (--no-names).
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
    filename:join(Dir, "air_core_corpus_" ++ integer_to_list(Unique) ++ ".bin").

%% Golden digest, computed once from this exact corpus + this exact
%% canonicalization and pinned here so a future change to either the
%% AND/join logic or the corpus itself must consciously update this
%% constant rather than silently drifting.
-define(GOLDEN_DIGEST, <<"7277d8a08de8a936103a8baaa65244dcc8676fed8cf857634bb70ff55e508829">>).

%% (1) The corpus's AND/join step ("merge") is NOT dispatched when only
%% gather_a has completed (event 2), and IS dispatched -- exactly once --
%% once gather_b completes too (event 4). This is the direct, human-legible
%% proof that PROJ-756's fix is exercised by this corpus, independent of
%% the digest check below.
corpus_exercises_and_join_test() ->
    {_FinalContext, ReverseTrail} = run_corpus(),
    Trail = lists:reverse(ReverseTrail),
    ?assertEqual(
        [
            {step_completed, <<"init">>, [<<"audit">>, <<"gather_a">>, <<"gather_b">>]},
            {step_completed, <<"gather_a">>, []},
            {step_failed, <<"audit">>, []},
            {step_completed, <<"gather_b">>, [<<"merge">>]},
            {step_completed, <<"merge">>, [<<"finalize">>]},
            {step_completed, <<"finalize">>, []}
        ],
        Trail
    ).

%% (2) Running the whole corpus from scratch three independent times
%% produces byte-identical canonical output and therefore byte-identical
%% BLAKE3 digests every time -- not asserted, actually run and compared.
corpus_digest_is_deterministic_across_repeated_runs_test() ->
    Bytes1 = canonical_bytes(run_corpus()),
    Bytes2 = canonical_bytes(run_corpus()),
    Bytes3 = canonical_bytes(run_corpus()),
    ?assertEqual(Bytes1, Bytes2),
    ?assertEqual(Bytes2, Bytes3),

    Digest1 = blake3_hex(Bytes1),
    Digest2 = blake3_hex(Bytes2),
    Digest3 = blake3_hex(Bytes3),
    ?assertEqual(Digest1, Digest2),
    ?assertEqual(Digest2, Digest3),

    %% Golden digest: pinned expected value for this exact corpus. Update
    %% ?GOLDEN_DIGEST deliberately (with a comment why) if the corpus or
    %% the readiness logic it exercises ever legitimately changes.
    ?assertEqual(?GOLDEN_DIGEST, Digest1).
