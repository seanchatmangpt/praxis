-module(arazzo_runner_app).
-behaviour(application).

-export([start/2, stop/1]).

start(_StartType, _StartArgs) ->
    arazzo_runner_sup:start_link().

stop(_State) ->
    ok.
