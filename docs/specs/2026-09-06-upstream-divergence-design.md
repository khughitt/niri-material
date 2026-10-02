# Upstream divergence document and sync strategy: design

**Status:** implemented 2026-09-06; plan
`../plans/2026-09-06-upstream-divergence.md`. The document it specifies is
`../materials/upstream-divergence.md`, the baseline it pins is
`../materials/upstream-baseline.toml`, and the canary is
`.github/workflows/upstream-drift.yml`.

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
comparison in this design pins the baseline explicitly and compares trees. A
tree hash cannot support `git log`, though, so the record also pins the release
commit, and history counting walks from there.

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

The 38 modified upstream files are where textual rebase conflicts can
arise. They are not the whole cost: an upstream change can break the fork
semantically without touching any of them.

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

`packaging/arch/PKGBUILD` carries a **static** `pkgver` and a `source=` line
pinning one commit, so a build is reproducible from the recipe alone. They were
rewritten by hand and drifted: the pinned `r278` was a count no rule produces.
`just package-pin <commit>` now derives `pkgver`, the `source=` commit and
`NIRI_BUILD_COMMIT` from the commit being packaged, and `check` refuses a
PKGBUILD whose three values disagree. `r<N>` counts the commits this fork
carries over `patched_commit` in the baseline record — `git describe` cannot
serve, because no upstream tag is an ancestor of this history.

### CI

`.github/workflows/ci.yml` is already modified by the fork, so it is a seam
file, not untouched upstream.

It does not invoke `just check`, but most of what `check` runs is covered by
CI's own jobs. `check_cmd` in the `justfile` is:

    cargo fmt --all -- --check && cargo clippy --all --all-targets
      && python3 -m unittest discover -s tools && tasks check

and `ci.yml` runs the first two in its `rustfmt` (line 233) and `clippy`
(line 218) jobs. The genuine CI gap is therefore narrow and specific: **the
tooling tests and `tasks check`** — plus the freshness check this design adds.

That narrowness is load-bearing. Closing the gap needs python3 and git only:
no Rust toolchain, no apt dependencies, no `cargo` build.

## Decisions

1. One document, `docs/materials/upstream-divergence.md`, split into
   hand-written prose and machine-generated blocks. Prose explains *why* a
   divergence exists and whether it is a seam candidate; generated blocks carry
   every fact, so the factual half cannot drift.
2. The baseline is identified by a **pinned record** — release tag, release
   commit, release tree, patched-branch commit, and the patch-ids of the
   carried commits — validated on every run and resolved by SHA, never by
   branch name. Merge-base is never consulted.
3. Drift is detected by `git merge-tree --merge-base=<baseline>`, whose
   guarantee is stated narrowly (below).
4. Conflict **acknowledgments are hand-maintained in a separate file**, keyed
   by path alone. Regenerating the report can never approve a new conflict.
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

       tag            = "v26.04"
       tree           = "7b010d1b..."   # tree OF THE RELEASE, not of patched-*
       patched_commit = "..."           # pinned SHA of the patched branch tip
       [[carried]]
       subject  = "..."
       patch_id = "..."

2. Assert `git rev-parse <tag>^{tree}` equals the recorded `tree`. The tag's
   commit is never compared: locally `v26.04` is the rewritten copy `aece2b0c`,
   while a GitHub checkout gets upstream's own tag object, which names
   `8ed0da44`. Both carry the release tree, and only the tree is portable.
3. Assert `<patched_commit>~<n>^{tree}` equals the recorded `tree`, where `n`
   is the length of `carried`. This compares **trees, not commit identities**,
   and it must: in this repository
   `patched-26.04~2` is `8ed0da44` while the local `v26.04` is `aece2b0c` —
   different commits carrying the identical tree `7b010d1b`. Requiring
   `<patched_commit>~<n>` to equal the tag's commit fails against the very fork
   this design describes.
4. Assert each carried commit's `git patch-id --stable` matches the recorded
   one, in order, at depths `n-1 .. 0` below `patched_commit`.

Four properties this shape buys, each in response to a way the simpler version
was wrong:

- **The recorded tree is the upstream release tree, never the patched tree.**
  Those differ whenever any patch is carried, and the validation compares
  against the release. Recording the patched tree would make step 2 fail on
  every cycle where `carried` is non-empty.
- **History is walked from `patched_commit~<n>`, never from the tag.** A tree
  cannot identify history, so history counting — upstream commits since the
  baseline, per-file churn, and the merge base of the drift check — needs a
  commit. `patched_commit~<n>` is upstream's real release commit, reachable from
  the fork's branch in every clone; the local tag's rewritten copy is on no
  pushed branch and not on upstream's line, so `git log` cannot walk from it.
  No commit hash for the tag is recorded: an earlier `tag_commit` key named the
  local copy, which a CI clone never has, and its identity check failed every
  CI run.
- **The fork side is anchored only by tree.** The rewritten history means the
  fork's copy of the release commit is a different object with the same
  content. Content is what the baseline actually asserts; commit identity is
  not available to assert and is not needed. The two sides are validated
  separately and meet at the tree hash.
- **`carried` is validated by ordered patch identity, not merely by depth.**
  Counting to depth `n` proves only that something sits there. Patch-ids prove
  it is the patch we think it is, and survive the rewriting a rebase does to
  commit SHAs.

Every ref is resolved through the **pinned SHAs**, never through branch names.
A fresh CI checkout has no local `patched-26.04` branch, so name-based
resolution would fail there; pinning also survives the branch rename each
rebase performs.

Any mismatch is a **hard error with a message naming both hashes**. There is no
fallback to merge-base — a silent fallback here reintroduces exactly the
5743-file lie this design exists to prevent.

## What the canary does and does not prove

`merge-tree` performs a three-way merge of two **final trees**. Stated
precisely, a clean result means: *merging the current material tree with the
current upstream tree, against the recorded baseline, produces no textual
conflict.*

It does **not** prove:

- **That a rebase will be clean.** A rebase replays 280 commits individually
  and can conflict at intermediate states that the final-tree merge never
  visits. The two results are not ordered: a rebase can hit conflicts the
  canary missed, and can also resolve differently. The canary is a change
  detector, not a measure of rebase cost.
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
- Never infer cleanliness from an empty conflicted-file list. Git's own
  documentation is explicit: *"Do NOT interpret an empty Conflicted file info
  list as a clean merge; check the exit status. A merge can have conflicts
  without having individual files conflict (there are a few types of directory
  rename conflicts that fall into this category)."* Exit `1` with no parsed
  paths is a **legitimate conflict**, not a tool error and not a pass. It is
  reported as an unattributed conflict, and because it has no path it can never
  be acknowledged: it fails the run until a human resolves it.

  Note this is specifically **directory-rename** conflicts. File/directory
  conflicts do produce an entry — with exactly one higher-order stage — so they
  are not the case to guard against here.
- **Do not classify conflicts.** The same documentation warns that the
  conflicted-file list carries insufficient information to infer conflict type,
  and that the informational-message section "is not designed to be machine
  parseable." The tool collects paths and the exit status, and reports the
  message text verbatim for a human. It derives nothing from it.
- If batching is ever added via `--stdin`, note that the per-merge *Merge
  status* integer inverts the process convention — `0` means conflicts and `1`
  means clean — and that the process exit status then no longer reports
  conflictedness at all.

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
- **B — seam edits to upstream files.** The 38. This is the only class where
  "upstream candidate" is meaningful, and where conflicts are expected as a
  matter of course.
- **C — vendored tooling and project scaffolding.** `tools/tt`, `.githooks/`,
  `justfile`, `packaging/`. Fork-only by construction, but tracked because
  `tools/tt` has an external source of truth in ops.
- **D — carried upstream patches.** The two #4147 commits. Retired when their
  content is present in the release tag being moved to — which is not the same
  moment upstream merges them, and the rebase procedure tests for the former.

**Class is an inventory aid, not a conflict filter.** Conflict detection runs
over the whole tree and is never scoped by class. A class-A file we add at a
path upstream independently adds conflicts add/add; upstream can rename a
directory out from under files in any class. The inventory says where our
changes are, not where conflicts can be.

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

The inventory also leaves out `tasks/`: task records never exist upstream and
cannot conflict on a rebase, and counting them as class A made every
`tasks add` stale the report until it was regenerated.
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
    path = "src/render_helpers/blur.rs"
    note = "we insert the material pass before blur; expected to conflict every cycle"

The tool compares generated findings against this file and fails on any
conflicting path that is not listed, and on any unattributed conflict.

**Acknowledgment is path-granular, and that is the whole contract.** An earlier
draft added a conflict-hunk count so that a *new* conflict inside an
already-acknowledged file would still trip the alarm. That is dropped, because
`merge-tree` does not supply hunk counts: the structured output reports
conflicted paths and stage entries, so any count would have to be derived by
re-parsing merged content, and rename, delete, and binary conflicts have no
textual hunks to count at all. A count would also have been defeated by a new
conflict that replaces or merges with an existing one, leaving the total
unchanged.

Two limitations, stated in the file's header comment because they are easy to
forget:

- **A new conflict inside an already-acknowledged file is not detected.**
  Acknowledging a path accepts every conflict in it, now and later. The
  mitigation is the drift block's per-file upstream churn count, which rises
  visibly when upstream works in a file we have accepted.
- Acknowledgment is not approval of upstream's change. It records that we
  expect to resolve this conflict by hand each cycle.

## Freshness enforcement

The two blocks are enforced differently because they have different inputs.

**Local block.** `just check` (and therefore the pre-commit hook) runs
`upstream-report --check`, which reads **the tree the index describes** and
fails if regenerating would change the report. Everything the check consumes
comes from the index, with no exceptions:

- the source files the inventory describes,
- `upstream-baseline.toml`,
- `upstream-conflicts.toml`,
- the report itself.

Reading the baseline record from the working tree while reading sources from
the index would let an unstaged baseline edit produce a report that validates
locally and fails in CI. One tree in, one answer out.

This fixes the regeneration order, which is otherwise easy to get wrong:

1. Stage the source changes **and** any configuration change
   (`upstream-baseline.toml`, `upstream-conflicts.toml`).
2. Run `just upstream-report`, which generates from the index.
3. Stage the regenerated report.
4. Commit.

Generating before staging configuration produces a report built from the old
baseline, which then disagrees with the staged configuration.

The pre-commit hooks run steps 2 and 3 themselves (`upstream-report --stage`,
added 2026-10-02 under material-7f85b8): the class-B line counts move with
nearly every source commit, and the manual step failed most code commits. It
splices into the staged document, so it needs no working-tree read, and it
refuses when the report has unstaged edits rather than sweep them into the
commit or overwrite them. The hooks then run `--check` as before, and `just
check` and CI only check, so a commit still cannot land without its
inventory.

Keying on the index rather than on a branch or on `HEAD` is what lets one code
path serve both callers. In the pre-commit hook the index is the tree about to
be committed, so a commit cannot land source changes without the matching
inventory. In a fresh CI checkout the index matches the checked-out commit, so
the same command validates that commit. Comparing against `materials-26.04`
instead would miss work on feature branches; comparing against `HEAD` would
miss the staged changes being committed right now. There is deliberately no
mode flag to get wrong.

Class A paths are counted in the local block, not listed, so as a consequence
editing an existing class-A file's contents can never make the report stale —
only adding or removing one changes the count. Someone probing the freshness
gate with a class-A content edit and finding it still passes has not found a
bug; that is the class-A contract working as designed.

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
2. **Check whether #4147's content is present in the selected target tag** —
   not merely whether it merged into upstream `main`. A patch can be merged to
   `main` and still be absent from the tag we are moving to, and dropping it
   then would silently remove the IPC surface.

   `git patch-id --stable` against `git log <newtag>` is **evidence,
   not the decision**. Upstream routinely squashes or revises a patch during
   review, which changes its id while keeping its behaviour, so a miss does not
   prove absence; and a patch that did land can later be reverted, so a hit
   does not prove presence. Use the id scan to find candidates, then **read the
   target tag's tree** for the IPC surface itself and decide from that. This
   step is manual by design and needs no automation.

   If the content is present, drop the commits and remove class D. Otherwise
   cherry-pick them onto the new tag and re-record their patch-ids.
3. Create `patched-<newtag>` from the new tag plus whatever step 2 decided, and
   branch `materials-<newtag>` from it — step 4's rebase target, which no
   earlier step otherwise creates. Record in the baseline file: the new `tag`,
   **the tree of the release commit** (not of
   `patched-<newtag>`, which differs whenever anything is still carried), the
   new `patched_commit`, and the patch-ids of whatever `carried` now holds.
4. In a fresh `.worktrees/` worktree, rebase the material work with an
   **explicit old boundary**:

       git rebase --onto patched-<newtag> patched-<oldtag> materials-<newtag>

   Without `--onto`, the rewritten ancestry reappears as a rebase that replays
   far more than the 280 material commits.

   The branch contains 7 merge commits. **Flatten them, but audit them first.**
   Flattening is the policy because the merge topology carries no upstream
   meaning and `--rebase-merges` preserves structure at a real
   conflict-resolution cost. What flattening must not do is discard the
   *resolutions* those merges recorded, and this fork has substantive ones:

       git show --remerge-diff 855ac7af

   shows hand resolutions across `src/layout/tile.rs` (91 lines),
   `src/render_helpers/material.rs`, and
   `src/render_helpers/shaders/material.frag`, including the combination of
   signal emission with noise and saturation processing. `f8bcb34c` carries
   further resolutions in `niri-config/`.

   So: before rebasing, run `git show --remerge-diff` over each merge in
   `git log --merges patched-<oldtag>..materials-<oldtag>`, record every
   non-empty result, and verify after the rebase that each resolution is
   reproduced in the flattened history. Having no upstream significance does
   not make a resolution disposable — it is the fork's own work.
5. **Regenerate and stage the baseline record and the report**, then `just gate`.
   This order is mandatory now that the gate enforces freshness: running the
   gate against a stale baseline fails on the report rather than on the rebase,
   which is a confusing way to learn nothing.
6. **Nested GLES smoke** on the headless host, using the existing harnesses
   under `docs/materials/scripts/`.
7. **Physical DRM acceptance**, separately, on real hardware with a human
   watching scanout. This is not interchangeable with step 6:
   `2026-08-27-v1-drm-acceptance-design.md` states that headless and nested
   hosts "cannot close the explicit real-hardware gate." A rebase can silently
   reorder render passes, so the render evidence is re-earned, not inherited.
8. Finish the document: prune acknowledgments that no longer apply, re-review
   the feature table's posture column, and add one rebase-log line recording
   the tag, the conflicts resolved, the merge resolutions carried across, and
   the evidence runs.
9. Re-pin packaging: push first, then `just package-pin <commit>`, which
   derives `pkgver`, the `source=` commit and `NIRI_BUILD_COMMIT` from the
   commit being packaged.
10. Keep the old branches as archives. Then **change the repository's default
    branch on GitHub** — `gh repo edit --default-branch materials-<newtag>` —
    because the weekly workflow only schedules from the file on the default
    branch, so a rebase that leaves the default behind silently stops the
    drift check. Updating the local `origin/HEAD` symbolic ref is a separate,
    purely local convenience and does not change anything on the server.

## Scheduled workflow contract

`.github/workflows/upstream-drift.yml`, a new file. `ci.yml` is a seam already
and does not grow.

- **Schedule:** weekly. Scheduled workflows only run from the file present on
  the default branch, so this lands on `materials-26.04` to run at all.
- **Checkout:** `fetch-depth: 0` with tags, then an explicit
  `git remote add upstream` / `git fetch upstream --tags`. A fetch failure
  **fails the job**; it must never be reported as a clean drift result.
- **Refs:** baseline validation resolves the pinned SHAs from
  `upstream-baseline.toml`, never branch names. `actions/checkout` fetches
  history but creates no local `patched-26.04`, so a name-based lookup would
  fail in CI even at `fetch-depth: 0`. Full history is fetched precisely so
  those pinned commits are present.
- **Permissions:** `contents: read`, `issues: write`.
- **Reporting:** one deduplicated issue, found by a fixed marker string in the
  body and updated in place rather than reopened per run. It fails the job on
  any conflicting path that `upstream-conflicts.toml` does not acknowledge, and
  on any unattributed conflict.
- **Toolchain:** python3 and git only. The job must **not** run `just check`.
  Two of `check_cmd`'s four commands need the Rust toolchain and apt
  dependencies, and `tasks check` needs the `tasks` CLI installed — which would
  contradict the reason this design keeps drift reporting on GitHub issues
  rather than in `tasks/`.
- **On pull requests** it runs only the genuinely missing lightweight checks:
  `python3 -m unittest discover -s tools` and `upstream-report --check`.
  Format and clippy are already covered by `ci.yml`'s `rustfmt` and `clippy`
  jobs and are not duplicated.

**`tasks check` stays local-only, and this is a deliberate gap.** Provisioning
the `tasks` CLI in CI to validate task metadata is not worth the coupling for
v1. The residual risk is narrow and worth naming: a push from a machine without
`core.hooksPath` set can land task metadata that has drifted from its plan or
spec, and nothing in CI will say so. It is caught at the next local `just
check`. Revisit if that ever actually happens.

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
- **Baseline record.** The central fixture reproduces this fork's shape: the
  commit at `<patched_commit>~<n>` is a **different object from the tag's
  commit with an identical tree**. Assert validation *passes*. Then assert it fails
  when that commit's tree differs, proving the check tests content and not
  identity.

  Assert each other validation fails independently: a `tree` that is the
  patched tree rather than the release tree; a `tag` whose tree is not `tree`;
  a `tag` moved to another commit with the same tree still validates (the CI
  clone's case); a `carried` list of
  the right length but wrong patch-ids. Assert resolution succeeds in a clone
  with **no local branches at all**, proving nothing depends on a branch name.
- **Index freshness.** Stage a source change without regenerating; assert
  `--check` fails. Regenerate and stage; assert it passes. Assert a change
  present in the working tree but unstaged does not affect the verdict, and
  that a fresh checkout with an empty staging area validates the checked-out
  commit rather than reporting a vacuous pass.
- **Conflict and error handling.** A fixture with a known conflict asserts exit
  1 and the exact path set. A deliberately bad ref asserts the exit-status-`>1`
  path fails loudly.

  A **directory-rename** fixture — verified while writing the test to actually
  produce exit 1 with an empty conflicted-file list on the pinned git version,
  not assumed to — asserts the unattributed-conflict path: the run fails, and
  the output distinguishes it from a tool error. A second case pairs that
  unattributed conflict with an ordinary conflict on an **acknowledged** path,
  asserting the run still fails: a non-empty, fully acknowledged path list must
  not be able to hide it.
- **Self-reference.** Assert the report excludes itself, the baseline record,
  and the acknowledgment file from its inventory, and that running it twice is
  a fixpoint.
- **Acknowledgments.** Assert an unacknowledged path fails and an acknowledged
  one passes. Assert an unattributed conflict fails even when every named path
  is acknowledged.
- **Configuration comes from the index.** Edit `upstream-baseline.toml` in the
  working tree without staging it; assert `--check` ignores the edit and
  reports on the staged record. Stage it; assert the verdict changes. This is
  the test that would have caught reading configuration from the working tree
  while reading sources from the index.

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
