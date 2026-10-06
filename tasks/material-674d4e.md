---
id: material-674d4e
title: "Owner's look at the backdrop noise site: keep, reshape, or drop it"
status: todo
priority: 2
size: xs
complexity: low
process: direct
needs: [owner]
created: 2026-10-06T08:56:17Z
updated: 2026-10-06T21:17:05Z
depends: [material-aaa714]
parent: material-3aa1f2
tags: []
agent: claude-code/claude-opus-5-5
---

Spec 2026-10-05-noise-placement-design.md §7.1 makes the backdrop variant conditional on the owner's look. The captures (evidence 2026-10-05-noise-placement-evidence.md) measured backdrop grain at 4% of glass after one blur pass and nothing above the 8-bit floor from three passes or any roughness above 0, far below the model's 19/7/5%: site=backdrop reads as grain only with backdrop-blur false and roughness 0. Look at the sheets and a live config, record the verdict in the evidence doc's 'Owner's look' line, and if the site is dropped or reshaped, file the change; prism-be5abe's rack copy depends on the answer.

## Notes

- 2026-10-06T21:17:04Z (materials-26.04): Review: the owner judges after material-aaa714 establishes the backdrop grain's dark-channel bias, so a keep verdict does not accept an unexplained bias (render-anomalies brief, alternative 3). This holds prism-be5abe until both close.
