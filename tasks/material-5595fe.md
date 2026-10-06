---
id: material-5595fe
title: Move the workstream lanes onto tasks' lane and needs fields
status: done
priority: 2
size: s
complexity: low
process: direct
owner: materials-26.04
created: 2026-10-03T16:18:06Z
updated: 2026-10-03T16:53:52Z
started: 2026-10-03T16:46:09Z
completed: 2026-10-03T16:53:51Z
depends: []
tags: []
source: docs/notes/2026-10-02-workstreams-brief.md
agent: claude-code
---

Why: the six workstream lanes and their host gates were recorded with the ad hoc tag `lane` and tags `needs-quiet`/`needs-nested`/`needs-owner`, because tasks had no first-class support. tasks now has lanes, needs and exclusive holds (tasks repo docs/specs/2026-10-03-lanes-needs-groups-design.md; skills/tasks/SKILL.md describes the commands), so `tasks lanes` and `prime` can show this organisation directly; today they show nothing for this project.

Done when:
1. tasks/.config.toml declares a [needs] vocabulary: `quiet` (exclusive = true; meaning: an idle host, a TTY with the desktop stopped), `nested` (a nested compositor on the headless weston unit; exclusive only if that unit can run one capture at a time - check the headless-verification setup before deciding) and `owner` (the owner supplies or judges an image; not exclusive). Declare the vocabulary before adding any need: `--need` refuses undeclared names.
2. The six lane goals carry the field (`tasks edit <id> --lane`) and lose the `lane` tag: material-2834d7, material-5d6b2c, material-6062fd, material-3aa1f2, material-53f873, material-6f606b. Each lane body leads with one guidance paragraph (why the lane exists, then its first milestone), because `tasks lanes`/`prime` show the first paragraph; today several begin "Lane goal (see ...)" - rewrite that opening and keep the rest.
3. Tagged tasks carry the field and lose the tag: `--need quiet` on material-1af3c6, material-7afc31, material-31074f; `--need nested` on material-07bac9; `--need owner` on material-fa4eec. Re-scan for other open tasks whose checks need a quiet host or the owner (the brief's lane table and the `tasks quiet` queue are the starting points) and add needs where clear.
4. Leftovers from the brief's "Pending regrouping": material-be611b goes under material-6062fd if its glass-edges worktree copy no longer blocks the write (otherwise note it on the task); material-77db8a goes under material-53f873 once its claim ends (otherwise leave a note). material-f7eb0b is already done - nothing to do.
5. docs/notes/2026-10-02-workstreams-brief.md says lanes and needs now live in the tracker (`tasks lanes`, `next --under <lane>`, `TASKS_WITHOUT=quiet`), and its "Pending regrouping" section is updated.
6. Check: `tasks check` is clean; `tasks --pretty lanes` lists six lanes, each with guidance and a state; `TASKS_WITHOUT=quiet tasks ready` hides the quiet-bound steps; `tasks list --tag lane` and the three needs-* tag queries return nothing.

Out of scope: setting TASKS_WITHOUT=quiet in the desktop session environment (that is the dotfiles project; mention it in the close note as a suggestion) and any change to the lane cut itself.

Where: tasks/.config.toml, the six lane task records, docs/notes/2026-10-02-workstreams-brief.md. Task-record and config edits only, through the tasks CLI; no worktree needed for task-record maintenance, but .config.toml is a committed file, so commit it with the records.

## Notes

- 2026-10-03T16:46:09Z (materials-26.04): started
  provenance: {"harness_session":"codex:01a102a8-0ebc-7e20-8c67-95d01b480cad","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-03T16:48:43Z (materials-26.04): Direct tracker maintenance in the main checkout as scoped; trial-arm: not enrolled. Nested is nonexclusive: material-signals-smoke.sh creates a per-launch Weston unit/socket; glass-optic-smoke-lib.sh uses a per-run runtime directory/socket. Quiet evidence retains its separate exclusive need. Additional clear needs: quiet on material-188aaa (capture hold validation), material-925518 (renderer-aware capture protocol) and material-3db428 (measured smoke scripts); owner on material-2592a2 (ring-glow visual judgement). Preparation can be split from capture when these tasks are taken up.
- 2026-10-03T16:48:43Z (materials-26.04): Pending worktree records: material-be611b reparent to material-6062fd refused with stale_copy (newer record in .worktrees/glass-edges); add owner there after merge. material-77db8a has no live claim but exists only in .worktrees/material-77db8a with its spec/plan; reparent to material-53f873 and add owner after merge. material-124f1f exists only in .worktrees/glass-edges and remains quiet-parked; add quiet and nested after merge. Preserve these parked records and branches. material-f7eb0b, material-80caf4 and material-3acc86 are done and need no migration.
- 2026-10-03T16:51:42Z (materials-26.04): Verification: tasks check clean; tasks --pretty lanes shows exactly the six expected lanes, all with guidance and state; next --under selects within the requested lane. TASKS_WITHOUT=quiet hides exactly the three ready quiet-bound steps (material-1af3c6, material-31074f, material-188aaa). All four retired-tag queries are empty. just test-fast passed with no changed workspace packages; git diff --check clean.
- 2026-10-03T16:52:46Z (materials-26.04): review: impl round 1 — verdict: revise; findings: Important 1; reviewer: codex
- 2026-10-03T16:53:06Z (materials-26.04): Review correction: material-bb3fe5 explicitly requires a smoke recording cost (material-optics spec section 11); added quiet. Its implementation can be separated from that capture when taken up.
- 2026-10-03T16:53:30Z (materials-26.04): Verification after review correction: tasks check clean; TASKS_WITHOUT=quiet tasks ready hides exactly four quiet-bound steps (material-1af3c6, material-31074f, material-188aaa, material-bb3fe5). Brief lane table now names measured-cost quiet access for the material library.
- 2026-10-03T16:53:51Z (materials-26.04): review: impl round 2 — verdict: accept; findings: none; reviewer: codex
- 2026-10-03T16:53:51Z (materials-26.04): done
  provenance: {"harness_session":"codex:01a102a8-0ebc-7e20-8c67-95d01b480cad","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-03T16:53:51Z (materials-26.04): Migrated six workstream goals to lane fields with guidance, declared quiet/nested/owner needs, replaced all retired tags, and added clear capture/review needs. Verified lanes, scoped selection and quiet filtering; review accepted. Newer/worktree-only glass-edges and view-tilt regrouping and needs remain documented for their integrating agents. Suggest dotfiles set TASKS_WITHOUT=quiet for busy desktop sessions; no host environment changes made.
  provenance: {"harness_session":"codex:01a102a8-0ebc-7e20-8c67-95d01b480cad","harness_session_source":"CODEX_SESSION_ID"}
