-module(arazzo_runner_workflow).

%% API
-export([start_link/1, dispatch_event/2]).

%% Internal Callbacks
-export([workflow_loop/1, io_worker_loop/0, infra_loop/0]).

%% NOTE: process_transition/2's {ok,_}/{io_request,_,_}/{error,_}/{stop,_,_}
%% branches assume a transition/2 return shape air_core does not currently
%% produce (it returns a bare context()) -- see air_core.erl and
%% docs/jira/v26.7.11/tickets/index.md PROJ-755/756. Only the fallback
%% context-tuple clause is reachable today; the others are kept as the
%% target shape for when PROJ-755 lands, not as working code now.

%% API

start_link(WorkflowId) ->
    setup_infrastructure(),

    InitialCoreState = case erlang:function_exported(air_core, initial_state, 0) of
        true -> air_core:initial_state();
        false -> undefined
    end,

    %% Lock-free insert of initial state
    ets:insert(arazzo_workflow_states, {WorkflowId, InitialCoreState}),

    %% Spawn a lightweight receive loop to act as the process boundary.
    Pid = proc_lib:spawn_link(?MODULE, workflow_loop, [WorkflowId]),
    {ok, Pid}.

dispatch_event(Pid, Event) ->
    Pid ! {event, Event},
    ok.

%% Infrastructure and Queue Management

setup_infrastructure() ->
    case whereis(arazzo_runner_infra) of
        undefined ->
            Pid = spawn(?MODULE, infra_loop, []),
            try register(arazzo_runner_infra, Pid) of
                true ->
                    Pid ! init_infra,
                    wait_for_infra()
            catch
                error:badarg ->
                    %% Already registered by a concurrent start
                    wait_for_infra()
            end;
        _ ->
            wait_for_infra()
    end.

wait_for_infra() ->
    case ets:info(arazzo_workflow_states) of
        undefined ->
            timer:sleep(1),
            wait_for_infra();
        _ ->
            ok
    end.

infra_loop() ->
    receive
        init_infra ->
            %% Lock-free ETS table for workflow states.
            ets:new(arazzo_workflow_states, [public, named_table, set,
                                             {write_concurrency, true},
                                             {read_concurrency, true}]),

            NumWorkers = erlang:system_info(schedulers_online) * 16,
            [spawn_link(?MODULE, io_worker_loop, []) || _I <- lists:seq(1, NumWorkers)],
            infra_loop();
        _ ->
            infra_loop()
    end.

%% Workflow Process Loop

workflow_loop(WorkflowId) ->
    receive
        {event, Event} ->
            process_transition(WorkflowId, Event),
            workflow_loop(WorkflowId);
        {io_reply, Reply} ->
            process_transition(WorkflowId, {io_reply, Reply}),
            workflow_loop(WorkflowId);
        _Other ->
            workflow_loop(WorkflowId)
    end.

process_transition(WorkflowId, Event) ->
    %% Lock-free optimistic state read
    case ets:lookup(arazzo_workflow_states, WorkflowId) of
        [{WorkflowId, CoreState}] ->
            case try air_core:transition(Event, CoreState) catch C:R:S -> {exception, C, R, S} end of
                {ok, NewCoreState} ->
                    ets:insert(arazzo_workflow_states, {WorkflowId, NewCoreState});
                {io_request, Req, NewCoreState} ->
                    ets:insert(arazzo_workflow_states, {WorkflowId, NewCoreState}),
                    enqueue_io(self(), Req);
                {error, Reason} ->
                    error_logger:error_msg("Workflow ~p transition refused: ~p. Terminating.", [WorkflowId, Reason]),
                    ets:delete(arazzo_workflow_states, WorkflowId),
                    exit(normal);
                {exception, Class, Reason, Stack} ->
                    error_logger:error_msg("Workflow ~p crashed: ~p:~p ~p. Terminating.", [WorkflowId, Class, Reason, Stack]),
                    ets:delete(arazzo_workflow_states, WorkflowId),
                    exit(normal);
                {stop, normal, _NewCoreState} ->
                    ets:delete(arazzo_workflow_states, WorkflowId),
                    exit(normal);
                NewCoreState when is_tuple(NewCoreState), element(1, NewCoreState) =:= context ->
                    ets:insert(arazzo_workflow_states, {WorkflowId, NewCoreState})
            end;
        [] ->
            ok
    end.

%% Cluster-wide I/O Queue Routing

enqueue_io(ReplyPid, Req) ->
    case pg:get_members(arazzo_io_workers) of
        [] ->
            %% Fallback: wait and retry if cluster pg hasn't synced yet
            timer:sleep(10),
            enqueue_io(ReplyPid, Req);
        Workers ->
            WorkersTuple = list_to_tuple(Workers),
            Index = erlang:unique_integer([positive]),
            WorkerId = (Index rem tuple_size(WorkersTuple)) + 1,
            Worker = element(WorkerId, WorkersTuple),
            Worker ! {execute_io, ReplyPid, Req}
    end.

%% I/O worker pool: simplified Raft-style leader election so exactly one
%% worker owns heartbeat/coordination duties at a time. This is real,
%% ordinary leader-election bookkeeping for the worker pool -- it does not
%% grant any worker authority to modify code, files, or workflow semantics.

-define(ELECTION_TIMEOUT_MIN, 150).
-define(ELECTION_TIMEOUT_MAX, 300).
-define(HEARTBEAT_INTERVAL, 50).

io_worker_loop() ->
    pg:join(arazzo_io_workers, self()),
    State = #{
        role => follower,
        current_term => 0,
        voted_for => undefined,
        leader => undefined,
        votes_received => 0,
        timer_ref => start_election_timer()
    },
    io_worker_receive_loop(State).

start_election_timer() ->
    Timeout = ?ELECTION_TIMEOUT_MIN + rand:uniform(?ELECTION_TIMEOUT_MAX - ?ELECTION_TIMEOUT_MIN),
    erlang:send_after(Timeout, self(), election_timeout).

start_heartbeat_timer() ->
    erlang:send_after(?HEARTBEAT_INTERVAL, self(), send_heartbeat).

io_worker_receive_loop(State = #{role := Role, current_term := Term}) ->
    receive
        election_timeout ->
            NewTerm = Term + 1,
            NewState = State#{
                role => candidate,
                current_term => NewTerm,
                voted_for => self(),
                votes_received => 1,
                timer_ref => reset_timer(maps:get(timer_ref, State), election)
            },
            broadcast({request_vote, NewTerm, self()}),
            io_worker_receive_loop(NewState);

        {request_vote, CandidateTerm, CandidateId} ->
            if
                CandidateTerm > Term ->
                    CandidateId ! {vote_granted, CandidateTerm, self()},
                    NewState = State#{
                        role => follower,
                        current_term => CandidateTerm,
                        voted_for => CandidateId,
                        timer_ref => reset_timer(maps:get(timer_ref, State), election)
                    },
                    io_worker_receive_loop(NewState);
                true ->
                    io_worker_receive_loop(State)
            end;

        {vote_granted, VoteTerm, _VoterId} ->
            if
                Role =:= candidate, VoteTerm =:= Term ->
                    Votes = maps:get(votes_received, State) + 1,
                    Majority = (length(pg:get_members(arazzo_io_workers)) div 2) + 1,
                    if
                        Votes >= Majority ->
                            NewState = State#{
                                role => leader,
                                leader => self(),
                                timer_ref => reset_timer(maps:get(timer_ref, State), heartbeat)
                            },
                            broadcast({append_entries, Term, self()}),
                            io_worker_receive_loop(NewState);
                        true ->
                            io_worker_receive_loop(State#{votes_received => Votes})
                    end;
                true ->
                    io_worker_receive_loop(State)
            end;

        send_heartbeat ->
            if
                Role =:= leader ->
                    broadcast({append_entries, Term, self()}),
                    NewState = State#{timer_ref => reset_timer(maps:get(timer_ref, State), heartbeat)},
                    io_worker_receive_loop(NewState);
                true ->
                    io_worker_receive_loop(State)
            end;

        {append_entries, LeaderTerm, LeaderId} ->
            if
                LeaderTerm >= Term ->
                    NewState = State#{
                        role => follower,
                        current_term => LeaderTerm,
                        leader => LeaderId,
                        voted_for => undefined,
                        timer_ref => reset_timer(maps:get(timer_ref, State), election)
                    },
                    io_worker_receive_loop(NewState);
                true ->
                    io_worker_receive_loop(State)
            end;

        %% -- I/O Data Plane --
        {execute_io, ReplyPid, Req} ->
            Reply = execute_io_request(Req),
            ReplyPid ! {io_reply, Reply},
            io_worker_receive_loop(State);

        _ ->
            io_worker_receive_loop(State)
    end.

broadcast(Msg) ->
    Members = pg:get_members(arazzo_io_workers),
    [Pid ! Msg || Pid <- Members, Pid =/= self()].

reset_timer(OldRef, Type) ->
    erlang:cancel_timer(OldRef),
    %% Flush specific timers to prevent race conditions during state transitions
    receive election_timeout -> ok after 0 -> ok end,
    receive send_heartbeat -> ok after 0 -> ok end,
    if
        Type =:= election -> start_election_timer();
        Type =:= heartbeat -> start_heartbeat_timer()
    end.

%% Actual actuation (HTTP/RDMA/etc.) belongs behind the broker (PRD section
%% 13, docs/jira/v26.7.11/tickets/index.md PROJ-758), not dispatched
%% directly from an I/O worker. Until PROJ-758 lands this is an inert
%% placeholder that echoes the request back rather than performing any I/O.
execute_io_request(Req) ->
    {ok, {processed, Req}}.
