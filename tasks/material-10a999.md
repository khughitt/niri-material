---
id: material-10a999
title: Documentation review
status: todo
priority: 3
size: s
complexity: low
process: direct
created: 2026-10-24T12:48:15Z
updated: 2026-10-24T15:29:36Z
depends: []
tags: [docs]
source: curate-docs
agent: codex
---

The reusable record of curate-docs passes over this project: one note per pass naming the rubric version, commit, documents covered, exclusions and findings by verdict; resolution of findings is recorded apart from the review. Never closed by a pass; ops owns its lifecycle.

## Notes

- 2026-10-24T12:48:31Z (materials-26.04): doc-review: eligible since 2026-10-24: baseline
- 2026-10-24T12:50:19Z (materials-26.04): doc-review: selected 2026-10-24; eligible since 2026-10-24; baseline; run /curate-docs --project material
- 2026-10-24T13:45:54Z (materials-26.04): curate-docs: rubric v2; commit bfea33f0; mechanical: ops-docs 2, ops-check 0: tools/ops-docs absent; covered: README.md (1-133), AGENTS.md (1-98), .github/README.md (undeclared, 1-93), niri-ipc/README.md (undeclared, 1-16), niri-visual-tests/README.md (undeclared, 1-14), docs/materials/README.md (undeclared, 1-143), docs/wiki/README.md (undeclared, 1-15), docs/materials/adding-an-optic.md (1-189), docs/materials/capture-host-setup.md (1-140), docs/materials/material-config.md sections: Material configuration 1-202, Optics introduction 203-211, saturation 212-220, noise 221-231, iridescence 232-242, reflection 243-253, edge-highlight 254-264, aurora 265-303, Presets 304-324; excluded: docs/specs/, docs/plans/, docs/notes/, docs/superpowers/, docs/materials/plans/ and dated docs/materials filenames: decision records or historical evidence; tools/: excluded tooling tree; docs/materials/scripts/, docs/hooks/ and non-prose assets: source evidence only; hosted upstream links: not fetched under read-only source/records scope; unread: docs/materials/material-config.md 325-586, docs/materials/performance.md 1-242, docs/materials/render-pipeline.md 1-230, docs/materials/upstream-divergence.md 1-285, every tracked docs/wiki/*.md except README.md in full; keep: 7, 10; not judged: 1: identity.toml absent, 2: no declared authority paths, 5: no Rules section at baseline, 6: retired in v2; budget checked mechanically but ops-docs unavailable; proposed: AGENTS.md: delete global tracker restatement at 11-12 and tt-report restatement at 48-49, docs/materials/README.md: correct backdrop-blur plan path to ../superpowers/plans/2026-08-29-material-backdrop-blur.md; stale: README.md and docs/materials/README.md: replace guide status/history prose with tracker/evidence pointers, docs/materials/adding-an-optic.md: worked Noise config and uniform examples retain the obsolete single-node representation, niri-ipc/README.md: link existing API/protocol usage documentation; decision: .github/, niri-ipc/, niri-visual-tests/, docs/materials/, docs/wiki/: declare guide coverage or reduce README to a pointer; unresolved: 6
- 2026-10-24T13:50:18Z (materials-26.04): curate-docs: resolution; mechanical tools/ops-docs and identity.toml absent: complete guide adoption -> material-ecfad5 (reused); README.md and docs/materials/README.md: replace guide status/history prose with tracker/evidence pointers -> material-ecfad5 (reused); AGENTS.md: remove four duplicated global-instruction lines -> material-ecfad5 (reused); docs/materials/adding-an-optic.md: correct layered Noise worked config and uniforms -> material-095049 (filed); docs/materials/README.md: correct backdrop-blur plan path -> material-4498c6 (filed); niri-ipc/README.md: link existing API/protocol usage documentation -> material-2a38d4 (filed); decision: 5 awaiting the human
