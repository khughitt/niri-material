# Native materials repository migration plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use
> superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task. Steps use
> checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create `niri-material` as the sole production niri fork and
`niri-experiments` as its evidence companion without modifying the source fork
or legacy prototype implementation; the prototype's planning records receive
only their final status commit after acceptance.

**Architecture:** Clone the reviewed niri history into a clean production
repository with only the two production branches and the canonical upstream
remote. Initialize the evidence repository independently, migrate the approved
documents according to the mapping table, and accept the split only after
cross-repository, source-preservation, build, test, and fixture gates pass.

**Tech Stack:** Git, zsh, ripgrep, ImageMagick, Cargo.

**Spec:** `docs/materials/2026-08-22-repository-split.md`

**Status:** completed 2026-08-22. Execution evidence is the two target commit
identities recorded by the closing task; checkboxes remain the immutable
procedure rather than a second execution log.

## Global constraints

- The dispatcher supplies six absolute paths: `$PROJECTS_ROOT` (parent of the
  legacy prototype repository), `$NIRI_GLASS` (legacy prototype repository),
  `$NIRI_SOURCE` (existing source fork), `$MIGRATION_SOURCE` (this approved
  docs worktree), `$NIRI_MATERIAL` (`$PROJECTS_ROOT/niri-material`), and
  `$NIRI_EXPERIMENTS` (`$PROJECTS_ROOT/niri-experiments`). They are exported
  for every task; steps do not rely on shell-local state from an earlier step.
- Targets are exactly `$PROJECTS_ROOT/niri-material` and
  `$PROJECTS_ROOT/niri-experiments`. Fail if either path exists.
- Source refs are exact: `patched-26.04` = `5e53b949`, `materials-26.04` =
  `3792432a`, `pr-4147` = `d26ab5f2`, and
  `ipc-animation-timing-spike` = `cba1119b`.
- `niri-material` is the only future writer for `patched-26.04` and
  `materials-26.04`; do not create a synchronization mechanism.
- Do not modify, move, delete, commit in, or reconfigure `$NIRI_SOURCE` or its
  worktrees. Tasks 1–4 also leave `$NIRI_GLASS` and `$MIGRATION_SOURCE`
  untouched; Task 5 may update only the approved design and migration-plan
  status lines in `$MIGRATION_SOURCE` after acceptance.
- Do not create a hosted repository or push. `niri-material` receives only the
  canonical `upstream` URL; `niri-experiments` has no remote.
- The production migration changes only `docs/materials/`; upstream's root
  `README.md` and every production source file remain unchanged.
- Do not copy PNGs, logs, CSV files, Tracy captures, or other raw generated
  artifacts. Copy the diagnostic-grid SVG and create the reviewed checker SVG.
- Cross-repository document references name both repository and full path.
  Machine-local paths never enter committed files.
- Use conventional commits and no attribution trailers.
- This migration is fail-closed: stop at the first failed assertion. Do not
  repair an unexpected target or source state in place.

---

### Task 1: Create the production fork

**Files:**
- Create: `$PROJECTS_ROOT/niri-material/` (full niri clone)

**Interfaces:**
- Consumes: dispatcher paths and the four reviewed source refs.
- Produces: a clean `niri-material` checkout on `materials-26.04`, with
  `patched-26.04` preserved and only the canonical `upstream` remote.

- [ ] **Step 1: Validate the dispatcher paths and source state**

Run from `$MIGRATION_SOURCE`:

```bash
set -euo pipefail
: "${PROJECTS_ROOT:?dispatcher must set PROJECTS_ROOT}"
: "${NIRI_GLASS:?dispatcher must set NIRI_GLASS}"
: "${NIRI_SOURCE:?dispatcher must set NIRI_SOURCE}"
: "${MIGRATION_SOURCE:?dispatcher must set MIGRATION_SOURCE}"
: "${NIRI_MATERIAL:?dispatcher must set NIRI_MATERIAL}"
: "${NIRI_EXPERIMENTS:?dispatcher must set NIRI_EXPERIMENTS}"

test "$(realpath "$(dirname "$NIRI_GLASS")")" = "$(realpath "$PROJECTS_ROOT")"
test "$NIRI_MATERIAL" = "$PROJECTS_ROOT/niri-material"
test "$NIRI_EXPERIMENTS" = "$PROJECTS_ROOT/niri-experiments"
test "$(git -C "$MIGRATION_SOURCE" rev-parse --is-inside-work-tree)" = true
git -C "$MIGRATION_SOURCE" merge-base --is-ancestor dadc93a HEAD
git -C "$MIGRATION_SOURCE" diff --quiet
git -C "$MIGRATION_SOURCE" diff --cached --quiet

test ! -e "$NIRI_MATERIAL"
test ! -e "$NIRI_EXPERIMENTS"
test "$(git -C "$NIRI_SOURCE" rev-parse --short=8 patched-26.04)" = 5e53b949
test "$(git -C "$NIRI_SOURCE" rev-parse --short=8 materials-26.04)" = 3792432a
test "$(git -C "$NIRI_SOURCE" rev-parse --short=8 pr-4147)" = d26ab5f2
test "$(git -C "$NIRI_SOURCE" rev-parse --short=8 ipc-animation-timing-spike)" = cba1119b
test "$(git -C "$NIRI_SOURCE" rev-list --count 8ed0da44..patched-26.04)" = 2
test "$(git -C "$NIRI_SOURCE" rev-list --count patched-26.04..materials-26.04)" = 5
git -C "$NIRI_SOURCE" diff --quiet
git -C "$NIRI_SOURCE" diff --cached --quiet
git -C "$NIRI_SOURCE/.worktrees/materials-26.04" diff --quiet
git -C "$NIRI_SOURCE/.worktrees/materials-26.04" diff --cached --quiet
git -C "$NIRI_SOURCE" worktree list --porcelain
```

Expected: all assertions pass. The listing records two live production
worktrees (`patched-26.04` and `materials-26.04`) and currently also reports a
prunable timing-spike registration whose directory no longer exists. That
prunable registration is context, not an acceptance invariant. Untracked
evidence in the materials worktree is allowed; tracked changes are not.

- [ ] **Step 2: Clone without shared object hardlinks and materialize the two production branches**

```bash
git clone --no-hardlinks "$NIRI_SOURCE" "$NIRI_MATERIAL"
git -C "$NIRI_MATERIAL" branch materials-26.04 3792432a
git -C "$NIRI_MATERIAL" switch materials-26.04
git -C "$NIRI_MATERIAL" remote remove origin
git -C "$NIRI_MATERIAL" remote add upstream https://github.com/niri-wm/niri
```

The local clone starts with `patched-26.04` because that is the source
checkout's current branch. Removing `origin` also removes source-fork tracking
refs; do not preserve `pr-4147` or the timing-spike branch in the production
fork.

- [ ] **Step 3: Verify the fork identity and boundary**

```bash
test "$(git -C "$NIRI_MATERIAL" rev-parse --short=8 patched-26.04)" = 5e53b949
test "$(git -C "$NIRI_MATERIAL" rev-parse --short=8 HEAD)" = 3792432a
test "$(git -C "$NIRI_MATERIAL" branch --show-current)" = materials-26.04
test "$(git -C "$NIRI_MATERIAL" rev-list --count patched-26.04..HEAD)" = 5
test "$(git -C "$NIRI_MATERIAL" branch --format='%(refname:short)' | sort)" = $'materials-26.04\npatched-26.04'
test "$(git -C "$NIRI_MATERIAL" remote)" = upstream
test "$(git -C "$NIRI_MATERIAL" remote get-url upstream)" = https://github.com/niri-wm/niri
test "$(git -C "$NIRI_MATERIAL" rev-parse 'HEAD^{tree}')" = "$(git -C "$NIRI_SOURCE" rev-parse '3792432a^{tree}')"
test -z "$(git -C "$NIRI_MATERIAL" status --short)"
```

Expected: exact reviewed history, two local branches, one canonical remote,
and a clean tree. There is no commit in this task; the existing niri history is
the deliverable.

---

### Task 2: Create the experiments repository

**Files:**
- Create: `$NIRI_EXPERIMENTS/README.md`
- Create: `$NIRI_EXPERIMENTS/docs/research/2026-08-22-gap-analysis.md`
- Create: `$NIRI_EXPERIMENTS/docs/research/legacy-visual-verification.md`
- Create: `$NIRI_EXPERIMENTS/docs/results/2026-08-22-slice0.md`
- Create: `$NIRI_EXPERIMENTS/fixtures/diagnostic-grid.svg`
- Create: `$NIRI_EXPERIMENTS/fixtures/opacity-checker.svg`

**Interfaces:**
- Consumes: reviewed documents and fixtures from `$MIGRATION_SOURCE`, plus
  niri commits already present in `$NIRI_MATERIAL`.
- Produces: the committed evidence companion on `main`, with no remote or raw
  generated artifacts.

- [ ] **Step 1: Initialize the repository and copy the reviewed sources**

```bash
git init -b main "$NIRI_EXPERIMENTS"
mkdir -p "$NIRI_EXPERIMENTS/docs/research" "$NIRI_EXPERIMENTS/docs/results" "$NIRI_EXPERIMENTS/fixtures"
cp "$MIGRATION_SOURCE/docs/superpowers/2026-08-22-native-materials-gap-analysis.md" \
  "$NIRI_EXPERIMENTS/docs/research/2026-08-22-gap-analysis.md"
cp "$MIGRATION_SOURCE/docs/ref/visual-verification.md" \
  "$NIRI_EXPERIMENTS/docs/research/legacy-visual-verification.md"
cp "$MIGRATION_SOURCE/docs/superpowers/2026-08-22-slice0-results.md" \
  "$NIRI_EXPERIMENTS/docs/results/2026-08-22-slice0.md"
cp "$MIGRATION_SOURCE/assets/diagnostic-grid.svg" \
  "$NIRI_EXPERIMENTS/fixtures/diagnostic-grid.svg"
```

- [ ] **Step 2: Add the repository index**

Create `$NIRI_EXPERIMENTS/README.md` with exactly:

```markdown
# niri experiments

Research, experiment results, and source fixtures supporting the native
material system in the sibling `niri-material` fork.

## Contents

- `docs/research/`: gap analyses and legacy methods of record.
- `docs/results/`: reviewed experiment outcomes and regression baselines.
- `fixtures/`: small source fixtures; generated rasters remain untracked.

Production compositor code, stable specifications, and implementation plans
live in `niri-material`. The legacy `niri-glass` repository remains the source
for explicitly qualified pre-split evidence and the frozen Quickshell client.
```

- [ ] **Step 3: Add the checker source fixture**

Create `$NIRI_EXPERIMENTS/fixtures/opacity-checker.svg` with exactly:

```svg
<svg xmlns="http://www.w3.org/2000/svg" width="1920" height="1080" viewBox="0 0 1920 1080">
  <defs>
    <pattern id="checker" width="240" height="240" patternUnits="userSpaceOnUse">
      <rect width="240" height="240" fill="#000"/>
      <rect x="120" width="120" height="120" fill="#fff"/>
      <rect y="120" width="120" height="120" fill="#fff"/>
    </pattern>
  </defs>
  <rect width="1920" height="1080" fill="url(#checker)"/>
</svg>
```

- [ ] **Step 4: Apply only the approved document rewrites**

In `docs/research/2026-08-22-gap-analysis.md`:

- replace `docs/superpowers/2026-08-22-slice0-results.md` with
  `niri-experiments/docs/results/2026-08-22-slice0.md`;
- replace “niri-glass today is” with “The legacy niri-glass client is”;
- replace the prose-only IPC-spike citation with
  `niri-glass/docs/2026-08-09-niri-ipc-animation-timing-spike-results.md`
  while preserving its dates and measurements;
- replace the fork-lane name `niri-glass/native` with `materials-26.04` in
  `niri-material` and its future versioned successors.

In `docs/research/legacy-visual-verification.md`, replace both occurrences of
`docs/ref/qt-quick3d-custommaterial.md` with
`niri-glass/docs/ref/qt-quick3d-custommaterial.md`. Make no other technical
changes; the document remains explicitly Quickshell-scoped.

Do not alter the measurements, caveats, hashes, warning text, or status in
`docs/results/2026-08-22-slice0.md`.

- [ ] **Step 5: Verify the evidence repository and checker fidelity**

```bash
set -euo pipefail
expected_files=$'README.md\ndocs/research/2026-08-22-gap-analysis.md\ndocs/research/legacy-visual-verification.md\ndocs/results/2026-08-22-slice0.md\nfixtures/diagnostic-grid.svg\nfixtures/opacity-checker.svg'
actual_files="$(find "$NIRI_EXPERIMENTS" -path "$NIRI_EXPERIMENTS/.git" -prune -o -type f -printf '%P\n' | sort)"
test "$actual_files" = "$expected_files"
test -z "$(git -C "$NIRI_EXPERIMENTS" remote)"
git -C "$NIRI_MATERIAL" cat-file -e '5e53b949^{commit}'
git -C "$NIRI_MATERIAL" cat-file -e '3792432a^{commit}'

test -z "$(rg -l -F -e "$HOME" -e "$PROJECTS_ROOT" -e '$NIRI_GLASS' "$NIRI_EXPERIMENTS" || true)"
test -z "$(rg -l 'docs/superpowers|T[B]D|T[O]DO' "$NIRI_EXPERIMENTS" || true)"
rg -q 'Quickshell instance' "$NIRI_EXPERIMENTS/docs/research/legacy-visual-verification.md"
rg -q 'niri-glass/docs/ref/qt-quick3d-custommaterial.md' "$NIRI_EXPERIMENTS/docs/research/legacy-visual-verification.md"
rg -q 'niri-glass/docs/2026-08-09-niri-ipc-animation-timing-spike-results.md' "$NIRI_EXPERIMENTS/docs/research/2026-08-22-gap-analysis.md"

checker_tmp="$(mktemp -d)"
magick "$NIRI_EXPERIMENTS/fixtures/opacity-checker.svg" -depth 8 gray:"$checker_tmp/source.gray"
magick "$NIRI_SOURCE/.worktrees/materials-26.04/slice0-checker.png" -depth 8 gray:"$checker_tmp/accepted.gray"
cmp "$checker_tmp/source.gray" "$checker_tmp/accepted.gray"
```

Expected: exact initial file set, no remote, both niri commit references
resolve, all repository references are qualified, the legacy method remains
legacy, and the checker pixels are identical to the retained accepted PNG.
The temporary comparison directory is left to normal system cleanup.

- [ ] **Step 6: Commit the experiments repository**

```bash
git -C "$NIRI_EXPERIMENTS" add README.md docs fixtures
git -C "$NIRI_EXPERIMENTS" commit -m "docs: initialize niri experiments"
test -z "$(git -C "$NIRI_EXPERIMENTS" status --short)"
```

---

### Task 3: Migrate production documentation and verify niri

**Files:**
- Create: `$NIRI_MATERIAL/docs/materials/README.md`
- Create: `$NIRI_MATERIAL/docs/materials/2026-08-22-repository-split.md`
- Create: `$NIRI_MATERIAL/docs/materials/2026-08-22-v1-design.md`
- Create: `$NIRI_MATERIAL/docs/materials/plans/2026-08-22-repository-migration.md`
- Create: `$NIRI_MATERIAL/docs/materials/plans/2026-08-22-slice0.md`

**Interfaces:**
- Consumes: the approved migration source and committed experiments
  repository.
- Produces: the documentation-only migration commit on `materials-26.04`,
  with the split design marked implemented only after all acceptance gates
  pass.

- [ ] **Step 1: Copy the approved production documents**

```bash
mkdir -p "$NIRI_MATERIAL/docs/materials/plans"
cp "$MIGRATION_SOURCE/docs/superpowers/specs/2026-08-22-native-materials-repository-split-design.md" \
  "$NIRI_MATERIAL/docs/materials/2026-08-22-repository-split.md"
cp "$MIGRATION_SOURCE/docs/superpowers/specs/2026-08-22-native-materials-v1-design.md" \
  "$NIRI_MATERIAL/docs/materials/2026-08-22-v1-design.md"
cp "$MIGRATION_SOURCE/docs/superpowers/plans/2026-08-22-native-materials-repository-migration.md" \
  "$NIRI_MATERIAL/docs/materials/plans/2026-08-22-repository-migration.md"
cp "$MIGRATION_SOURCE/docs/superpowers/plans/2026-08-22-native-materials-slice0.md" \
  "$NIRI_MATERIAL/docs/materials/plans/2026-08-22-slice0.md"
```

- [ ] **Step 2: Add the material-project index**

Create `$NIRI_MATERIAL/docs/materials/README.md` with exactly:

```markdown
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
```

- [ ] **Step 3: Rewrite the v1 design's repository boundary and references**

Make these exact semantic changes in
`docs/materials/2026-08-22-v1-design.md`:

Set the header references to:

```markdown
**Status:** approved 2026-08-22 after three review cycles; slice 0 implemented
and verified passed-with-caveats in fork `materials-26.04` at `3792432a`
(`niri-experiments/docs/results/2026-08-22-slice0.md`).
**Companion:** `niri-experiments/docs/research/2026-08-22-gap-analysis.md`
```

Replace §1 with:

```markdown
## 1. Fork and repository layout

- Branch `materials-26.04` is stacked on `patched-26.04` in `niri-material`.
  The two #4147 IPC commits stay because they serve external IPC clients and
  the open PR; native materials do not need them.
- Production compositor and shader sources live in `niri-material`, beside
  their compiler and tests (`src/render_helpers/shaders/`). Stable material
  specifications and implementation plans live in `docs/materials/`.
- Research, results, and source fixtures live in the sibling
  `niri-experiments` repository.
- The legacy `niri-glass` repository retains the frozen Quickshell reference
  implementation and its parameter sources. It is not a source of truth for
  production shaders, specifications, plans, or experiment results.
- Rebase budget per gap analysis §4: about three integration cycles per year
  plus one potentially heavy renderer-refactor rebase (#3833/#3834).
```

Replace the §5 acceptance sentence with:

```markdown
**Then:** parity pass against the frozen Quickshell client in the legacy
`niri-glass` repository, physical DRM smoke, v1 acceptance.
```

Replace the nested-visual-verification bullet in §6 with:

```markdown
- The completed slice-0 nested procedure remains in
  `docs/materials/plans/2026-08-22-slice0.md`. The Quickshell-specific method
  at `niri-experiments/docs/research/legacy-visual-verification.md` is research
  input, not a native compositor procedure. A reusable native method is
  deferred until slice 1 needs it.
```

Preserve every renderer, config, parameter, slice, and risk decision.

- [ ] **Step 4: Rewrite only repository paths in the historical slice-0 plan**

In `docs/materials/plans/2026-08-22-slice0.md`:

- point `**Spec:**` to `docs/materials/2026-08-22-v1-design.md`;
- replace the first two repository bullets under Global Constraints with:

  ```markdown
  - Code tasks originally ran in the source fork at
    `niri-patched/.worktrees/materials-26.04`, branch `materials-26.04`
    stacked on `patched-26.04`. That path is execution provenance;
    `niri-material` is now the sole production owner of both branches.
  - Reproduction supplies the `niri-material` checkout path and the companion
    evidence path as `$NIRI_EXPERIMENTS`. Results live at
    `niri-experiments/docs/results/2026-08-22-slice0.md`.
  ```

- replace the diagnostic-grid variable and path with
  `$NIRI_EXPERIMENTS/fixtures/diagnostic-grid.svg`;
- replace the result destination with
  `niri-experiments/docs/results/2026-08-22-slice0.md` and identify its commit
  as belonging to `niri-experiments`;
- remove remaining `$NIRI_GLASS`, unqualified `docs/superpowers`, and
  machine-local paths.

Except for the repository-path substitutions required by the bullets above,
do not alter other code snippets, test commands, measurements, acceptance
semantics, or the original worktree commands; those are historical execution
provenance.

- [ ] **Step 5: Update migrated planning paths**

In `docs/materials/plans/2026-08-22-repository-migration.md`, change `**Spec:**`
to `docs/materials/2026-08-22-repository-split.md`. Keep its source-copy paths
as explicitly qualified migration provenance.

In `docs/materials/2026-08-22-repository-split.md`, keep the status approved
but point its migration-plan path to
`docs/materials/plans/2026-08-22-repository-migration.md`. Replace the
self-referential sentence naming old `docs/superpowers` paths with “Pre-split
document and worktree paths are replaced with the destination layouts above.”

- [ ] **Step 6: Run documentation and production-source scope gates**

```bash
set -euo pipefail
test -f "$NIRI_EXPERIMENTS/docs/results/2026-08-22-slice0.md"
test -f "$NIRI_EXPERIMENTS/docs/research/2026-08-22-gap-analysis.md"
test -f "$NIRI_EXPERIMENTS/docs/research/legacy-visual-verification.md"

git -C "$NIRI_MATERIAL" diff --exit-code 3792432a -- . ':(exclude)docs/materials'
git -C "$NIRI_MATERIAL" diff --exit-code 3792432a -- README.md

test -z "$(rg -l -F -e "$HOME" -e "$PROJECTS_ROOT" -e '$NIRI_GLASS' "$NIRI_MATERIAL/docs/materials" \
  --glob '!2026-08-22-repository-migration.md' || true)"
test -z "$(rg -l 'docs/superpowers' "$NIRI_MATERIAL/docs/materials" \
  --glob '!2026-08-22-repository-migration.md' || true)"
rg -q 'niri-experiments/docs/results/2026-08-22-slice0.md' "$NIRI_MATERIAL/docs/materials/2026-08-22-v1-design.md"
rg -q 'niri-experiments/docs/research/legacy-visual-verification.md' "$NIRI_MATERIAL/docs/materials/2026-08-22-v1-design.md"
rg -q 'niri-patched/.worktrees/materials-26.04' "$NIRI_MATERIAL/docs/materials/plans/2026-08-22-slice0.md"
```

Expected: every changed file is under `docs/materials/`, root `README.md` and
production source match `3792432a`, and all live cross-repository references
are qualified. The migration plan is the sole exception for source-tree
`docs/superpowers` paths and `$NIRI_GLASS` because it records the copy
procedure and dispatcher contract itself.

- [ ] **Step 7: Run build, tests, and clippy**

```bash
set -euo pipefail
cargo build --manifest-path "$NIRI_MATERIAL/Cargo.toml"
test_output="$(cargo test --workspace --no-fail-fast --manifest-path "$NIRI_MATERIAL/Cargo.toml" 2>&1)"
printf '%s\n' "$test_output"
test "$(grep -c 'test result: ok. 198 passed; 0 failed' <<<"$test_output")" = 1
test "$(grep -c 'test result: ok. 18 passed; 0 failed' <<<"$test_output")" = 1
test "$(grep -c 'test result: ok. 3 passed; 0 failed' <<<"$test_output")" = 1
test "$(grep -c 'test result: ok. 1 passed; 0 failed' <<<"$test_output")" = 2

clippy_output="$(cargo clippy --workspace --manifest-path "$NIRI_MATERIAL/Cargo.toml" 2>&1)"
printf '%s\n' "$clippy_output"
test "$(grep -c '^warning: redundant reference in `println!` argument' <<<"$clippy_output")" = 1
test "$(grep -c "^warning: you seem to want to iterate on a map's values" <<<"$clippy_output")" = 1
test "$(grep -c "^warning: you seem to want to iterate on a map's keys" <<<"$clippy_output")" = 1
test "$(grep '^warning:' <<<"$clippy_output" | grep -vcE 'generated [0-9]+ warnings?')" = 3
```

Expected: build succeeds; the complete 221-test workspace suite passes (niri
198, niri-config 18 unit + 1 integration, niri-ipc 3 unit + 1 doc-test); and
clippy reports exactly the three warnings recorded in
`niri-experiments/docs/results/2026-08-22-slice0.md`.

- [ ] **Step 8: Mark the migrated split design implemented**

Only after Steps 6 and 7 pass, set this status in
`docs/materials/2026-08-22-repository-split.md`:

```markdown
**Status:** implemented 2026-08-22; migration plan:
`docs/materials/plans/2026-08-22-repository-migration.md`
```

Run `git -C "$NIRI_MATERIAL" diff --check` after the status edit.

- [ ] **Step 9: Commit the production documentation**

```bash
git -C "$NIRI_MATERIAL" add docs/materials
git -C "$NIRI_MATERIAL" commit -m "docs: establish native materials project"
test -z "$(git -C "$NIRI_MATERIAL" status --short)"
```

---

### Task 4: Run cross-repository acceptance

**Files:**
- Modify: none

**Interfaces:**
- Consumes: both committed target repositories and the untouched sources.
- Produces: an evidence-backed migration verdict and the two target commit
  identities.

- [ ] **Step 1: Verify source-fork refs, worktrees, and tracked state**

```bash
set -euo pipefail
test "$(git -C "$NIRI_SOURCE" rev-parse --short=8 patched-26.04)" = 5e53b949
test "$(git -C "$NIRI_SOURCE" rev-parse --short=8 materials-26.04)" = 3792432a
test "$(git -C "$NIRI_SOURCE" rev-parse --short=8 pr-4147)" = d26ab5f2
test "$(git -C "$NIRI_SOURCE" rev-parse --short=8 ipc-animation-timing-spike)" = cba1119b
test "$(git -C "$NIRI_SOURCE" branch --show-current)" = patched-26.04
test "$(git -C "$NIRI_SOURCE/.worktrees/materials-26.04" branch --show-current)" = materials-26.04
git -C "$NIRI_SOURCE" worktree list --porcelain
git -C "$NIRI_SOURCE" diff --quiet
git -C "$NIRI_SOURCE" diff --cached --quiet
git -C "$NIRI_SOURCE/.worktrees/materials-26.04" diff --quiet
git -C "$NIRI_SOURCE/.worktrees/materials-26.04" diff --cached --quiet
```

- [ ] **Step 2: Verify the production repository**

```bash
test "$(git -C "$NIRI_MATERIAL" rev-parse --short=8 patched-26.04)" = 5e53b949
git -C "$NIRI_MATERIAL" merge-base --is-ancestor 3792432a materials-26.04
test "$(git -C "$NIRI_MATERIAL" branch --format='%(refname:short)' | sort)" = $'materials-26.04\npatched-26.04'
test "$(git -C "$NIRI_MATERIAL" remote)" = upstream
test "$(git -C "$NIRI_MATERIAL" remote get-url upstream)" = https://github.com/niri-wm/niri
test -z "$(git -C "$NIRI_MATERIAL" diff --name-only 3792432a..materials-26.04 | grep -v '^docs/materials/' || true)"
git -C "$NIRI_MATERIAL" diff --exit-code 3792432a..materials-26.04 -- . ':(exclude)docs/materials'
git -C "$NIRI_MATERIAL" cat-file -e '5e53b949^{commit}'
git -C "$NIRI_MATERIAL" cat-file -e '3792432a^{commit}'
test -z "$(git -C "$NIRI_MATERIAL" status --short)"
```

- [ ] **Step 3: Verify the experiments repository and cross-references**

```bash
expected_files=$'README.md\ndocs/research/2026-08-22-gap-analysis.md\ndocs/research/legacy-visual-verification.md\ndocs/results/2026-08-22-slice0.md\nfixtures/diagnostic-grid.svg\nfixtures/opacity-checker.svg'
actual_files="$(find "$NIRI_EXPERIMENTS" -path "$NIRI_EXPERIMENTS/.git" -prune -o -type f -printf '%P\n' | sort)"
test "$actual_files" = "$expected_files"
test "$(git -C "$NIRI_EXPERIMENTS" branch --show-current)" = main
test -z "$(git -C "$NIRI_EXPERIMENTS" remote)"
test -z "$(git -C "$NIRI_EXPERIMENTS" status --short)"
test -z "$(find "$NIRI_EXPERIMENTS" -type f \( -name '*.png' -o -name '*.log' -o -name '*.csv' -o -name '*.tracy' \) -print -quit)"
test -z "$(rg -l -F -e "$HOME" -e "$PROJECTS_ROOT" -e '$NIRI_GLASS' "$NIRI_EXPERIMENTS" || true)"
test -z "$(rg -l 'docs/superpowers|T[B]D|T[O]DO' "$NIRI_EXPERIMENTS" || true)"
rg -q 'niri-material' "$NIRI_EXPERIMENTS/README.md"
rg -q 'niri-glass/docs/2026-08-09-niri-ipc-animation-timing-spike-results.md' "$NIRI_EXPERIMENTS/docs/research/2026-08-22-gap-analysis.md"
rg -q 'niri-patched/.worktrees/materials-26.04/slice0-checker.png' "$NIRI_EXPERIMENTS/docs/results/2026-08-22-slice0.md"
```

- [ ] **Step 4: Record the accepted target state**

Run:

```bash
git -C "$NIRI_MATERIAL" log -1 --oneline
git -C "$NIRI_EXPERIMENTS" log -1 --oneline
git -C "$NIRI_SOURCE" worktree list
```

Record both target commit IDs and the build/test/clippy result for Task 5. Do
not prune the archival materials worktree; its raw evidence still awaits a
separate retention decision.

---

### Task 5: Close the source and migrated planning records

**Files:**
- Modify: `$NIRI_MATERIAL/docs/materials/plans/2026-08-22-repository-migration.md`
- Modify: `$MIGRATION_SOURCE/docs/superpowers/specs/2026-08-22-native-materials-repository-split-design.md`
- Modify: `$MIGRATION_SOURCE/docs/superpowers/plans/2026-08-22-native-materials-repository-migration.md`

**Interfaces:**
- Consumes: the accepted state and commit identities from Task 4.
- Produces: truthful completion status in both plan copies and both split-design
  copies, committed separately in the production and prototype repositories.

- [ ] **Step 1: Update only status lines**

In both migration-plan copies, replace only the status paragraph before the
first `---`:

```markdown
**Status:** approved for execution 2026-08-22. During Tasks 1–4 the
dispatcher tracks progress externally so `$MIGRATION_SOURCE` stays clean; the
closing task records completion in both plan copies.
```

with:

```markdown
**Status:** completed 2026-08-22. Execution evidence is the two target commit
identities recorded by the closing task; checkboxes remain the immutable
procedure rather than a second execution log.
```

Leave both fenced status examples in this step byte-for-byte unchanged.

In the source split design, replace its approved status with:

```markdown
**Status:** implemented 2026-08-22; migration plan:
`docs/superpowers/plans/2026-08-22-native-materials-repository-migration.md`
```

The migrated split design was already marked implemented in Task 3. Change no
other content in these three files.

- [ ] **Step 2: Commit the migrated plan status**

```bash
git -C "$NIRI_MATERIAL" diff --check
test "$(git -C "$NIRI_MATERIAL" diff --name-only)" = docs/materials/plans/2026-08-22-repository-migration.md
git -C "$NIRI_MATERIAL" add docs/materials/plans/2026-08-22-repository-migration.md
git -C "$NIRI_MATERIAL" commit -m "docs: record repository migration completion"
```

- [ ] **Step 3: Commit the prototype status records**

```bash
git -C "$MIGRATION_SOURCE" diff --check
test "$(git -C "$MIGRATION_SOURCE" diff --name-only | sort)" = $'docs/superpowers/plans/2026-08-22-native-materials-repository-migration.md\ndocs/superpowers/specs/2026-08-22-native-materials-repository-split-design.md'
git -C "$MIGRATION_SOURCE" add \
  docs/superpowers/plans/2026-08-22-native-materials-repository-migration.md \
  docs/superpowers/specs/2026-08-22-native-materials-repository-split-design.md
git -C "$MIGRATION_SOURCE" commit -m "docs: record repository migration completion"
```

- [ ] **Step 4: Run the final clean-state and scope checks**

```bash
set -euo pipefail
test -z "$(git -C "$NIRI_MATERIAL" status --short)"
test -z "$(git -C "$NIRI_EXPERIMENTS" status --short)"
test -z "$(git -C "$MIGRATION_SOURCE" status --short)"
test -z "$(git -C "$NIRI_MATERIAL" diff --name-only 3792432a..materials-26.04 | grep -v '^docs/materials/' || true)"

test "$(git -C "$NIRI_SOURCE" rev-parse --short=8 patched-26.04)" = 5e53b949
test "$(git -C "$NIRI_SOURCE" rev-parse --short=8 materials-26.04)" = 3792432a
test "$(git -C "$NIRI_SOURCE" rev-parse --short=8 pr-4147)" = d26ab5f2
test "$(git -C "$NIRI_SOURCE" rev-parse --short=8 ipc-animation-timing-spike)" = cba1119b
test "$(git -C "$NIRI_SOURCE" branch --show-current)" = patched-26.04
test "$(git -C "$NIRI_SOURCE/.worktrees/materials-26.04" branch --show-current)" = materials-26.04

for status_file in \
  "$NIRI_MATERIAL/docs/materials/plans/2026-08-22-repository-migration.md" \
  "$MIGRATION_SOURCE/docs/superpowers/plans/2026-08-22-native-materials-repository-migration.md"
do
  status_head="$(sed -n '1,40p' "$status_file")"
  test "$(grep -c '^\*\*Status:\*\* completed 2026-08-22' <<<"$status_head" || true)" = 1
  test "$(grep -c '^\*\*Status:\*\* approved for execution 2026-08-22' <<<"$status_head" || true)" = 0
  test "$(grep -c '^\*\*Status:\*\* completed 2026-08-22' "$status_file" || true)" = 2
  test "$(grep -c '^\*\*Status:\*\* approved for execution 2026-08-22' "$status_file" || true)" = 1
done

split_head="$(sed -n '1,40p' "$MIGRATION_SOURCE/docs/superpowers/specs/2026-08-22-native-materials-repository-split-design.md")"
test "$(grep -c '^\*\*Status:\*\* implemented 2026-08-22' <<<"$split_head" || true)" = 1
test "$(grep -c '^\*\*Status:\*\* approved 2026-08-22' <<<"$split_head" || true)" = 0

git -C "$NIRI_MATERIAL" log -1 --oneline
git -C "$NIRI_EXPERIMENTS" log -1 --oneline
git -C "$MIGRATION_SOURCE" log -1 --oneline
git -C "$NIRI_SOURCE" worktree list
```

Report the three closing commit IDs, the 221-test workspace result, the three
known clippy warnings, and confirmation that the source fork's four branch
heads and two live production worktrees are unchanged. The prunable timing-
spike registration may be present or absent and is not part of the verdict.
