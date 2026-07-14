-module(arazzo_atomvm_SUITE).
-include_lib("eunit/include/eunit.hrl").

%% PROJ-761 (docs/jira/v26.7.11/tickets/index.md) is the real OTP/AtomVM
%% differential conformance corpus this suite was named for. The previous
%% version of this file had a test function that unconditionally returned
%% `ok` without comparing anything -- a test that always passes regardless
%% of behavior is worse than no test, so it was removed rather than left as
%% false signal that equivalence is checked here.
%%
%% PROJ-761's real implementation now lives at
%% apps/arazzo_runner/test/arazzo_runner_atomvm_differential_test.erl, not in
%% this file -- it needs #runner_state{}/#workflow_identity{}
%% (apps/arazzo_runner/include/arazzo_runner.hrl) to read the OTP path's core
%% context back out of arazzo_runner_workflow's ETS state, and Erlang record
%% definitions require a textual -include that resolves for free only from
%% within that app's own test/ directory (see that file's own module doc for
%% the full placement rationale). This file is left in place, empty of
%% tests, as the pointer.
