# Native materials v1 daily-driver rollout: design

**Status:** approved 2026-08-28; implemented 2026-08-29 and in daily-driver
use. Arch's `niri 26.04-1` was removed and `niri-material
26.04.r95.g138697be-1` installed 2026-08-28T20:20:00, then upgraded to
`26.04.r106.g7f6e69c3-1` on 2026-08-29T07:11:08 to pick up the reload fix in
`7f6e69c3`. Prism took ownership of the material definition on 2026-08-29.
Burn-in is in progress: the journal boundary was reset 2026-08-29 and the
normal-session scan, one cold start, and the operator PASS remain outstanding.

## Context

Native materials v1 is accepted. The corrected physical DRM run against
production source `138697be4cbb779c80425fe2a366ceca3610f38e` passed all 19
machine gates and all nine physical observations at `niri-experiments` result
commit `c0caa944db2edc5dc4844e6720951d7f32652b76`.

The accepted compositor is not yet the daily-driver installation. Arch's
`niri` package owns `/usr/bin/niri`, while an unmanaged
`/usr/local/bin/niri` at patched source `5e53b949` wins through `PATH`. The
normal niri config still launches the frozen Quickshell `niri-glass` client
with an allowlist intended for Kitty and Ghostty, although its exact `ghostty`
entry does not match Ghostty's live app ID.

This rollout turns the accepted implementation into a package-owned daily
driver without changing production material code. It also curates current
documentation claims that still describe already-merged slice branches or the
material surface as undeployed.

## Decisions

| Decision | Choice |
| --- | --- |
| Package manager | Arch `PKGBUILD` built with `makepkg`, installed with pacman |
| Package identity | `niri-material`, providing and conflicting with `niri` |
| Production source | Remote commit `138697be4cbb779c80425fe2a366ceca3610f38e` |
| Installed paths | The same literal paths and resources as Arch's `niri` package |
| Build storage | Required `$NIRI_MATERIAL_WORK_ROOT`; no in-repository or `/tmp` fallback |
| Initial material scope | Kitty and Ghostty, using their live exact app IDs |
| Native sampling | Accept six derived taps at chromatic aberration `0.68`, versus the legacy client's configured three samples |
| Legacy runtime | Quickshell glass no longer starts normally; its frozen source and inputs remain available for evidence work |
| Restart | Manual graphical-session restart after package and config preflight |
| Completion | One normal-use session and one cold start without a rollout defect |

## Package

`packaging/arch/PKGBUILD` is a narrow adaptation of Arch's niri package. It
keeps Arch's runtime dependencies, optional dependencies, release build,
installed resources, session integration, portal configuration, and shell
completions. Its complete recipe-level differences are:

- `pkgname=niri-material`;
- `pkgver=26.04.r95.g138697be` and `pkgrel=1`, with no `pkgver()` function;
- `provides=("niri=$pkgver" wayland-compositor)`;
- conflicts with `niri`, `niri-git`, and `niri-bin`;
- `options=(!debug)` produces one stripped package even when the host enables
  makepkg debug packages globally;
- `makedepends` adds `gtk4` and `libadwaita` because the pinned all-targets
  check builds `niri-visual-tests`;
- source is the named VCS checkout
  `niri::git+ssh://git@github.com/khughitt/niri-material.git#commit=138697be4cbb779c80425fe2a366ceca3610f38e`,
  using the private repository's existing SSH access,
  with `sha256sums=('SKIP')` because the immutable commit is the source pin;
- `prepare()`, `build()`, `check()`, and `package()` enter
  `"$srcdir/niri"` instead of a release-tarball directory;
- `build()` exports `NIRI_BUILD_COMMIT=138697be` directly instead of deriving
  it from a tar archive; and
- `check()` runs the accepted baseline command exactly:
  `cargo test --workspace --all-targets --locked`.

The fixed version is pacman-comparable with upstream `26.04` and identifies
both the 95 commits after `upstream/main` and the accepted source commit. A
dynamic `pkgver()` would add no information to a checkout that is deliberately
pinned to one commit.

Pinning the accepted production commit avoids a self-reference between a
committed `PKGBUILD` and the commit literal inside it. It is also stronger
than tagging the later documentation head: `git diff` confirms that every
change after `138697be` and before this design is under `docs/`, so the
production tree is identical to the candidate exercised on physical DRM.

The package recipe does not derive installed filenames from `$pkgname`.
`package()` reads `target/release/niri` and the literal `resources/niri*`
inputs, then installs `/usr/bin/niri`, `/usr/bin/niri-session`, the
`niri.service` and `niri-shutdown.target` user units, `niri.desktop`,
`niri-portals.conf`, the default config and README under
`/usr/share/doc/niri/`, and the three `niri` shell completions. Thus the
metadata name changes without moving any path in Arch's 27 pacman entries (11
regular files and 16 directories).
Pacman owns every deployed file; the package does not modify `/usr/local` from
an install hook.

The pinned `check()` command runs the existing workspace test suite without
`--all-features`. Its accepted baseline is 293 tests: 244 `niri`, 45
`niri-config`, one wiki parse, and three `niri-ipc` tests.

## Build and artifact lifecycle

The host-specific zshenv-time file sets:

```sh
export NIRI_MATERIAL_WORK_ROOT=/mnt/ssd3/niri-material
```

The documented `makepkg` invocation fails if that variable is absent, creates
its package directories explicitly, and exports makepkg's native `BUILDDIR`,
`SRCDEST`, `PKGDEST`, and `LOGDEST` beneath it. Source clones, Cargo output,
package archives, and logs therefore stay outside Dropbox and outside the
quota-limited temporary filesystem.

Build and installation remain separate operations. First `makepkg` builds,
tests, and logs. Then pacman metadata and the archive file list are inspected.
Only the inspected archive is passed explicitly to `pacman -U`.

During burn-in, retain the installed package archive, build log, and renamed
old compositor. After acceptance, delete makepkg's source/build trees and
logs, remove the renamed old compositor, and retain only the package archive
outside Dropbox until another package supersedes it. Failed artifacts remain
long enough to diagnose or roll back, then follow the same explicit cleanup.

## Host transition and rollback

Installing `niri-material` removes the conflicting Arch `niri` package and
makes `/usr/bin/niri` package-owned. It does not affect the already-running
compositor process.

After installation:

1. verify `/usr/bin/niri` is owned by `niri-material` and reports the accepted
   commit;
2. validate the proposed daily config with that exact binary;
3. rename the unmanaged `/usr/local/bin/niri` to a versioned rollback name,
   making `/usr/bin/niri` authoritative for the next session;
4. confirm command resolution and the systemd user service's `ExecStart` path;
5. leave the current graphical session running until the operator chooses to
   restart it; and
6. after restart, verify the live PID resolves to `/usr/bin/niri`.

The immediate rollback is deliberately independent of the network and of a
new build: restore the preceding dotfiles commit and rename the saved
`/usr/local` binary back to `niri`, then restart the graphical session. Pacman
may continue to hold `niri-material` underneath because `/usr/local/bin`
precedes `/usr/bin`; after recovery, reinstalling the Arch package completes a
full package rollback when desired.

No package script edits or deletes the unmanaged binary. The transition
procedure resolves and prints that exact file before renaming it.

## Daily terminal material

Dotfiles gains a composed `niri/materials.kdl`, included after generated
`prism.kdl`. It defines:

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
```

These are the active legacy values where v1 has a corresponding public
control. `bevel 9` is the reviewed inverse mapping from legacy `paneLip 5`
and four-pixel shifts: `5 + max(abs(4), abs(4))`. Roughness is absent because
v1 does not expose it. Sample count is derived from the strongest multi-tap
effect; chromatic aberration `0.68` yields six taps. This is an accepted
divergence from the legacy JSON's `samples: 3`: native v1 deliberately removed
the user-facing GPU-cost dial and derives the count from effect strength. It
doubles this effect's loop count, not necessarily total frame cost, and the
normal-use burn-in is responsible for rejecting observable stutter.

One rule assigns `terminal-glass` to the live exact app IDs and clears both
rendering controls that keep Prism's background effect visible:

```kdl
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

Ghostty's live app ID is `com.mitchellh.ghostty`. The legacy client's exact
string allowlist contained `ghostty`, so it did not actually treat Ghostty;
generated Prism's unanchored matcher does. Including Ghostty here follows the
intended two-terminal scope and is an explicit rollout expansion, not a claim
of identical legacy matching.

Because this rule follows generated Prism rules, it retains Prism's
focus-conditioned opacity. `blur false` alone is insufficient: Prism's
nonzero `noise` keeps the background-effect render element visible, so
`noise 0` is also required. Explicit `saturation 1` keeps suppression
self-contained if Prism's generated saturation changes later. With blur,
noise, and saturation inert, automatic xray selection is unreachable. The
material owns the transmitted background; leaving a separate blur/noise
effect beneath its non-opaque render element would spend another pass without
contributing the intended native result.

The main config includes `materials.kdl` and removes
`spawn-at-startup "qs" "-c" "niri-glass"`. The frozen niri-glass checkout,
JSON, setup links, and evidence contracts are retained but dormant. Setup and
health tests stop treating legacy autostart as a required daily-driver
contract.

## Documentation curation

The rollout updates only current-state claims:

- `docs/materials/README.md` gains the accepted status, package/rollout entry
  point, concise build/use guidance, and its missing config-surface design,
  Slice 3, config-surface, and parity plan index entries;
- the v1 design status becomes a concise accepted result with authoritative
  evidence pins;
- Slice 2 and Slice 3 plan headers and README prose record that their
  production commits are ancestors of `materials-26.04`, rather than merely
  naming deleted feature branches;
- the glass config-surface design makes its “not deployed” statement
  explicitly historical; and
- the rollout design and plan own package, rollback, and burn-in procedure.

Historical command blocks, conditional failure paths, result templates, and
the frozen niri-glass documentation are not rewritten. They describe their
original procedures rather than current project status.

After each stale status correction, grep user-facing material docs, including
`docs/materials/material-config.md`, for the same claim. Commit ancestry, not
checked boxes or prose alone, is the evidence for merged status.

## Verification and verdict

Pre-install gates:

- material and dotfiles feature worktrees are clean;
- remote source resolves to the literal accepted commit;
- makepkg completes, including all 293 tests;
- package metadata and installed-file inventory match this design;
- the package archive and build log live below the required work root; and
- the packaged binary validates the proposed dotfiles config.

Post-restart gates:

- the live compositor executable is package-owned `/usr/bin/niri` and its
  version names `138697be`;
- no `niri-glass` Quickshell process or layer namespace beginning with
  `glasspanes` is present;
- Kitty and Ghostty show native material treatment through focus changes,
  movement, overview, workspace switching, close/remap, and valid config
  reload;
- an invalid material edit is rejected while the last valid config remains
  active; and
- the journal contains no unexpected material, shader, renderer, or config
  error.

The rollout remains **in burn-in** until one normal-use session and one cold
start pass. It becomes **deployed** only after those gates and the operator's
visual confirmation are recorded. A crash, corruption, failed reload, missing
material, legacy/native double rendering, or unexpected material warning is a
rollout failure and triggers rollback. The deployment procedure does not fix
production code in place.

## Alternatives rejected

**Side-by-side `niri-material` executable.** This avoids a package conflict but
splits `niri-session`, systemd, IPC commands, config validation, and user PATH
between two binaries. Package replacement gives pacman one owner and one
command surface.

**Package the current documentation head.** It produces the same compositor
but weakens the direct connection to the physically accepted source and
creates a commit-pin self-reference for the in-tree PKGBUILD.

**Apply a patch series over Arch's source tarball.** This mirrors distro
packaging at the cost of maintaining a second representation of the 95-commit
fork. The fork commit is already the reviewed source of truth.

**Build directly with Cargo and copy to `/usr/local`.** This repeats the
unmanaged installation that the rollout is correcting and makes ownership,
inventory, upgrade, and uninstall opaque.

**Keep legacy glass running during native burn-in.** Double rendering makes
visual attribution impossible and preserves the external synchronization path
that native materials replace.
