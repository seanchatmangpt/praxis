-module(arazzo_runner_blake3_test).
-include_lib("eunit/include/eunit.hrl").

%% PROJ-781 (PRD v26.7.11 15 -- Receipt and Replay).
%%
%% Proves arazzo_runner_blake3:hex/1 computes REAL BLAKE3 (against the
%% algorithm's own published test vectors), not a substitute hash or a
%% stand-in constant -- the same independent-verification discipline this
%% session's Rust-side receipt work uses (recompute outside the
%% constructor, assert equality), applied to the one Erlang-side hash this
%% ticket introduces to production code.

%% Published BLAKE3 test vectors (github.com/BLAKE3-team/BLAKE3 test
%% vectors, input length 0 and the well-known "abc" input), independently
%% reconfirmed against this machine's real b3sum CLI before writing this
%% test (`printf '' | b3sum --no-names` and `printf 'abc' | b3sum
%% --no-names`).
empty_input_matches_published_blake3_vector_test() ->
    {ok, Digest} = arazzo_runner_blake3:hex(<<>>),
    ?assertEqual(
        <<"af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262">>,
        Digest).

abc_input_matches_published_blake3_vector_test() ->
    {ok, Digest} = arazzo_runner_blake3:hex(<<"abc">>),
    ?assertEqual(
        <<"6437b3ac38465133ffb63b75273a8db548c558465d79db03fd359c6cd5bd9d85">>,
        Digest).

%% Content-sensitivity: not a constant stub -- two different inputs
%% genuinely produce two different digests.
distinct_inputs_produce_distinct_digests_test() ->
    {ok, D1} = arazzo_runner_blake3:hex(<<"input-one">>),
    {ok, D2} = arazzo_runner_blake3:hex(<<"input-two">>),
    ?assertNotEqual(D1, D2).

%% Determinism: hashing the same bytes 3 independent times (3 independent
%% subprocess round trips) produces byte-identical output every time --
%% run, not asserted from memory.
hex_is_deterministic_across_repeated_runs_test() ->
    Bytes = <<"arazzo_runner_blake3 determinism probe">>,
    {ok, D1} = arazzo_runner_blake3:hex(Bytes),
    {ok, D2} = arazzo_runner_blake3:hex(Bytes),
    {ok, D3} = arazzo_runner_blake3:hex(Bytes),
    ?assertEqual(D1, D2),
    ?assertEqual(D2, D3).

%% Real, triggerable typed failure: b3sum genuinely unreachable (empty
%% PATH) refuses with {error, b3sum_not_found} instead of crashing the
%% calling process -- restores the real PATH immediately after, so this
%% does not disturb any other test module sharing this BEAM VM.
hex_refuses_when_b3sum_not_found_test() ->
    OriginalPath = os:getenv("PATH"),
    true = os:putenv("PATH", ""),
    Result = try
        arazzo_runner_blake3:hex(<<"whatever">>)
    after
        true = os:putenv("PATH", OriginalPath)
    end,
    ?assertEqual({error, b3sum_not_found}, Result),
    %% PATH genuinely restored: a real b3sum call works again immediately
    %% after this test, proving the restore (not just the assertion above)
    %% actually took effect.
    ?assertMatch({ok, _}, arazzo_runner_blake3:hex(<<"post-restore-probe">>)).

%% Swarm audit wnl2yhbgm finding #10: tmp_file_path/0 previously used only
%% erlang:unique_integer([positive]) for a filename in the SHARED /tmp
%% filesystem -- unique across calls within THIS BEAM VM, but two separate,
%% concurrently-running VM instances (this repo's own F16 driver spawns one
%% escript per dispatch) each restart that counter independently, so their
%% filenames could collide on the shared filesystem. os:getpid/0 is unique
%% among concurrently-live OS processes at any instant, so its presence in
%% the generated path is the structural property that makes cross-VM
%% collision impossible. hex/1 itself can't be used to observe this (it
%% deletes its temp file before returning), so this drives tmp_file_path/0
%% directly -- exported for exactly this reason.
tmp_file_path_includes_the_calling_os_process_id_test() ->
    Path = arazzo_runner_blake3:tmp_file_path(),
    Pid = os:getpid(),
    ?assert(string:find(Path, Pid) =/= nomatch).

%% Regression guard for the property that actually prevents the collision:
%% two calls FROM THIS SAME PROCESS must still differ from each other (the
%% pre-existing unique_integer component still does its job within one VM;
%% the PID addition is additive, not a replacement that collapses distinct
%% calls in the same process onto the same path).
tmp_file_path_still_differs_across_repeated_calls_in_the_same_process_test() ->
    Path1 = arazzo_runner_blake3:tmp_file_path(),
    Path2 = arazzo_runner_blake3:tmp_file_path(),
    ?assertNotEqual(Path1, Path2).
