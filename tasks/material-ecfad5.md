---
id: material-ecfad5
title: "Adopt the project-guide contract: identity.toml, generated regions, ops-docs check in the gate"
status: todo
priority: 3
size: s
complexity: mid
process: direct
created: 2026-10-23T20:14:57Z
updated: 2026-10-24T13:48:14Z
depends: []
tags: [docs]
source: ops-2c5fbc
agent: claude-code/claude-opus-5-5
---

Why: niri-material has no root identity.toml (ops `bin/ops-projects check`, 2026-10-23), so its line in the global Projects block is hand-edited in ops's identity-mirror.toml, no identity region or authority line reaches its guides, and `ops-docs check` reads nothing here. ops-2c5fbc reconciled the project-guides rollout on 2026-10-23: the old sequential wave 3 continues as one adoption task per project, owned by that project. This is material's.

Contract (ops repository): docs/specs/2026-09-22-project-guides-design.md §3 (identity.toml keys and limits), §4 (README and AGENTS.md section order, the identity region directly after the H1, CLAUDE.md as a one-line `@AGENTS.md` include or symlink, no status prose), §5 (`just docs`; `ops-docs check` after `ops-check` in the check recipe) and §9 step 3 (one adoption commit: identity.toml, the regions, sections reordered, status prose replaced by a pointer to the tracker or the authority document, global-layer restatements deleted, a budget raise declared where AGENTS.md cannot fit the default 120 lines; subsystems declared only where a boundary already has its own README). The default contract applies: this project belongs to no family (docs/specs/2026-10-09-family-guide-templates-design.md). Templates: ops templates/identity.toml, templates/README.md, templates/AGENTS.md and templates/justfile. Vendoring: ops docs/runbooks/project-lifecycle.md §2 step 2 (`vendored adopt ops-docs` in this checkout; bare `vendored` may not be on PATH, so use the explicit invocation in ops README's Consumers section). Before reshaping a guide, grep this repository's tests for reads of README.md or AGENTS.md.

Copy `name`, `aliases` and `purpose` from this project's table in ops's identity-mirror.toml unchanged. A new purpose changes the global Projects block, which needs `just projects` in ops and `just assemble` in lore under their own approval gate: record it here as a proposal instead.

At 2026-10-23: README.md 133 lines, AGENTS.md 98, no CLAUDE.md. tools/ops-check, tools/tt, .githooks and a check recipe are present. The trunk is materials-26.04 (there is no main).

Done when: `tools/ops-docs check` exits 0 here with identity.toml present; the project's check route runs it (and so its pre-commit hook, where it has one); and `bin/ops-projects check` in ops no longer lists material and still exits 0.

Profile: personal; integrate under this repository's own rules.

## Notes

- 2026-10-24T13:48:07Z (materials-26.04): curate-docs: material-10a999 at bfea33f0 confirms: Mechanical adoption gap: tools/ops-docs and root identity.toml are absent; ops-docs check exits 2, so guide regions/budgets/declarations are not verified. Complete the existing identity/checker/gate adoption.
- 2026-10-24T13:48:14Z (materials-26.04): curate-docs: material-10a999 at bfea33f0 confirms: README.md:53-56 and docs/materials/README.md:22-143 retain guide status/history prose. Replace completion labels and chronology with tracker/evidence pointers, preserving unique evidence in dated records and current compatibility/use guidance.
