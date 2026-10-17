---
id: material-fa9ecb
title: "Harden hermetic git tests: non-vacuous focused run, honest inventory scope, CI Git version"
status: todo
priority: 3
size: s
complexity: low
process: direct
created: 2026-10-17T19:00:58Z
updated: 2026-10-17T19:00:58Z
depends: []
tags: [testing]
source: material-561927
agent: claude-code/claude-opus-5-5
---

Deferred minors from material-561927's final review (claude-opus-5-5, impl round 1; report summary in that task's notes).

- [ ] tools/test_hermetic_git.py FocusedRunTests checks only the child's returncode; a future skip on PrepareTests.test_prepare_validates_inventory_and_cleans_runtime would pass vacuously. Also assert 'Ran 1 test' in the child's stderr and no 'skipped'.
- [ ] InventoryTests: the GIT_USE comment claims it catches every git call, but test_glass_optic_smoke calls isolate without quoting 'git', so the probe never guards it. Reword the comment to 'a module that quotes the command name', and either drop isolate from test_glass_optic_smoke or cover it explicitly. Match discovery's test*.py glob rather than test_*.py.
- [ ] SuppressionTests.test_config_declared_hook_in_host_config: the comment calls CI's Git 'packaged' and pre-2.54; GitHub's ubuntu-24.04 may ship >= 2.54 through the git-core PPA, so the inert-control branch may run nowhere. Print git --version in ci-tooling-test, read which branch CI exercises, and reword the comment (and the spec's CI premise) to match.
