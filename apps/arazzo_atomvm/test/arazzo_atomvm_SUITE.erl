-module(arazzo_atomvm_SUITE).
-include_lib("eunit/include/eunit.hrl").

%% PROJ-761 (docs/jira/v26.7.11/tickets/index.md) is the real OTP/AtomVM
%% differential conformance corpus this suite is named for. It does not
%% exist yet. The previous version of this file had a test function that
%% unconditionally returned `ok` without comparing anything -- a test that
%% always passes regardless of behavior is worse than no test, so it has
%% been removed rather than left as false signal that equivalence is
%% checked here.
