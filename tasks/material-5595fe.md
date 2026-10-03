---
id: material-5595fe
title: Move the workstream lanes onto tasks' lane and needs fields
status: todo
priority: 2
size: s
complexity: low
process: direct
created: 2026-10-03T16:18:06Z
updated: 2026-10-03T16:18:06Z
depends: []
tags: []
source: docs/notes/2026-10-02-workstreams-brief.md
agent: claude-code
---

Why: the six workstream lanes and their host gates were recorded with the ad hoc tag `lane` and tags `needs-quiet`/`needs-nested`/`needs-owner`, because tasks had no first-class support. tasks now has lanes, needs and exclusive holds (tasks repo docs/specs/2026-10-03-lanes-needs-groups-design.md; skills/tasks/SKILL.md describes the commands), so `tasks lanes` and `prime` can show this organisation directly; today they show nothing for this project.

Done when:
1. tasks/.config.toml declares a [needs] vocabulary: `quiet` (exclusive = true; meaning: an idle host, a TTY with the desktop stopped), `nested` (a nested compositor on the headless weston unit; exclusive only if that unit can run one capture at a time - check the headless-verification setup before deciding) and `owner` (the owner supplies or judges an image; not exclusive). Declare the vocabulary before adding any need: `--need` refuses undeclared names.
2. The six lane goals carry the field (`tasks edit <id> --lane`) and lose the `lane` tag: material-2834d7, material-5d6b2c, material-6062fd, material-3aa1f2, material-53f873, material-6f606b. Each lane body leads with one guidance paragraph (why the lane exists, then its first milestone), because `tasks lanes`/`prime` show the first paragraph; today several begin "Lane goal (see ...)" - rewrite that opening and keep the rest.
3. Tagged tasks carry the field and lose the tag: `--need quiet` on material-1af3c6, material-7afc31, material-31074f; `--need nested` on material-07bac9; `--need owner` on material-fa4eec. Re-scan for other open tasks whose checks need a quiet host or the owner (the brief's lane table and the `tasks quiet` queue are the starting points) and add needs where clear.
4. Leftovers from the brief's "Pending regrouping": material-be611b goes under material-6062fd if its glass-edges worktree copy no longer blocks the write (otherwise note it on the task); material-77db8a goes under material-53f873 once its claim ends (otherwise leave a note). material-f7eb0b is already done - nothing to do.
5. docs/notes/2026-10-02-workstreams-brief.md says lanes and needs now live in the tracker (`tasks lanes`, `next --under <lane>`, `TASKS_WITHOUT=quiet`), and its "Pending regrouping" section is updated.
6. Check: `tasks check` is clean; `tasks --pretty lanes` lists six lanes, each with guidance and a state; `TASKS_WITHOUT=quiet tasks ready` hides the quiet-bound steps; `tasks list --tag lane` and the three needs-* tag queries return nothing.

Out of scope: setting TASKS_WITHOUT=quiet in the desktop session environment (that is the dotfiles project; mention it in the close note as a suggestion) and any change to the lane cut itself.

Where: tasks/.config.toml, the six lane task records, docs/notes/2026-10-02-workstreams-brief.md. Task-record and config edits only, through the tasks CLI; no worktree needed for task-record maintenance, but .config.toml is a committed file, so commit it with the records.
