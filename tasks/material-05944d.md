---
id: material-05944d
title: "idle-budget supervisor: stop automated runs with TERM and say so when INT is ignored"
status: done
priority: 3
size: xs
complexity: low
process: direct
owner: materials-26.04
created: 2026-09-28T08:44:24Z
updated: 2026-09-30T14:03:42Z
started: 2026-09-30T14:00:01Z
completed: 2026-09-30T14:03:42Z
depends: []
tags: [capture]
agent: claude-code/claude-opus-5-5
---

On 2026-09-27 an agent started idle-budget-supervise.sh as a background job; a non-interactive shell starts such jobs with INT ignored, so the supervisor's INT trap never installed and the interrupt was lost (the run completed; the retry used env --default-signal=INT). Rejected fix: re-exec the supervisor with INT restored, which silently undoes a deliberate shell disposition. Instead, in niri-experiments results/idle-budget: (1) when INT is ignored at entry (SigIgn bit 2 in /proc/self/status), the supervisor prints one line on stderr naming TERM as the way to stop the run, and changes nothing else; (2) add a SuperviseTests case for TERM: forwarded to the fixture shell alone, the fixture finishes cleanup, the wrapper is not signalled, exit 143, mirroring the INT test; (3) in docs/results/2026-09-11-idle-budget.md, replace the pitfall bullet that recommends env --default-signal=INT with: agents stop a background supervised run with TERM (stop.json kind signal, message TERM, exit 143); Ctrl-C at a TTY is the human path.

## Notes

- 2026-09-30T14:00:01Z (materials-26.04): started
- 2026-09-30T14:03:42Z (materials-26.04): done
- 2026-09-30T14:03:42Z (materials-26.04): Supervisor (niri-experiments results/idle-budget ffbde3e) prints one stderr line naming TERM when INT is ignored at entry and changes nothing else; SuperviseTests gained a TERM case mirroring the INT one (fixture cleanup finishes, wrapper untouched, exit 143); the results doc's env --default-signal=INT pitfall bullet now says agents stop a background supervised run with TERM and Ctrl-C at a TTY is the human path. Fixture suite: 68 tests OK.
