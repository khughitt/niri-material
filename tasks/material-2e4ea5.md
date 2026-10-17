---
id: material-2e4ea5
title: "Capture gates judge measured contamination, not circumstantial host events"
status: doing
priority: 2
size: m
complexity: high
process: planned
owner: capture-gates-evidence
created: 2026-10-17T00:51:03Z
updated: 2026-10-17T10:08:55Z
started: 2026-10-17T10:01:20Z
depends: []
tags: [rendering]
agent: claude-code/claude-opus-5-5
spec: docs/specs/2026-10-17-capture-evidence-gates-design.md
---

Why: on 2026-10-16 three of eight quiet-host attempts were stopped by gates whose trigger did not touch the measurement. (1) A step in the idle GPU power floor (10.4 <-> 17 W, or 8.9 -> 10.4 W when the fb0-blanked display woke) failed the settle check, which compares every sub-run's idle watts with one run-start baseline (+-1.5 W). The refused noise A/B's completed sub-runs (noise-idle-ab-full-1792148041: none a 0.2877, b1 0.3246, b2 0.2918 ms) equal the passing run's (noise-idle-ab-full-1792149757) to one 1024 ns GPU-timer step. (2) A 20 s console login on tty2 (getty@tty2 restart, no CPU or GPU load) made the journal scan mark a nested headless pilot disturbed.

The gates exist for real reasons (2026-09-24: build-load tails, the idle lock raising P8, a desktop GPU blip), and the same-session alternation (a, b, a, b) plus the a/a, b/b repeat check already measure noise directly. The gates should stop runs on evidence of contention, and record circumstantial events for review instead of failing on them.

Design questions for the spec (process planned; the hold and host-conditions designs, docs/specs/2026-10-04-capture-disturber-hold-design.md and docs/specs/2026-10-08-capture-host-conditions-design.md, are reviewed protocol, so changes go through spec review):
- Settle: judge each sub-run by its own window (cpu busy, GPU util, clients, pstate, power IQR) and a step in the idle floor against the previous sub-run, rather than absolute watts against the run-start baseline; record the floor per sub-run.
- Disturbance scan: classify journal events by lane. A VT login or getty restart matters to the DRM and dedicated lanes (VT switches), not to nested headless runs; there it is an annotation. Keep the full list in capture.json.
- Severity: fail / annotate / ignore, decided per lane and recorded, so a reviewer can re-judge a run without rerunning it.
- Validation: replay tonight's refused capture.json records through the new rules (each refused attempt should pass with annotations, and the 2026-09-24 refusals should still refuse).
Related: material-04818d (what selects the 10.4 vs 17 W floor), material-d2e552 (clock-state selection of draws).

## Notes

- 2026-10-17T10:01:20Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:63b33ff5-4e3e-4682-9faf-fed3c89e90e9","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-17T10:01:32Z (capture-gates-evidence): resumed
  provenance: {"harness_session":"claude-code:63b33ff5-4e3e-4682-9faf-fed3c89e90e9","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-17T10:08:55Z (capture-gates-evidence): Spec drafted: power on headless judged as contention only with a GPU client in the window (else annotated); VT getty/autovt restarts annotated on nested lanes; read-only judge verb. Evidence: none-a-1 median 0.287744 ms at both the 17.1 W and 10.5 W floors.
