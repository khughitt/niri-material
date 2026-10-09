---
id: material-977bd9
title: "capture-meta: refuse before the hold on every host fault it can see"
status: todo
priority: 3
size: s
complexity: low
process: direct
created: 2026-10-09T04:22:12Z
updated: 2026-10-09T04:22:12Z
depends: []
tags: [capture]
source: material-18c2a1
agent: claude-code/claude-opus-5-5
---

Deferred minors from material-18c2a1's final review. The rollback is clean in each case, but the run takes the host before failing, and one leaves no record of why:

- A present-but-broken nvidia-smi fails a pixels run after the hold (GpuReader is built when nvidia-smi is on PATH and first used after the hold). Probe it before the hold, or drop it on pixels.
- When listing GPU clients fails in pre_hold_reasons, capture.json gets no preflight section; the lock is released. Write a refused section naming the failure.
- optic-settling-smoke.sh runs dedicated_prerequisites after the hold. Move it before.

Done when each case has a fake-host test showing the refusal comes before any hold action and is recorded.
