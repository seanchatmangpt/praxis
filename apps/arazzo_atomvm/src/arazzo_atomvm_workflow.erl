-module(arazzo_atomvm_workflow).
-export([start/1, dispatch_event/2, stop/1]).
-export([loop/2, loop_waiting_for_io/2, execute_io_worker/2]).

%% API

start(WorkflowId) ->
    InitialCoreState = case erlang:function_exported(air_core, initial_state, 0) of
        true -> air_core:initial_state();
        false -> undefined
    end,
    Pid = spawn(?MODULE, loop, [WorkflowId, InitialCoreState]),
    {ok, Pid}.

dispatch_event(Pid, Event) ->
    Pid ! {event, Event},
    ok.

stop(Pid) ->
    Pid ! stop,
    ok.

%% Actor loop

loop(WorkflowId, CoreState) ->
    receive
        {event, Event} ->
            case air_core:transition(Event, CoreState) of
                {ok, NewCoreState} ->
                    loop(WorkflowId, NewCoreState);
                {io_request, Req, NewCoreState} ->
                    spawn_worker(Req),
                    loop_waiting_for_io(WorkflowId, NewCoreState);
                {error, Reason} ->
                    exit({error, Reason});
                {stop, normal, _NewCoreState} ->
                    exit(normal);
                %% Fallback to handle direct context return just in case
                NewCoreState when is_tuple(NewCoreState), element(1, NewCoreState) =:= context ->
                    loop(WorkflowId, NewCoreState)
            end;
        stop ->
            exit(normal);
        _Other ->
            loop(WorkflowId, CoreState)
    end.

loop_waiting_for_io(WorkflowId, CoreState) ->
    receive
        {io_reply, _Reply} = Msg ->
            case air_core:transition(Msg, CoreState) of
                {ok, NewCoreState} ->
                    loop(WorkflowId, NewCoreState);
                {io_request, Req, NewCoreState} ->
                    spawn_worker(Req),
                    loop_waiting_for_io(WorkflowId, NewCoreState);
                {error, Reason} ->
                    exit({error, Reason});
                {stop, normal, _NewCoreState} ->
                    exit(normal);
                NewCoreState when is_tuple(NewCoreState), element(1, NewCoreState) =:= context ->
                    loop(WorkflowId, NewCoreState)
            end;
        stop ->
            exit(normal);
        _Other ->
            loop_waiting_for_io(WorkflowId, CoreState)
    end.

spawn_worker(Req) ->
    spawn(?MODULE, execute_io_worker, [self(), Req]).

execute_io_worker(Parent, Req) ->
    Reply = execute_io_request(Req),
    Parent ! {io_reply, Reply}.

execute_io_request(Req) ->
    %% Placeholder for real active I/O execution
    {ok, {processed, Req}}.
