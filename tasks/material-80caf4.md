---
id: material-80caf4
title: Verify optic settling under a real idle inhibitor
status: todo
priority: 1
size: s
complexity: mid
process: direct
created: 2026-10-02T08:06:31Z
updated: 2026-10-02T08:06:31Z
depends: []
parent: material-f86183
tags: [performance]
source: "docs/materials/2026-09-30-optic-settling-evidence.md#unverified-idle-inhibitor"
agent: codex
spec: docs/specs/2026-09-29-sustained-optic-settling-design.md
---

Remaining idle-inhibitor capture acceptance from material-2ee11e; the real-handler fixture already covers the wiring. Extend the existing bounded driver with a real idle-inhibitor client, prove that inhibition does not notify input activity or restart Aurora deadlines while held, and retain normal client updates. Pilot before matrix, explicit worktree binary, trace liveness controls, hashes, cleanup and run notes. Update the acceptance evidence; do not substitute the fixture test for the missing lifecycle capture.
