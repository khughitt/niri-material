---
id: material-5184bf
title: Rewrite Fixture commit authorship in this repository
status: doing
priority: 2
size: s
complexity: low
process: direct
owner: materials-26.04
created: 2026-10-16T20:07:27Z
updated: 2026-10-16T20:09:12Z
started: 2026-10-16T20:07:43Z
depends: []
tags: [git]
source: git-identity-leak-2026-09-06
agent: claude-code
---

A fixture test written on 2026-09-06 (before tools/test_upstream_report.py gained its GIT_* scrub) ran git config user.name/user.email inside the pre-commit hook, where GIT_DIR redirected the writes into this repository's own .git/config. The local override Fixture <fixture@example.invalid> then authored every commit here from 3933fc80 (2026-09-06) to 2026-10-16: 1055 commits on all refs, many pushed to origin. ops-5749d9's t@t inventory missed it because the identity differs and the global config was never touched here. Local override removed 2026-10-16. Rewrite to Keith Hughitt <keith.hughitt@gmail.com> under the ops-e5b39c pilot procedure, and add a guard so a leaked repo-local identity fails the next commit.

## Notes

- 2026-10-16T20:07:38Z (materials-26.04): concerns: ops-5749d9 extension — the t@t inventory searched one identity and missed this repository's repo-local Fixture leak
- 2026-10-16T20:07:43Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:c27da384-b593-4ec1-ba12-eba1a97de08f","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-16T20:08:03Z (materials-26.04): correction: the concerns line above is void — ops-5749d9 is still open, so this is scope added to an open goal, not a concern on closed work
- 2026-10-16T20:09:12Z (materials-26.04): Plan (ops-e5b39c pilot procedure, as hq/atoms/mind6): Europa idle per user, no material worktrees there, stale index preserved. bundle --all; mirror clone to WORK_ROOT outside Dropbox; filter-repo --mailmap (Fixture <fixture@example.invalid> -> Keith Hughitt) over every refs/heads plus refs/stash in one pass; verify commit objects differ only in mapped parents and identity; one guarded update-ref --stdin transaction for 18 branches + stash, plus update-ref --no-deref for the detached material-5b3107-baseline worktree (522a09fe); tags untouched (no tag target in the window). Attach changed-entry SHA map. Remap the functional PKGBUILD pin 99196eba via just package-pin. Push only the mapped twins of the current origin tips (materials-26.04 477446ff, fix/ci-format-freebsd b2ac60a1) with explicit leases, so nothing unpublished is published. Guard: just check refuses a repo-local user.name/user.email; fixtures assert confinement before their first git config write.
