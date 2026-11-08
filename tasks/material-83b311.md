---
id: material-83b311
title: Adopt current ops documentation citations
status: todo
priority: 2
size: s
complexity: low
process: direct
created: 2026-11-05T11:40:43Z
updated: 2026-11-05T11:40:43Z
depends: [ops-713299]
tags: [docs]
source: ops-7f0b75
---

Follow-up from the approved ops public-document curation, ops-7f0b75. The private archive is records commit a83536d369734fcf0cb2ad6b8c876e8f7d5dadb3, under ops/<original path>; the migration receipt is ops/docs/notes/2026-11-05-public-docs-migration-receipt.md. Current references are authored in ops on docs/ops-7f0b75-public-docs; wait for its separately approved paired integration, ops-713299, before adoption.

Refresh and disposition the current citations found in: .githooks/pre-push, justfile, tools/test-affected. Replace authored historical-current guidance pointers with the appropriate ops subject reference (tooling, project-guides, CLI conventions, key vocabulary, operations, execution evidence or a runbook). Keep genuine historical evidence tied to the private archive revision rather than rewriting its meaning. Preserve frozen fixtures byte-for-byte with explicit historical locators. For tools/cli.toml, tools/cli_surface.py or tools/keys.toml, adopt one compatible landed snapshot with normal conformance checks; do not hand-edit or broadcast tables. Family-copy comments follow that family's staged adoption, not a premature family publish. tt/ops-docs broadcast copies belong to the paired ops rollout and are not this task's edits.

Done: each named current match has a valid subject/private locator or explicit historical/non-applicable disposition; required hygiene and affected focused checks pass under checkout-local policy. Attach this receipt to any future publication prerequisite; an open task is ownership, not proof that citations are already fixed. No push, publication, live global-instruction delivery, host change or unrelated docs migration is authorized. Where generated output is involved, update its owning source and regenerate under the normal activation gate.

## Notes

- 2026-11-05T11:40:43Z (add): created
