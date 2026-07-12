-module(arazzo_runner_app).
-behaviour(application).

-export([start/2, stop/1]).

%% F16 (V12-016): starts the new `arazzo_runner_root_sup` (Root Supervisor,
%% atlas L2) instead of `arazzo_runner_sup` directly -- see that module's own
%% header comment for why this is additive, not a behavior change to the
%% pre-existing workflow supervisor.
start(_StartType, _StartArgs) ->
    arazzo_runner_root_sup:start_link().

stop(_State) ->
    ok.
