---
id: material-7ff4bc
title: Attribute render errors and grain bias before fixes
status: todo
priority: 2
size: m
complexity: high
process: direct
created: 2026-10-06T20:44:04Z
updated: 2026-10-06T20:44:04Z
depends: []
parent: material-2834d7
tags: [harness]
source: docs/notes/2026-10-06-render-anomalies-brief.md
agent: codex
---

Establish reproducible causes for the renderer's GL error signature and the backdrop grain's dark-channel lift before choosing fixes. First milestone: isolate each observation using preserved journal provenance and deterministic controls; do not infer a shared cause from two rendering symptoms.

Done when each idea has an evidence-backed disposition or a separately scoped fix/design justified by its bounded investigation. This goal belongs to the evidence-instruments lane; the work adds attribution and controls, not a new runtime subsystem. Keep renderer changes, acceptance of color bias and host capture permission separate from the scope pass. Handoff: docs/notes/2026-10-06-render-anomalies-brief.md.
