---
id: material-416a0d
title: "Balance fast tooling buckets by measured case time, not case count"
status: todo
priority: 2
size: s
complexity: mid
process: planned
created: 2026-10-17T15:27:43Z
updated: 2026-10-18T22:48:09Z
depends: []
tags: [testing]
agent: claude-code/claude-opus-5-5
---

Why: fast tooling packs whole modules into 4 buckets by case count (tools/tooling_tests.py `module_buckets`: largest module first into the bucket with the fewest cases). On 2026-10-17 (529 cases, global hook excluded) the buckets finished at 5.5/19.5/18.7/5.0 s of case time; balancing the same modules by time gives ~12.3 s each, ~7 s off the fast pre-commit route. Needs a source of per-module durations (last run's timings, a committed weights file) and a staleness rule. Data and context: material-561927, docs/specs/2026-10-17-pre-commit-fast-hermetic-git-design.md (Rejected alternatives).

Since the hermetic-git fix (material-561927, 4860a257) the fast route runs 32–39 s on titan against its 45 s limit (tt runs, 2026-10-17 18:16–18:36Z, 535 cases), so this is headroom, not an incident. The 2026-10-17 bucket figures were measured hook-free, which is the condition the fix now gives every run.

Established (scope 2026-10-18, at 27015551):
- The coordinator already times every case: each worker's receipt carries `elapsed` per case id, and the parent prints `case <id>: <s>` lines (tooling_tests.py near lines 111–132 and 264). No new timing instrument is needed, only somewhere to keep the totals between runs.
- tt keeps only the five slowest tests per run (`SLOWEST_KEEP = 5` in tools/tt), so runs.jsonl cannot supply per-module weights.
- `module_buckets` also packs the full route's non-lifecycle remainder (four buckets beside one job per lifecycle case), so one time-weighted packer serves both routes.
- tools/tooling_tests.py and tools/test_tooling_tests.py are in `tooling_full_paths`: the change commits through the full pre-commit route.
- material-5c9b37 (P4, open) adds the missing empty-bucket test to the same function.

Decision for the design spec: where the weights live and how they go stale.
- Lean: after each successful run the coordinator writes per-module case-time totals to a per-checkout, machine-local file (never committed, kept out of Dropbox like target/), and the next run packs longest-first into the least-loaded bucket. A module with no measurement gets its case count × the measured mean per-case time, and the `Fast tooling:` header says how many modules were estimated. It refreshes itself and needs no regeneration step.
- Rejected for now: a committed weights file. It is deterministic and would also help CI, but every new or renamed test module (14 added in the last month) needs a timed regeneration, a staleness test would fail the gate on each one, and one host's timings would be committed for every host.
- The spec also settles which runs may update the file (fast and full, or fast only; never a failed or cancelled run), two worktrees or two hook runs writing at once, and a malformed file (fail loudly or rebuild).

Done:
- The packer balances by measured weight; the inventory guard (no case lost or duplicated) is unchanged. tools/test_tooling_tests.py covers longest-first order, the estimate for unmeasured modules, the missing and malformed file, and one module with count=4 giving one bucket (material-5c9b37's test, folded in or landed first).
- Measured: three hook-pre-commit-fast runs on titan after the merge, with the slowest bucket within about 15% of the mean bucket case time and the route median at least 5 s under the 32–39 s baseline above.

Where: tools/tooling_tests.py (`module_buckets`, `run_tooling`, the worker receipts), tools/test_tooling_tests.py, the AGENTS.md Gates paragraph on fast tooling, docs/specs/2026-10-17-pre-commit-fast-hermetic-git-design.md (Rejected alternatives), material-561927 (bucket data and diagnosis notes).
