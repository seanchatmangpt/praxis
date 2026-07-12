-module(arazzo_runner_root_sup).
-behaviour(supervisor).

-export([start_link/0]).
-export([init/1]).

%% ---------------------------------------------------------------------
%% F16 (atlas ticket V12-016, L2 "Component Topology") -- the "Root
%% Supervisor" the atlas names as the top of the tree, with the Workflow
%% Dynamic Supervisor and the Dispatch Worker Supervisor as two of its
%% children. Introduced this session, additively:
%%
%%  - `arazzo_runner_sup` (pre-existing, UNCHANGED -- neither its `init/1`
%%    body nor its exported functions were touched) is started as this
%%    supervisor's first child instead of directly by
%%    `arazzo_runner_app:start/2`. It keeps its own `{local, arazzo_runner_sup}`
%%    registration (set in its own `start_link/0`), so every existing caller
%%    (`arazzo_runner_sup:start_workflow/1`,
%%    `supervisor:which_children(arazzo_runner_sup)` in
%%    `arazzo_runner_workflow_test.erl`) keeps working identically -- this
%%    supervisor only changes WHO starts `arazzo_runner_sup`, not what it is
%%    or how it behaves.
%%  - `arazzo_runner_dispatch_sup` (new this session) is started as a second,
%%    independent child -- the atlas's Dispatch Worker Supervisor.
%%
%% `one_for_one`: the two children are independent supervision domains (a
%% dispatch-worker crash storm should not tear down or restart live workflow
%% processes, and vice versa) -- not `one_for_all`/`rest_for_one`, which
%% would couple their restart behavior for no invariant this family actually
%% requires.
%% ---------------------------------------------------------------------

start_link() ->
    supervisor:start_link({local, ?MODULE}, ?MODULE, []).

init([]) ->
    SupFlags = #{strategy => one_for_one,
                 intensity => 10,
                 period => 1},
    ChildSpecs = [
        #{id => arazzo_runner_sup,
          start => {arazzo_runner_sup, start_link, []},
          restart => permanent,
          shutdown => 5000,
          type => supervisor,
          modules => [arazzo_runner_sup]},
        #{id => arazzo_runner_dispatch_sup,
          start => {arazzo_runner_dispatch_sup, start_link, []},
          restart => permanent,
          shutdown => 5000,
          type => supervisor,
          modules => [arazzo_runner_dispatch_sup]}
    ],
    {ok, {SupFlags, ChildSpecs}}.
