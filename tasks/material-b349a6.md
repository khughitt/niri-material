---
id: material-b349a6
title: Publish niri-material as a public repository with a fork-aware front page
status: doing
priority: 2
size: m
complexity: mid
process: direct
owner: materials-26.04
created: 2026-09-27T17:14:18Z
updated: 2026-09-27T17:14:24Z
started: 2026-09-27T17:14:24Z
depends: []
tags: [docs, release]
agent: claude-code/claude-opus-5-5
---

khughitt/niri-material is private and standalone on GitHub (not in niri-wm/niri's fork network). Make it public, presented as what it is: an unofficial, permanent fork of niri carrying the native material system. Prism (public) needs it for glass.

Approach:
- Keep the repo standalone and flip it public. A GitHub fork gains nothing: history was rewritten (no shared ancestry with upstream), and seams go upstream from upstream-based branches anyway.
- Front page in .github/README.md, which GitHub shows ahead of the root README. The upstream README.md stays byte-identical, so rebases see no new conflict. It covers: what the fork adds, unofficial and not affiliated with niri, which niri release it tracks, build/install (packaging/arch), config docs (docs/materials/material-config.md), prism, GPL-3.0-or-later with a modification notice, credit to upstream, and a note to report fork bugs here, not upstream.
- .github hygiene for a fork: FUNDING.yml (sponsor button names upstream's author), ISSUE_TEMPLATE (routes to niri discussions and Matrix), dependabot.yml (PR noise), release.yml. Each change is a class B seam, so record it in upstream-divergence.
- Audit (done 2026-09-27): gitleaks over all refs found only one false positive, on an unpushed branch. Docs and tasks hold ~300 local absolute paths (evidence roots). Ship history as-is, as prism did.
- Publish only materials-26.04 (default) and patched-26.04; task branches stay local.
- Hero screenshot of the glass: the user supplies it.

Making the repo public is outward-facing: confirm with the user first.

## Notes

- 2026-09-27T17:14:24Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:8c710f98-bc15-4d6b-b0cb-7b43df9f3a75","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
