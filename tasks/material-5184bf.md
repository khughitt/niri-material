---
id: material-5184bf
title: Rewrite Fixture commit authorship in this repository
status: todo
priority: 2
size: s
complexity: low
process: direct
created: 2026-10-16T20:07:27Z
updated: 2026-10-16T20:07:27Z
depends: []
tags: [git]
source: git-identity-leak-2026-09-06
agent: claude-code
---

A fixture test written on 2026-09-06 (before tools/test_upstream_report.py gained its GIT_* scrub) ran git config user.name/user.email inside the pre-commit hook, where GIT_DIR redirected the writes into this repository's own .git/config. The local override Fixture <fixture@example.invalid> then authored every commit here from 3933fc80 (2026-09-06) to 2026-10-16: 1055 commits on all refs, many pushed to origin. ops-5749d9's t@t inventory missed it because the identity differs and the global config was never touched here. Local override removed 2026-10-16. Rewrite to Keith Hughitt <keith.hughitt@gmail.com> under the ops-e5b39c pilot procedure, and add a guard so a leaked repo-local identity fails the next commit.
