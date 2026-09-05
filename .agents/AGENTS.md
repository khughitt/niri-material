## Tasks workflow

- Run `tasks prime` at the start of a work session and `tasks ready` before choosing work.
- Run `tasks start ID` before implementation, add concise notes as evidence changes, and close the task with a one-line result in the same commit as the work.
- Never edit `tasks/*.md` directly; use the `tasks` CLI for every task mutation.
- Before completion, run `tasks check`. Require zero errors and report every warning. Registration-only `unreachable_dep` and `cycle_unverifiable` warnings are environmental on machines without all referenced projects; resolve every other warning.

## Deployment

- Before treating new Niri config syntax emitted by Prism as deployed, update and install `packaging/arch/PKGBUILD` at a commit containing the parser support, then verify the installed `niri` reports that commit.
