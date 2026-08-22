# niri material

This branch carries the native material system inside the full niri fork.
Upstream project documentation remains in the repository root and `docs/wiki/`.

## Branches

- `patched-26.04`: niri 26.04 plus the two #4147 IPC commits.
- `materials-26.04`: production material work, stacked on `patched-26.04`.

All future production rebases and material commits happen in this repository.

## Material documentation

- `2026-08-22-repository-split.md`: repository ownership and provenance.
- `2026-08-22-v1-design.md`: accepted v1 material architecture.
- `plans/2026-08-22-repository-migration.md`: repository migration procedure.
- `plans/2026-08-22-slice0.md`: implemented renderer-seam plan.

Research, results, and source fixtures live in the sibling
`niri-experiments` repository. Explicitly qualified pre-split evidence and the
frozen Quickshell reference remain in the legacy `niri-glass` repository.
