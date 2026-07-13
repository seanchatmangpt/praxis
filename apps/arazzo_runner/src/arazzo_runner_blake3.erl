-module(arazzo_runner_blake3).
-export([hex/1]).
%% tmp_file_path/0 exported for direct unit testing only (see
%% arazzo_runner_blake3_test.erl) -- hex/1 deletes its temp file before
%% returning, so the PID-inclusion property tmp_file_path/0 is responsible
%% for is not otherwise observable from hex/1's own external behavior.
-export([tmp_file_path/0]).

%% PROJ-781 (PRD v26.7.11 15 -- Receipt and Replay).
%%
%% Real BLAKE3 hashing, reachable from production Erlang code (not just
%% eunit, unlike its one prior use in this repo, air_core_corpus_test.erl's
%% PROJ-756 test-only helper of the same shape). Shells out to the `b3sum`
%% CLI (the reference BLAKE3 implementation) over a temp file, for the same
%% reason PROJ-756 chose it: apps/air_core/native/air_core_nif does not
%% wire up blake3 (only eval_expr_nif), and extending it would mean a new
%% NIF export, a new Cargo dependency, and a fresh cdylib build outside
%% this repo's just-only crates/ discipline. This is real BLAKE3 (confirmed
%% against the known empty-input test vector in
%% arazzo_runner_blake3_test.erl), not a substitute hash.
%%
%% Every call returns {ok, _} | {error, _} -- no erlang:error/1, unlike the
%% test-only precedent this mirrors -- because this module is now called
%% from production dispatch paths (arazzo_runner_broker:do_dispatch/6) that
%% must be able to refuse cleanly (typed {error, Reason}) rather than crash
%% the calling process when b3sum is unavailable or misbehaves.

%% # Complexity
%% O(bytes) to write the temp file plus O(bytes) for b3sum's own hash pass;
%% one subprocess round trip (bounded by the 10s receive timeout below).
-spec hex(binary()) -> {ok, binary()} | {error, term()}.
hex(Bytes) when is_binary(Bytes) ->
    case os:find_executable("b3sum") of
        false -> {error, b3sum_not_found};
        Path -> hex_via_port(Path, Bytes)
    end.

hex_via_port(B3sumPath, Bytes) ->
    TmpPath = tmp_file_path(),
    case file:write_file(TmpPath, Bytes) of
        ok ->
            Result = run_b3sum(B3sumPath, TmpPath),
            _ = file:delete(TmpPath),
            Result;
        {error, Reason} ->
            {error, {tmp_file_write_failed, Reason}}
    end.

run_b3sum(B3sumPath, TmpPath) ->
    try
        Port = erlang:open_port(
            {spawn_executable, B3sumPath},
            [{args, ["--no-names", TmpPath]}, binary, exit_status, use_stdio, stderr_to_stdout]
        ),
        collect_port_output(Port, <<>>)
    catch
        error:Reason -> {error, {b3sum_port_failed, Reason}}
    end.

%% # Complexity
%% O(k) recursive calls where k is the number of discrete {data, _} chunks
%% the OS delivers on the port (bounded in practice by b3sum's own small,
%% fixed-size hex-digest output plus stdout/stderr buffering); each
%% recursion step is a bounded binary append, not a rescan of Acc.
collect_port_output(Port, Acc) ->
    receive
        {Port, {data, Data}} ->
            collect_port_output(Port, <<Acc/binary, Data/binary>>);
        {Port, {exit_status, 0}} ->
            {ok, string:trim(Acc)};
        {Port, {exit_status, Status}} ->
            {error, {b3sum_failed, Status, Acc}}
    after 10000 ->
        {error, b3sum_timeout}
    end.

%% Filename choice only (neither erlang:unique_integer/1 nor the OS pid
%% feeds the hashed bytes themselves), so its own non-determinism across
%% runs does not compromise digest determinism -- same reasoning as
%% air_core_corpus_test.erl's tmp_file_path/0.
%%
%% Swarm audit wnl2yhbgm finding #10: erlang:unique_integer/1's counter is
%% scoped to the calling BEAM VM, not the shared /tmp filesystem two
%% SEPARATE VM instances actually write into -- two concurrently-spawned
%% VMs (e.g. this repo's own F16 driver spawns one escript per dispatch,
%% and recursion_crosses_engines_full_8x2_fanout genuinely drives many
%% concurrent dispatches) can each start their unique_integer counter near
%% the same low baseline and collide on the identical filename. A collision
%% here is not merely a robustness concern: hex_via_port/2 writes Bytes to
%% TmpPath, then reads it back via b3sum -- two VMs racing on the same path
%% risk one VM's b3sum hashing the OTHER VM's bytes, silently producing a
%% WRONG digest for the right input, in the exact receipt-chain primitive
%% this repo's invariant #2 (receipts are computed, never asserted) depends
%% on. os:getpid/0 is unique among concurrently-running OS processes at any
%% given instant (the OS itself guarantees this), so combining it with the
%% existing per-VM unique_integer makes cross-VM collision structurally
%% impossible for any two genuinely distinct, concurrently-live processes.
tmp_file_path() ->
    Dir = case os:getenv("TMPDIR") of
        false -> "/tmp";
        D -> D
    end,
    Pid = os:getpid(),
    Unique = erlang:unique_integer([positive]),
    filename:join(
        Dir,
        "arazzo_runner_blake3_" ++ Pid ++ "_" ++ integer_to_list(Unique) ++ ".bin"
    ).
