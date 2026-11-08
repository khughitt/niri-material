---
id: material-5184bf
title: Rewrite Fixture commit authorship in this repository
status: done
priority: 2
size: s
complexity: low
process: direct
owner: materials-26.04
created: 2026-10-16T20:07:27Z
updated: 2026-10-16T20:24:30Z
started: 2026-10-16T20:07:43Z
completed: 2026-10-16T20:24:28Z
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
- 2026-10-16T20:13:24Z (materials-26.04): attached: 2026-10-16-identity-commit-map.txt (88642 bytes): Authorship rewrite map: 1081 changed commit IDs (old new); resolves ~800 historical SHA citations in task notes, docs and commit messages without editing append-only text
- 2026-10-16T20:24:28Z (material-5184bf): Runbook finding (ops-e5b39c): git filter-repo silently drops non-standard commit headers (upstream niri's jj change-id), which rewrote 680 upstream commits and cascaded to 4098 changes here; the byte-level check caught it in the mirror. Used a plumbing rewrite instead (rev-list --topo-order --reverse, edit only parent/author/committer lines of commits with the identity or a rewritten parent, hash-object -w), which preserves every other byte. Also found: tag v1.0 (lightweight, on 871183304b87, a one-file 'release' commit committed by Fixture, on no branch, not on origin) was the 2026-09-06 fixture's own tag leaked into this repository, and the leak also set commit.gpgsign=false locally (global unset); both removed.
- 2026-10-16T20:24:28Z (material-5184bf): done
  provenance: {"harness_session":"claude-code:c27da384-b593-4ec1-ba12-eba1a97de08f","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-16T20:24:28Z (material-5184bf): Rewrote 1055 Fixture author / 1056 committer identities: 1082 commits re-hashed (1081 kept in the attached map; v1.0's junk commit dropped), every object byte-identical apart from mapped parents and identity, all trees identical, 0 upstream commits touched, 0 Fixture on any local ref, branch, tag or stash. One guarded update-ref transaction moved 16 branches + stash (single stash entry restored with its message), deleted v1.0; detached material-5b3107-baseline moved with --no-deref (522a09fe -> c171081d); all 10 Titan worktrees untouched (material-5b3107 keeps its uncommitted file). Europa: no worktrees, refs synced, 0 Fixture, tracked files match HEAD. Origin: rewritten twins of the existing tips pushed with leases through the pre-push gate (materials-26.04 477446ff -> 995eeab5, fix/ci-format-freebsd b2ac60a1 -> bdbb6d5f); nothing unpublished was published. PKGBUILD pin remapped 99196eba -> 1710a4f0 (same tree). Guard: tools/identity-check in check_after_cmd; fixtures assert confinement before config writes; full tooling run with hook-style GIT_DIR on a scratch clone left its config and refs unchanged. Bundle, original config/index and audit kept on Titan under WORK_ROOT at niri-material/identity-rewrite-20261016T200925Z.
