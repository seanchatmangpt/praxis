-module(arazzo_atomvm_SUITE).
-include_lib("eunit/include/eunit.hrl").

runner_equivalence_test() ->
    %% We mock air_core:transition/2 by loading a mock module for the test,
    %% or we just rely on passing {event, ...} and observing the process state.
    %% Since we cannot inspect the internal state of gen_statem easily without sys:get_state,
    %% and sys:get_state works on gen_statem, we can use it.
    %% For the actor loop, we can add a synchronous get_state message.

    ok.
