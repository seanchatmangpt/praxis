-module(arazzo_runner_sup).
-behaviour(supervisor).

-export([start_link/0, start_workflow/1]).
-export([init/1]).

start_link() ->
    supervisor:start_link({local, ?MODULE}, ?MODULE, []).

%% StartSpec: the map arazzo_runner_workflow:start_link/1 expects (10 PRD
%% 7.8 identity fields plus, for a fresh workflow, workflow_def/
%% active_steps/env/history). See arazzo_runner_workflow.erl.
start_workflow(StartSpec) ->
    supervisor:start_child(?MODULE, [StartSpec]).

init([]) ->
    %% Using simple_one_for_one because all children are identical workflow processes
    %% The 80/20 rule dictates one permanent process per workflow instance.
    SupFlags = #{strategy => simple_one_for_one,
                 intensity => 10,
                 period => 1},

    ChildSpecs = [
        #{id => arazzo_runner_workflow,
          start => {arazzo_runner_workflow, start_link, []},
          %% PROJ-757: was `temporary` (never restarted, regardless of exit
          %% reason) -- directly contradicts PRD 7.8's "The OTP runner
          %% SHALL survive execution-process restart". `permanent` is also
          %% wrong in the other direction: it would restart a workflow that
          %% exited `normal` (finished successfully, or was deliberately
          %% refused by an admission-result reaction -- see
          %% arazzo_runner_workflow:handle_reaction/3's
          %% {admission_result,{refused,_}} clause) forever. `transient`
          %% is the correct OTP semantics for "retry on crash, do not retry
          %% on a clean/deliberate stop": it restarts the child for any
          %% exit reason other than `normal` or `shutdown`/`{shutdown,_}`,
          %% which is exactly "survive an abnormal execution-process
          %% restart" without also fighting a workflow that legitimately
          %% finished. On restart, simple_one_for_one re-invokes
          %% start_link/1 with the *original* StartSpec args, but
          %% start_link/1 always consults arazzo_runner_identity:load/1
          %% first and reconstructs from there when present -- so identity
          %% and any progress made before the crash survive even though the
          %% args handed back by the supervisor are the pre-crash ones.
          restart => transient,
          shutdown => 5000,
          type => worker,
          modules => [arazzo_runner_workflow]}
    ],

    {ok, {SupFlags, ChildSpecs}}.
