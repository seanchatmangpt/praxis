-module(arazzo_runner_identity).
-include("arazzo_runner.hrl").

%% PROJ-757 (PRD v26.7.11 7.8, Layer 8 -- OTP Outer Runner).
%%
%% Two responsibilities, kept in one module because they're two views of the
%% same fact: (1) construct/validate the 10-field #workflow_identity{} from
%% an untyped start-spec map, refusing loudly if any field is missing;
%% (2) durably persist #runner_state{} (identity + reconstructable execution
%% state) to DETS -- real disk-backed OTP term storage, keyed by
%% workflow_id -- so a workflow instance survives the death of every Erlang
%% process that ever held it in memory, not just a single crash-and-restart
%% within one still-running VM.
%%
%% What this is NOT: PROJ-758 (the broker/admission-graph integration) does
%% not exist yet, so there is no real receipt-chain or replay-log service in
%% this codebase for Erlang to read back from. DETS here is a genuine,
%% locally-durable persistence mechanism (not a simulation -- it really
%% survives process death and node restart, verified in
%% arazzo_runner_workflow_test.erl by actually killing the owning processes
%% before reading it back), but it is this ticket's own local
%% implementation of "admitted state and replay surfaces", not a wrapper
%% around a pre-existing receipt system. See PROJ-757's ticket text and
%% docs/jira/v26.7.11/tickets/index.md for the tracked follow-up.

-export([
    from_map/1,
    to_map/1,
    state_dir/0,
    table_name/0,
    open_table/0,
    close_table/0,
    persist/1,
    load/1,
    delete/1
]).

-define(REQUIRED_IDENTITY_FIELDS, [
    workflow_id,
    parent_workflow_id,
    arazzo_workflow_id,
    source_powl_region_id,
    dispatch_id,
    correlation_id,
    source_digest,
    projection_digest,
    receipt_head,
    replay_id
]).

%% ---------------------------------------------------------------------
%% Identity construction
%% ---------------------------------------------------------------------

-spec from_map(map()) ->
    {ok, #workflow_identity{}} | {error, {missing_identity_fields, [atom()]}}.
from_map(Map) when is_map(Map) ->
    Missing = [F || F <- ?REQUIRED_IDENTITY_FIELDS, not maps:is_key(F, Map)],
    case Missing of
        [] ->
            {ok, #workflow_identity{
                workflow_id = maps:get(workflow_id, Map),
                parent_workflow_id = maps:get(parent_workflow_id, Map),
                arazzo_workflow_id = maps:get(arazzo_workflow_id, Map),
                source_powl_region_id = maps:get(source_powl_region_id, Map),
                dispatch_id = maps:get(dispatch_id, Map),
                correlation_id = maps:get(correlation_id, Map),
                source_digest = maps:get(source_digest, Map),
                projection_digest = maps:get(projection_digest, Map),
                receipt_head = maps:get(receipt_head, Map),
                replay_id = maps:get(replay_id, Map)
            }};
        _ ->
            {error, {missing_identity_fields, Missing}}
    end.

-spec to_map(#workflow_identity{}) -> map().
to_map(#workflow_identity{} = Id) ->
    Fields = record_info(fields, workflow_identity),
    [workflow_identity | Values] = tuple_to_list(Id),
    maps:from_list(lists:zip(Fields, Values)).

%% ---------------------------------------------------------------------
%% Durable persistence (DETS)
%% ---------------------------------------------------------------------

%% Local runtime-state directory. Overridable via the `state_dir`
%% application env (used by tests to get a fresh, isolated DETS file per
%% test run); defaults to the OS-appropriate per-user cache directory so a
%% real deployment doesn't need any extra configuration to get durable
%% restart-survival.
-spec state_dir() -> file:filename().
state_dir() ->
    case application:get_env(arazzo_runner, state_dir) of
        {ok, Dir} -> Dir;
        undefined -> filename:basedir(user_cache, "arazzo_runner")
    end.

-spec table_name() -> atom().
table_name() ->
    case application:get_env(arazzo_runner, dets_table) of
        {ok, Name} -> Name;
        undefined -> arazzo_runner_state
    end.

state_file() ->
    filename:join(state_dir(), "runner_state.dets").

%% dets:open_file/2 is reentrant: calling it again for a table that's
%% already open (by this or another process) under the same name and file
%% just increments the internal reference count, it does not error or
%% duplicate work. Every entry point below calls this first, so the table
%% is opened lazily and stays open as long as at least one process has
%% touched it.
-spec open_table() -> ok | {error, term()}.
open_table() ->
    File = state_file(),
    ok = filelib:ensure_dir(File),
    Name = table_name(),
    case dets:open_file(Name, [{file, File}, {type, set}]) of
        {ok, Name} -> ok;
        {error, Reason} -> {error, {dets_open_failed, Reason}}
    end.

-spec close_table() -> ok | {error, term()}.
close_table() ->
    dets:close(table_name()).

-spec persist(#runner_state{}) -> ok.
persist(#runner_state{identity = #workflow_identity{workflow_id = WorkflowId}} = RunnerState) ->
    ok = open_table(),
    ok = dets:insert(table_name(), {WorkflowId, RunnerState}),
    %% Synchronous flush to disk: durability is the point of this table
    %% (restart-survival correctness), not raw throughput, and step-events
    %% are not a hot loop.
    ok = dets:sync(table_name()).

-spec load(binary()) -> {ok, #runner_state{}} | not_found.
load(WorkflowId) ->
    ok = open_table(),
    case dets:lookup(table_name(), WorkflowId) of
        [{WorkflowId, RunnerState}] -> {ok, RunnerState};
        [] -> not_found
    end.

-spec delete(binary()) -> ok.
delete(WorkflowId) ->
    ok = open_table(),
    dets:delete(table_name(), WorkflowId).
