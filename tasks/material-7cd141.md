---
id: material-7cd141
title: upstream-report --stage leaves the real index stale on a pathspec commit
status: doing
priority: 1
size: xs
complexity: low
process: direct
owner: materials-26.04
created: 2026-10-02T10:33:26Z
updated: 2026-10-02T10:33:28Z
started: 2026-10-02T10:33:26Z
depends: []
tags: [tooling]
agent: claude-code/claude-opus-5-5
---

git commit <paths> runs pre-commit on a temporary index (GIT_INDEX_FILE=.git/next-index-*.lock); --stage's git add lands there, so the commit is right but the real index keeps the old report. Seen 2026-10-02 after caf8c8c6 (vendor ops-check 6): HEAD and working tree +264, index +260, and the next commit's --stage refused with 'unstaged edits'. Fix: when regeneration is needed under a next-index temporary index, refuse with instructions instead of staging; -a and plain commits use the real index (or index.lock, which git commits back) and keep auto-staging.

## Notes

- 2026-10-02T10:33:26Z (materials-26.04): concerns: material-7f85b8 defect — --stage does not handle pathspec commits, whose temporary index never reaches the real index
- 2026-10-02T10:33:26Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:cebfaf5f-51dd-49c1-ae0b-f56f976f9f14","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
