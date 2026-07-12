#!/usr/bin/env escript
%% F15 AIR Single Semantic Core (V12-015) -- the real, minimal, SAFE
%% Rust<->Erlang bridge this family's own source (f15_air_transition_core.rs)
%% disclosed as HAND_WRITE_REQUIRED and not yet built.
%%
%% Problem this closes: apps/air_core/native/air_core_nif is `cdylib`-only
%% (Erlang calling Rust, one direction, and only from inside a loaded BEAM
%% NIF call). There was no direction for a *standalone* Rust process (e.g.
%% crates/multifractal-workflow, which cannot link air_core.erl the way a
%% same-BEAM caller can) to invoke the real air_core:transition/2 and get
%% back a real answer. This script is that missing direction: an OS-process
%% bridge, not a second reimplementation of AIR semantics. It calls the
%% REAL, rebar3-compiled air_core:new/1, air_core:transition/2, and
%% air_core:ready_steps/1 (apps/air_core/src/air_core.erl) -- including the
%% real PROJ-756 AND/join readiness bitmask logic -- and reports back
%% exactly what those functions computed.
%%
%% Why a port/escript, not the Erlang distribution protocol: a distribution
%% client (a fake Erlang node joining the cluster, cookie auth, EPMD) is
%% materially more machinery for the same observable result and a larger
%% attack surface for one family's minimal bridge; escript is a single
%% already-available OTP tool (confirmed on PATH:
%% /Users/sac/.erlmcp/otp-28.3.1/bin/escript) with a well-defined,
%% narrow contract: one JSON line in, one JSON line out, exit 0. No code is
%% generated, compiled, or hot-loaded by this script or its caller -- it
%% only invokes air_core's own, already-compiled, already-tested functions.
%%
%% Request (one line of JSON on stdin):
%%   {"workflow": {"steps": {"<id>": {"next": ["<id>", ...]}, ...}},
%%    "active_steps": ["<id>", ...],
%%    "events": [{"type": "step_completed", "step_id": "<id>", "result": <any>}
%%             | {"type": "step_failed", "step_id": "<id>", "reason": <any>},
%%               ...]}
%%
%% `events` is applied in array order against ONE `air_core:new/1` context
%% (air_core.erl's own real fold, `lists:foldl` over `transition/2` calls --
%% not reimplemented here), matching how a real caller drives many events
%% through a single long-lived context; this script itself holds no state
%% between separate invocations. `commands` in the response is every
%% command produced by every event, concatenated in event order.
%%
%% Response (one line of JSON on stdout, exit 0 either way -- an
%% application-level refusal is data, not a script crash):
%%   success: {"ok": true, "ready_steps": ["<id>", ...],
%%             "commands": [{"step_id": "<id>"}, ...]}
%%   failure: {"ok": false, "error": "<message>"}
%%
%% Disclosed scope limit: StepDef `outputs` (bind_outputs/3's expr-AST bind
%% rules, which reach eval_expr_nif) has no JSON encoding here and is
%% always treated as absent -- this bridge exercises air_core:new/1,
%% transition/2's step_completed/step_failed dispatch, and the AND/join
%% newly_ready_successors/5 readiness predicate for real, but not the
%% expression-VM path. Inventing a JSON encoding for the expr() AST is real
%% future work (tracked under V12-015), not attempted here -- it would be
%% new surface, not a minimal bridge.
%%
%% ERL_LIBS must include a directory containing air_core/ebin (and
%% air_core/priv, for the NIF this module's eval_expr/2,3 -- not called by
%% this script -- would need) as an immediate subdirectory, e.g.
%% `ERL_LIBS=/Users/sac/praxis/_build/default/lib`. The Rust caller sets
%% this; the script does not hardcode any absolute repo path itself.

main(_Args) ->
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
    Workflow = to_workflow(maps:get(<<"workflow">>, Req, #{})),
    ActiveSteps = [to_step_id(S) || S <- maps:get(<<"active_steps">>, Req, [])],
    EventsJson = maps:get(<<"events">>, Req),
    true = is_list(EventsJson) andalso EventsJson =/= [],
    Events = [to_event(E) || E <- EventsJson],

    %% The real air_core.erl calls this bridge exists to reach: one
    %% context, folded through every event in order via the real
    %% transition/2 -- not a Rust reimplementation of the AND/join
    %% bitmask logic (air_core.erl:191-277, PROJ-756).
    Ctx0 = air_core:new(#{workflow => Workflow, active_steps => ActiveSteps}),
    {CtxN, AllCommands} = lists:foldl(
        fun(Event, {Ctx, CmdsAcc}) ->
            {Ctx1, Cmds} = air_core:transition(Event, Ctx),
            {Ctx1, CmdsAcc ++ Cmds}
        end,
        {Ctx0, []},
        Events
    ),
    ReadySteps = air_core:ready_steps(CtxN),

    Resp = #{
        ok => true,
        ready_steps => ReadySteps,
        commands => [command_to_json(C) || C <- AllCommands]
    },
    emit(Resp).

to_workflow(WfJson) ->
    StepsJson = maps:get(<<"steps">>, WfJson, #{}),
    Steps = maps:fold(
        fun(Id, StepJson, Acc) ->
            Next = [to_step_id(N) || N <- maps:get(<<"next">>, StepJson, [])],
            maps:put(to_step_id(Id), #{next => Next}, Acc)
        end,
        #{},
        StepsJson
    ),
    #{steps => Steps}.

to_step_id(Id) when is_binary(Id) -> Id;
to_step_id(Id) when is_list(Id) -> unicode:characters_to_binary(Id).

to_event(EventJson) ->
    Type = maps:get(<<"type">>, EventJson),
    StepId = to_step_id(maps:get(<<"step_id">>, EventJson)),
    case Type of
        <<"step_completed">> ->
            Result = json_to_term(maps:get(<<"result">>, EventJson, null)),
            {step_completed, StepId, Result};
        <<"step_failed">> ->
            ReasonTerm = json_to_term(maps:get(<<"reason">>, EventJson, null)),
            {step_failed, StepId, ReasonTerm};
        Other ->
            throw({unknown_event_type, Other})
    end.

%% Structural JSON -> Erlang term conversion for the opaque Result/Reason
%% payload only (never for control fields like step ids or the event type,
%% which are matched as literal binaries above). `null` -> `undefined`
%% matches air_core.erl's own `undefined` sentinel (see eval_expr_nif's use
%% of it in apps/air_core/native/air_core_nif/src/lib.rs).
json_to_term(null) ->
    undefined;
json_to_term(M) when is_map(M) ->
    maps:fold(fun(K, V, Acc) -> maps:put(K, json_to_term(V), Acc) end, #{}, M);
json_to_term(L) when is_list(L) ->
    [json_to_term(X) || X <- L];
json_to_term(Other) ->
    %% binary, number, bool already have no closer Erlang shape.
    Other.

command_to_json({dispatch_step, StepId, _StepDef}) ->
    #{step_id => StepId}.

emit(Term) ->
    Json = json:encode(Term),
    io:format("~s~n", [Json]),
    halt(0).

fail(ReasonIoData) ->
    Json = json:encode(#{ok => false, error => iolist_to_binary(ReasonIoData)}),
    io:format("~s~n", [Json]),
    halt(0).
