---
id: material-27332e
title: "capture-meta judge: close the deferred review minors"
status: todo
priority: 3
size: s
complexity: low
process: direct
created: 2026-10-17T12:29:16Z
updated: 2026-10-17T12:29:16Z
depends: []
tags: [capture]
agent: claude-code/claude-opus-5-5
---

Deferred from material-2e4ea5's final review (2026-10-17), all in tools/capture-meta's judge and its tests:
- The summary prints recorded settle counts but not the current ones (gates design §5 asks for both); print e.g. 'now 5 settled, 0 refused'.
- threshold_supported checks every DEFAULT_THRESHOLDS key, including preflight-only cpu_busy_pct, load1 and gpu_util_pct that settle never reads; limit it to the keys settle applies.
- cmd_judge's catch-all (KeyError/TypeError/ValueError/AttributeError) reports a judge defect as 'malformed record'. Validate entry shapes up front (sub-run entries are dicts with verdict; disturbances carry kind/unit/manager/at) and drop the catch-all, or reword the message.
- judge_windows appends a memory-floor reason string that can never print; set failed directly.
- The malformed-record test covers only a missing key; add a non-dict sub_runs entry.
- tools/test_capture_meta.py: realign capture_record's continuation lines; drop the triple blank line after test_console_login_restart_is_annotated_and_release_passes.
Check: rerun the retained-record replay; exit codes must match docs/specs/2026-10-17-capture-evidence-gates-design.md §6.2.
