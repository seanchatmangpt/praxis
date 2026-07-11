-module(atomvm_runner).
-export([]).

%% PRD v26.7.11 section 7.9 (docs/jira/v26.7.11/PRD.md:425-436) requires this
%% module to execute the same AIR transition semantics as the OTP runner
%% (apps/arazzo_runner), with no separate semantic implementation.
%%
%% That does not exist yet. Tracked as PROJ-760
%% (docs/jira/v26.7.11/tickets/index.md) — a real wrapper over
%% apps/air_core's transition/2, plus the missing rebar build entrypoint
%% for this app (none exists today).
%%
%% This file previously contained unrelated placeholder code (process
%% spawning loops with no connection to AIR semantics) that has been
%% removed rather than left in place implying capability that isn't real.
