---
id: material-96ceee
title: Curate material public docs and migrate all retained historical documents
status: todo
priority: 0
size: l
complexity: mid
process: planned
created: 2026-11-04T17:59:16Z
updated: 2026-11-04T17:59:16Z
depends: [ops-b44874]
tags: []
source: ops-9a11f4
agent: codex
origin: agent
---

Why: material's project piece of ops-9a11f4, the user's highest-priority sweep to prepare public repositories. User clarification 2026-11-04: retained historical documents are migration targets even when earlier passes left them because code comments, AGENTS.md or other consumers cite them. Migration completion and reference-driven retention do not establish public-documentation completion.

Initial evidence (2026-11-04): code HEAD 9de61daaedc8b80676e7188ed2e3006435252455; records kind in-tree; 126 tracked candidates in historical-document directories or dated docs paths. Examples: docs/materials/2026-08-22-repository-split.md; docs/materials/2026-08-22-v1-design.md; docs/materials/2026-08-24-glass-config-surface-design.md; docs/materials/2026-08-24-v1-parity-design.md These are discovery seeds, not a complete semantic classification or a fixed inventory: refresh after prerequisite migrations, include nested/undated historical documents, non-Markdown retained inputs and cross-project citations. The full initial candidate list and project coverage are recorded on ops-9a11f4.

Done:
- Inventory every historical/working document still in the code tree, including the earlier migration's complete retained set. Treat every such entry as a migration target. Record its private records destination, inbound citations/runtime or test consumers, replacement public document (or why none is needed), and verification. A citation is work to resolve, never a permanent exception.
- Preserve historical bytes, history and internal search/reference access in the project's private records home resolved by tasks where. Couple removal with the necessary concise, curated replacement: update code comments, README, AGENTS.md, identity.toml authority and generated regions, other docs, fixtures and executable consumers before removing their historical sources. Preserve actual normative contracts and evidence through the transition; do not delete the only copy or silently drop validation.
- Curate README and any needed user/operator guide, human/agent developer guidance, and a minimal set of typed current public references (standards, data models, protocols or guides as appropriate). Verify behavior against code and keep recorded intended direction distinct. No document or abstraction is required merely to fill a category.
- Verify links, guide generation, document types, affected runtime/test behavior through project-declared routes under its own test policy, and operation without private records where the public tool should support it. Record explicit limits where private inputs are intentionally required. Documentation-only changes create no test obligation.
- Leave no historical source in the public code tree solely because it is referenced. Attach or reference a migration/curation receipt with per-target dispositions and the resulting public-doc list. Gate this project's publication on this task, independently of other project pieces. Optional history removal, remote writes and host changes remain separately approved actions.

Where to start: tasks show ops-9a11f4 and its project-coverage attachment; tasks doc lore-666ce3 migration-receipt (Reusable sequence, cleanup separate from optional history treatment); the current ops publication runbook and public-document-types standard; this project's instructions, current code, identity, public guides, first-pass migration receipt and all citation/consumer searches. Process is planned because the project-specific replacement document list and consumer changes need a reviewed design/plan; reuse the pilot approach instead of inventing new tooling. Existing periodic Documentation review tasks remain maintenance and do not replace this one-time migration.

First-pass prerequisite: ops-b44874; retain its existing timing, approvals and blockers. This task owns the retained targets left after that pass, not permission to bypass it.

## Notes

- 2026-11-04T17:59:16Z (add): created
  provenance: {"harness_session":"codex:01a1a7a1-1323-7e00-bce7-fd1b62c4f362","harness_session_source":"CODEX_THREAD_ID","origin":"agent"}
