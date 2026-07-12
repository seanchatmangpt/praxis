-module(arazzo_runner_dispatch_sup).
-behaviour(supervisor).

-export([start_link/0, start_dispatch/4]).
-export([init/1]).

%% ---------------------------------------------------------------------
%% F16 (atlas ticket V12-016) -- the "Dispatch Worker Supervisor" named in
%% the atlas L2 component-topology diagram. Real, distinct from
%% `arazzo_runner_sup.erl` (the existing Workflow Dynamic Supervisor, whose
%% own `simple_one_for_one` children are `arazzo_runner_workflow` processes,
%% not dispatch state machines) -- closes the disclosed gap
%% `crates/multifractal-workflow/src/f16_otp_runner.rs`'s
%% `check_dispatch_worker_supervisor_wired` names ("io-workers are
%% unsupervised spawn_link loops with no dedicated restart-policy supervisor
%% of their own"). This supervisor governs
%% `arazzo_runner_dispatch_statem` children specifically, not the io-worker
%% pool itself (`arazzo_runner_workflow:io_worker_loop/0` remains
%% unsupervised -- see that gap's still-disclosed remainder in
%% `f16_otp_runner.rs`'s updated doc comment).
%% ---------------------------------------------------------------------

start_link() ->
    supervisor:start_link({local, ?MODULE}, ?MODULE, []).

%% Starts one supervised `arazzo_runner_dispatch_statem` child for a single
%% step-dispatch lifecycle. Mirrors `arazzo_runner_sup:start_workflow/1`'s
%% own `simple_one_for_one` child-spawn convention.
-spec start_dispatch(binary(), #{}, binary(), map()) -> {ok, pid()} | {error, term()}.
start_dispatch(WorkflowId, Identity, StepId, StepDef) ->
    supervisor:start_child(?MODULE, [WorkflowId, Identity, StepId, StepDef]).

init([]) ->
    SupFlags = #{strategy => simple_one_for_one,
                 intensity => 10,
                 period => 1},
    ChildSpecs = [
        #{id => arazzo_runner_dispatch_statem,
          start => {arazzo_runner_dispatch_statem, start_link, []},
          %% `temporary`: a dispatch lifecycle is a one-shot, finite-lived
          %% state machine that always terminates in `completed` or `refused`
          %% (both a `normal` exit -- gen_statem's default `terminate/3`
          %% return of `ok` on a Data record that is not itself an error).
          %% There is no "retry the state machine" concept the way there is
          %% for a long-lived workflow process (`arazzo_runner_sup.erl`'s
          %% `transient` restart) -- a fresh dispatch should be started
          %% fresh by whoever wants to retry a step, not have this
          %% supervisor silently respawn one with stale Data.
          restart => temporary,
          shutdown => 5000,
          type => worker,
          modules => [arazzo_runner_dispatch_statem]}
    ],
    {ok, {SupFlags, ChildSpecs}}.
