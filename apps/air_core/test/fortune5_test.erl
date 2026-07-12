-module(fortune5_test).
-include_lib("eunit/include/eunit.hrl").

%% Simulate a Fortune 5 complex workflow transition graph
%% Supply Chain Operations:
%% Step 1: validate_order
%% Step 2: check_inventory (deps: [validate_order])
%% Step 3: check_compliance (deps: [validate_order])
%% Step 4: dispatch_freight (deps: [check_inventory, check_compliance])
%% Step 5: finalize_invoice (deps: [dispatch_freight])

fortune5_workflow_test() ->
    %% 1. Initialize context with the static Fortune 5 AIR program
    Context0 = air_core:new(#{
        workflow => #{
            steps => #{
                <<"validate_order">> => #{
                    outputs => [{bind, <<"order_valid">>, {literal, true}}],
                    next => [<<"check_inventory">>, <<"check_compliance">>]
                },
                <<"check_inventory">> => #{
                    outputs => [{bind, <<"inventory_reserved">>, {literal, true}}],
                    next => [<<"dispatch_freight">>]
                },
                <<"check_compliance">> => #{
                    outputs => [{bind, <<"compliance_ok">>, {literal, true}}],
                    next => [<<"dispatch_freight">>]
                },
                <<"dispatch_freight">> => #{
                    outputs => [{bind, <<"freight_id">>, {literal, <<"FR-999">>}}],
                    next => [<<"finalize_invoice">>]
                },
                <<"finalize_invoice">> => #{
                    outputs => [{bind, <<"invoice_status">>, {literal, <<"PAID">>}}],
                    next => []
                }
            }
        },
        active_steps => [<<"validate_order">>],
        env => #{},
        history => []
    }),

    %% Assert initial ready steps
    ?assertEqual([<<"validate_order">>], air_core:ready_steps(Context0)),

    %% Transition: validate_order completes.
    %% transition/2 now returns {NewContext, Commands} per PRD v26.7.11 7.7
    %% (delta_AIR: (S,E) -> (S',C)) -- PROJ-755. validate_order unlocks two
    %% next-steps, so Commands must carry exactly those two dispatch_step
    %% requests (order-independent: bitmask iteration order is an
    %% implementation detail, not part of the contract).
    {Context1, Commands1} = air_core:transition({step_completed, <<"validate_order">>, ok}, Context0),
    Env1 = air_core:get_env(Context1),
    ?assertEqual(true, maps:get(<<"order_valid">>, Env1)),
    Ready1 = air_core:ready_steps(Context1),
    ?assertEqual(true, lists:member(<<"check_inventory">>, Ready1)),
    ?assertEqual(true, lists:member(<<"check_compliance">>, Ready1)),
    ?assertEqual(2, length(Commands1)),
    ?assertEqual(
        lists:sort([<<"check_inventory">>, <<"check_compliance">>]),
        lists:sort([StepId || {dispatch_step, StepId, _StepDef} <- Commands1])
    ),
    ?assertEqual(
        true,
        lists:all(fun({dispatch_step, _StepId, StepDef}) -> is_map(StepDef) end, Commands1)
    ),

    %% Transition: check_inventory completes. dispatch_freight has TWO real
    %% predecessors (check_inventory, check_compliance) -- PROJ-756's
    %% AND/join fix means it must NOT become ready off this one alone.
    {Context2, Commands2} = air_core:transition({step_completed, <<"check_inventory">>, ok}, Context1),
    Env2 = air_core:get_env(Context2),
    ?assertEqual(true, maps:get(<<"inventory_reserved">>, Env2)),
    ?assertEqual([], Commands2),
    ?assertEqual(false, lists:member(<<"dispatch_freight">>, air_core:ready_steps(Context2))),

    %% Transition: check_compliance completes. This is dispatch_freight's
    %% *second* and last predecessor, so the real, event-driven AND/join
    %% logic (no set_active_steps workaround -- that manual seed is gone;
    %% this is the real transition logic doing real join-resolution) now
    %% marks it ready and emits exactly one dispatch_step command for it.
    {Context3, Commands3} = air_core:transition({step_completed, <<"check_compliance">>, ok}, Context2),
    Env3 = air_core:get_env(Context3),
    ?assertEqual(true, maps:get(<<"compliance_ok">>, Env3)),
    ?assertEqual(
        [<<"dispatch_freight">>],
        [StepId || {dispatch_step, StepId, _StepDef} <- Commands3]
    ),
    ?assertEqual(1, length(Commands3)),
    ?assertEqual(true, lists:member(<<"dispatch_freight">>, air_core:ready_steps(Context3))),

    %% Transition: dispatch_freight completes; unlocks exactly one next-step
    {Context5, Commands5} = air_core:transition({step_completed, <<"dispatch_freight">>, ok}, Context3),
    Env5 = air_core:get_env(Context5),
    ?assertEqual(<<"FR-999">>, maps:get(<<"freight_id">>, Env5)),
    Ready5 = air_core:ready_steps(Context5),
    ?assertEqual([<<"finalize_invoice">>], Ready5),
    ?assertEqual(
        [<<"finalize_invoice">>],
        [StepId || {dispatch_step, StepId, _StepDef} <- Commands5]
    ),
    ?assertEqual(1, length(Commands5)),

    %% Transition: finalize_invoice completes (a leaf step; unlocks nothing)
    {Context6, Commands6} = air_core:transition({step_completed, <<"finalize_invoice">>, ok}, Context5),
    Env6 = air_core:get_env(Context6),
    ?assertEqual(<<"PAID">>, maps:get(<<"invoice_status">>, Env6)),
    Ready6 = air_core:ready_steps(Context6),
    ?assertEqual([], Ready6),
    ?assertEqual([], Commands6),

    %% Assert history size
    History = air_core:get_history(Context6),
    ?assertEqual(5, length(History)),
    ok.
