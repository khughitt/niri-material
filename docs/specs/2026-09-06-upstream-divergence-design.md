# Upstream divergence document and sync strategy: design

**Status:** approved 2026-09-06; not implemented. Plan to follow.

**Task:** `material-a9447f`.

## Context

The fork carries a material system upstream niri does not have, and has no
written account of what it changed or how it will keep up. Two questions have
never been answered in the tree: what exactly diverges from upstream, and when
and how the fork moves forward.

The posture is settled and this design assumes it: **the fork is permanent.**
The material system is not going upstream. Upstreaming is worth pursuing only
for *seams* — small enabling changes to upstream files that, if accepted, would
shrink the diff and make every future rebase cheaper. "Upstream candidate" in
this design always means that, never feature donation.

## Current-build evidence

Every number below was measured on 2026-09-06 against the cached
`upstream/main` tip `3439d4ef` (2026-08-21). No fetch was performed; the drift
half of these figures is as stale as that cache.

### Ancestry is rewritten; trees are not

    git merge-base materials-26.04 v26.04   ->  64214407  (2023-08-14)

The recorded merge base is a 2023 commit. The fork's history was rewritten at
some point, so **ancestry-derived comparisons are wrong**, and wrong in a way
that looks plausible:

| Query | Reports | Truth |
| --- | --- | --- |
| `git diff <merge-base>..materials-26.04` | 5743 files, 218779 insertions | 176 files, 40611 insertions |
| `git log materials-26.04..upstream/main` | 2798 commits | 85 commits since `v26.04` |

The trees, however, are exact:

    patched-26.04~2^{tree} == v26.04^{tree} == 7b010d1b3ab29a1bee76b1554c6bc8eb09300ec6

So the baseline is verifiable by tree hash even though ancestry is not. Every
comparison in this design pins the baseline explicitly and compares trees.

### Branch shape

- `v26.04` = `aece2b0c`, 2026-04-25.
- `patched-26.04` = baseline + 2 IPC commits from the open upstream PR #4147.
  No merge commits.
- `materials-26.04` = 280 commits over `patched-26.04`, including **7 merge
  commits**.

### Diff shape

176 files change between `patched-26.04` and `materials-26.04`: **138 added,
38 modified, 0 deleted.** `Cargo.toml` and `Cargo.lock` are untouched — the
material system adds no dependency.

The 38 modified upstream files are the entire rebase cost:

- `src/` — 26 files
- `niri-visual-tests/` — 4
- `niri-config/` — 3
- `niri-ipc/` — 2
- `.github/workflows/ci.yml`, `.gitignore`, `docs/wiki/IPC.md` — 1 each

The 138 added files are overwhelmingly fork-only scaffolding: 71 under
`tasks/`, 46 under `docs/`, then `src/` (6), `.githooks/` (5), `tools/` (3),
`packaging/` (2), `niri-config/` (2), and `justfile`, `AGENTS.md`, `.agents/`.

### Conflicts today

    git merge-tree --merge-base=v26.04 upstream/main materials-26.04

exits 1 and reports six conflicting paths:

    src/layout/floating.rs
    src/layout/monitor.rs
    src/layout/tile.rs
    src/layout/workspace.rs
    src/protocols/foreign_toplevel.rs
    src/render_helpers/blur.rs

Six conflicts across 85 upstream commits. The drift is small and this is a good
moment to start measuring it.

### Release cadence

`v25.05` 2025-05-17, `v25.08` 2025-08-27, `v25.11` 2025-11-29, `v26.04`
2026-04-25 — roughly quarterly, with one longer gap.

### Packaging

`packaging/arch/PKGBUILD` carries a **static** `pkgver=26.04.r278.g5dbe182d`
and a `source=` line pinning commit `5dbe182d`. Nothing computes these; the
release procedure rewrites both by hand.

### CI

`.github/workflows/ci.yml` is already modified by the fork, so it is a seam
file, not untouched upstream. It runs `just ci-test` and `cargo check` feature
matrices. **It does not run `just check`** — format, clippy, tooling tests and
`tasks check` are enforced only by the local pre-commit hook.

## Decisions

1. One document, `docs/materials/upstream-divergence.md`, split into
   hand-written prose and machine-generated blocks. Prose explains *why* a
   divergence exists and whether it is a seam candidate; generated blocks carry
   every fact, so the factual half cannot drift.
2. The baseline is identified by **tree hash**, asserted on every run. Merge-base
   is never consulted.
3. Drift is detected by `git merge-tree --merge-base=<baseline>`, whose
   guarantee is stated narrowly (below).
4. Conflict **acknowledgments are hand-maintained in a separate file**.
   Regenerating the report can never approve a new conflict.
5. Generated content splits into a **local block** (needs only local refs,
   enforced fresh on every commit) and a **drift block** (needs a fetch,
   refreshed weekly and at rebase time, carrying a staleness stamp).
6. The fork rebases on upstream **release tags**, not `main`.
7. The drift check lives in a **new** workflow file. `ci.yml` is already a seam
   and must not grow.

## Baseline identity

`tools/upstream-report` resolves and verifies the baseline before doing
anything else:

1. Read the baseline record from `docs/materials/upstream-baseline.toml`:
   `tag`, `tag_commit`, `tree`, and `carried` (the list of commits applied on
   top, currently the two #4147 commits).
2. Assert `git rev-parse <tag>^{tree}` equals the recorded `tree`.
3. Assert `patched-<tag>~<n>^{tree}` equals the recorded `tree`, where `n` is
   the length of `carried`. This is what makes `carried` load-bearing rather
   than documentation: it is the depth at which the baseline tree must appear.

Any mismatch is a **hard error with a message naming both hashes**. There is no
fallback to merge-base — a silent fallback here reintroduces exactly the
5743-file lie this design exists to prevent.

Storing the baseline as data rather than deriving it from the branch name also
survives the branch rename that every rebase performs.

## What the canary does and does not prove

`merge-tree` performs a three-way merge of two **final trees**. Stated
precisely, a clean result means: *merging the current material tree with the
current upstream tree, against the recorded baseline, produces no textual
conflict.*

It does **not** prove:

- **That a rebase will be clean.** A rebase replays 280 commits individually
  and can conflict at intermediate states that the final-tree merge never
  visits. The canary is a lower bound on rebase pain, never an upper bound.
- **That the result compiles or behaves correctly.** Textual mergeability says
  nothing about semantics. An upstream signature change that our code calls
  correctly-but-differently merges cleanly and fails to build; a reordered
  render pass merges cleanly and changes what the screen shows.

The report states both limitations in its own header, so a reader who finds
"0 conflicts" cannot mistake it for "the rebase is done."

### Invocation and parsing

Measured on git 2.55.0: `merge-tree` writes **everything to stdout** and leaves
stderr empty. The implementation must therefore:

- Use `-z` for structured, NUL-separated sections rather than parsing the
  human-readable `Auto-merging` / `CONFLICT (content):` prose.
- Branch on **exit status**: `0` clean, `1` conflicts, **anything greater is an
  error** — a bad ref, a missing object, an unfetched upstream — and must fail
  the run loudly rather than be reported as "no conflicts."
- Never infer cleanliness from an empty conflicted-file list. Git can exit
  non-zero with no parsed paths; that combination is an error, not a pass.

## Document structure

    # Upstream divergence
    <baseline header: tag, tag commit, tree hash, carried patches, posture>

    ## Divergence classes            (prose)
    ## Feature table                 (prose)
    <!-- BEGIN GENERATED: local -->  (regenerated; enforced fresh per commit)
    ## Seam inventory
    <!-- END GENERATED: local -->
    <!-- BEGIN GENERATED: drift -->  (regenerated on fetch; stamped)
    ## Upstream drift
    <!-- END GENERATED: drift -->
    ## Rebase procedure              (prose)
    ## Rebase log                    (prose, one line per rebase)

### Divergence classes

- **A — additive fork-only files.** Lower textual conflict risk, not zero cost:
  upstream can still rename a module we add files beside, or restructure a
  directory we occupy. They carry no merge cost today.
- **B — seam edits to upstream files.** The 38. The entire rebase cost lives
  here, and this is the only class where "upstream candidate" is meaningful.
- **C — vendored tooling and project scaffolding.** `tools/tt`, `.githooks/`,
  `justfile`, `packaging/`. Fork-only by construction, but tracked because
  `tools/tt` has an external source of truth in ops.
- **D — carried upstream patches.** The two #4147 commits. Retired the moment
  upstream merges them; the rebase procedure checks for this explicitly.

### Feature table

Hand-written, one row per divergence: feature, where it lives, class, upstream
posture, status. Posture is one of `seam`, `fork-only`, or `carried`, and
status tracks the seam candidates through `unfiled` → `submitted #NNNN` →
`merged <tag>` / `declined: <reason>`.

### Seam inventory (generated, local)

Per class-B file: path, added/removed line counts, and its class. Derived
purely from `git diff <baseline-tree> <target-tree>`, so it needs no network.

Two self-reference rules:

- The report **excludes itself and its baseline record** from its own
  inventory. Otherwise regenerating changes the input to regeneration.
- The locally enforced block **embeds no commit SHA of the commit containing
  it**. That value cannot exist before the commit is made.

### Upstream drift (generated, needs fetch)

Per class-B file: upstream commits touching it since the baseline, whether
upstream renamed or deleted it (the highest-risk signal, and the one a
path-keyed conflict list misses entirely), and its conflict state.

Stamped with **baseline tree hash, fork SHA, upstream SHA, and date**, so
staleness is visible rather than silent.

## Acknowledgments

`docs/materials/upstream-conflicts.toml`, hand-edited, never written by the
tool. One entry per accepted conflict:

    [[acknowledged]]
    path  = "src/render_helpers/blur.rs"
    hunks = 2
    note  = "we insert the material pass before blur; expected to conflict every cycle"

The tool compares generated findings against this file and fails when it sees a
path that is not acknowledged, **or an acknowledged path whose conflict-hunk
count has risen**. Recording the count means a *new* conflict inside an
already-acknowledged file still trips the alarm.

Two limitations, stated in the file's header comment because they are easy to
forget:

- A conflict that **moves** within a file, leaving the count unchanged, is not
  detected. Path plus count is a coarse fingerprint, deliberately chosen over a
  content digest, which would churn on every upstream commit and train the
  reader to ignore it.
- Acknowledgment is not approval of upstream's change. It records that we
  expect to resolve this conflict by hand each cycle.

## Freshness enforcement

The two blocks are enforced differently because they have different inputs.

**Local block.** `just check` (and therefore the pre-commit hook) runs
`upstream-report --check`, which reads **the tree the index describes** — both
the source files and the report itself — and fails if regenerating would change
the report.

Keying on the index rather than on a branch or on `HEAD` is what lets one code
path serve both callers. In the pre-commit hook the index is the tree about to
be committed, so a commit cannot land source changes without the matching
inventory. In a fresh CI checkout the index matches the checked-out commit, so
the same command validates that commit. Comparing against `materials-26.04`
instead would miss work on feature branches; comparing against `HEAD` would
miss the staged changes being committed right now. There is deliberately no
mode flag to get wrong.

**Drift block.** Never enforced locally — it needs a fetch, and a pre-commit
hook must not touch the network. The scheduled workflow regenerates it against
the checked-out commit and fails on findings.

## Rebase procedure

Triggered by a new upstream release tag, or quarterly, whichever comes first.

1. `git fetch upstream --tags`, then
   `just upstream-report --drift --against <newtag>`. The baseline is still the
   old one at this point; `--against` only replaces the upstream side of the
   comparison, so this answers "what will conflict if we move to this tag"
   before anything is touched.
2. **Check whether #4147 merged upstream.** If it did, drop the two carried
   commits and remove class D from the document. If not, cherry-pick them onto
   the new tag.
3. Create `patched-<newtag>` from the new tag plus whatever step 2 decided.
   Record its tree hash in the baseline file.
4. In a fresh `.worktrees/` worktree, rebase the material work with an
   **explicit old boundary**:

       git rebase --onto patched-<newtag> patched-<oldtag> materials-<newtag>

   Without `--onto`, the rewritten ancestry reappears as a rebase that replays
   far more than the 280 material commits.

   The branch contains 7 merge commits. **Flatten them**: they are internal
   integration points with no upstream meaning, and `--rebase-merges` would
   preserve structure that costs conflict-resolution effort for no benefit. If
   a future merge encodes a resolution worth keeping, revisit this and say so
   in the rebase log.
5. `just gate`.
6. **Nested GLES smoke** on the headless host, using the existing harnesses
   under `docs/materials/scripts/`.
7. **Physical DRM acceptance**, separately, on real hardware with a human
   watching scanout. This is not interchangeable with step 6:
   `2026-08-27-v1-drm-acceptance-design.md` states that headless and nested
   hosts "cannot close the explicit real-hardware gate." A rebase can silently
   reorder render passes, so the render evidence is re-earned, not inherited.
8. Update the document: new baseline record, regenerate both blocks, prune
   acknowledgments that no longer apply, re-review the feature table's posture
   column, and add one rebase-log line recording the tag, the conflicts
   resolved, and the evidence runs.
9. Re-pin packaging: recompute the commit count since the pin, rewrite
   `pkgver` and the `source=` commit in `packaging/arch/PKGBUILD`, push before
   building.
10. Keep the old branches as archives. Move `origin/HEAD`.

## Scheduled workflow contract

`.github/workflows/upstream-drift.yml`, a new file. `ci.yml` is a seam already
and does not grow.

- **Schedule:** weekly. Scheduled workflows only run from the file present on
  the default branch, so this lands on `materials-26.04` to run at all.
- **Checkout:** `fetch-depth: 0` with tags, then an explicit
  `git remote add upstream` / `git fetch upstream --tags`. A fetch failure
  **fails the job**; it must never be reported as a clean drift result.
- **Permissions:** `contents: read`, `issues: write`.
- **Reporting:** one deduplicated issue, found by a fixed marker string in the
  body and updated in place rather than reopened per run. It fails the job when
  a path appears that `upstream-conflicts.toml` does not acknowledge, or an
  acknowledged path's hunk count rises.
- **Also runs on pull requests:** `just check`, which `ci.yml` does not run.
  Today those checks exist only in the local pre-commit hook, so a push from a
  machine without `core.hooksPath` set bypasses them entirely.

## Implementation surface

- `tools/upstream-report` — python3, stdlib only, following the
  `tools/test-affected` idiom.
- `tools/test_upstream_report.py` — picked up by the existing
  `unittest discover -s tools` that `just check` already runs.
- `justfile` — an `upstream-report` recipe passing arguments through, and
  `check` gains the `--check` call.
- `docs/materials/upstream-divergence.md`, `upstream-baseline.toml`,
  `upstream-conflicts.toml`.
- `.github/workflows/upstream-drift.yml`.
- `docs/materials/README.md` index entries; one `AGENTS.md` pointer.

## Verification

Unit tests, on fixture repositories built in `tmp` by the test itself:

- **Rewritten ancestry.** A fixture whose merge-base is unrelated to its
  baseline tree. Asserts the report matches the hand-computed tree diff, and
  that a tampered baseline tree hash produces a hard error naming both hashes
  rather than any fallback.
- **Index freshness.** Stage a source change without regenerating; assert
  `--check` fails. Regenerate and stage; assert it passes. Assert a change
  present in the working tree but unstaged does not affect the verdict, and
  that a fresh checkout with an empty staging area validates the checked-out
  commit rather than reporting a vacuous pass.
- **Conflict and error handling.** A fixture with a known conflict asserts exit
  1 and the exact path set. A deliberately bad ref asserts the exit-status-`>1`
  path fails loudly. A case with non-zero exit and an empty parsed path list
  asserts an error, not a pass.
- **Self-reference.** Assert the report excludes itself and the baseline record
  from its inventory, and that running it twice is a fixpoint.
- **Acknowledgments.** Assert an unacknowledged path fails; an acknowledged one
  passes; an acknowledged path with a raised hunk count fails.

Acceptance for the document itself: its generated numbers match the
hand-measured figures in the Current-build evidence section above, recomputed
at implementation time against a fresh fetch.

## Non-goals

- **No two-way sync.** Nothing flows from the fork to upstream automatically.
- **No automated rebasing.** The canary reports; a human rebases.
- **No per-hunk annotations in source files.** The seam list lives in the
  document, not in comments the rebase would have to carry.
- **No upstream PR automation.** Seam candidates are proposed by hand.
- **No tracking of `upstream/main` as a rebase target.** It is a signal source
  only.
