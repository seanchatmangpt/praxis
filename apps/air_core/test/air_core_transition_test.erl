-module(air_core_transition_test).
-include_lib("eunit/include/eunit.hrl").

%% PROJ-755: transition/2 must return {S', C} (delta_AIR: (S,E) -> (S',C),
%% PRD v26.7.11 section 7.7), where C is a finite set of dispatch_step
%% commands for steps newly unlocked by this transition. These tests are
%% independent of fortune5_test.erl (which also exercises the new shape
%% end-to-end) and target the {S', C} contract directly, including the
%% "already active, not newly ready" edge case that a naive
%% NextMask-is-the-answer implementation would get wrong.

two_step_workflow() ->
    #{
        workflow => #{
            steps => #{
                <<"a">> => #{
                    outputs => [],
                    next => [<<"b">>, <<"c">>]
                },
                <<"b">> => #{outputs => [], next => []},
                <<"c">> => #{outputs => [], next => []}
            }
        },
        active_steps => [<<"a">>],
        env => #{},
        history => []
    }.

%% (1) transition/2 returns a 2-tuple {NewContext, Commands}.
transition_returns_two_tuple_test() ->
    Context0 = air_core:new(two_step_workflow()),
    Result = air_core:transition({step_completed, <<"a">>, ok}, Context0),
    ?assertEqual(true, is_tuple(Result)),
    ?assertEqual(2, tuple_size(Result)),
    {_NewContext, Commands} = Result,
    ?assertEqual(true, is_list(Commands)).

%% (2) step_completed unlocking N next-steps produces exactly N commands,
%% each correctly identifying the unlocked step and carrying its StepDef.
step_completed_unlocks_matching_commands_test() ->
    Context0 = air_core:new(two_step_workflow()),
    {_NewContext, Commands} = air_core:transition({step_completed, <<"a">>, ok}, Context0),
    ?assertEqual(2, length(Commands)),
    StepIds = lists:sort([StepId || {dispatch_step, StepId, _StepDef} <- Commands]),
    ?assertEqual([<<"b">>, <<"c">>], StepIds),
    %% Every command is tagged `dispatch_step` and carries the real StepDef
    %% map for that step (not a placeholder/empty map), per the {dispatch_step,
    %% StepId, StepDef} shape documented in air_core.erl.
    lists:foreach(
        fun({dispatch_step, StepId, StepDef}) ->
            ?assertEqual(true, lists:member(StepId, [<<"b">>, <<"c">>])),
            ?assertEqual(#{outputs => [], next => []}, StepDef)
        end,
        Commands
    ).

%% (3) step_failed emits no commands, even when the failed step *has* a
%% non-empty `next` list -- verified against the code (handle_step_failed
%% never computes a NextMask at all), not assumed from the happy path.
step_failed_emits_no_commands_test() ->
    Context0 = air_core:new(two_step_workflow()),
    {NewContext, Commands} = air_core:transition({step_failed, <<"a">>, some_reason}, Context0),
    ?assertEqual([], Commands),
    %% And, consistent with "no command without a state change to back it",
    %% the failed step's downstream steps are not marked ready either.
    ?assertEqual([], air_core:ready_steps(NewContext)).

%% (4) PROJ-756 AND/join: "shared" has two real predecessors (p1 and p2).
%% It must NOT become ready when only one of them completes -- that would
%% be the old OR-semantics bug -- and must become ready, with exactly one
%% dispatch_step command, only once BOTH have completed. (This test used to
%% be named diamond_shared_next_step_not_recommanded_test/0 and asserted
%% the opposite: that "shared" went ready after p1 alone. That assertion
%% encoded the PROJ-756 bug itself, not a spec; it is corrected here, not
%% preserved, now that the real AND/join logic exists.)
two_predecessor_and_join_test() ->
    Diamond = #{
        workflow => #{
            steps => #{
                <<"p1">> => #{outputs => [], next => [<<"shared">>]},
                <<"p2">> => #{outputs => [], next => [<<"shared">>]},
                <<"shared">> => #{outputs => [], next => []}
            }
        },
        active_steps => [<<"p1">>, <<"p2">>],
        env => #{},
        history => []
    },
    Context0 = air_core:new(Diamond),

    %% p1 completes alone: "shared" is still waiting on p2, so it must NOT
    %% be marked ready and no command may be issued for it.
    {Context1, Commands1} = air_core:transition({step_completed, <<"p1">>, ok}, Context0),
    ?assertEqual([], Commands1),
    ?assertEqual(false, lists:member(<<"shared">>, air_core:ready_steps(Context1))),

    %% p2 completes second: every predecessor of "shared" has now
    %% completed, so it becomes ready with exactly one dispatch_step
    %% command (not re-dispatched again on any later, unrelated event).
    {Context2, Commands2} = air_core:transition({step_completed, <<"p2">>, ok}, Context1),
    ?assertEqual([{dispatch_step, <<"shared">>, #{outputs => [], next => []}}], Commands2),
    ?assertEqual(true, lists:member(<<"shared">>, air_core:ready_steps(Context2))).

%% (5) PROJ-756 full diamond: D depends on both B and C; B and C both
%% depend on A. D must not be ready after only B completes (still waiting
%% on C), and must become ready -- exactly once -- after both B and C have
%% completed. A's own completion unlocks B and C immediately since each of
%% those has the single predecessor A.
diamond_dependency_and_join_test() ->
    Diamond = #{
        workflow => #{
            steps => #{
                <<"a">> => #{outputs => [], next => [<<"b">>, <<"c">>]},
                <<"b">> => #{outputs => [], next => [<<"d">>]},
                <<"c">> => #{outputs => [], next => [<<"d">>]},
                <<"d">> => #{outputs => [], next => []}
            }
        },
        active_steps => [<<"a">>],
        env => #{},
        history => []
    },
    Context0 = air_core:new(Diamond),

    %% a completes: both b and c (single predecessor: a) become ready.
    {Context1, Commands1} = air_core:transition({step_completed, <<"a">>, ok}, Context0),
    ?assertEqual(
        lists:sort([<<"b">>, <<"c">>]),
        lists:sort([StepId || {dispatch_step, StepId, _StepDef} <- Commands1])
    ),

    %% b completes: d is NOT ready yet -- it is still waiting on c.
    {Context2, Commands2} = air_core:transition({step_completed, <<"b">>, ok}, Context1),
    ?assertEqual([], Commands2),
    ?assertEqual(false, lists:member(<<"d">>, air_core:ready_steps(Context2))),

    %% c completes: d's full predecessor set {b, c} is now complete, so it
    %% becomes ready with exactly one dispatch_step command.
    {Context3, Commands3} = air_core:transition({step_completed, <<"c">>, ok}, Context2),
    ?assertEqual([{dispatch_step, <<"d">>, #{outputs => [], next => []}}], Commands3),
    ?assertEqual(true, lists:member(<<"d">>, air_core:ready_steps(Context3))).
