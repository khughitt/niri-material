# Native materials v1 daily-driver rollout: design

**Status:** approved 2026-08-28; not implemented.

## Context

Native materials v1 is accepted. The corrected physical DRM run against
production source `138697be4cbb779c80425fe2a366ceca3610f38e` passed all 19
machine gates and all nine physical observations at `niri-experiments` result
commit `c0caa944db2edc5dc4844e6720951d7f32652b76`.

The accepted compositor is not yet the daily-driver installation. Arch's
`niri` package owns `/usr/bin/niri`, while an unmanaged
`/usr/local/bin/niri` at patched source `5e53b949` wins through `PATH`. The
normal niri config still launches the frozen Quickshell `niri-glass` client
for Kitty and Ghostty.

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
| Installed paths | The same paths and resources as Arch's `niri` package |
| Build storage | Required `$NIRI_MATERIAL_WORK_ROOT`; no in-repository or `/tmp` fallback |
| Initial material scope | Kitty and Ghostty, matching the existing legacy allowlist |
| Legacy runtime | Quickshell glass no longer starts normally; its frozen source and inputs remain available for evidence work |
| Restart | Manual graphical-session restart after package and config preflight |
| Completion | One normal-use session and one cold start without a rollout defect |

## Package

`packaging/arch/PKGBUILD` is a narrow adaptation of Arch's niri package. It
keeps Arch's runtime dependencies, optional dependencies, release build,
tests, installed resources, session integration, portal configuration, and
shell completions. The intentional differences are:

- `pkgname=niri-material`;
- `provides=("niri=$pkgver" wayland-compositor)`;
- conflicts with `niri`, `niri-git`, and `niri-bin`;
- source is the project remote pinned with
  `#commit=138697be4cbb779c80425fe2a366ceca3610f38e`; and
- the package version identifies the accepted commit.

Pinning the accepted production commit avoids a self-reference between a
committed `PKGBUILD` and the commit literal inside it. It is also stronger
than tagging the later documentation head: `git diff` confirms that every
change after `138697be` and before this design is under `docs/`, so the
production tree is identical to the candidate exercised on physical DRM.

The package installs `/usr/bin/niri`, `niri-session`, the systemd user units,
Wayland session entry, portal configuration, default config and README, and
the three shell completions. Pacman owns every deployed file; the package does
not modify `/usr/local` from an install hook.

The package `check()` runs the existing workspace test suite. The accepted
baseline is 293 tests: 244 `niri`, 45 `niri-config`, one wiki parse, and three
`niri-ipc` tests.

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
effect; chromatic aberration `0.68` yields six taps.

One rule assigns `terminal-glass` to exact Kitty and Ghostty app IDs. Because
it follows generated Prism rules, it also disables their redundant background
effect while retaining Prism's focus-conditioned opacity. The material owns
the transmitted background; leaving a separate blur/noise effect beneath its
non-opaque render element would spend another pass without contributing the
intended native result.

The main config includes `materials.kdl` and removes
`spawn-at-startup "qs" "-c" "niri-glass"`. The frozen niri-glass checkout,
JSON, setup links, and evidence contracts are retained but dormant. Setup and
health tests stop treating legacy autostart as a required daily-driver
contract.

## Documentation curation

The rollout updates only current-state claims:

- `docs/materials/README.md` gains the accepted status, package/rollout entry
  point, and concise build/use guidance;
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

After each stale status correction, grep user-facing material docs for the
same claim. Commit ancestry, not checked boxes or prose alone, is the evidence
for merged status.

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
- no `niri-glass` Quickshell process or `glasspanes` layer is present;
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
packaging at the cost of maintaining a second representation of the 103-commit
fork. The fork commit is already the reviewed source of truth.

**Build directly with Cargo and copy to `/usr/local`.** This repeats the
unmanaged installation that the rollout is correcting and makes ownership,
inventory, upgrade, and uninstall opaque.

**Keep legacy glass running during native burn-in.** Double rendering makes
visual attribution impossible and preserves the external synchronization path
that native materials replace.
