# Native Materials v1 Daily-Driver Rollout Implementation Plan

**Status:** implemented 2026-08-29. The accepted compositor is the daily-driver
installation (`26.04.r133.g52f74f10-1`, source `52f74f10`), and Prism now owns
the material definition. Burn-in passed 2026-08-30 after roughly 13 hours over
three sessions, two cold starts, clean scoped journal review, and an explicit
operator PASS. The unchecked steps below preserve the executed procedure and
are not current progress markers.

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Package the physically accepted native-material compositor, make it
the daily-driver binary for Kitty and Ghostty, retire the legacy glass runtime,
and record a reversible normal-use and cold-start burn-in.

**Architecture:** An Arch `PKGBUILD` installs the exact accepted source commit
under Arch's existing `niri` paths. A separate dotfiles worktree composes one
native material rule after Prism while retaining Prism's opacity and frozen
legacy evidence inputs. Build, package inspection, installation, session
restart, burn-in, and cleanup remain explicit checkpoints; no deployment
wrapper is added.

**Tech Stack:** Arch `makepkg`/pacman, Bash PKGBUILD functions, Rust/Cargo,
KDL, zsh, systemd user services, niri IPC, and Git worktrees.

**Spec:** `docs/materials/2026-08-28-v1-daily-driver-rollout-design.md`

## Global Constraints

- Package production source exactly
  `138697be4cbb779c80425fe2a366ceca3610f38e`, the source exercised by the
  accepted physical result at `niri-experiments`
  `c0caa944db2edc5dc4844e6720951d7f32652b76`.
- Do not change compositor, shader, or config-parser production code.
- Work on `v1-post-acceptance-planning` in the existing
  `.worktrees/v1-post-acceptance-planning` niri-material worktree. Create a
  separate dotfiles worktree; do not edit the dirty dotfiles main worktree.
- Preserve unrelated changes in both repositories. Fail before proceeding if
  any rollout-owned path is already modified.
- Use conventional commits with no attribution trailers. Stage named paths,
  never `git add .` or `git add -A`.
- Require `NIRI_MATERIAL_WORK_ROOT=/mnt/ssd3/niri-material`. There is no
  fallback to the repository, `/tmp`, or another disk.
- Keep package build/install separate. Do not run pacman, rename the existing
  compositor, restart the graphical session, or delete rollback artifacts
  without the operator's explicit approval at that checkpoint.
- The package archive, source/build tree, logs, staged binary, and rollback
  metadata stay below the required work root. Retain them through burn-in.
- The native rule matches exactly `kitty` and `com.mitchellh.ghostty`, sets
  `blur false`, `noise 0`, and `saturation 1`, and leaves Prism's
  focus-conditioned opacity intact.
- Six derived native taps at chromatic aberration `0.68` are an accepted
  divergence from legacy `samples: 3`. Observable stutter fails burn-in.
- The design remains “approved; not implemented” until package, config,
  transition, normal-use session, and cold start all pass. Correct its status
  in the final niri-material commit before merging.

## File Structure

| File | Responsibility | Task |
| --- | --- | --- |
| `packaging/arch/PKGBUILD` | Reproducible Arch package for the accepted source commit | 1 |
| dotfiles `niri/materials.kdl` | Native daily terminal material and exact app-ID rule | 2 |
| dotfiles `niri/config.kdl` | Include native materials after Prism and stop legacy autostart | 2 |
| host-local `$HOME/d/dotfiles/shell/local/titan.env.zsh` (ignored) | Required external build root; intentionally outside the dotfiles commit | 2 |
| dotfiles `tests/setup_and_health.zsh` | Static rollout/config contract and retained evidence links | 2 |
| `docs/materials/README.md` | Current accepted/package entry point and complete index | 3, 7 |
| `docs/materials/2026-08-22-v1-design.md` | Concise accepted v1 status | 3 |
| `docs/materials/2026-08-24-glass-config-surface-design.md` | Historicalize the pre-deployment statement | 3 |
| `docs/materials/plans/2026-08-23-slice2.md` | Record ancestry into `materials-26.04` | 3 |
| `docs/materials/plans/2026-08-24-slice3.md` | Record ancestry into `materials-26.04` | 3 |
| `docs/materials/2026-08-28-v1-daily-driver-rollout-design.md` | Final deployment status and evidence | 7 |
| `docs/materials/plans/2026-08-28-v1-daily-driver-rollout.md` | Executed procedure and checkpoints | 1–7 |

---

### Task 1: Add the pinned Arch package recipe

**Files:**
- Create: `packaging/arch/PKGBUILD`
- Modify: `docs/materials/plans/2026-08-28-v1-daily-driver-rollout.md`

**Interfaces:**
- Consumes: accepted source commit `138697be4cbb779c80425fe2a366ceca3610f38e`.
- Produces: package metadata `niri-material 26.04.r95.g138697be-1`, the
  standard `niri` payload paths, and the exact 293-test `check()` contract.

- [ ] **Step 1: Verify the source/version facts and clean package path**

```bash
cd "$HOME/d/niri-material/.worktrees/v1-post-acceptance-planning"
test -z "$(git status --short -- packaging)"
test "$(git describe --tags --long 138697be4cbb779c80425fe2a366ceca3610f38e)" = \
  v26.04-95-g138697be
test "$(git rev-list --count v26.04..138697be4cbb779c80425fe2a366ceca3610f38e)" = 95
git merge-base --is-ancestor \
  138697be4cbb779c80425fe2a366ceca3610f38e origin/materials-26.04
```

Expected: all checks exit zero and `packaging/` has no pre-existing change.

- [ ] **Step 2: Create the complete PKGBUILD**

Create `packaging/arch/PKGBUILD` with exactly:

```bash
pkgname=niri-material
pkgver=26.04.r95.g138697be
pkgrel=1
pkgdesc="A scrollable-tiling Wayland compositor with native materials"
arch=(x86_64)
url="https://github.com/khughitt/niri-material"
license=(GPL-3.0-or-later)
depends=(
  cairo
  glib2
  glibc
  libdisplay-info
  libgcc
  libinput
  libpipewire
  libxkbcommon
  mesa
  pango
  pixman
  seatd
  systemd-libs
  xdg-desktop-portal-impl
)
makedepends=(
  clang
  git
  gtk4
  libadwaita
  rust
)
optdepends=(
  'alacritty: a suggested GPU-accelerated terminal emulator'
  'bash: for niri-session script'
  'fuzzel: a suggested Wayland application launcher'
  'mako: a suggested Wayland notification daemon'
  'org.freedesktop.secrets: for apps to rely on secrets portal'
  'swaybg: a suggested Wayland wallpaper tool'
  'swaylock: a suggested Wayland screen locker'
  'waybar: a suggested Wayland customizable desktop bar'
  'xwayland-satellite: for running X11 apps in XWayland'
  'xdg-desktop-portal-gtk: a suggested XDG desktop portal'
  'xdg-desktop-portal-gnome: a XDG desktop portal required for screencasting'
)
provides=("niri=$pkgver" wayland-compositor)
conflicts=(niri niri-git niri-bin)
options=(!debug)
source=("niri::git+ssh://git@github.com/khughitt/niri-material.git#commit=138697be4cbb779c80425fe2a366ceca3610f38e")
sha256sums=('SKIP')

prepare() {
  cd "$srcdir/niri"
  cargo fetch --locked --target "$(rustc --print host-tuple)"
}

build() {
  cd "$srcdir/niri"
  export NIRI_BUILD_COMMIT=138697be
  CFLAGS+=" -ffat-lto-objects"
  cargo build --frozen --release --features default

  for shell in bash fish zsh; do
    cargo run --frozen --release --bin niri -- \
      completions "$shell" > "$shell-completions"
  done
}

check() {
  cd "$srcdir/niri"
  export XDG_RUNTIME_DIR="$srcdir/xdg-runtime"
  install -dm 700 "$XDG_RUNTIME_DIR"
  export RAYON_NUM_THREADS=1
  cargo test --workspace --all-targets --locked
}

package() {
  cd "$srcdir/niri"
  install -vDm 755 target/release/niri resources/niri-session \
    -t "$pkgdir/usr/bin/"
  install -vDm 644 resources/niri.service resources/niri-shutdown.target \
    -t "$pkgdir/usr/lib/systemd/user/"
  install -vDm 644 resources/niri.desktop \
    -t "$pkgdir/usr/share/wayland-sessions/"
  install -vDm 644 resources/niri-portals.conf \
    -t "$pkgdir/usr/share/xdg-desktop-portal/"
  install -vDm 644 resources/default-config.kdl README.md \
    -t "$pkgdir/usr/share/doc/niri/"
  install -vDm 644 bash-completions \
    "$pkgdir/usr/share/bash-completion/completions/niri"
  install -vDm 644 fish-completions \
    "$pkgdir/usr/share/fish/vendor_completions.d/niri.fish"
  install -vDm 644 zsh-completions \
    "$pkgdir/usr/share/zsh/site-functions/_niri"
}
```

- [ ] **Step 3: Validate syntax and generated metadata without building**

```bash
bash -n packaging/arch/PKGBUILD
srcinfo=$(makepkg -D packaging/arch --printsrcinfo)
print -r -- "$srcinfo"
grep -Fx $'pkgbase = niri-material' <<<"$srcinfo"
grep -Fx $'\tpkgver = 26.04.r95.g138697be' <<<"$srcinfo"
grep -Fx $'\tprovides = niri=26.04.r95.g138697be' <<<"$srcinfo"
grep -Fx $'\tconflicts = niri' <<<"$srcinfo"
grep -Fx $'\toptions = !debug' <<<"$srcinfo"
grep -F '138697be4cbb779c80425fe2a366ceca3610f38e' <<<"$srcinfo"
```

Expected: syntax passes and every metadata assertion prints its exact line.
Do not create or commit `.SRCINFO`; this is not an AUR repository.

`prepare()` fetches the locked dependency graph, so `build()` uses `--frozen`
to forbid either lockfile changes or network access. `check()` deliberately
uses `--locked` instead: that is the exact command that established the
293-test baseline, and the preceding fetch/build has already populated its
dependencies.

- [ ] **Step 4: Commit the package recipe**

```bash
git add packaging/arch/PKGBUILD
git commit -m "build(arch): package accepted material compositor"
```

---

### Task 2: Compose the native daily-driver dotfiles

**Files:**
- Create: dotfiles `niri/materials.kdl`
- Modify: dotfiles `niri/config.kdl`
- Modify: dotfiles `tests/setup_and_health.zsh`
- Modify outside Git: host-local `$HOME/d/dotfiles/shell/local/titan.env.zsh`

**Interfaces:**
- Consumes: Prism-generated terminal opacity/background-effect rules and the
  accepted v1 material config surface.
- Produces: dotfiles commit `feat(niri): deploy native terminal material` on
  `feat/niri-material-daily-driver`, a clean worktree suitable for
  packaged-binary validation, and the required variable in the intentionally
  ignored host-local environment file.

- [ ] **Step 1: Create an isolated dotfiles worktree after checking owned paths**

```bash
dotfiles_repo="$HOME/d/dotfiles"
dotfiles_wt="$dotfiles_repo/.worktrees/niri-material-daily-driver"
test ! -e "$dotfiles_wt"
git -C "$dotfiles_repo" diff --quiet -- \
  niri/config.kdl niri/materials.kdl tests/setup_and_health.zsh
git -C "$dotfiles_repo" diff --cached --quiet -- \
  niri/config.kdl niri/materials.kdl tests/setup_and_health.zsh
git -C "$dotfiles_repo" worktree add -b feat/niri-material-daily-driver \
  "$dotfiles_wt" main
git -C "$dotfiles_wt" status --short
```

Expected: the final status is empty. Unrelated dirty files in the main
dotfiles worktree remain untouched.

- [ ] **Step 2: Replace the legacy-autostart assertion with a failing native contract**

In `tests/setup_and_health.zsh`, replace
`test_setup_graphical_config_plans_niri_glass` with:

```zsh
test_setup_graphical_config_retains_glass_evidence_without_autostart() {
  local tmp output config materials
  tmp=$(make_tmpdir)
  register_tmp_cleanup "$tmp"
  mkdir -p "${tmp}/home" "${tmp}/config" "${tmp}/data"

  output=$(PRISM_TEST_HOSTNAME=titan \
    run_setup "$tmp" --dry-run --link-only --only graphical-config)

  [[ "$output" == *"${tmp}/home/d/niri-glass"* ]] || \
    fail "graphical setup did not retain the niri-glass evidence source"
  [[ "$output" == *"${tmp}/config/quickshell/niri-glass"* ]] || \
    fail "graphical setup did not retain the named Quickshell evidence config"
  [[ "$output" == *"niri-glass.json"* ]] || \
    fail "graphical setup did not retain the generated evidence config"

  config="${repo_root}/niri/config.kdl"
  materials="${repo_root}/niri/materials.kdl"
  ! rg -q -F 'spawn-at-startup "qs" "-c" "niri-glass"' "$config" || \
    fail "niri still autostarts the legacy glass runtime"
  rg -q -F 'include "./prism.kdl"' "$config" || fail "missing Prism include"
  rg -q -F 'include "./materials.kdl"' "$config" || fail "missing material include"
  rg -q -F 'material "terminal-glass"' "$materials" || fail "missing terminal material"
  rg -q -F 'match app-id=r#"^(kitty|com\.mitchellh\.ghostty)$"#' "$materials" || \
    fail "terminal material does not use the exact live app IDs"
  rg -q -F 'blur false' "$materials" || fail "material rule leaves Prism blur active"
  rg -q -F 'noise 0' "$materials" || fail "material rule leaves Prism noise active"
  rg -q -F 'saturation 1' "$materials" || \
    fail "material rule leaves Prism saturation active"
}
```

Replace its invocation at the bottom with:

```zsh
test_setup_graphical_config_retains_glass_evidence_without_autostart
```

- [ ] **Step 3: Run the focused contract and observe the expected failure**

```bash
cd "$dotfiles_wt"
zsh tests/setup_and_health.zsh
```

Expected: FAIL because `niri/config.kdl` still autostarts `niri-glass` and
`niri/materials.kdl` does not yet exist.

- [ ] **Step 4: Add the exact native material and override rule**

Create `niri/materials.kdl`:

```kdl
material "terminal-glass" {
    glass {
        ior 1.38
        thickness 32
        attenuation-color "#bbc7db"
        attenuation-distance 178
        chromatic-aberration 0.68
        distortion 0.32 scale=0.05
        anisotropic-blur 0
        jelly-flex 0.0038
        jelly-ripple 0.15
        bevel 9
        offset-x 4
        offset-y 4
    }
}

window-rule {
    match app-id=r#"^(kitty|com\.mitchellh\.ghostty)$"#
    material "terminal-glass"
    background-effect {
        blur false
        noise 0
        saturation 1
    }
}
```

In `niri/config.kdl`, keep Prism first and include the material immediately
after it:

```kdl
include "./prism.kdl"
include "./materials.kdl"
include "./host.kdl"
```

Delete only this autostart line:

```kdl
spawn-at-startup "qs" "-c" "niri-glass"
```

Do not remove the setup links, JSON symlink, frozen checkout, or health checks
for those retained evidence inputs.

- [ ] **Step 5: Add the external work root to the ignored host-local file**

Verify that `$HOME/d/dotfiles/shell/local/titan.env.zsh` is ignored, then
append after its `TMPDIR` block:

```zsh

# Native niri package builds and retained rollout artifacts stay off Dropbox.
export NIRI_MATERIAL_WORK_ROOT=/mnt/ssd3/niri-material
```

- [ ] **Step 6: Run the complete dotfiles test and static config checks**

```bash
zsh tests/setup_and_health.zsh
test "$(grep -c -F 'include "./materials.kdl"' niri/config.kdl)" = 1
! rg -n -F 'spawn-at-startup "qs" "-c" "niri-glass"' niri/config.kdl
zsh -c 'source "$HOME/d/dotfiles/shell/local/titan.env.zsh"; test "$NIRI_MATERIAL_WORK_ROOT" = /mnt/ssd3/niri-material'
git diff --check
```

Expected: the test prints `setup and health tests passed`; all remaining
checks exit zero.

- [ ] **Step 7: Commit the dotfiles rollout**

```bash
git add niri/config.kdl niri/materials.kdl tests/setup_and_health.zsh
git commit -m "feat(niri): deploy native terminal material"
git status --short
```

Expected: clean status. Record the commit for rollback:

```bash
dotfiles_rollout_commit=$(git rev-parse HEAD)
print -r -- "$dotfiles_rollout_commit"
```

---

### Task 3: Curate current material documentation

**Files:**
- Modify: `docs/materials/README.md`
- Modify: `docs/materials/2026-08-22-v1-design.md`
- Modify: `docs/materials/2026-08-24-glass-config-surface-design.md`
- Modify: `docs/materials/plans/2026-08-23-slice2.md`
- Modify: `docs/materials/plans/2026-08-24-slice3.md`

**Interfaces:**
- Consumes: verified commit ancestry and accepted evidence pins.
- Produces: current-state docs that distinguish merged history, acceptance,
  and the still-incomplete daily-driver deployment.

- [ ] **Step 1: Reverify every status claim against ancestry**

```bash
cd "$HOME/d/niri-material/.worktrees/v1-post-acceptance-planning"
for commit in ec0824c5 7e287517 01a4259c b8fe7b84 138697be; do
  git merge-base --is-ancestor "$commit" materials-26.04
done
git cat-file -e c0caa944db2edc5dc4844e6720951d7f32652b76^{commit} 2>/dev/null || \
  git -C "$HOME/d/niri-experiments" cat-file -e \
    c0caa944db2edc5dc4844e6720951d7f32652b76^{commit}
```

Expected: all production commits are ancestors and the evidence commit exists
in the evidence repository.

- [ ] **Step 2: Replace the v1 design's long status with the accepted result**

Replace only the opening status block in
`docs/materials/2026-08-22-v1-design.md` with:

```markdown
**Status:** accepted 2026-08-28. Production slices and the reviewed config
surface are ancestors of `materials-26.04`. Corrected frozen-reference parity
passed all 28 implementation rows and 14 combined parameters at
`niri-experiments` result `c4b71a4ebfbe3c82c56f964bfc24d4f7de1bde4f`;
the corrected physical DRM run passed all 19 machine gates and all nine
operator observations at result
`c0caa944db2edc5dc4844e6720951d7f32652b76`. Daily-driver deployment is
tracked by `2026-08-28-v1-daily-driver-rollout-design.md`.
```

Leave the historical body unchanged.

- [ ] **Step 3: Correct the Slice 2 and Slice 3 plan headers**

Set the Slice 2 header to:

```markdown
**Status:** completed; final-review fix `7e287517` is an ancestor of
`materials-26.04`. Accepted nested-winit evidence is recorded at
`niri-experiments` commit `a27eb8f`. The unchecked steps below preserve the
approved execution plan and are not current progress.
```

Set the Slice 3 header to:

```markdown
**Status:** completed; production fix `01a4259c` is an ancestor of
`materials-26.04`. Verification is recorded at `niri-experiments`
`results/slice3` commits `c37c653`, `50fb0e8`, and `3038246`. The unchecked
steps below preserve the executed plan and are not current progress.
```

Do not rewrite historical worktree commands later in either plan.

- [ ] **Step 4: Historicalize the config-surface deployment sentence**

In `docs/materials/2026-08-24-glass-config-surface-design.md`, replace:

```markdown
The material system has not shipped in a release, so no deployed
configuration is affected.
```

with:

```markdown
At the time of this pre-acceptance design, the material system had not shipped
in a release, so no deployed configuration was affected.
```

- [ ] **Step 5: Curate the README index and current progress**

Add index entries for:

```markdown
- `2026-08-24-glass-config-surface-design.md`: implemented and accepted v1 configuration surface.
- `2026-08-28-v1-daily-driver-rollout-design.md`: approved package, transition, rollback, and burn-in design.
- `plans/2026-08-24-slice3.md`: completed overview-crop correction plan.
- `plans/2026-08-24-glass-config-surface.md`: executed configuration-surface plan.
- `plans/2026-08-24-v1-parity.md`: executed frozen-reference parity plan.
- `plans/2026-08-28-v1-daily-driver-rollout.md`: package and daily-driver rollout procedure.
```

Rewrite the current Slice 2 and Slice 3 progress paragraphs to say that
`7e287517` and `01a4259c` are ancestors of `materials-26.04`; retain their
existing evidence pins. Add a concise rollout paragraph stating that v1 is
accepted but remains undeployed until this plan's package and two-stage
burn-in pass. Link `[packaging/arch/PKGBUILD](../../packaging/arch/PKGBUILD)`
as the build entry point and
`plans/2026-08-28-v1-daily-driver-rollout.md` as the install/rollback
procedure.

- [ ] **Step 6: Run the focused drift and formatting sweep**

```bash
git diff --check
! rg -n \
  'implemented on `slice2-glass`|implemented on `slice3-crop`|The material system has not shipped in a release' \
  docs/materials/README.md \
  docs/materials/2026-08-22-v1-design.md \
  docs/materials/2026-08-24-glass-config-surface-design.md \
  docs/materials/material-config.md
! sed -n '1,12p' docs/materials/plans/2026-08-23-slice2.md \
  docs/materials/plans/2026-08-24-slice3.md | \
  rg -n 'implemented on `slice2-glass`|implemented on `slice3-crop`'
rg -n '138697be|c4b71a4e|c0caa944|daily-driver' \
  docs/materials/README.md docs/materials/2026-08-22-v1-design.md
```

Expected: no current-state stale claim, and the accepted pins plus rollout
entry point are present. Historical instructions outside this focused set are
not drift.

- [ ] **Step 7: Commit the curation**

```bash
git add docs/materials/README.md \
  docs/materials/2026-08-22-v1-design.md \
  docs/materials/2026-08-24-glass-config-surface-design.md \
  docs/materials/plans/2026-08-23-slice2.md \
  docs/materials/plans/2026-08-24-slice3.md
git commit -m "docs(materials): curate accepted v1 status"
```

---

### Task 4: Build, test, and inspect the package outside Dropbox

**Files:**
- Verify: `packaging/arch/PKGBUILD`
- Read: dotfiles worktree `niri/config.kdl`, `niri/materials.kdl`
- Create outside repository: rollout build/source/package/log/stage artifacts

**Interfaces:**
- Consumes: Task 1 PKGBUILD and Task 2 clean dotfiles worktree.
- Produces: inspected package archive, build log with the 293-test baseline,
  payload comparison, staged accepted binary, and pre-install config pass.

- [ ] **Step 1: Require and materialize the exact external root**

```bash
dotfiles_wt="$HOME/d/dotfiles/.worktrees/niri-material-daily-driver"
source "$HOME/d/dotfiles/shell/local/titan.env.zsh"
: "${NIRI_MATERIAL_WORK_ROOT:?NIRI_MATERIAL_WORK_ROOT must be set}"
test "$NIRI_MATERIAL_WORK_ROOT" = /mnt/ssd3/niri-material

rollout_root="$NIRI_MATERIAL_WORK_ROOT/v1-daily-driver-138697be"
builddir="$rollout_root/build"
srcdest="$rollout_root/sources"
pkgdest="$rollout_root/packages"
logdest="$rollout_root/logs"
stagedir="$rollout_root/stage"
test ! -e "$rollout_root"
install -d "$builddir" "$srcdest" "$pkgdest" "$logdest" "$stagedir"
export BUILDDIR="$builddir" SRCDEST="$srcdest" PKGDEST="$pkgdest" LOGDEST="$logdest"
```

Expected: a fresh, explicit rollout subtree exists only under the required
SSD root. If it already exists, stop and inspect it; do not overwrite it.

- [ ] **Step 2: Verify the remote branch contains the pinned source**

```bash
cd "$HOME/d/niri-material/.worktrees/v1-post-acceptance-planning"
set -e
remote_tip=$(git ls-remote ssh://git@github.com/khughitt/niri-material.git \
  refs/heads/materials-26.04 | awk '{print $1}')
test -n "$remote_tip"
git fetch origin materials-26.04
test "$(git rev-parse origin/materials-26.04)" = "$remote_tip"
git merge-base --is-ancestor \
  138697be4cbb779c80425fe2a366ceca3610f38e origin/materials-26.04
```

Expected: the remote tip is fetched and contains the accepted commit.

- [ ] **Step 3: Snapshot Arch's current package inventory**

```bash
pacman -Q niri
pacman -Qlq niri > "$rollout_root/arch-niri-26.04-1.entries"
test "$(wc -l < "$rollout_root/arch-niri-26.04-1.entries")" = 27
test "$(awk '!/\/$/' "$rollout_root/arch-niri-26.04-1.entries" | wc -l)" = 11
```

Expected: installed package `niri 26.04-1`, 27 pacman entries, 11 files.

- [ ] **Step 4: Build and run the pinned check() suite**

```bash
makepkg -D packaging/arch --cleanbuild --log
```

Expected: makepkg exits zero, builds from the pinned commit, and runs
`cargo test --workspace --all-targets --locked`. Confirm the log contains all
four accepted crate counts:

```bash
rg -n 'test result: ok\. 244 passed' "$logdest"
rg -n 'test result: ok\. 45 passed' "$logdest"
rg -n 'test result: ok\. 1 passed' "$logdest"
rg -n 'test result: ok\. 3 passed' "$logdest"
```

- [ ] **Step 5: Resolve and inspect the single package archive**

```bash
archive=$(makepkg -D packaging/arch --packagelist)
test -f "$archive"
test "$(find "$pkgdest" -maxdepth 1 -type f -name 'niri-material-*.pkg.tar.*' | wc -l)" = 1
pacman -Qip "$archive" | tee "$rollout_root/package-info.txt"
rg -F 'Name            : niri-material' "$rollout_root/package-info.txt"
rg -F 'Version         : 26.04.r95.g138697be-1' "$rollout_root/package-info.txt"
rg -F 'Provides        : niri=26.04.r95.g138697be' "$rollout_root/package-info.txt"
rg -F 'Conflicts With  : niri' "$rollout_root/package-info.txt"
```

Expected: exactly one archive and exact package identity. Additional conflict
names may follow `niri` on the same metadata line.

- [ ] **Step 6: Compare the archive payload with Arch's 27 entries**

```bash
bsdtar -tf "$archive" | awk '!/^\./ { print "/" $0 }' | LC_ALL=C sort \
  > "$rollout_root/niri-material.entries"
LC_ALL=C sort "$rollout_root/arch-niri-26.04-1.entries" \
  > "$rollout_root/arch-niri-26.04-1.entries.sorted"
diff -u "$rollout_root/arch-niri-26.04-1.entries.sorted" \
  "$rollout_root/niri-material.entries"
test "$(wc -l < "$rollout_root/niri-material.entries")" = 27
test "$(awk '!/\/$/' "$rollout_root/niri-material.entries" | wc -l)" = 11
```

Expected: byte-empty diff, 27 payload entries, 11 regular files.

- [ ] **Step 7: Extract the package and validate the proposed composed config**

```bash
bsdtar -xf "$archive" -C "$stagedir"
test "$($stagedir/usr/bin/niri --version)" = 'niri 26.04 (138697be)'

config_stage="$rollout_root/config"
install -d "$config_stage"
cp "$dotfiles_wt/niri/config.kdl" "$dotfiles_wt/niri/materials.kdl" \
  "$dotfiles_wt/niri/host-titan.kdl" "$config_stage/"
cp "$HOME/.local/state/prism/generated/prism.kdl" "$config_stage/prism.kdl"
cp "$HOME/.config/niri/noctalia.kdl" "$config_stage/noctalia.kdl"
ln -s host-titan.kdl "$config_stage/host.kdl"
"$stagedir/usr/bin/niri" validate -c "$config_stage/config.kdl"
sha256sum "$archive" "$logdest"/* | tee "$rollout_root/artifacts.sha256"
```

Expected: exact accepted version, valid composed config, and recorded archive
plus log hashes. No host package or running process has changed yet.

---

### Task 5: Install the package and stage the reversible host transition

**Files:**
- Install: inspected `niri-material` package archive
- Merge: dotfiles branch `feat/niri-material-daily-driver`
- Rename: unmanaged `/usr/local/bin/niri` to a versioned rollback file
- Create outside repository: rollback metadata

**Interfaces:**
- Consumes: Task 4 inspected archive and validated dotfiles commit.
- Produces: package-owned `/usr/bin/niri`, active main dotfiles config, saved
  old compositor, and `/usr/bin` command resolution for the next session.

- [ ] **Step 1: Record rollback identities before any host mutation**

```bash
source "$HOME/d/dotfiles/shell/local/titan.env.zsh"
: "${NIRI_MATERIAL_WORK_ROOT:?NIRI_MATERIAL_WORK_ROOT must be set}"
test "$NIRI_MATERIAL_WORK_ROOT" = /mnt/ssd3/niri-material
rollout_root="$NIRI_MATERIAL_WORK_ROOT/v1-daily-driver-138697be"
pkgdest="$rollout_root/packages"
archive=$(find "$pkgdest" -maxdepth 1 -type f \
  -name 'niri-material-26.04.r95.g138697be-1-x86_64.pkg.tar.*' -print -quit)
test -f "$archive"

dotfiles_repo="$HOME/d/dotfiles"
dotfiles_wt="$dotfiles_repo/.worktrees/niri-material-daily-driver"
dotfiles_base=$(git -C "$dotfiles_repo" rev-parse main)
dotfiles_rollout_commit=$(git -C "$dotfiles_wt" rev-parse HEAD)
old_binary=/usr/local/bin/niri
rollback_binary=/usr/local/bin/niri-v26.04-2-g5e53b949.rollback
test -x "$old_binary"
test ! -e "$rollback_binary"
test "$(readlink -f /proc/$(systemctl --user show niri.service -p MainPID --value)/exe)" = \
  "$old_binary"
{
  printf 'dotfiles_base=%s\n' "$dotfiles_base"
  printf 'dotfiles_rollout_commit=%s\n' "$dotfiles_rollout_commit"
  printf 'old_binary=%s\n' "$old_binary"
  printf 'rollback_binary=%s\n' "$rollback_binary"
  sha256sum "$old_binary"
  "$old_binary" --version
} | tee "$rollout_root/rollback.txt"
```

Expected: the live process and rollback source are the unmanaged old binary,
and the rollback target does not exist.

- [ ] **Step 2: Pause for explicit operator approval, then install only the inspected archive**

Show the operator `package-info.txt`, `artifacts.sha256`, the payload diff
result, and `rollback.txt`. After explicit approval, run:

```bash
sudo pacman -U "$archive"
```

Do not add `--noconfirm`; the operator must see pacman's replacement of
`niri 26.04-1` by `niri-material 26.04.r95.g138697be-1`.

- [ ] **Step 3: Verify package ownership before touching command resolution**

```bash
pacman -Q niri-material
! pacman -Q niri
test "$(pacman -Qo /usr/bin/niri | awk '{print $5}')" = niri-material
test "$(/usr/bin/niri --version)" = 'niri 26.04 (138697be)'
pacman -Qlq niri-material | LC_ALL=C sort > "$rollout_root/installed.entries"
diff -u "$rollout_root/arch-niri-26.04-1.entries.sorted" \
  "$rollout_root/installed.entries"
```

Expected: exact version, package ownership, and unchanged 27-entry inventory.
The running compositor still resolves to the old inode.

- [ ] **Step 4: Fast-forward the reviewed dotfiles commit into main**

```bash
test "$(git -C "$dotfiles_repo" branch --show-current)" = main
git -C "$dotfiles_repo" diff --quiet -- \
  niri/config.kdl niri/materials.kdl tests/setup_and_health.zsh
git -C "$dotfiles_repo" diff --cached --quiet -- \
  niri/config.kdl niri/materials.kdl tests/setup_and_health.zsh
git -C "$dotfiles_repo" merge --ff-only feat/niri-material-daily-driver
test "$(git -C "$dotfiles_repo" rev-parse HEAD)" = "$dotfiles_rollout_commit"
test "$(readlink -f "$HOME/.config/niri/config.kdl")" = \
  "$(realpath "$dotfiles_repo/niri/config.kdl")"
```

Expected: only the rollout commit advances main. The old running compositor
may reject the newly material-enabled file once; it must keep its prior valid
config until restart.

- [ ] **Step 5: Validate the merged config with the package-owned binary**

```bash
/usr/bin/niri validate -c "$dotfiles_repo/niri/config.kdl"
zsh "$dotfiles_repo/tests/setup_and_health.zsh"
```

Expected: config valid and complete dotfiles tests pass.

- [ ] **Step 6: Pause for approval, then move the exact old binary aside**

After explicit approval, recheck and rename only the recorded file:

```bash
test "$(sha256sum "$old_binary")" = "$(grep '  /usr/local/bin/niri$' "$rollout_root/rollback.txt")"
sudo mv -- "$old_binary" "$rollback_binary"
hash -r
test "$(command -v niri)" = /usr/bin/niri
test "$(niri --version)" = 'niri 26.04 (138697be)'
systemctl --user show niri.service -p ExecStart --no-pager | \
  rg -F 'path=niri ; argv[]=niri --session'
```

Expected: new commands resolve to `/usr/bin/niri`; the current compositor
process continues from the renamed old inode until the manual restart.

- [ ] **Step 7: Preserve the exact immediate rollback procedure**

If any later gate fails, switch to the recovery VT and run:

```bash
git -C "$dotfiles_repo" revert --no-edit "$dotfiles_rollout_commit"
sudo mv -- "$rollback_binary" "$old_binary"
hash -r
test "$(command -v niri)" = /usr/local/bin/niri
niri --version
```

Then manually restart the graphical session. This rollback needs neither the
network nor a rebuild; `niri-material` may remain installed underneath until
the Arch package is restored deliberately.

- [ ] **Step 8: Manually restart the graphical session**

Save work, switch to the recovery VT if desired, and restart the graphical
session using the normal login/session flow. Do not automate this step. After
login, continue with Task 6.

---

### Task 6: Run the daily-driver and cold-start burn-in

**Files:**
- Read: installed package, live process, niri IPC, systemd journal
- Temporarily edit and restore: dotfiles `niri/materials.kdl`
- Create outside repository: burn-in observations and journal capture

**Interfaces:**
- Consumes: package-owned compositor and merged native-material dotfiles.
- Produces: machine and operator evidence for one normal-use session and one
  cold start, or an immediate rollback decision.

- [ ] **Step 1: Verify the new live process and absence of legacy rendering**

```bash
source "$HOME/d/dotfiles/shell/local/titan.env.zsh"
: "${NIRI_MATERIAL_WORK_ROOT:?NIRI_MATERIAL_WORK_ROOT must be set}"
test "$NIRI_MATERIAL_WORK_ROOT" = /mnt/ssd3/niri-material
rollout_root="$NIRI_MATERIAL_WORK_ROOT/v1-daily-driver-138697be"
dotfiles_repo="$HOME/d/dotfiles"

pid=$(systemctl --user show niri.service -p MainPID --value)
test "$pid" -gt 0
test "$(readlink -f "/proc/$pid/exe")" = /usr/bin/niri
test "$(pacman -Qo /usr/bin/niri | awk '{print $5}')" = niri-material
test "$(/usr/bin/niri --version)" = 'niri 26.04 (138697be)'
test "$(niri msg version | rg -c '138697be')" = 2
! pgrep -af '[q]s([[:space:]].*)?-c[[:space:]]+niri-glass'
niri msg --json layers | jq -e \
  'all(.[]; (.namespace | startswith("glasspanes") | not))'
```

Expected: package-owned process, accepted version, no legacy process, and no
`glasspanes` or `glasspanes-preview` layer.

- [ ] **Step 2: Confirm exact Kitty and Ghostty assignment**

Open one fresh Kitty and one fresh Ghostty. Run:

```bash
niri msg --json windows | jq -r \
  '.[] | select(.app_id == "kitty" or .app_id == "com.mitchellh.ghostty") | [.id, .app_id, .title] | @tsv'
```

Expected: both exact app IDs appear and both windows visibly carry the native
material. Ghostty is now intentionally treated even though the frozen legacy
client's incorrect exact `ghostty` entry did not match it.

- [ ] **Step 3: Exercise the physical daily-driver matrix**

For both terminals, observe all of the following and append `PASS` or the
specific defect to `$rollout_root/burn-in.txt`:

1. focus and unfocus preserve Prism's opacity change while material remains;
2. move/scroll and resize animate without a detached pane or observable
   six-tap stutter;
3. overview entry/exit keeps the material attached;
4. workspace switching keeps the material attached through transitions;
5. close and remap produces a treated replacement immediately; and
6. overlapping translucent content shows native refraction/attenuation with
   no legacy/native double rendering.

Any crash, corruption, missing material, double rendering, or observable
stutter is a rollout failure; execute Task 5 Step 7.

- [ ] **Step 4: Prove valid reload and invalid-config retention**

```bash
materials="$dotfiles_repo/niri/materials.kdl"
saved="$rollout_root/materials.kdl.accepted"
cp "$materials" "$saved"
perl -0pi -e 's/attenuation-distance 178/attenuation-distance 179/' "$materials"
sleep 2
journalctl --user -u niri.service --since '10 seconds ago' --no-pager | \
  rg -F 'loaded config'
cp "$saved" "$materials"
sleep 2

perl -0pi -e 's/offset-x 4/offset-x 10/' "$materials"
! /usr/bin/niri validate -c "$dotfiles_repo/niri/config.kdl"
sleep 2
journalctl --user -u niri.service --since '10 seconds ago' --no-pager | \
  rg -F 'offset must not exceed bevel'
cp "$saved" "$materials"
sleep 2
/usr/bin/niri validate -c "$dotfiles_repo/niri/config.kdl"
cmp "$saved" "$materials"
git -C "$dotfiles_repo" diff --quiet -- niri/materials.kdl
```

Expected: valid edit and restoration reload; invalid edit is rejected while
the last valid material remains visible; final tracked bytes are unchanged.
If interrupted, restore with `cp "$saved" "$materials"` before continuing.

- [ ] **Step 5: Establish a clean journal boundary and use the session normally**

```bash
acceptance_since=$(date --iso-8601=seconds)
printf 'acceptance_since=%s\n' "$acceptance_since" | tee -a "$rollout_root/burn-in.txt"
```

Use the system normally for the rest of the session. At its end:

```bash
acceptance_since=$(sed -n 's/^acceptance_since=//p' \
  "$rollout_root/burn-in.txt" | tail -n 1)
test -n "$acceptance_since"
journalctl --user -u niri.service --since "$acceptance_since" --no-pager \
  > "$rollout_root/niri-normal-session.log"
scan_status=0
rg -ni '(warn|error).*(material|shader|render|config)|(material|shader|render|config).*(warn|error)' \
  "$rollout_root/niri-normal-session.log" || scan_status=$?
test "$scan_status" = 1
printf 'normal-use session: PASS\n' >> "$rollout_root/burn-in.txt"
```

Expected: no matching warning/error and no observed rollout defect.

- [ ] **Step 6: Perform one cold start and repeat the machine/visual essentials**

Shut down fully, start the host, log into niri normally, then rerun Task 6
Step 1. Open fresh Kitty and Ghostty windows and repeat focus, movement,
overview, workspace switching, and close/remap. Record:

```bash
printf 'cold start: PASS\n' >> "$rollout_root/burn-in.txt"
journalctl --user -u niri.service -b --no-pager \
  > "$rollout_root/niri-cold-start.log"
scan_status=0
rg -ni '(warn|error).*(material|shader|render|config)|(material|shader|render|config).*(warn|error)' \
  "$rollout_root/niri-cold-start.log" || scan_status=$?
test "$scan_status" = 1
```

Expected: package-owned accepted binary, no legacy process/layer, both exact
app IDs treated, no visible defect, and clean relevant journal.

- [ ] **Step 7: Obtain the operator's explicit burn-in verdict**

Show `burn-in.txt`, the normal-session log scan, and the cold-start log scan.
Continue only after the operator records an explicit PASS. Otherwise execute
the rollback and leave all artifacts intact for diagnosis.

---

### Task 7: Record deployment, integrate, publish, and clean retained artifacts

**Files:**
- Modify: `docs/materials/2026-08-28-v1-daily-driver-rollout-design.md`
- Modify: `docs/materials/README.md`
- Modify: `docs/materials/plans/2026-08-28-v1-daily-driver-rollout.md`
- Delete after approval: exact external build/source/log/stage paths and old
  rollback binary
- Retain: package archive below `$NIRI_MATERIAL_WORK_ROOT`

**Interfaces:**
- Consumes: explicit burn-in PASS and artifact hashes.
- Produces: truthful deployed status on `materials-26.04`, merged and
  published dotfiles/niri-material branches, retained package archive, and no
  stale heavy build artifacts.

- [ ] **Step 1: Record the concrete deployment evidence**

Initialize the final paths and read the package SHA from `artifacts.sha256`,
the dotfiles commit from `rollback.txt`, and the two PASS lines from
`burn-in.txt`:

```bash
source "$HOME/d/dotfiles/shell/local/titan.env.zsh"
: "${NIRI_MATERIAL_WORK_ROOT:?NIRI_MATERIAL_WORK_ROOT must be set}"
test "$NIRI_MATERIAL_WORK_ROOT" = /mnt/ssd3/niri-material
rollout_root="$NIRI_MATERIAL_WORK_ROOT/v1-daily-driver-138697be"
pkgdest="$rollout_root/packages"
archive=$(find "$pkgdest" -maxdepth 1 -type f \
  -name 'niri-material-26.04.r95.g138697be-1-x86_64.pkg.tar.*' -print -quit)
test -f "$archive"
dotfiles_repo="$HOME/d/dotfiles"
dotfiles_rollout_commit=$(sed -n 's/^dotfiles_rollout_commit=//p' \
  "$rollout_root/rollback.txt")
package_sha=$(sha256sum "$archive" | awk '{print $1}')
deployment_date=$(date +%F)
test "$(grep -c 'session: PASS\|cold start: PASS' "$rollout_root/burn-in.txt")" = 2
```

Change the rollout design status to an implemented/deployed statement
containing:

- the exact `$deployment_date`;
- package `niri-material 26.04.r95.g138697be-1`;
- source `138697be4cbb779c80425fe2a366ceca3610f38e`;
- dotfiles rollout commit;
- package archive SHA-256; and
- normal-use plus cold-start PASS.

Update the README rollout paragraph from “accepted but undeployed” to the same
concise deployed result. Mark only actually executed checkboxes in this plan
and change its status to `executed; deployed`.

- [ ] **Step 2: Run final dotfiles, package, and drift checks**

```bash
cd "$HOME/d/niri-material/.worktrees/v1-post-acceptance-planning"
git diff --check
git -C "$dotfiles_repo" status --short
zsh "$dotfiles_repo/tests/setup_and_health.zsh"
/usr/bin/niri validate -c "$dotfiles_repo/niri/config.kdl"
pacman -Q niri-material
test "$(readlink -f /proc/$(systemctl --user show niri.service -p MainPID --value)/exe)" = \
  /usr/bin/niri
! rg -n \
  'implemented on `slice2-glass`|implemented on `slice3-crop`|The material system has not shipped in a release' \
  docs/materials/README.md \
  docs/materials/2026-08-22-v1-design.md \
  docs/materials/2026-08-24-glass-config-surface-design.md \
  docs/materials/material-config.md
! sed -n '1,12p' docs/materials/plans/2026-08-23-slice2.md \
  docs/materials/plans/2026-08-24-slice3.md | \
  rg -n 'implemented on `slice2-glass`|implemented on `slice3-crop`'
```

Expected: dotfiles tests/config pass; live package identity holds; no
current-state drift remains. Existing unrelated dirty dotfiles paths are
listed but unchanged. Task 4's package `check()` is the authoritative
293-test run against the exact packaged source; rerunning it on this
PKGBUILD-and-docs branch would add no production coverage.

- [ ] **Step 3: Commit the truthful deployment result**

```bash
git add docs/materials/2026-08-28-v1-daily-driver-rollout-design.md \
  docs/materials/README.md \
  docs/materials/plans/2026-08-28-v1-daily-driver-rollout.md
git commit -m "docs(materials): record daily-driver deployment"
git status --short
```

Expected: clean niri-material feature worktree.

- [ ] **Step 4: Fast-forward the niri-material branch**

```bash
material_repo="$HOME/d/niri-material"
feature="$material_repo/.worktrees/v1-post-acceptance-planning"
test "$(git -C "$material_repo" branch --show-current)" = materials-26.04
test -z "$(git -C "$material_repo" status --short)"
git -C "$material_repo" merge --ff-only v1-post-acceptance-planning
git -C "$material_repo" merge-base --is-ancestor \
  138697be4cbb779c80425fe2a366ceca3610f38e materials-26.04
```

Expected: fast-forward only; the deployed design status lands in the same
merge as the package and curated docs.

- [ ] **Step 5: Review and explicitly approve the two remote updates**

```bash
git -C "$dotfiles_repo" fetch origin main
git -C "$material_repo" fetch origin materials-26.04
git -C "$dotfiles_repo" merge-base --is-ancestor origin/main main
git -C "$material_repo" merge-base --is-ancestor \
  origin/materials-26.04 materials-26.04
git -C "$dotfiles_repo" log --oneline origin/main..main
git -C "$material_repo" log --oneline origin/materials-26.04..materials-26.04
```

Show both pending commit lists to the operator. Stop if either contains a
commit outside the intended local history. After explicit approval, publish
both fast-forward updates:

```bash
git -C "$dotfiles_repo" push origin main:main
git -C "$material_repo" push origin materials-26.04:materials-26.04
test "$(git -C "$dotfiles_repo" rev-parse main)" = \
  "$(git -C "$dotfiles_repo" rev-parse origin/main)"
test "$(git -C "$material_repo" rev-parse materials-26.04)" = \
  "$(git -C "$material_repo" rev-parse origin/materials-26.04)"
```

Expected: the deployed config and deployment-status documentation are both
durable on their existing remote branches.

- [ ] **Step 6: Pause for cleanup approval and validate exact targets**

```bash
rollback_binary=/usr/local/bin/niri-v26.04-2-g5e53b949.rollback
test "$rollout_root" = /mnt/ssd3/niri-material/v1-daily-driver-138697be
test -f "$archive"
sha256sum --check <(grep -F "$archive" "$rollout_root/artifacts.sha256")
test -x "$rollback_binary"
test "$(readlink -f /proc/$(systemctl --user show niri.service -p MainPID --value)/exe)" = \
  /usr/bin/niri
```

Show the operator these resolved paths before deletion:

```text
/mnt/ssd3/niri-material/v1-daily-driver-138697be/build
/mnt/ssd3/niri-material/v1-daily-driver-138697be/sources
/mnt/ssd3/niri-material/v1-daily-driver-138697be/logs
/mnt/ssd3/niri-material/v1-daily-driver-138697be/stage
/mnt/ssd3/niri-material/v1-daily-driver-138697be/config
/usr/local/bin/niri-v26.04-2-g5e53b949.rollback
```

The package archive under `packages/`, `package-info.txt`, payload inventories,
hash manifest, rollback metadata, and burn-in record remain retained.

- [ ] **Step 7: After explicit approval, delete only the validated heavy/rollback targets**

```bash
rm -rf -- \
  /mnt/ssd3/niri-material/v1-daily-driver-138697be/build \
  /mnt/ssd3/niri-material/v1-daily-driver-138697be/sources \
  /mnt/ssd3/niri-material/v1-daily-driver-138697be/logs \
  /mnt/ssd3/niri-material/v1-daily-driver-138697be/stage \
  /mnt/ssd3/niri-material/v1-daily-driver-138697be/config
sudo rm -- /usr/local/bin/niri-v26.04-2-g5e53b949.rollback
test -f "$archive"
test ! -e /usr/local/bin/niri-v26.04-2-g5e53b949.rollback
```

These deletions are intentional and non-recoverable; the retained package
archive remains the reinstall artifact until superseded.

- [ ] **Step 8: Remove merged worktrees and branches**

Run from the two main worktrees:

```bash
dotfiles_repo="$HOME/d/dotfiles"
dotfiles_wt="$dotfiles_repo/.worktrees/niri-material-daily-driver"
material_repo="$HOME/d/niri-material"
feature="$material_repo/.worktrees/v1-post-acceptance-planning"

git -C "$dotfiles_repo" worktree remove "$dotfiles_wt"
git -C "$dotfiles_repo" worktree prune
git -C "$dotfiles_repo" branch -d feat/niri-material-daily-driver

git -C "$material_repo" worktree remove "$feature"
git -C "$material_repo" worktree prune
git -C "$material_repo" branch -d v1-post-acceptance-planning
```

Expected: both feature commits remain reachable from their main branches,
both feature worktrees are gone, and only the package archive plus compact
rollout records remain under the external work root.
