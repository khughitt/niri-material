# Upstream divergence

**Posture:** the fork is permanent. The material system is not going upstream.
"Upstream candidate" here means a *seam*: a small change to an upstream file that,
if accepted, would shrink our diff and make every rebase cheaper.

The baseline is pinned in `upstream-baseline.toml` and validated on every run by
comparing **trees**. This repository's history was rewritten: `git merge-base
materials-26.04 v26.04` returns a 2023 commit, and a diff from it reports 5743 files
against a true 176. Nothing here consults ancestry.

## Divergence classes

- **A — additive fork-only files.** Lower textual conflict risk, not zero cost:
  upstream can rename a directory we occupy, or add a file at a path we also add.
- **B — seam edits to upstream files.** Where conflicts are expected as a matter of
  course, and the only class where "upstream candidate" is meaningful.
- **C — vendored tooling and project scaffolding.** `tools/`, `.githooks/`,
  `justfile`, `packaging/`. Fork-only by construction, but tracked because
  `tools/tt` has an external source of truth in ops.
- **D — carried upstream patches.** The #4147 IPC commits, listed in
  `upstream-baseline.toml` as `carried`. Retired when their content is present in
  the release tag we move to — which is not the same moment upstream merges them.

**Class is an inventory aid, not a conflict filter.** Conflict detection runs over
the whole tree and is never scoped by class.

## Feature table

| Feature | Where | Class | Posture | Status |
| --- | --- | --- | --- | --- |
| Material rendering and glass shader | `src/render_helpers/material/`, `shaders/material/` | A | fork-only | — |
| Material and glass configuration | `niri-config/src/material/` | A | fork-only | — |
| Per-window signals | `src/render_helpers/signal.rs`, `src/window/signal.rs` | A | fork-only | — |
| Workspace IPC field | `niri-ipc/`, `src/ipc/`, `src/layout/workspace.rs` | D | carried | submitted #4147 |
| Material pass ordering hook | `src/render_helpers/blur.rs`, `effect_buffer.rs` | B | seam | unfiled |
| Tile material state | `src/layout/tile.rs` | B | fork-only | — |
| Test front door and timing | `justfile`, `tools/` | C | fork-only | — |

Posture is `seam`, `fork-only`, or `carried`. Status tracks seam candidates through
`unfiled` → `submitted #NNNN` → `merged <tag>` / `declined: <reason>`.

## Seam inventory

Generated from the git index against the baseline tree. Do not edit by hand; run
`just upstream-report`.

<!-- BEGIN GENERATED: local -->
Baseline `v26.04` (`8ed0da44d974`), tree `7b010d1b3ab2`, carrying 2 patch(es).

201 paths differ: 48 class B (seam), 129 class A (fork-only), 24 class C (scaffolding).

| Path | Class | Status | +/- |
| --- | --- | --- | --- |
| `.githooks/post-checkout` | C | A | +3/-0 |
| `.githooks/post-commit` | C | A | +3/-0 |
| `.githooks/post-merge` | C | A | +3/-0 |
| `.githooks/pre-commit` | C | A | +24/-0 |
| `.githooks/pre-push` | C | A | +64/-0 |
| `.github/FUNDING.yml` | B | D | +0/-1 |
| `.github/ISSUE_TEMPLATE/bug_report.md` | B | M | +2/-0 |
| `.github/ISSUE_TEMPLATE/config.yml` | B | M | +6/-9 |
| `.github/dependabot.yml` | B | D | +0/-24 |
| `.github/workflows/ci.yml` | B | M | +6/-2 |
| `.github/workflows/release.yml` | B | D | +0/-66 |
| `.gitignore` | B | M | +3/-1 |
| `AGENTS.md` | C | A | +56/-0 |
| `Cargo.toml` | B | M | +4/-0 |
| `docs/wiki/IPC.md` | B | M | +71/-0 |
| `docs/wiki/Nvidia.md` | B | M | +7/-31 |
| `justfile` | C | A | +91/-0 |
| `niri-config/src/animations.rs` | B | M | +35/-0 |
| `niri-config/src/lib.rs` | B | M | +1354/-0 |
| `niri-config/src/window_rule.rs` | B | M | +7/-0 |
| `niri-ipc/src/lib.rs` | B | M | +162/-1 |
| `niri-ipc/src/state.rs` | B | M | +108/-0 |
| `niri-visual-tests/src/cases/layout.rs` | B | M | +1/-0 |
| `niri-visual-tests/src/cases/tile.rs` | B | M | +11/-4 |
| `niri-visual-tests/src/cases/window.rs` | B | M | +1/-0 |
| `niri-visual-tests/src/test_window.rs` | B | M | +5/-0 |
| `packaging/arch/.gitignore` | C | A | +6/-0 |
| `packaging/arch/PKGBUILD` | C | A | +96/-0 |
| `src/animation/clock.rs` | B | M | +90/-0 |
| `src/backend/tty.rs` | B | M | +1/-0 |
| `src/backend/winit.rs` | B | M | +1/-0 |
| `src/cli.rs` | B | M | +41/-0 |
| `src/handlers/compositor.rs` | B | M | +1/-0 |
| `src/handlers/xdg_shell.rs` | B | M | +1/-0 |
| `src/input/pick_color_grab.rs` | B | M | +1/-0 |
| `src/ipc/client.rs` | B | M | +56/-1 |
| `src/ipc/server.rs` | B | M | +113/-1 |
| `src/layout/floating.rs` | B | M | +16/-5 |
| `src/layout/mod.rs` | B | M | +97/-9 |
| `src/layout/monitor.rs` | B | M | +136/-25 |
| `src/layout/scrolling.rs` | B | M | +53/-7 |
| `src/layout/tests.rs` | B | M | +265/-4 |
| `src/layout/tile.rs` | B | M | +2363/-78 |
| `src/layout/workspace.rs` | B | M | +34/-5 |
| `src/lib.rs` | B | M | +1/-0 |
| `src/niri.rs` | B | M | +293/-0 |
| `src/protocols/foreign_toplevel.rs` | B | M | +6/-4 |
| `src/render_helpers/blur.rs` | B | M | +113/-61 |
| `src/render_helpers/effect_buffer.rs` | B | M | +338/-1 |
| `src/render_helpers/mod.rs` | B | M | +27/-0 |
| `src/render_helpers/resize.rs` | B | M | +6/-1 |
| `src/render_helpers/shaders/mod.rs` | B | M | +175/-0 |
| `src/screencasting/mod.rs` | B | M | +1/-0 |
| `src/tests/client.rs` | B | M | +48/-0 |
| `src/tests/mod.rs` | B | M | +3/-0 |
| `src/window/mapped.rs` | B | M | +43/-0 |
| `src/window/mod.rs` | B | M | +36/-2 |
| `tools/capture-meta` | C | A | +892/-0 |
| `tools/ops-check` | C | A | +260/-0 |
| `tools/optic_settling.py` | C | A | +413/-0 |
| `tools/package-pin` | C | A | +109/-0 |
| `tools/test-affected` | C | A | +135/-0 |
| `tools/test_affected.py` | C | A | +49/-0 |
| `tools/test_capture_meta.py` | C | A | +865/-0 |
| `tools/test_gates.py` | C | A | +111/-0 |
| `tools/test_glass_optic_smoke.py` | C | A | +635/-0 |
| `tools/test_glass_render_order_metrics.py` | C | A | +184/-0 |
| `tools/test_optic_settling.py` | C | A | +265/-0 |
| `tools/test_package_pin.py` | C | A | +117/-0 |
| `tools/test_upstream_report.py` | C | A | +864/-0 |
| `tools/tt` | C | A | +276/-0 |
| `tools/upstream-report` | C | A | +689/-0 |

Class A paths are counted, not listed: fork-only additions with no upstream counterpart. Class C is listed because `tools/tt` has an external source of truth in ops. Because class A is counted, editing an existing class-A file's contents can never make this block stale; only adding or removing one can. Task records under `tasks/` are not counted.
<!-- END GENERATED: local -->

## Upstream drift

Generated against a fetched upstream. Refreshed weekly by
`.github/workflows/upstream-drift.yml` and at every rebase; the stamp says how stale
it is. Do not edit by hand; run `just upstream-report --drift`.

<!-- BEGIN GENERATED: drift -->
As of **2026-09-27**: baseline tree `7b010d1b3ab2`, fork `390dfc3e5fd1`, upstream `upstream/main` at `1f03391ea644`.

This is a three-way merge of final trees. It does not predict a rebase, which replays commits individually, and it says nothing about whether the result compiles or behaves correctly.

**11 conflicting path(s).**

| Seam path | Upstream name | Upstream commits since baseline | Upstream status | Conflict | Acknowledged |
| --- | --- | --- | --- | --- | --- |
| `.github/workflows/ci.yml` | same | 5 | - | - | - |
| `.gitignore` | same | 0 | - | - | - |
| `Cargo.toml` | same | 13 | - | - | - |
| `docs/wiki/IPC.md` | same | 1 | - | - | - |
| `niri-config/src/animations.rs` | same | 1 | - | - | - |
| `niri-config/src/lib.rs` | same | 11 | - | yes | yes |
| `niri-config/src/window_rule.rs` | same | 3 | - | - | - |
| `niri-ipc/src/lib.rs` | same | 2 | - | - | - |
| `niri-ipc/src/state.rs` | same | 0 | - | - | - |
| `niri-visual-tests/src/cases/layout.rs` | same | 0 | - | - | - |
| `niri-visual-tests/src/cases/tile.rs` | same | 0 | - | - | - |
| `niri-visual-tests/src/cases/window.rs` | same | 0 | - | - | - |
| `niri-visual-tests/src/test_window.rs` | same | 0 | - | - | - |
| `src/backend/tty.rs` | same | 9 | - | - | - |
| `src/backend/winit.rs` | same | 7 | - | - | - |
| `src/cli.rs` | same | 1 | - | - | - |
| `src/handlers/compositor.rs` | same | 2 | - | - | - |
| `src/handlers/xdg_shell.rs` | same | 4 | - | - | - |
| `src/input/pick_color_grab.rs` | same | 0 | - | - | - |
| `src/ipc/client.rs` | same | 6 | - | - | - |
| `src/ipc/server.rs` | same | 2 | - | - | - |
| `src/layout/floating.rs` | same | 4 | - | yes | yes |
| `src/layout/mod.rs` | same | 9 | - | - | - |
| `src/layout/monitor.rs` | same | 8 | - | yes | yes |
| `src/layout/scrolling.rs` | same | 12 | - | yes | yes |
| `src/layout/tests.rs` | same | 2 | - | - | - |
| `src/layout/tile.rs` | same | 2 | - | yes | yes |
| `src/layout/workspace.rs` | same | 6 | - | yes | yes |
| `src/lib.rs` | same | 0 | - | - | - |
| `src/niri.rs` | same | 18 | - | yes | yes |
| `src/protocols/foreign_toplevel.rs` | same | 1 | - | yes | yes |
| `src/render_helpers/blur.rs` | same | 1 | - | yes | yes |
| `src/render_helpers/effect_buffer.rs` | same | 0 | - | - | - |
| `src/render_helpers/mod.rs` | same | 3 | - | - | - |
| `src/render_helpers/resize.rs` | same | 0 | - | - | - |
| `src/render_helpers/shaders/mod.rs` | same | 0 | - | - | - |
| `src/screencasting/mod.rs` | same | 4 | - | - | - |
| `src/tests/client.rs` | same | 3 | - | yes | yes |
| `src/tests/mod.rs` | same | 2 | - | yes | yes |
| `src/window/mapped.rs` | same | 0 | - | - | - |
| `src/window/mod.rs` | same | 3 | - | - | - |

Conflict notices, verbatim and unparsed:

    CONFLICT (content): Merge conflict in niri-config/src/lib.rs
    CONFLICT (content): Merge conflict in src/layout/floating.rs
    CONFLICT (content): Merge conflict in src/layout/monitor.rs
    CONFLICT (content): Merge conflict in src/layout/scrolling.rs
    CONFLICT (content): Merge conflict in src/layout/tile.rs
    CONFLICT (content): Merge conflict in src/layout/workspace.rs
    CONFLICT (content): Merge conflict in src/niri.rs
    CONFLICT (content): Merge conflict in src/protocols/foreign_toplevel.rs
    CONFLICT (content): Merge conflict in src/render_helpers/blur.rs
    CONFLICT (content): Merge conflict in src/tests/client.rs
    CONFLICT (content): Merge conflict in src/tests/mod.rs
<!-- END GENERATED: drift -->

## Rebase procedure

Triggered by a new upstream release tag, or quarterly, whichever comes first. The
authoritative version, with the reasoning for each step, is
`../specs/2026-09-06-upstream-divergence-design.md`.

1. `git fetch upstream --tags`, then `just upstream-report --drift --against <newtag>`.
   Never pass `--force` to this fetch: upstream's own `v26.04` tag object names a
   different commit than this fork's local tag (both carry the release tree), and
   forcing the fetch would move the local tag and break baseline resolution. Exit 1
   here just means there are findings to read — unacknowledged conflicts against a
   new tag is the expected, informative outcome at this stage, not a broken tool.
2. Check whether #4147's content is present **in the target tag**, not merely merged
   to `main`. `git patch-id --stable` is evidence; read the tag's tree and decide.
3. Create `patched-<newtag>` and branch `materials-<newtag>` from it; record the new
   `tag`, the release tree, release commit, new patched commit, and carried
   patch-ids in `upstream-baseline.toml`.
4. Rebase in a fresh `.worktrees/` worktree with an explicit old boundary:
   `git rebase --onto patched-<newtag> patched-<oldtag> materials-<newtag>`.
   Flatten the merges — but first `git show --remerge-diff` each one in
   `git log --merges patched-<oldtag>..materials-<oldtag>`, record every non-empty
   result, and verify afterwards that each resolution survived.
5. Regenerate and stage the baseline record and the report, then `just gate`.
6. Nested GLES smoke on the headless host.
7. Physical DRM acceptance on real hardware, with a human watching scanout. Not
   interchangeable with step 6.
8. Prune stale acknowledgments, re-review the posture column, add a rebase-log line.
9. Re-pin packaging: push first, then `just package-pin <commit>`, which derives
   `pkgver`, the `source=` commit and `NIRI_BUILD_COMMIT` from that commit.
10. Keep the old branches as archives, then change the repository's default branch on
    GitHub (`gh repo edit --default-branch materials-<newtag>`) — the weekly workflow
    only schedules from the default branch. Moving local `origin/HEAD` does not do this.

## Rebase log

| Date | From | To | Conflicts resolved | Merge resolutions carried | Evidence |
| --- | --- | --- | --- | --- | --- |
| — | — | `v26.04` | — | — | baseline established 2026-09-06 |
