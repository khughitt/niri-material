---
id: material-2f8ed6
title: Upstream the single-pixel resize-snapshot fix to niri
status: doing
priority: 3
size: s
complexity: mid
process: direct
owner: materials-26.04
created: 2026-10-06T14:01:43Z
updated: 2026-10-06T14:12:05Z
started: 2026-10-06T14:12:05Z
depends: []
tags: [rendering, upstream]
source: material-698875
agent: claude-code/claude-opus-5-5
---

material-698875 fixed render_snapshot_from_surface_tree (src/render_helpers/surface.rs) so a wp_single_pixel_buffer buffer bakes into a resize snapshot as a solid-colour element instead of being skipped. Upstream niri has the same code path; docs/materials/upstream-divergence.md lists it as a seam, status unfiled.

Verify first, on current upstream main (not v26.04):
- the code path is unchanged and still drops single-pixel buffers;
- the bug reproduces there: a test client with single-pixel buffers loses the snapshot mid-resize (upstream has no material, so check that RenderSnapshot::texture is None / the resize shader falls back, e.g. with an upstream-style test in src/tests/);
- no open upstream issue or PR already covers it.

Then prepare the PR on a branch off upstream main: the fix without fork-only code (BakedSurface, BakedSurfaceRenderElement, the LayoutElementRenderSnapshot change), a test in upstream's own test style, and a PR description in upstream's conventions. Opening the PR is an outward action: present the branch and description for the owner's go-ahead before pushing. Once submitted, set the seam's status in upstream-divergence.md to submitted #NNNN.

## Notes

- 2026-10-06T14:01:48Z (materials-26.04): concerns: material-698875 extension — take the landed single-pixel snapshot fix upstream
- 2026-10-06T14:12:05Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:d57c1528-57e5-424d-9b5a-e9635e5720d4","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
