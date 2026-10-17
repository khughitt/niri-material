# Pre-commit fast route: hermetic git for tooling fixtures

Status: Draft (material-561927).

## 1. Incident

`tt-latency` filed material-561927 at 2026-10-17T14:00:24Z:
`hook-pre-commit-fast` median 54.226 s against the 45 s limit, 28 judged runs
on 6 days. The full route (`hook-pre-commit`) stands at 33.2 s and the docs
route at 5.2 s in the same window; only the fast route breached.

## 2. What the runs show

The fast route held at 33–36 s from 2026-10-15 through 2026-10-17T01:36Z
(487 tooling cases at the last of those runs). From 2026-10-17T11:06Z every
run on the same 487 cases took 53–67 s. The suite did not grow in that gap;
something under it got slower.

At 2026-10-17T06:26 the dotfiles added a global git hook
(`hook.identity-check`, events `pre-commit` and `pre-merge-commit`) that runs
ops's `git-identity-check` in every repository on the host. Config-declared
hooks run beside a repository's own `core.hooksPath`, so they also run in the
throwaway repositories the tooling fixtures build and commit to. The fixtures
scrub inherited `GIT_*` variables but still read the host's global git
config.

Measured on the idle host, 2026-10-17:

| Probe | With the global hook | Hook-free global config |
| --- | ---: | ---: |
| one fixture `git commit` | 0.128 s | 0.010 s |
| `test_upstream_report` (67 cases) | 26.1 s | 7.3 s |
| fast tooling (529 cases, 4 children) | 42.0 s | 23.3 s |

`git-identity-check` costs ~110 ms per call: interpreter start, the
`ops-profile` and `tt` imports, and nine subprocesses. A repository whose
profile is the default (every fixture) is not judged, so for fixtures this is
pure overhead. The other fast-route steps take ~14 s together (clippy 7.6 s,
ops-check 4.1 s, rustfmt 1.4 s, the rest under 1 s each) and did not move.

The hook is the cause: removing it from the fixtures' view takes back the
~19 s the route gained.

## 3. Design

Tooling tests run git hermetically: no inherited `GIT_*` variable, and no
host global or system git config. This is the rule lore-e1c2db adds to the
global instructions and ops-646c0e applies to ops's own suite; CI's tooling
job already runs that way, since its runner has no global config.

1. **One helper.** A new `tools/hermetic_git.py` exposes `isolate(environ)`,
   which deletes every `GIT_*` key, then sets `GIT_CONFIG_GLOBAL` to
   `os.devnull` and `GIT_CONFIG_NOSYSTEM=1`. With the global config pointing
   at `/dev/null`, a fixture's stray `git config --global` fails at once rather
   than writing anywhere. A test that needs a global config of its own (the
   identity-check sandbox) sets `GIT_CONFIG_GLOBAL` in its own subprocess
   environment, as it does today.
2. **Every worker.** The tooling coordinator calls `isolate(os.environ)` in
   each worker before it imports a test module, and in the parent before
   discovery. Every case then runs hermetically whichever bucket it lands in.
3. **Every module that runs git.** The six module-level scrub loops
   (`test_upstream_report`, `test_identity_check`, `test_target_dir_check`,
   `test_package_pin`, `test_capture_meta`, `test_glass_optic_smoke`) become a
   call to `isolate(os.environ)`. `test_gates`, which builds fixture
   repositories with no scrub today, gains the same call. A focused
   `just test-one` run of one module, which bypasses the coordinator, stays
   hermetic too. The call is idempotent.
4. **Tests.** A test of the helper: given an environment carrying `GIT_DIR`,
   `GIT_INDEX_FILE` and a `GIT_CONFIG_GLOBAL` whose file declares a
   `pre-commit` config hook that writes a marker, a fixture commit after
   `isolate` leaves no marker, and no `GIT_*` key but the two it sets
   survives. A coordinator test pins that a worker runs with
   `GIT_CONFIG_GLOBAL=/dev/null` and `GIT_CONFIG_NOSYSTEM=1`.

`test_gates`'s missing scrub is a latent hazard of its own. Today, whether an
inherited `GIT_INDEX_FILE` reaches its fixtures depends on whether a scrubbing
module was imported first in the same worker. Step 3 closes it as part of the
same change.

### Rejected alternatives

- **Disable only `hook.identity-check` in fixtures** (`-c
  hook.identity-check.enabled=false`, or a `GIT_CONFIG_COUNT` override). That
  ties this repository to one ops hook's name, so the next global hook or
  setting brings the incident back, and other host settings keep leaking in.
- **Make `git-identity-check` faster in ops.** That is worth doing for every
  project, and it is filed with ops below, but a Python hook stays several
  times git's own 10 ms per commit, and fixtures should not depend on host
  hooks at all.
- **Rebalance fast-route buckets by measured time.** Buckets are balanced by
  case count. Without the hook they finish at 5.5/19.5/18.7/5.0 s of case
  time, where time balancing would give ~12.3 s each. That would save another
  ~7 s, but it needs a source of case durations and has its own staleness
  question. It is a separate lever, not the cause, so it is filed as a
  follow-up.

## 4. Verification

1. Tooling passes in both modes, with the case count and skip set unchanged:
   `just check` (fast) and `just check-full` (full), both with the global hook
   still installed on the host.
2. The cause is gone: with the hook installed, `test_upstream_report` takes at
   most 9 s and fast tooling at most 26 s on the idle host.
3. Fast route: three staged fast-route hook runs on the idle host, with a
   median of at most 40 s. They use the private-index procedure, with
   divergence-report save and restore, from the 2026-10-07 recurrence plan.
   The projection is ~37 s: about 14 s of non-tooling steps plus ~23 s of
   tooling.
4. `tt-latency verify material-561927 --after <remedy timestamp> --dry-run`
   during acceptance; the single recording verify runs after the final review
   passes, right before closing the task, as in material-5094f2.

Verification counts every `hook-pre-commit-fast` run on the host after the
remedy timestamp, from any checkout. A worktree branched before the remedy
records slow runs until it merges `materials-26.04`. The remedy timestamp is
when the fix lands on `materials-26.04`; open worktrees whose fast-route
commits would pollute the window merge it first, or their runs are named in
the verify note.

## 5. Out of scope

- The 35 s fast-route target from the 2026-10-04 design. This change restores
  the route to its pre-hook ~37 s; the bucket rebalance above is the lever for
  the rest.
- `git-identity-check`'s own cost (ops feedback below), and removing
  `tools/identity-check` (material-e0d6b1).
- The toplevel assertion lore-e1c2db also recommends: it guards against
  writes, not latency, and every fixture here already builds its repository
  under a temporary directory.
- Rust tests: none of them run git.
