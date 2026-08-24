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
- `material-config.md`: material and glass configuration reference.
- `plans/2026-08-22-repository-migration.md`: repository migration procedure.
- `plans/2026-08-22-slice0.md`: implemented renderer-seam plan.
- `plans/2026-08-22-slice1.md`: implemented config-and-assignment plan.
- `plans/2026-08-23-slice2.md`: implemented glass shader and composition plan.

Research, results, and source fixtures live in the sibling
`niri-experiments` repository. Explicitly qualified pre-split evidence and the
frozen Quickshell reference remain in the legacy `niri-glass` repository.

## Progress

Slice 1 is merged into `materials-26.04` at `ec0824c5`. Its nested-winit
verification, run against `2a55ab14`, is recorded at `niri-experiments`
commit `d155e90` in `docs/results/2026-08-22-slice1.md`.

Slice 2 is implemented on `slice2-glass` through `7e287517`. Its accepted
nested-winit verification, including the final Slice 1 per-render-target fix,
the transparent-workspace regression, and the matched opaque final-fix pair is
recorded at `niri-experiments` commit `a27eb8f` in
`docs/results/2026-08-23-slice2.md`; it closes the earlier texture-creation and
matched GPU-time measurement debt.

Slice 3 is implemented on `slice3-crop` through `01a4259c`. Its nested-winit
verification — characterization, live verification, performance record, and
final write-up — is recorded at `niri-experiments` commit `3038246` in
`docs/results/2026-08-24-slice3.md` (building on characterization at
`c37c653` and live verification + Tracy record at `50fb0e8`). The
coordinate-space half of the original G9 HACK premise did not reproduce on
current shaders; the shipped fix instead opens the strip-end per-workspace
crop bounds.

Next: a parity pass against the frozen Quickshell client, a physical DRM
smoke, and v1 acceptance.
