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
  through `tools/tt`, so run tests through `just`, not `cargo` directly.
- Fresh clone: `git config core.hooksPath .githooks` installs the hooks. Pre-commit runs
  `just check`; pre-push runs `just gate`. They also run the Git LFS hooks, which
  `core.hooksPath` would otherwise bypass.
- Before removing a worktree, run `tt-report` so its fallback test-timing log is
  harvested.
- `tools/tt` is a vendored copy of ops `bin/tt`: change it there and re-copy.
