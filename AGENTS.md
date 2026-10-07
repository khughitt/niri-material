# niri-material — agent guide

A fork of niri carrying the material rendering work. Tasks live in `tasks/`; the `tasks`
skill applies. Designs and plans live under `docs/materials/`, `docs/specs/`, and
`docs/plans/`. Before changing anything under `src/render_helpers/` or the
shaders, read `docs/materials/render-pipeline.md`: it is the pass order every
rendering design assumes.

## Session protocol

- `tasks prime`, then `tasks start <id>` before changing anything; `tasks done <id>` in
  the same commit; `tasks check` before every commit.
- Fresh clone: `git config core.hooksPath .githooks` installs the hooks, including
  the Git LFS handoff.
- .cargo/config.toml and target/ are per-machine and must not reach another
  machine: both carry `user.com.dropbox.ignored=1`, so the shared tree holds neither.
  Without a .cargo/config.toml cargo builds into target/, which is what a fresh
  machine gets. To keep build output on a fast local disk, write your own
  .cargo/config.toml with `build.target-dir` and mark it ignored
  (`setfattr -n user.com.dropbox.ignored -v 1 .cargo`). A `target-dir` naming a path
  that exists on one machine only fails `cargo fmt` before any gate can run, so no
  commit is possible there.
- Every checkout builds into a target dir of its own: that `target-dir` is for the
  main checkout only. A worktree gets no `build.target-dir` or `build.build-dir` (or,
  under a parent config that sets one, both `target-dir = "target"` and
  `build-dir = "target"` in its own .cargo/config.toml) and builds into its own target/, which sits on local storage
  when .worktrees is a work-link symlink. Two checkouts sharing either dir serve each other's workspace
  crates as fresh (cargo keys them by root-relative path and checks them by mtime): a
  "no field" error the source contradicts, or a binary mixing the other tree's crates
  with no error at all. `just setup` after `git worktree add` and `just check` run
  `tools/target-dir-check`, which refuses a worktree that shares; the first build in a
  worktree is a full one.
- Live captures that need an idle host (settle-gated traces, DRM power runs) are parked
  for `tasks quiet`. When the user starts a session from a TTY with the desktop stopped
  and hands the host over, run that queue yourself: one run at a time, the lane's pilot
  before any full run, and a `run:` note for every attempt. The fixture's reproduction
  notes in the `niri-experiments` results doc give the commands and known pitfalls.
  A TTY has no display sleep: run ops's `display-dim status` first (a stale
  record means an earlier session did not restore; restore it),
  `display-dim set 0.25` before the first run and `display-dim restore` after
  the last, before the final report. `display-dim` is on `~/.local/bin` via
  ops's install recipe; when the command is missing, that is a host setup
  fault: say so in the report and file it against ops, never skip silently.
  Dimming never gates a run; a monitor that rejects DDC stays undimmed and
  the report says so.
  The packages and host configuration every capture lane needs (sudoers, udev,
  AUR tools) are recorded in `docs/materials/capture-host-setup.md`.
- Before removing a worktree, run `tt-report` so its fallback test-timing log is
  harvested.
- `tools/tt` is a vendored copy of ops `bin/tt`: change it there and re-copy.
- Before rebasing onto a new upstream release, read `docs/materials/upstream-divergence.md`:
  it carries the baseline, the acknowledged conflicts, and the rebase procedure. Never
  use `git merge-base` against upstream — this repository's history was rewritten and it
  returns a 2023 commit.

## Gates

- While editing Rust: `just test-one -p niri <name-filter>` or
  `just test-one -E 'test(foo) | test(bar)'` (nextest arguments). Then
  `just test-fast`, which selects changed workspace packages and their dependents
  against HEAD, excluding the visual viewer. Neither runs doctests.
- For Python tooling: `just --set one_cmd 'env NIRI_TOOLING_FAST=0 python3 -m unittest' test-one tools.test_gates`;
  `just --set fast_cmd 'env NIRI_TOOLING_FAST=0 python3 -m tools.tooling_tests --full' test-fast`
  runs all tooling in parallel. The Rust affected selector does not select Python.
  Sequential reference discovery uses the same override with
  `env NIRI_TOOLING_FAST=0 python3 -m unittest discover -s tools 2>&1`.
  Raw discovery inherits exported `NIRI_TOOLING_FAST=1` and skips DriverCleanupTests
  and VtLibTests; only unset, `0` and `1` are valid. Repository commands set their mode.
  Fast tooling uses `python3 -m tools.tooling_tests --fast` with four children and
  keeps whole modules in one worker. Full tooling defaults to ten children total;
  both routes are capped by `NEXTEST_TEST_THREADS` when
  set; the cap must be a positive ASCII decimal integer.
- `just check` runs hygiene, target isolation, rustfmt, clippy, fast tooling and
  task/report/pin checks; `just check-full` substitutes parallel full tooling.
  Pre-commit uses full tooling for the narrow lifecycle subjects and gate wiring in
  `tooling_full_paths`, fast tooling for other code, and static source plus consistency
  checks for root guides/tasks/specs/plans/notes. Both code routes record the same
  `hook-pre-commit` target; every route checks lifecycle source coverage statically.
  Material docs and wiki retain code checks because Rust tests read their inputs.
- CI runs the full suite (including doctests) for every branch/tag pushed to `origin`.
  Pre-push there runs `check` plus config and IPC tests through nextest;
  other remotes or unclassified pushes run `just gate` (full check plus Rust tests).
  `just test`, `ci-test` and `ci-test-release` remain Rust-only, including doctests;
  the release job enables randomized/slow tests through environment variables.
  The separate Ubuntu `ci-tooling-test` job runs full parallel tooling, rejecting
  dependency skips and allowing only the five named optional skips in the design.
- Recipes record runs through `tools/tt`. Host recipes use `host-budget run` to set
  `CARGO_BUILD_JOBS` and `NEXTEST_TEST_THREADS`; CI recipes do not need ops tooling.
