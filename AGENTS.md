# niri-material — agent guide

A fork of niri carrying the material rendering work. Tasks live in `tasks/`; the `tasks`
skill applies. Designs and plans live under `docs/materials/`, `docs/specs/`, and
`docs/plans/`. Before changing anything under `src/render_helpers/` or the
shaders, read `docs/materials/render-pipeline.md`: it is the pass order every
rendering design assumes.

## Session protocol

- `tasks prime`, then `tasks start <id>` before changing anything; `tasks done <id>` in
  the same commit; `tasks check` before every commit.
- Tests: `just test` runs the suite CI runs. `just check` runs format, clippy, the
  tooling tests, and `tasks check`; `just gate` is both. Every recipe records its run
  through `tools/tt`, so run tests through `just`, not `cargo` directly. The host recipes
  run under ops `host-budget run`, which sets `CARGO_BUILD_JOBS` and
  `NEXTEST_TEST_THREADS` from the host's CPU budget; the `ci-` recipes do not.
- Fresh clone: `git config core.hooksPath .githooks` installs the hooks. Pre-commit runs
  `just check`; pre-push runs `just gate`. They also run the Git LFS hooks, which
  `core.hooksPath` would otherwise bypass.
- `.cargo/config.toml` and `target/` are per-machine and must not reach another
  machine: both carry `user.com.dropbox.ignored=1`, so the shared tree holds neither.
  Without a `.cargo/config.toml` cargo builds into `target/`, which is what a fresh
  machine gets. To keep build output on a fast local disk, write your own
  `.cargo/config.toml` with `build.target-dir` and mark it ignored
  (`setfattr -n user.com.dropbox.ignored -v 1 .cargo`). A `target-dir` naming a path
  that exists on one machine only fails `cargo fmt` before any gate can run, so no
  commit is possible there.
- Live captures that need an idle host (settle-gated traces, DRM power runs) are parked
  for `tasks quiet`. When the user starts a session from a TTY with the desktop stopped
  and hands the host over, run that queue yourself: one run at a time, the lane's pilot
  before any full run, and a `run:` note for every attempt. The fixture's reproduction
  notes in the `niri-experiments` results doc give the commands and known pitfalls.
- Before removing a worktree, run `tt-report` so its fallback test-timing log is
  harvested.
- `tools/tt` is a vendored copy of ops `bin/tt`: change it there and re-copy.
- Before rebasing onto a new upstream release, read `docs/materials/upstream-divergence.md`:
  it carries the baseline, the acknowledged conflicts, and the rebase procedure. Never
  use `git merge-base` against upstream — this repository's history was rewritten and it
  returns a 2023 commit.
