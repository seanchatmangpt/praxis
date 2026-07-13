#!/usr/bin/env escript
%% F16 (V12-016) -- a real, second, independent Rust<->Erlang production entrypoint into the
%% atlas L5 gen_statem lifecycle (`arazzo_runner_dispatch_statem.erl`), closing `F15 -> F16` for
%% the crown-witness EXTERNAL tail without touching `arazzo_runner_workflow.erl:apply_transition/4`
%% at all.
%%
%% Why NOT rewire apply_transition/4 instead: f16_otp_runner.rs's own module doc (and
%% docs/jira/v26.7.12/CROWN_STATUS.md, re-confirmed three independent times) already establishes
%% that `apply_transition/4` calls `arazzo_runner_broker:dispatch/4` *synchronously*, and several
%% `arazzo_runner_workflow_test.erl` assertions depend on that synchronous completion ordering --
%% while `arazzo_runner_dispatch_statem:dispatch/1` is deliberately *asynchronous* (replies `ok`
%% before the spawned worker's broker round trip completes, its own proven core feature). Naively
%% rewiring the call site would flip that ordering guarantee and risk breaking existing, currently-
%% passing tests. This script sidesteps that regression risk entirely: it does not modify
%% `arazzo_runner_workflow.erl` or `apply_transition/4` in any way. It reuses ONLY real, already-
%% supervised, already-tested production entrypoints exactly as `arazzo_runner_dispatch_statem_test`
%% .erl's own `test_lawful_path_advances_live_workflow/0` and `test_broker_refusal_lands_in_refused/0`
%% already prove real: `application:ensure_all_started(arazzo_runner)`,
%% `arazzo_runner_sup:start_workflow/1` (starts a real, supervised workflow process), and
%% `arazzo_runner_dispatch_statem:start_link/4` + `mark_ready/1` + `dispatch/1` (starts a real,
%% supervised dispatch worker driving the real, unmodified `arazzo_runner_broker:dispatch/4`).
%%
%% Request (one line of JSON on stdin):
%%   {"workflow_id": "...", "correlation_id": "...", "source_digest": "...",
%%    "projection_digest": "...", "receipt_head": "...", "replay_id": "...",
%%    "step_id": "...", "bind_name": "...", "bind_value": true|false}
%%
%% Disclosed scope narrowing: `parent_workflow_id`/`arazzo_workflow_id`/`source_powl_region_id`/
%% `dispatch_id` (4 of the identity record's 10 fields) are synthesized from `workflow_id`/
%% `correlation_id` rather than accepted as independent request fields -- this bridge exercises one
%% step's real lawful dispatch through the real 8-state lifecycle, not the full 10-field identity
%% surface a richer caller might need. `bind_value` is always folded through a `{bind, BindName,
%% {literal, BindValue}}` output rule (the same literal-bind shape
%% `arazzo_runner_dispatch_statem_test.erl`'s own `test_lawful_path_advances_live_workflow/0` uses)
%% -- no other output-rule shape is reachable through this bridge.
%%
%% Response (one line of JSON on stdout, exit 0 either way -- a real REFUSED terminal state is data,
%% not a script crash):
%%   completed: {"ok": true, "outcome": "completed", "step_id": "<binary as string>",
%%               "transition_log": ["manufactured","ready",...,"completed"],
%%               "dispatch_token": "<binary as string>", "refusal_atom": null}
%%   refused:   {"ok": true, "outcome": "refused", "step_id": "<binary as string>",
%%               "transition_log": ["manufactured","ready",...,"refused"],
%%               "dispatch_token": null, "refusal_atom": "<erlang atom as string>"}
%%   bridge/protocol failure: {"ok": false, "error": "..."}
%%
%% `step_id` is read back from the real running gen_statem's own internal
%% `#d.step_id` record field via `arazzo_runner_dispatch_statem:get_step_id/1`
%% (a live query of the actual process, not this script's own copy of the
%% request field) -- so a Rust caller can independently confirm which step
%% this specific dispatch really ran, not merely which step it asked for.
%%
%% Each invocation runs in its own fresh escript BEAM VM (same "stateless per call" property
%% `air_core_bridge.escript` discloses), so the `arazzo_runner` OTP application, its DETS state
%% under a freshly-generated `/tmp` directory, and every spawned process are naturally torn down
%% when the VM exits; this script also explicitly stops the application and removes its state
%% directory before emitting its response, matching `arazzo_runner_dispatch_statem_test.erl`'s own
%% `cleanup/1` fixture, not relying on VM exit alone.
%%
%% ERL_LIBS must include a directory containing `arazzo_runner/ebin` and `air_core/ebin` (the
%% workflow process's own real dependency) as immediate subdirectories, e.g.
%% `ERL_LIBS=/Users/sac/praxis/_build/default/lib`. The Rust caller sets this; the script does not
%% hardcode any absolute repo path itself.

main(_Args) ->
    %% Suppress Erlang's own logger output (e.g. the kernel app's real
    %% "=INFO REPORT====" lines emitted when arazzo_runner stops) at the
    %% primary-logger level, before any handler runs. Without this, that
    %% report text lands ahead of this script's one JSON response line on
    %% the SAME stream the Rust caller reads (call_dispatch_statem_bridge
    %% captures `output.stdout` as a whole and parses it as one JSON value),
    %% breaking the documented "one JSON line out" contract. Discovered via
    %% a live integration-test failure this session: MalformedResponse{
    %% reason: "expected value at line 1 column 1", raw: "=INFO REPORT...
    %% \n{...real json...}\n"} -- a real defect in this script, not in the
    %% Rust parser.
    logger:set_primary_config(level, none),
    case io:get_line(standard_io, "") of
        eof ->
            fail("no input on stdin");
        {error, ReadReason} ->
            fail(io_lib:format("stdin read error: ~p", [ReadReason]));
        Line ->
            try
                Req = json:decode(unicode:characters_to_binary(Line)),
                handle_request(Req)
            catch
                Class:Err:Stack ->
                    fail(io_lib:format("~p:~p ~p", [Class, Err, Stack]))
            end
    end.

handle_request(Req) ->
    WorkflowId = b(maps:get(<<"workflow_id">>, Req)),
    CorrelationId = b(maps:get(<<"correlation_id">>, Req)),
    SourceDigest = b(maps:get(<<"source_digest">>, Req)),
    ProjectionDigest = b(maps:get(<<"projection_digest">>, Req)),
    ReceiptHead = b(maps:get(<<"receipt_head">>, Req)),
    ReplayId = b(maps:get(<<"replay_id">>, Req)),
    StepId = b(maps:get(<<"step_id">>, Req)),
    BindName = b(maps:get(<<"bind_name">>, Req)),
    BindValue = maps:get(<<"bind_value">>, Req, true),

    %% os:getpid/0 (the OS process id, genuinely unique across concurrently-running OS
    %% processes on this machine) is combined with erlang:unique_integer/1 (unique only
    %% *within* one BEAM VM instance) because this escript's own "stateless per call"
    %% design means every invocation is a fresh, separate OS process/VM -- unique_integer
    %% alone does not guarantee two concurrently-spawned escript instances pick different
    %% values, since each VM's counter starts fresh. Reproduced live this session: running
    %% this crate's ignored/live tests with cargo's default parallel test runner (multiple
    %% concurrent escript spawns) hit a real `application:ensure_all_started(arazzo_runner)`
    %% failure downstream (`air_core:new` reported `undef`) that did not reproduce when the
    %% same test ran alone -- consistent with two concurrent escript VMs colliding on the
    %% same unique_integer value and racing on the same state_dir.
    Dir = filename:join(
        "/tmp",
        "arazzo_dispatch_statem_bridge_" ++ os:getpid() ++ "_" ++
            integer_to_list(erlang:unique_integer([positive]))
    ),
    ok = filelib:ensure_dir(filename:join(Dir, "x")),
    ok = application:set_env(arazzo_runner, state_dir, Dir),
    {ok, _Started} = application:ensure_all_started(arazzo_runner),

    IdMap = #{
        workflow_id => WorkflowId,
        parent_workflow_id => undefined,
        arazzo_workflow_id => WorkflowId,
        source_powl_region_id => <<"f16-dispatch-bridge">>,
        dispatch_id => CorrelationId,
        correlation_id => CorrelationId,
        source_digest => SourceDigest,
        projection_digest => ProjectionDigest,
        receipt_head => ReceiptHead,
        replay_id => ReplayId
    },

    Result =
        case arazzo_runner_identity:from_map(IdMap) of
            {error, IdErr} ->
                {error, io_lib:format("identity construction refused: ~p", [IdErr])};
            {ok, Id} ->
                StepDef = #{outputs => [{bind, BindName, {literal, BindValue}}], next => []},
                StartSpec = maps:merge(IdMap, #{
                    workflow_def => #{steps => #{StepId => StepDef}},
                    active_steps => [StepId],
                    env => #{},
                    history => []
                }),
                case arazzo_runner_sup:start_workflow(StartSpec) of
                    {error, WfErr} ->
                        {error, io_lib:format("start_workflow refused: ~p", [WfErr])};
                    {ok, _WfPid} ->
                        drive_dispatch(WorkflowId, Id, StepId, StepDef)
                end
        end,

    catch application:stop(arazzo_runner),
    catch arazzo_runner_identity:close_table(),
    catch os:cmd("rm -rf " ++ Dir),

    case Result of
        {ok, Resp} -> emit(Resp);
        {error, Reason} -> fail(Reason)
    end.

drive_dispatch(WorkflowId, Id, StepId, StepDef) ->
    case arazzo_runner_dispatch_statem:start_link(WorkflowId, Id, StepId, StepDef) of
        {error, StatemErr} ->
            {error, io_lib:format("dispatch_statem start_link refused: ~p", [StatemErr])};
        {ok, Pid} ->
            ok = arazzo_runner_dispatch_statem:mark_ready(Pid),
            ok = arazzo_runner_dispatch_statem:dispatch(Pid),
            case wait_for_terminal(Pid, 400) of
                timeout ->
                    {error, "timed out waiting for a terminal dispatch-statem state"};
                Terminal ->
                    TransitionLog = arazzo_runner_dispatch_statem:get_transition_log(Pid),
                    Outcome = arazzo_runner_dispatch_statem:get_outcome(Pid),
                    %% Read back from the real running process's own state
                    %% (not this script's local `StepId` variable) -- see the
                    %% module doc's `step_id` field note.
                    ObservedStepId = arazzo_runner_dispatch_statem:get_step_id(Pid),
                    to_response(Terminal, ObservedStepId, TransitionLog, Outcome)
            end
    end.

wait_for_terminal(_Pid, 0) ->
    timeout;
wait_for_terminal(Pid, N) ->
    case arazzo_runner_dispatch_statem:get_lifecycle_state(Pid) of
        completed -> completed;
        refused -> refused;
        _ ->
            timer:sleep(5),
            wait_for_terminal(Pid, N - 1)
    end.

to_response(completed, StepId, TransitionLog, {ok, DispatchToken}) when is_binary(DispatchToken) ->
    {ok, #{
        ok => true,
        outcome => <<"completed">>,
        step_id => StepId,
        transition_log => [atom_to_binary(S, utf8) || S <- TransitionLog],
        dispatch_token => DispatchToken,
        refusal_atom => null
    }};
to_response(refused, StepId, TransitionLog, {refused, Atom, _Ctx}) when is_atom(Atom) ->
    {ok, #{
        ok => true,
        outcome => <<"refused">>,
        step_id => StepId,
        transition_log => [atom_to_binary(S, utf8) || S <- TransitionLog],
        dispatch_token => null,
        refusal_atom => atom_to_binary(Atom, utf8)
    }};
to_response(Terminal, _StepId, _TransitionLog, Outcome) ->
    {error, io_lib:format("unexpected terminal/outcome pair: ~p ~p", [Terminal, Outcome])}.

b(V) when is_binary(V) -> V;
b(V) when is_list(V) -> unicode:characters_to_binary(V).

emit(Term) ->
    Json = json:encode(Term),
    io:format("~s~n", [Json]),
    halt(0).

fail(ReasonIoData) ->
    Json = json:encode(#{ok => false, error => iolist_to_binary(ReasonIoData)}),
    io:format("~s~n", [Json]),
    halt(0).
