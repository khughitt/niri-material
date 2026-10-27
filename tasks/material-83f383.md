---
id: material-83f383
title: Diagnose why GitHub CI ci.yml fails on most pushes while the local pre-push gate passes
status: todo
priority: 2
size: s
complexity: mid
process: direct
created: 2026-10-27T22:27:47Z
updated: 2026-10-27T22:27:47Z
depends: []
tags: [testing]
agent: claude-code/claude-opus-5-5
origin: user
---

Evidence from obs's CI input-contract sample (obs-0e4620, 2026-10-27, read-only GitHub API): ci.yml failed on 51 of the latest 55 push runs (2026-08-24 to 2026-10-26); 29 of those commits had a passing local full hook-pre-push tt run at the same revision and 22 a passing fast pre-push only. upstream-drift.yml also failed on 10 of 12 scheduled or dispatched runs. The one sampled failure (run 40152883835) failed in job 'full tooling tests', step 'Test tooling with guarded optional skips'. The count shows persistent redness, not 51 distinct defects: repeated failures may share one cause. Question: which failing jobs and steps recur, whether they share a cause, and whether the fix is in the code, the workflow or the local gate's coverage. Read job and step conclusions first (gh run view / jobs API); read logs only for the recurring step. Does not wait on obs's collector (obs-1af9fb).

## Notes

- 2026-10-27T22:27:47Z (add): created
  provenance: {"harness_session":"claude-code:5258301d-9399-4ef3-980b-d4ef962aff21","harness_session_source":"CLAUDE_CODE_SESSION_ID","origin":"user"}
