# Native materials repository split

**Status:** implemented 2026-08-22; migration plan:
`docs/materials/plans/2026-08-22-repository-migration.md`

## Goal

Give native niri materials a clean project identity and source-of-truth
boundary without carrying the implementation structure or naming conventions
of the legacy Quickshell prototype.

The production repository is the actual niri fork. A separate companion
repository holds experiments, research, and reproducibility evidence. The
existing prototype and source-fork repositories remain unchanged as historical
provenance, not as active peers.

## Decision

Create two sibling repositories under the dispatcher-supplied project root
(the parent of the existing prototype repository):

1. `niri-material`: the full niri fork and production source of truth.
2. `niri-experiments`: research, experiment reports, small fixtures, and
   reproducibility bookkeeping.

This boundary keeps experimental churn and generated evidence out of the
production fork while allowing experiments to remain discoverable and
reproducible.

The split has one writer for production history: `niri-material` exclusively
owns future commits and upstream integration on `patched-26.04` and
`materials-26.04`. The existing source fork remains only for maintenance of
the open #4147 PR and for timing-spike provenance. Its copies and worktrees of
the two production branches are archival: they are not rebased, advanced, or
synchronized. There is deliberately no two-way sync mechanism. Retirement of
the source fork after the PR closes is a separate decision.

## Repository responsibilities

### `niri-material`

The repository contains full upstream niri history and preserves these local
branches:

- `patched-26.04`, including the two existing IPC commits;
- `materials-26.04`, including the five reviewed slice-0 material commits,
  ending at `3792432a`.

It owns:

- production compositor and shader code;
- stable material architecture and interface specifications;
- implementation plans that translate accepted findings into production
  changes;
- a material-project index at `docs/materials/README.md`.

The upstream niri `README.md` remains unchanged. Material-specific context
does not replace upstream project documentation.

`docs/materials/` is an accepted, small rebase surface inside upstream's
documentation tree. Upstream publishes `wiki/`, so this directory is not part
of the MkDocs site unless that configuration changes.

Its niri remote is named `upstream`. It has no `origin` until an owned remote
exists, preventing accidental pushes to the upstream project.

Initial material documentation:

```text
docs/materials/
  README.md
  2026-08-22-repository-split.md
  2026-08-22-v1-design.md
  plans/
    2026-08-22-repository-migration.md
    2026-08-22-slice0.md
```

### `niri-experiments`

The repository starts with a new `main` branch and no remote. It owns:

- gap analyses and research notes;
- spike and experiment results;
- small reusable fixtures;
- reproducibility commands and experiment bookkeeping;
- future tooling only after a repeated workflow justifies it.

Initial layout:

```text
README.md
docs/
  research/
    2026-08-22-gap-analysis.md
    legacy-visual-verification.md
  results/
    2026-08-22-slice0.md
fixtures/
  diagnostic-grid.svg
  opacity-checker.svg
```

Raw Tracy captures, generated images, logs, and CSV exports are not committed
initially. The reviewed result document is the durable evidence summary. Raw
artifacts remain untouched in the existing worktree until an artifact-retention
need justifies storage or Git LFS.

## Migration mapping

| Current artifact | Destination | Treatment |
| --- | --- | --- |
| Native materials v1 design | `niri-material/docs/materials/2026-08-22-v1-design.md` | Preserve accepted technical decisions; explicitly replace its old repository-boundary decision: production code/specs/plans live in `niri-material`, research/results/fixtures live in `niri-experiments`, and the frozen Quickshell reference remains in the legacy repository. Replace the live visual-method citation with the legacy method as research input; the slice-0 plan retains its specific procedure, and reusable native adaptation is deferred to slice 1. Update all companion paths. |
| Repository-split design | `niri-material/docs/materials/2026-08-22-repository-split.md` | Preserve the repository boundary and migration constraints as project history. |
| Repository-migration plan | `niri-material/docs/materials/plans/2026-08-22-repository-migration.md` | Preserve the approved execution procedure; update its spec path after migration. |
| Slice-0 implementation plan | `niri-material/docs/materials/plans/2026-08-22-slice0.md` | Preserve the corrected historical plan; rename repository variables and paths. |
| Visual-verification method | `niri-experiments/docs/research/legacy-visual-verification.md` | Preserve the Quickshell-specific method of record; update repository-qualified paths only. Native adaptation is not part of migration. |
| Gap analysis | `niri-experiments/docs/research/2026-08-22-gap-analysis.md` | Preserve research and provenance; describe the production fork as `niri-material`. |
| IPC timing-spike results | Remains at `niri-glass/docs/2026-08-09-niri-ipc-animation-timing-spike-results.md` | Do not copy; rewrite the gap analysis's prose citation to this repository-qualified full path. |
| Slice-0 results | `niri-experiments/docs/results/2026-08-22-slice0.md` | Preserve reviewed measurements, caveats, implementation commit references, and the recorded test/clippy baseline. |
| Diagnostic grid | `niri-experiments/fixtures/diagnostic-grid.svg` | Copy the source fixture only; generated raster output remains untracked. |
| Accepted opacity checker | `niri-experiments/fixtures/opacity-checker.svg` | Recreate a source fixture matching the accepted 1920×1080 checker: alternating black/white 120 px cells. Do not copy the generated PNG. |

The repository-split design itself migrates to the material documentation
index as project history. The old repository is not rewritten or deleted.

## Naming and cross-references

- `niri-material` is the production project and repository name.
- `niri-experiments` is the companion evidence repository.
- The old project name appears only when explicitly identifying the legacy
  Quickshell reference implementation or a repository-qualified provenance
  source retained there, including pre-split evidence and parameter files.
- Repository-root variables in reproducibility instructions use
  `$NIRI_EXPERIMENTS`, not the old project name.
- Pre-split document and worktree paths are replaced with the destination
  layouts above.
- Until repository remotes exist, cross-repository references name the
  companion repository and full document path without inventing broken web
  links. Never refer across repositories by basename alone.

Production plans may contain the feature-specific commands needed to execute
their acceptance gates. `niri-experiments` owns research about verification
methods, source fixtures, completed-run evidence, and experiment bookkeeping.
A reusable native verification method is added only when slice 1 needs it.

## Migration procedure

1. The dispatcher supplies `$PROJECTS_ROOT`, the parent directory of the
   existing prototype repository. Resolve the targets as
   `$PROJECTS_ROOT/niri-material` and `$PROJECTS_ROOT/niri-experiments`; fail if
   either exists, and never merge into or overwrite an unexpected directory.
   This intentionally makes the targets siblings of the prototype repository,
   not siblings of the source fork in its nested directory.
2. Record the existing source fork's `patched-26.04`, `materials-26.04`,
   `pr-4147`, and `ipc-animation-timing-spike` heads and worktree list. The
   expected heads are respectively `5e53b949`, `3792432a`, `d26ab5f2`, and
   `cba1119b`. Clone it as `niri-material`, materialize local
   `patched-26.04` and `materials-26.04` branches at their exact source refs,
   check out `materials-26.04`, and verify ancestry and source identity before
   adding documentation.
3. Replace the clone's source remote with `upstream` pointing to the canonical
   niri upstream URL; leave `origin` absent.
4. Add the material index, stable design, corrected slice-0 plan, and this
   repository decision. Do not modify production source during migration.
5. Initialize `niri-experiments` on `main`; add its README, research, results,
   and source SVG fixtures.
6. Apply only the treatments in the mapping table: rewrite project-home
   terminology and cross-references, the v1 repository-boundary decision, and
   its visual-verification citation. Preserve all other technical content,
   historical facts, and explicit legacy-client provenance.
7. Commit each repository independently with conventional commits and no
   attribution trailers.

## Verification

`niri-material` is accepted when:

- `patched-26.04` and `materials-26.04` retain the expected ancestry;
- `3792432a` is reachable and production files are identical to the reviewed
  slice-0 implementation before the documentation-only migration commit;
- `upstream` points to the niri upstream repository and `origin` is absent;
- build, the complete 221-test workspace suite (including the niri library's
  198 tests), and clippy complete with only the three recorded baseline
  warnings documented in
  `niri-experiments/docs/results/2026-08-22-slice0.md`;
- the migration commits change only files under `docs/materials/`;
- material documents contain no stale project-home names, old repository
  variables, old document paths, placeholders, or machine-local paths, except
  explicit legacy-client or source-fork provenance;
- the working tree is clean.

`niri-experiments` is accepted when:

- it contains only the documented initial files;
- the niri commit references in research/results exist in `niri-material`;
- no raw captures, generated raster images, logs, or CSV exports were copied;
- the opacity-checker source rasterizes to a 1920×1080 alternating black/white
  checker with 120 px cells;
- the migrated visual-verification document remains explicitly
  Quickshell-scoped and is not presented as a native compositor procedure;
- documents contain no stale project-home names except explicit legacy-client
  provenance, and no machine-local paths or placeholders;
- the working tree is clean.

The original repositories and worktrees remain byte-for-byte untouched by the
migration, apart from this design and migration plan receiving their final
status-only commit in the prototype repository after acceptance. The source
fork's four recorded branch heads and two live production worktrees must be
unchanged after migration. Its prunable timing-spike registration is reported
but is not an acceptance invariant. Its production-branch copies are archival,
and all future integration happens in `niri-material`. No hosted remote is
created, no branch is pushed, and nothing is merged.

After both repositories pass migration verification, pruning the archival
materials worktree is allowed only as a separate, destructive follow-up after
its untracked evidence has an explicit retention decision. Until then, the
single-writer rule is enforced by convention and the recorded-head checks.

## Alternatives rejected

### One fork with an `experiments/` subtree

This reduces cross-repository references, but experimental reports and
generated evidence predictably accumulate around production code. The
separation cost is small compared with keeping the fork reviewable.

### Experimental branches in the production fork

This avoids a second repository but makes experiments harder to discover,
compare, and retain independently. Branches are a poor catalog for research
artifacts.

### A third repository for the legacy client

The legacy Quickshell implementation already has a repository and remains a
reference. Copying it would recreate the naming and ownership ambiguity this
split is intended to remove.

### Rename the existing source fork in place

This would guarantee one copy of each production branch, but it would also
turn the repository backing the open PR, timing spike, and existing worktrees
into the new project. A fresh clone gives the production project a clean
identity while the explicit single-writer rule prevents drift; the old refs
remain fixed provenance and are not peers.

## Non-goals

- publishing or configuring owned remotes;
- moving or deleting the legacy prototype repository;
- committing raw profiling artifacts;
- redesigning the material system or changing production code;
- planning slice 1 or later material functionality.
