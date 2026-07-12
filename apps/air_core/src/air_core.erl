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
    set_active_steps/2
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
    %% Static per-workflow predecessor bitmask, PROJ-756: pred_mask_map[T]
    %% has bit P set iff step P lists T in its `next`. Built once in new/1
    %% by inverting every step's forward `next` edges (the workflow map
    %% only carries forward/successor edges; this is the only way to learn
    %% a step's full predecessor set). Never mutated after construction.
    pred_mask_map = #{} :: #{step_id() => integer()},
    %% Cumulative "has ever step_completed" bitmask, PROJ-756. Distinct
    %% from state_mask: state_mask is the current *active/ready* frontier
    %% (a step's bit is cleared from it the instant that step completes),
    %% so it cannot answer "has step X completed at some point in the
    %% past" once X has moved on -- exactly the question AND/join
    %% readiness needs answered for every predecessor of a candidate
    %% successor. completed_mask only ever grows (OR-only), one bit per
    %% completed step, so it is the "still waiting on" ground truth: a
    %% successor is ready iff its pred_mask_map bits are all present here.
    %% step_failed intentionally does NOT set a bit here (see
    %% handle_step_failed/3) -- a failed predecessor permanently blocks any
    %% AND/join depending on it, which is the correct default absent a
    %% compensation protocol (PROJ-759, not yet built).
    completed_mask = 0 :: integer(),
    env = #{} :: env(),
    history = [] :: [term()]
}).
-type context() :: #context{}.

-type event() :: {step_completed, step_id(), term()}
               | {step_failed, step_id(), term()}.

%% A command is a finite, typed request for a future broker (PROJ-758, not
%% yet built) to route through a lawful surface -- the transition core
%% itself performs no I/O or actuation (PRD v26.7.11 section 7.7:
%% delta_AIR: (S,E) -> (S',C), "C is a finite set of requested
%% commands/consequences to route through lawful surfaces"). The only
%% concrete command shape today is `{dispatch_step, StepId, StepDef}`,
%% requesting dispatch of a step that just became newly ready as a direct
%% effect of this transition. Add new tagged variants here (not new shapes
%% of this one) if a second broker action is ever needed.
-type command() :: {dispatch_step, step_id(), map()}.

%% PROJ-756: AND/join dependency-readiness. A step with multiple
%% predecessors (multiple other steps naming it in their `next`) only
%% becomes ready once ALL of them have completed, not the instant any one
%% of them does -- that OR-shaped bug is what PROJ-756 closes. See
%% pred_mask_map / completed_mask on #context{} above for the two static
%% + cumulative pieces of state this requires, and
%% newly_ready_successors/5 below for the readiness predicate itself.
%% transition/2's return *shape* (the {S', C} pair) was fixed by PROJ-755;
%% this closes the *readiness semantics* feeding both S' and C.

-spec new(map()) -> context().
new(Opts) ->
    Wf = maps:get(workflow, Opts, #{}),
    Steps = maps:get(steps, Wf, #{}),
    StepIds = maps:keys(Steps),
    {StepToBit, BitToStep} = build_bit_maps(StepIds, 0, #{}, #{}),
    PredMaskMap = build_pred_mask_map(Steps, StepToBit),
    ActiveStepsList = maps:get(active_steps, Opts, []),
    ActiveMask = list_to_bitmask(ActiveStepsList, StepToBit),
    #context{
        workflow = Wf,
        state_mask = ActiveMask,
        step_bit_map = StepToBit,
        bit_step_map = BitToStep,
        pred_mask_map = PredMaskMap,
        completed_mask = 0,
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

%% Inverts every step's forward `next` edges into a per-step predecessor
%% bitmask: the result map's entry for step T has bit P set iff step P
%% lists T in its `next`. `next` is the only edge direction the workflow
%% map carries, so this inversion (done once, here, at context
%% construction) is the only way handle_step_completed/3 can later ask
%% "has EVERY predecessor of this candidate successor completed" in O(1)
%% per candidate instead of re-walking the whole step map on every event.
%%
%% # Complexity
%% O(|steps| + sum(|next| over all steps)) -- one outer fold over the step
%% map plus one inner fold per step's `next` list; same cost class as
%% build_bit_maps/4 above. Space: O(|steps| + sum(|next|)) for the map and
%% its bitmask values.
-spec build_pred_mask_map(#{step_id() => map()}, #{step_id() => integer()}) ->
    #{step_id() => integer()}.
build_pred_mask_map(Steps, S2B) ->
    maps:fold(
        fun(StepId, StepDef, PredAcc) ->
            StepBit = maps:get(StepId, S2B, 0),
            NextSteps = maps:get(next, StepDef, []),
            lists:foldl(
                fun(NextId, Acc2) ->
                    Prev = maps:get(NextId, Acc2, 0),
                    maps:put(NextId, Prev bor StepBit, Acc2)
                end,
                PredAcc,
                NextSteps
            )
        end,
        #{},
        Steps
    ).

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

-spec transition(event(), context()) -> {context(), [command()]}.
transition({step_completed, <<StepId/binary>>, Result}, Context) ->
    handle_step_completed(StepId, Result, Context);
transition({step_failed, <<StepId/binary>>, Reason}, Context) ->
    handle_step_failed(StepId, Reason, Context).

-spec handle_step_completed(step_id(), term(), context()) -> {context(), [command()]}.
handle_step_completed(StepId, Result, #context{
        workflow = Wf,
        state_mask = Mask,
        step_bit_map = S2B,
        pred_mask_map = PredMaskMap,
        completed_mask = CompletedMask,
        env = Env,
        history = History
    } = Context) ->
    Steps = maps:get(steps, Wf, #{}),
    StepDef = maps:get(StepId, Steps, #{}),
    Outputs = maps:get(outputs, StepDef, []),
    NextSteps = maps:get(next, StepDef, []),

    %% Bind the outputs of the step into the environment
    NewEnv = bind_outputs(Outputs, Result, Env),

    %% Advance active steps via 64-bit bitmask operations (0-byte allocation)
    StepBit = maps:get(StepId, S2B, 0),
    Mask1 = Mask band (bnot StepBit),
    %% completed_mask only ever grows: this step has now completed, for
    %% good, regardless of whether it later leaves the active frontier.
    CompletedMask1 = CompletedMask bor StepBit,

    %% PROJ-756 AND/join: a candidate successor is newly ready iff (a) it
    %% is not already active, and (b) EVERY bit in its static predecessor
    %% mask is now present in CompletedMask1 -- i.e. no predecessor bit
    %% remains outside the completed set (PredMask band bnot CompletedMask1
    %% =:= 0). This replaces the old "every direct successor is
    %% unconditionally ready" bug, where a step with two predecessors
    %% became ready the instant the first one finished (OR semantics)
    %% instead of waiting for all of them (AND semantics).
    {ReadyIds, ReadyMask} = newly_ready_successors(NextSteps, S2B, PredMaskMap, CompletedMask1, Mask1),
    Mask2 = Mask1 bor ReadyMask,

    NewHistory = [{step_completed, StepId, Result} | History],

    NewContext = Context#context{
        state_mask = Mask2,
        completed_mask = CompletedMask1,
        env = NewEnv,
        history = NewHistory
    },

    %% C = dispatch_step commands for exactly the steps newly_ready_successors
    %% determined are ready this transition. O(|next|) -- same cost class as
    %% the pre-PROJ-756 NextMask walk, just with an added O(1) predecessor
    %% check per candidate.
    Commands = [{dispatch_step, Id, maps:get(Id, Steps, #{})} || Id <- ReadyIds],

    {NewContext, Commands}.

%% Filters a step's direct successors (`next`) down to the ones that are
%% newly ready this transition: not already active, and with every
%% predecessor bit (per PredMaskMap) now present in CompletedMask1. Returns
%% both the ordered id list (for building Commands, order-preserving over
%% NextSteps) and the corresponding bitmask (for folding into state_mask).
%%
%% # Complexity
%% O(|NextSteps|) -- one O(1) map lookup + bitmask comparison per candidate,
%% no traversal of the full step/bit space.
-spec newly_ready_successors([step_id()], #{step_id() => integer()}, #{step_id() => integer()}, integer(), integer()) ->
    {[step_id()], integer()}.
newly_ready_successors(NextSteps, S2B, PredMaskMap, CompletedMask1, Mask1) ->
    newly_ready_successors(NextSteps, S2B, PredMaskMap, CompletedMask1, Mask1, [], 0).

newly_ready_successors([], _S2B, _PredMaskMap, _CompletedMask1, _Mask1, Ids, ReadyMask) ->
    {lists:reverse(Ids), ReadyMask};
newly_ready_successors([Id | Rest], S2B, PredMaskMap, CompletedMask1, Mask1, Ids, ReadyMask) ->
    Bit = maps:get(Id, S2B, 0),
    AlreadyActive = (Mask1 band Bit) =/= 0,
    AlreadyMarkedReady = (ReadyMask band Bit) =/= 0,
    PredMask = maps:get(Id, PredMaskMap, 0),
    AllPredsDone = (PredMask band (bnot CompletedMask1)) =:= 0,
    case (not AlreadyActive) andalso (not AlreadyMarkedReady) andalso AllPredsDone of
        true ->
            newly_ready_successors(Rest, S2B, PredMaskMap, CompletedMask1, Mask1, [Id | Ids], ReadyMask bor Bit);
        false ->
            newly_ready_successors(Rest, S2B, PredMaskMap, CompletedMask1, Mask1, Ids, ReadyMask)
    end.

-spec handle_step_failed(step_id(), term(), context()) -> {context(), [command()]}.
handle_step_failed(StepId, Reason, #context{state_mask = Mask, step_bit_map = S2B, history = History} = Context) ->
    StepBit = maps:get(StepId, S2B, 0),
    Mask1 = Mask band (bnot StepBit),
    NewHistory = [{step_failed, StepId, Reason} | History],
    NewContext = Context#context{
        state_mask = Mask1,
        history = NewHistory
    },
    %% A failed step advances no next-steps and -- deliberately, PROJ-756 --
    %% never sets its bit in completed_mask either (unlike
    %% handle_step_completed/3), so any AND/join successor depending on it
    %% stays permanently unsatisfied: it must never become "ready" from a
    %% predecessor that failed rather than completed. C is therefore always
    %% the empty list here, verified by the absence of any successor
    %% computation above, not assumed.
    {NewContext, []}.

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
