---
id: material-7963b8
title: "Drift canary --check compares tag commit identity, contradicting its tree contract"
status: done
priority: 1
size: s
complexity: low
process: direct
owner: materials-26.04
created: 2026-09-19T13:07:35Z
updated: 2026-09-27T11:04:53Z
started: 2026-09-27T10:21:19Z
completed: 2026-09-27T11:04:53Z
depends: []
tags: [upstream, tooling, bug]
model: claude-opus-5-5
agent: "claude-code/claude-opus-5[1m]"
---

The weekly upstream-drift workflow has never passed: its only scheduled run (2026-09-14, run 34845581393) failed at `python3 tools/upstream-report --check` with `tag v26.04 resolves to 8ed0da44d974..., recorded tag_commit is aece2b0c4e1f...`, before the fetch and drift steps ran.

Cause: `resolve_baseline` in `tools/upstream-report` first compares `v26.04^{commit}` against `tag_commit` by identity. Locally the tag is the rewritten copy `aece2b0c`; on GitHub the fork's `v26.04` is upstream's tag object (`6a0a862b` -> `8ed0da44`), so a CI checkout resolves it differently. Both commits carry tree `7b010d1b`, and both `upstream-baseline.toml`'s header and the function's own docstring say validation compares trees, never commit identity — the first check does the opposite.

Fix: validate `<tag>^{tree}` against `tree` instead of comparing commits (keep `tag_commit` as the recorded local reference, or drop it from the required keys), add a tooling test that resolves the tag to a different commit with the same tree, then trigger the workflow with `gh workflow run upstream-drift.yml -R khughitt/niri-material` and confirm it reaches the drift step. Until this lands, the drift section of docs/materials/upstream-divergence.md only refreshes from local runs (last: 2026-09-06).

## Notes

- 2026-09-27T10:21:19Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:3f4a9869-e47a-4bc9-a90f-ec437ad135a0","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-27T10:26:49Z (materials-26.04): Fixed in a81806be (merged 7bddb325): tag validated by tree; tag_commit dropped (it named aece2b0c, which a CI clone never has, so --drift's merge-tree base would have failed next); merge base and report hash use baseline_commit. Verified in a file:// clone with upstream tags fetched from GitHub and aece2b0c absent: --check exits 0, --drift reaches its verdict and exits 1 on 5 unacknowledged conflicts (filed material-555b14). Remaining: push materials-26.04 (35 commits ahead of origin) and gh workflow run; that run will open the drift issue.
- 2026-09-27T10:26:49Z (materials-26.04): parked (waiting on user, approval): User approves pushing materials-26.04 (35 commits ahead of origin); then gh workflow run upstream-drift.yml -R khughitt/niri-material --ref materials-26.04, confirm it reaches the Drift step (expect exit 1 and a 'Upstream drift' issue for material-555b14's five paths), then tasks done.
  provenance: {"harness_session":"claude-code:3f4a9869-e47a-4bc9-a90f-ec437ad135a0","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-27T10:45:01Z (materials-26.04): resumed
  provenance: {"harness_session":"claude-code:3f4a9869-e47a-4bc9-a90f-ec437ad135a0","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-27T10:50:46Z (materials-26.04): First dispatched run (36313708159) failed earlier, at Tooling tests: test_within_aurora_* set EVIDENCE=/tmp but the smoke lib reads NIRI_MATERIAL_WORK_ROOT, which only this host exports. Fixed in 18288a7b; suite passes with the variable unset.
- 2026-09-27T11:04:53Z (materials-26.04): done
  provenance: {"harness_session":"claude-code:3f4a9869-e47a-4bc9-a90f-ec437ad135a0","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-27T11:04:53Z (materials-26.04): Baseline tag validated by tree; tag_commit dropped, drift merge base is baseline_commit (a81806be). Also fixed to reach the drift step in CI: the smoke-lib offline test needed NIRI_MATERIAL_WORK_ROOT and ImageMagick (18288a7b, a2a2c28a), and the issue step resolved gh's repository to the upstream remote, niri-wm/niri (d0106616). Dispatched run 36314529632 passes through Drift and opened issue #1 for material-555b14's five conflicts.
  provenance: {"harness_session":"claude-code:3f4a9869-e47a-4bc9-a90f-ec437ad135a0","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
