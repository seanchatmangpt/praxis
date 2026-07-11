-module(arazzo_runner_sup).
-behaviour(supervisor).

-export([start_link/0, start_workflow/1]).
-export([init/1]).

start_link() ->
    supervisor:start_link({local, ?MODULE}, ?MODULE, []).

start_workflow(WorkflowId) ->
    supervisor:start_child(?MODULE, [WorkflowId]).

init([]) ->
    %% Using simple_one_for_one because all children are identical workflow processes
    %% The 80/20 rule dictates one permanent process per workflow instance.
    SupFlags = #{strategy => simple_one_for_one,
                 intensity => 10,
                 period => 1},
    
    ChildSpecs = [
        #{id => arazzo_runner_workflow,
          start => {arazzo_runner_workflow, start_link, []},
          restart => temporary,
          shutdown => 5000,
          type => worker,
          modules => [arazzo_runner_workflow]}
    ],
    
    {ok, {SupFlags, ChildSpecs}}.
