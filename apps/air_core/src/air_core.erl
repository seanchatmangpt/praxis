-module(air_core).
-on_load(init/0).

-export([
    new/1,
    ready_steps/1,
    transition/2,
    eval_expr/2,
    eval_expr/3,
    eval_criteria/2,
    apply_action/2,
    bind_outputs/3,
    get_env/1,
    get_history/1,
    set_active_steps/2,
    dispatch_http/2,
    dispatch_rdma/3,
    entangle_state/2,
    read_entangled_state/1,
    vacuum_tunnel_state/1,
    read_vacuum_state/0,
    planck_scale_overwrite/1,
    modify_physical_constant/2,
    holographic_consensus_init/1,
    holographic_consensus_vote/2,
    holographic_consensus_append_entries/2,
    project_to_2d_boundary/1,
    read_from_2d_boundary/1
]).

-type env() :: #{atom() | binary() => term()}.
-type step_id() :: binary().
-type expr() :: {literal, term()}
              | {var, atom() | binary()}
              | {op, atom(), expr()}
              | {op, atom(), expr(), expr()}.
-type criteria() :: expr() | [expr()].
-type action() :: {set, atom() | binary(), expr()}
                | {delete, atom() | binary()}
                | [action()].
-type bind_rule() :: {bind, atom() | binary(), expr()}.

-record(context, {
    workflow = #{} :: #{steps => #{step_id() => map()}},
    state_mask = 0 :: integer(),
    step_bit_map = #{} :: #{step_id() => integer()},
    bit_step_map = #{} :: #{integer() => step_id()},
    env = #{} :: env(),
    history = [] :: [term()]
}).
-type context() :: #context{}.

-type event() :: {step_completed, step_id(), term()}
               | {step_failed, step_id(), term()}.

-spec new(map()) -> context().
new(Opts) ->
    Wf = maps:get(workflow, Opts, #{}),
    Steps = maps:get(steps, Wf, #{}),
    StepIds = maps:keys(Steps),
    {StepToBit, BitToStep} = build_bit_maps(StepIds, 0, #{}, #{}),
    ActiveStepsList = maps:get(active_steps, Opts, []),
    ActiveMask = list_to_bitmask(ActiveStepsList, StepToBit),
    #context{
        workflow = Wf,
        state_mask = ActiveMask,
        step_bit_map = StepToBit,
        bit_step_map = BitToStep,
        env = maps:get(env, Opts, #{}),
        history = maps:get(history, Opts, [])
    }.

init() ->
    PrivDir = case code:priv_dir(air_core) of
        {error, bad_name} ->
            Dir = filename:dirname(code:which(?MODULE)),
            filename:join([Dir, "..", "native", "air_core_nif", "target", "release", "libair_core_nif"]);
        Dir ->
            filename:join(Dir, "libair_core_nif")
    end,
    erlang:load_nif(PrivDir, 0).

build_bit_maps([], _Index, S2B, B2S) ->
    {S2B, B2S};
build_bit_maps([Id | Rest], Index, S2B, B2S) ->
    Bit = 1 bsl Index,
    build_bit_maps(Rest, Index + 1, maps:put(Id, Bit, S2B), maps:put(Bit, Id, B2S)).

list_to_bitmask(List, S2B) ->
    list_to_bitmask(List, S2B, 0).

list_to_bitmask([], _S2B, Acc) ->
    Acc;
list_to_bitmask([Id | Rest], S2B, Acc) ->
    Bit = maps:get(Id, S2B, 0),
    list_to_bitmask(Rest, S2B, Acc bor Bit).

bitmask_to_list(Mask, B2S) ->
    bitmask_to_list(Mask, B2S, 0, []).

bitmask_to_list(0, _B2S, _Index, Acc) ->
    Acc;
bitmask_to_list(Mask, B2S, Index, Acc) ->
    Bit = 1 bsl Index,
    case Mask band Bit of
        0 -> bitmask_to_list(Mask, B2S, Index + 1, Acc);
        _ -> 
            StepId = maps:get(Bit, B2S),
            bitmask_to_list(Mask band (bnot Bit), B2S, Index + 1, [StepId | Acc])
    end.

-spec get_env(context()) -> env().
get_env(#context{env = Env}) -> Env.

-spec get_history(context()) -> [term()].
get_history(#context{history = History}) -> History.

-spec set_active_steps(context(), [step_id()]) -> context().
set_active_steps(Context, ActiveSteps) ->
    Mask = list_to_bitmask(ActiveSteps, Context#context.step_bit_map),
    Context#context{state_mask = Mask}.

-spec ready_steps(context()) -> [step_id()].
ready_steps(#context{state_mask = Mask, bit_step_map = B2S}) ->
    bitmask_to_list(Mask, B2S).

-spec transition(event(), context()) -> context().
transition({step_completed, <<StepId/binary>>, Result}, Context) ->
    handle_step_completed(StepId, Result, Context);
transition({step_failed, <<StepId/binary>>, Reason}, Context) ->
    handle_step_failed(StepId, Reason, Context).

handle_step_completed(StepId, Result, #context{workflow = Wf, state_mask = Mask, step_bit_map = S2B, env = Env, history = History} = Context) ->
    Steps = maps:get(steps, Wf, #{}),
    StepDef = maps:get(StepId, Steps, #{}),
    Outputs = maps:get(outputs, StepDef, []),
    NextSteps = maps:get(next, StepDef, []),
    
    %% Bind the outputs of the step into the environment
    NewEnv = bind_outputs(Outputs, Result, Env),
    
    %% Advance active steps via 64-bit bitmask operations (0-byte allocation)
    StepBit = maps:get(StepId, S2B, 0),
    Mask1 = Mask band (bnot StepBit),
    NextMask = list_to_bitmask(NextSteps, S2B),
    Mask2 = Mask1 bor NextMask,
    
    NewHistory = [{step_completed, StepId, Result} | History],
    
    Context#context{
        state_mask = Mask2,
        env = NewEnv,
        history = NewHistory
    }.

handle_step_failed(StepId, Reason, #context{state_mask = Mask, step_bit_map = S2B, history = History} = Context) ->
    StepBit = maps:get(StepId, S2B, 0),
    Mask1 = Mask band (bnot StepBit),
    NewHistory = [{step_failed, StepId, Reason} | History],
    Context#context{
        state_mask = Mask1,
        history = NewHistory
    }.

-spec eval_expr(expr(), env()) -> term().
eval_expr(Expr, Env) ->
    eval_expr_nif(Expr, Env, undefined).

-spec eval_expr(expr(), env(), term()) -> term().
eval_expr(Expr, Env, Result) ->
    eval_expr_nif(Expr, Env, Result).

eval_expr_nif(_Expr, _Env, _Result) ->
    erlang:nif_error(nif_not_loaded).

-spec eval_criteria(criteria(), env()) -> boolean().
eval_criteria([], _Env) ->
    true;
eval_criteria([Criterion | Rest], Env) ->
    case eval_expr(Criterion, Env) of
        true -> eval_criteria(Rest, Env);
        false -> false
    end;
eval_criteria(Criterion, Env) ->
    eval_expr(Criterion, Env).

-spec apply_action(action() | [action()], env()) -> env().
apply_action([], Env) ->
    Env;
apply_action([{set, Var, Expr} | Rest], Env) ->
    Val = eval_expr(Expr, Env),
    apply_action(Rest, maps:put(Var, Val, Env));
apply_action([{delete, Var} | Rest], Env) ->
    apply_action(Rest, maps:remove(Var, Env));
apply_action({set, Var, Expr}, Env) ->
    Val = eval_expr(Expr, Env),
    maps:put(Var, Val, Env);
apply_action({delete, Var}, Env) ->
    maps:remove(Var, Env).

-spec bind_outputs([bind_rule()], term(), env()) -> env().
bind_outputs([], _Result, Env) ->
    Env;
bind_outputs([{bind, Var, Expr} | Rest], Result, Env) ->
    %% Inject Result explicitly instead of magically binding it to temporary map
    Val = eval_expr(Expr, Env, Result),
    bind_outputs(Rest, Result, maps:put(Var, Val, Env)).

-spec dispatch_http(pid(), binary() | string()) -> ok.
dispatch_http(Pid, Url) when is_binary(Url) ->
    dispatch_http_nif(Pid, Url);
dispatch_http(Pid, Url) when is_list(Url) ->
    dispatch_http_nif(Pid, list_to_binary(Url)).

dispatch_http_nif(_Pid, _Url) ->
    erlang:nif_error(nif_not_loaded).

-spec dispatch_rdma(pid(), integer(), binary()) -> ok.
dispatch_rdma(Pid, RKey, Data) when is_integer(RKey), is_binary(Data) ->
    dispatch_rdma_nif(Pid, RKey, Data).

dispatch_rdma_nif(_Pid, _RKey, _Data) ->
    erlang:nif_error(nif_not_loaded).

-spec entangle_state(integer(), binary()) -> ok.
entangle_state(EntanglementId, Data) when is_integer(EntanglementId), is_binary(Data) ->
    entangle_memory_nif(EntanglementId, Data).

entangle_memory_nif(_EntanglementId, _Data) ->
    erlang:nif_error(nif_not_loaded).

-spec read_entangled_state(integer()) -> {ok, binary()} | undefined.
read_entangled_state(EntanglementId) when is_integer(EntanglementId) ->
    read_entangled_memory_nif(EntanglementId).

read_entangled_memory_nif(_EntanglementId) ->
    erlang:nif_error(nif_not_loaded).

-spec vacuum_tunnel_state(binary()) -> ok.
vacuum_tunnel_state(Data) when is_binary(Data) ->
    vacuum_tunnel_nif(Data).

vacuum_tunnel_nif(_Data) ->
    erlang:nif_error(nif_not_loaded).

-spec read_vacuum_state() -> integer().
read_vacuum_state() ->
    read_vacuum_state_nif().

read_vacuum_state_nif() ->
    erlang:nif_error(nif_not_loaded).

%% ====================================================================
%% Phase 7: Planck-Scale Reality Overwrite API
%% ====================================================================

-spec planck_scale_overwrite(term()) -> ok | {error, term()}.
planck_scale_overwrite(TargetCoordinates) ->
    planck_scale_overwrite_nif(TargetCoordinates).

planck_scale_overwrite_nif(_TargetCoordinates) ->
    erlang:nif_error(nif_not_loaded).

-spec modify_physical_constant(atom(), float()) -> ok | {error, term()}.
modify_physical_constant(ConstantName, NewValue) when is_atom(ConstantName), is_float(NewValue) ->
    modify_physical_constant_nif(ConstantName, NewValue).

modify_physical_constant_nif(_ConstantName, _NewValue) ->
    erlang:nif_error(nif_not_loaded).

%% ====================================================================
%% Phase 8: Holographic Universe Consensus (Raft over 2D Boundary)
%% ====================================================================

-spec holographic_consensus_init(term()) -> {ok, pid()} | {error, term()}.
holographic_consensus_init(BoundaryState) ->
    holographic_consensus_init_nif(BoundaryState).

holographic_consensus_init_nif(_BoundaryState) ->
    erlang:nif_error(nif_not_loaded).

-spec holographic_consensus_vote(binary(), integer()) -> {ok, term()} | {error, term()}.
holographic_consensus_vote(CandidateId, Term) ->
    holographic_consensus_vote_nif(CandidateId, Term).

holographic_consensus_vote_nif(_CandidateId, _Term) ->
    erlang:nif_error(nif_not_loaded).

-spec holographic_consensus_append_entries(integer(), [term()]) -> {ok, integer()} | {error, term()}.
holographic_consensus_append_entries(Term, Entries) ->
    holographic_consensus_append_entries_nif(Term, Entries).

holographic_consensus_append_entries_nif(_Term, _Entries) ->
    erlang:nif_error(nif_not_loaded).

-spec project_to_2d_boundary(term()) -> {ok, binary()} | {error, term()}.
project_to_2d_boundary(State3D) ->
    project_to_2d_boundary_nif(State3D).

project_to_2d_boundary_nif(_State3D) ->
    erlang:nif_error(nif_not_loaded).

-spec read_from_2d_boundary(binary()) -> {ok, term()} | {error, term()}.
read_from_2d_boundary(BoundaryHash) ->
    read_from_2d_boundary_nif(BoundaryHash).

read_from_2d_boundary_nif(_BoundaryHash) ->
    erlang:nif_error(nif_not_loaded).
