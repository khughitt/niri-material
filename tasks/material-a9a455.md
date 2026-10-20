---
id: material-a9a455
title: "idle-budget fixture: finish its DRM sub-runs in capture.json"
status: idea
priority: 2
reviewed: true
created: 2026-10-06T13:03:07Z
updated: 2026-10-20T21:43:35Z
depends: [material-282edc]
parent: material-2834d7
tags: [capture]
agent: claude-code/claude-opus-5-5
---

Why: material-c44509 closes sub-runs in glass-optic-smoke-lib.sh's stop_nested and optic-settling-smoke.sh's stop_drm. The idle-budget fixture (niri-experiments, results/capture-protocol and results/idle-budget) sources the same lib but launches niri on DRM through its own start_drm, so its sub-runs settle and never finish. Scope: whether the fixture still runs; if so, call the lib's finish_sub_run from its DRM stop path.

## Current scope (2026-10-20)

material-9cf378 records continued idle-budget support and results/idle-budget advanced to 4d03b5b on 2026-10-09 (renderer verification and release-status propagation). This settles activity, not whether normal DRM stop calls finish_sub_run. Reuse material-282edc to inspect that retained consumer revision and distinguish normal completion from aborted cleanup; no capture is needed.

Read [ref:brief] for the existing alternatives, constraints and bounded handoff.

## References

- brief (input, required, kind=brief): docs/notes/2026-10-06-capture-lifecycle-brief.md — existing scope handoff and follow-up boundaries

## Notes

- 2026-10-06T13:03:07Z (materials-26.04): concerns: material-c44509 extension — the lib's sub-run finish does not reach a consumer with its own host stop
- 2026-10-06T19:17:33Z (materials-26.04): scope: briefed; local DRM stop already finishes sub-runs; the external idle-budget consumer is not present in this checkout and needs a bounded lifecycle inventory before a patch; brief: docs/notes/2026-10-06-capture-lifecycle-brief.md
