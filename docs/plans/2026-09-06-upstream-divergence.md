# Upstream Divergence Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give the fork a written account of what it changed relative to upstream niri, a generated inventory that cannot drift, and a weekly canary that reports when upstream movement starts conflicting with our seams.

**Architecture:** One python3 stdlib tool, `tools/upstream-report`, reads a pinned baseline record and emits two generated blocks into `docs/materials/upstream-divergence.md`: a *local* block derived from the git index alone, enforced fresh by `just check`, and a *drift* block derived from a fetched `upstream/main`, refreshed by a new weekly workflow. Every comparison is against the recorded baseline **tree**; ancestry and merge-base are never consulted, because this fork's history was rewritten and its recorded merge base is a 2023 commit.

**Tech Stack:** python3 (stdlib only: `argparse`, `pathlib`, `subprocess`, `tomllib`), git 2.55, `just`, GitHub Actions, `unittest` via `python3 -m unittest discover -s tools`.

**Spec:** `docs/specs/2026-09-06-upstream-divergence-design.md`. Task record: `material-a9447f`.

## Global Constraints

- **Python is stdlib only.** No new dependency reaches `Cargo.toml`, `Cargo.lock`, or a `requirements` file. `tools/upstream-report` follows `tools/test-affected`: `#!/usr/bin/env python3`, a module docstring stating usage and design source, module-level constants, small pure functions, `subprocess` for git.
- **Everything the tool reads comes from the git index**, via `git show :<path>` and `git diff-index --cached`. Never the working tree, never a branch name, never `HEAD`. The one exception is the drift block, which additionally reads a fetched upstream ref and uses `HEAD` for the fork side. Output is written to the working tree.
- **Never call `git merge-base`.** In this repository `git merge-base materials-26.04 v26.04` returns `64214407` (2023-08-14) and a diff from it reports 5743 files against a true 176. Any code path that would fall back to ancestry is a defect.
- **Baseline validation compares trees, never commit identities.** `patched-26.04~2` is `8ed0da44` while `v26.04` is `aece2b0c`; both carry tree `7b010d1b`. Asserting commit equality fails against this fork.
- **Exit statuses.** `git merge-tree`: `0` clean, `1` conflicts, anything else is an error that must abort loudly. The tool itself: `0` success, `1` a reportable finding (stale report, unacknowledged conflict), `2` an error (bad baseline, git failure, missing markers).
- **Real values, pinned at implementation time.** `tag = "v26.04"`, `tag_commit = "aece2b0c4e1fed364f80b8d0083374d921fe710f"`, `tree = "7b010d1b3ab29a1bee76b1554c6bc8eb09300ec6"`. `patched_commit` is `git rev-parse patched-26.04` read at Task 1. Do not copy a truncated hash into the record; record full 40-character hashes.
- Tests live in `tools/test_upstream_report.py` and are found by the existing `python3 -m unittest discover -s tools`, which `just check` already runs. Follow `tools/test_affected.py`: load the hyphenated script through `importlib.util.spec_from_loader` + `SourceFileLoader`.
- Fixture repositories are built under `tempfile.mkdtemp()` and removed in `tearDown`. Each fixture sets `user.email`/`user.name` locally and passes `-c commit.gpgsign=false`, so the suite does not depend on the operator's git config.
- Commits pass the pre-commit hook (`just check`: rustfmt, clippy, tooling tests, `tasks check`). Conventional commits with scopes `feat(tools)`, `test(tools)`, `docs(material)`, `build(just)`, `ci`. No AI attribution trailer.
- Task lifecycle: every `### Task N` heading has a record linked with `plan:` and `step:`. Run `tasks start <id>` before a task. Task 7 closes the parent `material-a9447f`.
- **Closing sequence, in this exact order, for every task.** Getting it wrong makes the task's own commit fail its own gate:

  1. `tasks done <id> "<result>"` — this REWRITES a file under `tasks/`, which is a class-A path in the inventory, so it must happen before anything is staged.
  2. `git add` every input: source, config, `tasks/`.
  3. `python3 tools/upstream-report` — regenerate, now that the index is final.
  4. `git add docs/materials/upstream-divergence.md`.
  5. `git commit`.

  Steps 3-4 are no-ops before Task 5 creates the document, and harmless. From Task 6 on, `just check` runs `upstream-report --check` in the pre-commit hook, and skipping them makes the commit fail with `docs/materials/upstream-divergence.md is stale`.

| Task | Record | | Task | Record |
|---|---|---|---|---|
| 1 | `material-a26c7b` | | 5 | `material-46c3be` |
| 2 | `material-7d0d29` | | 6 | `material-331614` |
| 3 | `material-13b933` | | 7 | `material-d467fb` |
| 4 | `material-e62c55` | | | |

## File structure

| File | Responsibility |
| --- | --- |
| `docs/materials/upstream-baseline.toml` | the pinned baseline: tag, tag commit, release tree, patched commit, carried patch-ids |
| `docs/materials/upstream-conflicts.toml` | hand-maintained conflict acknowledgments, keyed by path |
| `docs/materials/upstream-divergence.md` | the document: prose classes and procedure, plus two generated blocks |
| `tools/upstream-report` | baseline resolution, inventory, drift, freshness check |
| `tools/test_upstream_report.py` | unit tests with git fixture repositories |
| `justfile` | `upstream-report` recipe; `check_cmd` gains the freshness check |
| `.github/workflows/upstream-drift.yml` | weekly drift canary and PR-time lightweight checks |
| `docs/materials/README.md`, `AGENTS.md` | index entry and the pointer agents read |

---

### Task 1: Baseline record and validation

**Files:**
- Create: `docs/materials/upstream-baseline.toml`
- Create: `tools/upstream-report`
- Create: `tools/test_upstream_report.py`

**Interfaces:**
- Consumes: nothing.
- Produces: `ReportError(Exception)`; `git(root, *args) -> str`; `rev(root, spec) -> str`; `read_index(root, path) -> str`; `patch_id(root, commit) -> str`; `load_baseline(root) -> dict`; `resolve_baseline(root, record) -> dict`; module constants `BASELINE_PATH`, `CONFLICTS_PATH`, `REPORT_PATH`, `SELF_PATHS`.

- [ ] **Step 1: Write the failing test**

Create `tools/test_upstream_report.py`:

```python
"""Unit tests for tools/upstream-report: `python3 -m unittest discover -s tools`."""
import importlib.machinery
import importlib.util
import pathlib
import shutil
import subprocess
import tempfile
import textwrap
import unittest

spec = importlib.util.spec_from_loader(
    "upstream_report",
    importlib.machinery.SourceFileLoader(
        "upstream_report", str(pathlib.Path(__file__).with_name("upstream-report"))
    ),
)
report = importlib.util.module_from_spec(spec)
spec.loader.exec_module(report)


class Fixture(unittest.TestCase):
    """A throwaway git repository whose history is rewritten the way the fork's is."""

    def setUp(self):
        self.root = pathlib.Path(tempfile.mkdtemp())
        self.git("init", "-q", "-b", "main")
        self.git("config", "user.email", "fixture@example.invalid")
        self.git("config", "user.name", "Fixture")
        self.git("config", "commit.gpgsign", "false")

    def tearDown(self):
        shutil.rmtree(self.root, ignore_errors=True)

    def git(self, *args):
        result = subprocess.run(
            ["git", "-C", str(self.root), *args], capture_output=True, text=True, check=True
        )
        return result.stdout

    def write(self, path, text):
        target = self.root / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(textwrap.dedent(text).lstrip("\n"))

    def commit(self, message):
        self.git("add", "-A")
        self.git("commit", "-q", "-m", message)
        return self.git("rev-parse", "HEAD").strip()

    def baseline_repo(self):
        """Build the fork's shape: a release commit, a rewritten copy of it with a
        different SHA and an identical tree, and two carried patches on the copy.

        Returns (tag_commit, tree, patched_commit).
        """
        self.write("src/lib.rs", "fn upstream() {}\n")
        tag_commit = self.commit("release")
        self.git("tag", "v1.0")
        tree = self.git("rev-parse", "v1.0^{tree}").strip()

        # The rewritten copy: same tree, different commit object.
        copy = self.git(
            "commit-tree", tree, "-m", "release (rewritten)",
        ).strip()
        self.git("checkout", "-q", copy)
        self.write("carried_a.rs", "fn a() {}\n")
        self.commit("carry a")
        self.write("carried_b.rs", "fn b() {}\n")
        patched_commit = self.commit("carry b")
        self.assertNotEqual(copy, tag_commit)
        return tag_commit, tree, patched_commit

    def write_baseline(self, tag_commit, tree, patched_commit, carried):
        entries = "\n".join(
            f'[[carried]]\nsubject = "{s}"\npatch_id = "{p}"\n' for s, p in carried
        )
        self.write(
            report.BASELINE_PATH,
            f"""
            tag = "v1.0"
            tag_commit = "{tag_commit}"
            tree = "{tree}"
            patched_commit = "{patched_commit}"

            {entries}
            """,
        )
        self.git("add", report.BASELINE_PATH)

    def carried_ids(self, patched_commit, n):
        """Patch-ids of the n carried commits, oldest first."""
        return [
            (
                self.git("log", "-1", "--format=%s", f"{patched_commit}~{d}").strip(),
                report.patch_id(self.root, f"{patched_commit}~{d}"),
            )
            for d in reversed(range(n))
        ]


class Baseline(Fixture):
    def test_rewritten_ancestry_with_identical_tree_validates(self):
        tag_commit, tree, patched = self.baseline_repo()
        self.write_baseline(tag_commit, tree, patched, self.carried_ids(patched, 2))
        record = report.resolve_baseline(self.root, report.load_baseline(self.root))
        self.assertEqual(record["tree"], tree)

    def test_merge_base_is_never_consulted(self):
        tag_commit, tree, patched = self.baseline_repo()
        self.write_baseline(tag_commit, tree, patched, self.carried_ids(patched, 2))
        merge_base = subprocess.run(
            ["git", "-C", str(self.root), "merge-base", patched, tag_commit],
            capture_output=True, text=True,
        )
        self.assertNotEqual(merge_base.returncode, 0)  # unrelated histories
        report.resolve_baseline(self.root, report.load_baseline(self.root))

    def test_fork_tree_mismatch_fails(self):
        tag_commit, tree, patched = self.baseline_repo()
        self.git("checkout", "-q", patched)
        self.write("src/lib.rs", "fn drifted() {}\n")
        drifted = self.commit("drift the baseline tree")
        self.write_baseline(tag_commit, tree, drifted, self.carried_ids(patched, 2))
        with self.assertRaises(report.ReportError) as caught:
            report.resolve_baseline(self.root, report.load_baseline(self.root))
        self.assertIn(tree[:8], str(caught.exception))

    def test_tag_commit_tree_mismatch_fails(self):
        tag_commit, tree, patched = self.baseline_repo()
        self.write_baseline(tag_commit, "0" * 40, patched, self.carried_ids(patched, 2))
        with self.assertRaises(report.ReportError):
            report.resolve_baseline(self.root, report.load_baseline(self.root))

    def test_tag_moved_off_tag_commit_fails(self):
        tag_commit, tree, patched = self.baseline_repo()
        self.git("tag", "-f", "v1.0", patched)
        self.write_baseline(tag_commit, tree, patched, self.carried_ids(patched, 2))
        with self.assertRaises(report.ReportError):
            report.resolve_baseline(self.root, report.load_baseline(self.root))

    def test_wrong_carried_patch_ids_fail(self):
        tag_commit, tree, patched = self.baseline_repo()
        carried = [(s, "0" * 40) for s, _ in self.carried_ids(patched, 2)]
        self.write_baseline(tag_commit, tree, patched, carried)
        with self.assertRaises(report.ReportError):
            report.resolve_baseline(self.root, report.load_baseline(self.root))

    def test_resolution_needs_no_branches(self):
        """A fresh CI checkout has no local patched-26.04, so nothing may resolve by
        branch name.

        `git branch --format` is NOT usable here: baseline_repo leaves HEAD detached,
        and it then prints `(HEAD detached at abc1234)`, which splits into the tokens
        `(HEAD`, `detached`, `at`, `abc1234)`. for-each-ref lists refs only."""
        tag_commit, tree, patched = self.baseline_repo()
        self.write_baseline(tag_commit, tree, patched, self.carried_ids(patched, 2))
        branches = self.git(
            "for-each-ref", "--format=%(refname:short)", "refs/heads/"
        ).split()
        for branch in branches:
            self.git("branch", "-D", branch)
        self.assertEqual(
            self.git("for-each-ref", "--format=%(refname:short)", "refs/heads/").strip(),
            "",
        )
        report.resolve_baseline(self.root, report.load_baseline(self.root))


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `python3 -m unittest discover -s tools -v`
Expected: FAIL — `FileNotFoundError` or `AttributeError`, because `tools/upstream-report` does not exist.

- [ ] **Step 3: Write `tools/upstream-report`**

```python
#!/usr/bin/env python3
"""Upstream divergence report: `python3 tools/upstream-report [--check] [--drift] [--against REF]`.

Reports what this fork changes relative to upstream niri, and whether upstream has
moved into our seams. Everything is measured against the baseline recorded in
docs/materials/upstream-baseline.toml, comparing TREES: this repository's history was
rewritten, its recorded merge base is a 2023 commit, and a diff from that merge base
reports 5743 files against a true 176. `git merge-base` is never called.

All input is read from the git index (`git show :<path>`, `git diff-index --cached`),
never the working tree and never a branch name, so one code path serves both the
pre-commit hook (where the index is the tree about to be committed) and CI (where it
is the checked-out commit). Output is written to the working tree.

Exit status: 0 success, 1 a reportable finding (stale report, unacknowledged
conflict), 2 an error (bad baseline, git failure, missing markers).

Design: docs/specs/2026-09-06-upstream-divergence-design.md.
"""
import argparse
import collections
import datetime
import pathlib
import subprocess
import sys
import tomllib

BASELINE_PATH = "docs/materials/upstream-baseline.toml"
CONFLICTS_PATH = "docs/materials/upstream-conflicts.toml"
REPORT_PATH = "docs/materials/upstream-divergence.md"
SELF_PATHS = (BASELINE_PATH, CONFLICTS_PATH, REPORT_PATH)

REQUIRED_KEYS = ("tag", "tag_commit", "tree", "patched_commit")


class ReportError(Exception):
    """Anything that must stop the run loudly rather than degrade into a guess."""


def git(root, *args):
    """Run git, returning stdout. Any non-zero status is a ReportError."""
    result = subprocess.run(
        ["git", "-C", str(root), *args], capture_output=True, text=True
    )
    if result.returncode != 0:
        raise ReportError(
            f"git {' '.join(args)} failed ({result.returncode}): "
            f"{result.stderr.strip() or result.stdout.strip()}"
        )
    return result.stdout


def rev(root, spec):
    return git(root, "rev-parse", "--verify", spec).strip()


def read_index(root, path):
    """File content as staged in the index."""
    return git(root, "show", f":{path}")


def patch_id(root, commit):
    """Stable patch-id of a single commit."""
    patch = git(root, "diff-tree", "-p", "--no-color", commit)
    result = subprocess.run(
        ["git", "-C", str(root), "patch-id", "--stable"],
        input=patch, capture_output=True, text=True,
    )
    if result.returncode != 0 or not result.stdout.split():
        raise ReportError(f"git patch-id --stable produced nothing for {commit}")
    return result.stdout.split()[0]


def _hash(value):
    return len(value) == 40 and all(c in "0123456789abcdef" for c in value)


def load_baseline(root):
    """The baseline record, as staged, with every field's SHAPE validated.

    Valid TOML is not a valid record. `carried = 1` parses fine and then explodes on
    `len()` with a traceback and the wrong exit code; a truncated hash pasted from
    the design doc validates against nothing. Both are caught here, as ReportError,
    so they reach the error boundary and exit 2."""
    record = tomllib.loads(read_index(root, BASELINE_PATH))
    if not isinstance(record, dict):
        raise ReportError(f"{BASELINE_PATH}: not a table")

    missing = [key for key in REQUIRED_KEYS if key not in record]
    if missing:
        raise ReportError(f"{BASELINE_PATH}: missing {', '.join(missing)}")
    for key in REQUIRED_KEYS:
        if not isinstance(record[key], str) or not record[key]:
            raise ReportError(
                f"{BASELINE_PATH}: {key} must be a non-empty string, "
                f"got {record[key]!r}"
            )
    for key in ("tag_commit", "tree", "patched_commit"):
        if not _hash(record[key]):
            raise ReportError(
                f"{BASELINE_PATH}: {key} must be a full 40-character hex hash, "
                f"got {record[key]!r}"
            )

    carried = record.setdefault("carried", [])
    if not isinstance(carried, list):
        raise ReportError(
            f"{BASELINE_PATH}: carried must be an array of tables, got {carried!r}"
        )
    for index, entry in enumerate(carried):
        if not isinstance(entry, dict):
            raise ReportError(
                f"{BASELINE_PATH}: carried[{index}] must be a table, got {entry!r}"
            )
        for key in ("subject", "patch_id"):
            if not isinstance(entry.get(key), str) or not entry[key]:
                raise ReportError(
                    f"{BASELINE_PATH}: carried[{index}].{key} must be a "
                    f"non-empty string"
                )
    return record


def resolve_baseline(root, record):
    """Validate the record against the object store. Trees on both sides; never
    commit identity, because the fork's copy of the release commit is a different
    object with the same content."""
    tree = record["tree"]
    tag_commit = record["tag_commit"]

    resolved = rev(root, f"{record['tag']}^{{commit}}")
    if resolved != tag_commit:
        raise ReportError(
            f"tag {record['tag']} resolves to {resolved}, "
            f"recorded tag_commit is {tag_commit}"
        )

    tag_tree = rev(root, f"{tag_commit}^{{tree}}")
    if tag_tree != tree:
        raise ReportError(
            f"tree of tag_commit {tag_commit} is {tag_tree}, recorded tree is {tree}"
        )

    carried = record["carried"]
    depth = len(carried)
    fork_tree = rev(root, f"{record['patched_commit']}~{depth}^{{tree}}")
    if fork_tree != tree:
        raise ReportError(
            f"tree of {record['patched_commit']}~{depth} is {fork_tree}, "
            f"recorded tree is {tree}"
        )

    # carried is oldest-first; the oldest sits at depth-1 below patched_commit.
    for index, entry in enumerate(carried):
        commit = f"{record['patched_commit']}~{depth - 1 - index}"
        found = patch_id(root, commit)
        if found != entry.get("patch_id"):
            raise ReportError(
                f"carried[{index}] ({entry.get('subject', '?')}): patch-id at "
                f"{commit} is {found}, recorded {entry.get('patch_id')}"
            )
    return record


def run(root, args):
    """The whole job. Returns 0 for success and 1 for a finding; every operational
    failure raises and is caught by the boundary in main()."""
    record = resolve_baseline(root, load_baseline(root))
    if args.show_baseline:
        print(f"{record['tag']} {record['tag_commit']} tree {record['tree']}")
        print(f"patched {record['patched_commit']} carried {len(record['carried'])}")
    return 0


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--show-baseline", action="store_true",
                        help="resolve and print the baseline, then exit")
    args = parser.parse_args(argv)
    # ONE error boundary. Every expected operational failure — a git error, a bad
    # baseline, malformed TOML, an unreadable file — exits 2. Exit 1 is reserved for
    # findings, so CI can tell "the check found something" from "the check broke".
    try:
        root = pathlib.Path(
            git(pathlib.Path.cwd(), "rev-parse", "--show-toplevel").strip()
        )
        return run(root, args)
    except (ReportError, tomllib.TOMLDecodeError, OSError) as error:
        print(f"upstream-report: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `python3 -m unittest discover -s tools -v`
Expected: PASS, 7 tests in `Baseline` plus the existing `test-affected` suite.

- [ ] **Step 5: Write the real baseline record**

Read the live values and write `docs/materials/upstream-baseline.toml`. Compute, do not transcribe from this plan:

```bash
cd "$(git rev-parse --show-toplevel)"
git rev-parse v26.04^{commit}   # expect aece2b0c4e1fed364f80b8d0083374d921fe710f
git rev-parse v26.04^{tree}     # expect 7b010d1b3ab29a1bee76b1554c6bc8eb09300ec6
git rev-parse patched-26.04
for d in 1 0; do
  git log -1 --format='%s' "patched-26.04~$d"
  git diff-tree -p --no-color "patched-26.04~$d" | git patch-id --stable | cut -d' ' -f1
done
```

Write the file with full 40-character hashes:

```toml
# The upstream baseline this fork is measured against.
#
# Validation compares TREES, never commit identities: patched-26.04~2 is a different
# commit object from v26.04 carrying the identical tree, because this repository's
# history was rewritten. See docs/specs/2026-09-06-upstream-divergence-design.md.
#
# carried is oldest-first. Its length is the depth at which the baseline tree must
# appear below patched_commit.

tag = "v26.04"
tag_commit = "aece2b0c4e1fed364f80b8d0083374d921fe710f"
tree = "7b010d1b3ab29a1bee76b1554c6bc8eb09300ec6"
patched_commit = "<git rev-parse patched-26.04>"

[[carried]]
subject = "<subject of patched-26.04~1>"
patch_id = "<its stable patch-id>"

[[carried]]
subject = "<subject of patched-26.04~0>"
patch_id = "<its stable patch-id>"
```

- [ ] **Step 6: Verify the tool validates the real repository**

Run: `git add docs/materials/upstream-baseline.toml && python3 tools/upstream-report --show-baseline`
Expected: exit 0, two lines naming `v26.04`, `aece2b0c…`, tree `7b010d1b…`, and `carried 2`.

Then confirm the failure path is real:

Run: `git stash && python3 tools/upstream-report --show-baseline; git stash pop`
Expected: exit 2 with `upstream-report: git show :docs/materials/upstream-baseline.toml failed`.

- [ ] **Step 7: Commit**

```bash
chmod +x tools/upstream-report
tasks done material-a26c7b "baseline record and tree-based validation; fixture tests including the rewritten-ancestry case"
git add tools/upstream-report tools/test_upstream_report.py docs/materials/upstream-baseline.toml tasks/
git commit -m "feat(tools): resolve the upstream baseline by tree, not ancestry"
```

The document does not exist yet, so there is nothing to regenerate. From Task 5 on, the full closing sequence in Global Constraints applies.

---

### Task 2: Index-backed seam inventory

**Files:**
- Modify: `tools/upstream-report`
- Modify: `tools/test_upstream_report.py`

**Interfaces:**
- Consumes: `ReportError`, `git`, `read_index`, `resolve_baseline`, `SELF_PATHS` from Task 1.
- Produces: `classify(status, path) -> str`; `Row = namedtuple("Row", "status path previous added removed")`; `inventory(root, tree) -> list[Row]` sorted by path; `render_local(record, rows) -> str`; constants `SCAFFOLD_PREFIXES`, `SCAFFOLD_FILES`.

- [ ] **Step 1: Write the failing test**

Append to `tools/test_upstream_report.py`:

```python
class Classify(unittest.TestCase):
    def test_modified_upstream_files_are_class_b(self):
        self.assertEqual(report.classify("M", "src/layout/tile.rs"), "B")
        self.assertEqual(report.classify("M", ".github/workflows/ci.yml"), "B")

    def test_deleted_and_renamed_upstream_files_are_class_b(self):
        self.assertEqual(report.classify("D", "src/gone.rs"), "B")
        self.assertEqual(report.classify("R100", "src/moved.rs"), "B")

    def test_added_scaffolding_is_class_c(self):
        for path in ("tools/tt", ".githooks/pre-commit", "packaging/arch/PKGBUILD",
                     ".agents/x.md", "justfile", "AGENTS.md"):
            self.assertEqual(report.classify("A", path), "C", path)

    def test_other_additions_are_class_a(self):
        self.assertEqual(report.classify("A", "src/render_helpers/material.rs"), "A")
        self.assertEqual(report.classify("A", "docs/materials/render-pipeline.md"), "A")


class Inventory(Fixture):
    def stage_divergence(self):
        """A baseline plus one modified upstream file, one added source file, one
        added scaffold file, and the tool's own files."""
        tag_commit, tree, patched = self.baseline_repo()
        self.write_baseline(tag_commit, tree, patched, self.carried_ids(patched, 2))
        self.write("src/lib.rs", "fn upstream() { changed(); }\n")
        self.write("src/material.rs", "fn material() {}\n")
        self.write("tools/tt", "#!/bin/sh\n")
        self.write(report.REPORT_PATH, "placeholder\n")
        self.write(report.CONFLICTS_PATH, "acknowledged = []\n")
        self.git("add", "-A")
        return tree

    def test_inventory_reports_status_path_and_line_counts(self):
        tree = self.stage_divergence()
        rows = report.inventory(self.root, tree)
        by_path = {row.path: row for row in rows}
        self.assertEqual(by_path["src/lib.rs"].status, "M")
        self.assertEqual(by_path["src/material.rs"].status, "A")
        self.assertEqual(by_path["tools/tt"].status, "A")
        self.assertEqual(by_path["src/material.rs"].added, 1)
        self.assertEqual(by_path["src/material.rs"].removed, 0)
        self.assertIsNone(by_path["src/material.rs"].previous)

    def test_inventory_excludes_the_tools_own_files(self):
        tree = self.stage_divergence()
        paths = [row.path for row in report.inventory(self.root, tree)]
        for own in report.SELF_PATHS:
            self.assertNotIn(own, paths)

    def test_inventory_reads_the_index_not_the_working_tree(self):
        tree = self.stage_divergence()
        (self.root / "src" / "unstaged.rs").write_text("fn unstaged() {}\n")
        paths = [row.path for row in report.inventory(self.root, tree)]
        self.assertNotIn("src/unstaged.rs", paths)

    def test_rename_joins_both_diff_formats_on_the_new_path(self):
        """--name-status gives `R100\0old\0new`; --numstat gives `0\t0\t\0old\0new`.
        Joining them naively yields the nonexistent path `src/{old.rs => new.rs}`."""
        tag_commit, tree, patched = self.baseline_repo()
        self.write_baseline(tag_commit, tree, patched, self.carried_ids(patched, 2))
        self.git("mv", "src/lib.rs", "src/renamed.rs")
        self.git("add", "-A")
        rows = report.inventory(self.root, tree)
        by_path = {row.path: row for row in rows}
        self.assertIn("src/renamed.rs", by_path)
        self.assertFalse([p for p in by_path if "=>" in p or "{" in p], list(by_path))
        row = by_path["src/renamed.rs"]
        self.assertTrue(row.status.startswith("R"), row.status)
        self.assertEqual(report.classify(row.status, row.path), "B")
        # The baseline path survives, so drift can ask upstream about the old name.
        self.assertEqual(row.previous, "src/lib.rs")
        self.assertEqual(report.seam_paths(rows)["src/renamed.rs"], "src/lib.rs")

    def test_disagreeing_diff_formats_are_an_error(self):
        tag_commit, tree, patched = self.baseline_repo()
        self.write_baseline(tag_commit, tree, patched, self.carried_ids(patched, 2))
        self.git("add", "-A")
        original = report._numstat
        try:
            report._numstat = lambda root, tree: {}
            with self.assertRaises(report.ReportError):
                report.inventory(self.root, tree)
        finally:
            report._numstat = original

    def test_inventory_is_sorted_by_path(self):
        tree = self.stage_divergence()
        paths = [row.path for row in report.inventory(self.root, tree)]
        self.assertEqual(paths, sorted(paths))

    def test_churn_follows_the_baseline_path_across_a_fork_rename(self):
        """The fork renamed src/lib.rs; upstream kept editing it under the old name.
        Asking upstream about the fork's new name returns zero and hides the churn."""
        tag_commit, tree, patched = self.baseline_repo()
        self.write_baseline(tag_commit, tree, patched, self.carried_ids(patched, 2))

        # Upstream continues from the release, editing the file under its old name.
        self.git("checkout", "-q", "-b", "upstream-main", tag_commit)
        self.write("src/lib.rs", "fn upstream() { moved_on(); }\n")
        self.commit("upstream edits src/lib.rs")

        self.git("checkout", "-q", patched)
        self.git("mv", "src/lib.rs", "src/renamed.rs")
        self.git("add", "-A")

        watched = report.seam_paths(report.inventory(self.root, tree))
        self.assertEqual(watched["src/renamed.rs"], "src/lib.rs")
        churns = report.churn(self.root, tag_commit, "upstream-main", watched)
        count, letter, baseline, upstream_path = churns["src/renamed.rs"]
        self.assertEqual(baseline, "src/lib.rs")
        self.assertEqual(count, 1, "upstream churn must follow the baseline path")

    def test_local_block_counts_classes(self):
        tree = self.stage_divergence()
        record = report.resolve_baseline(self.root, report.load_baseline(self.root))
        body = report.render_local(record, report.inventory(self.root, tree))
        self.assertIn("| `src/lib.rs` | B |", body)
        self.assertIn("| `tools/tt` | C |", body)
        self.assertNotIn("src/material.rs", body)  # class A is counted, not listed
        self.assertIn("v1.0", body)
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `python3 -m unittest discover -s tools -v`
Expected: FAIL with `AttributeError: module 'upstream_report' has no attribute 'classify'`.

- [ ] **Step 3: Implement classification, inventory, and the local block**

Add to `tools/upstream-report`, after `read_index`:

```python
SCAFFOLD_PREFIXES = ("tools/", ".githooks/", "packaging/", ".agents/")
SCAFFOLD_FILES = ("justfile", "AGENTS.md")

# `previous` is the baseline path for a rename, else None.
Row = collections.namedtuple("Row", "status path previous added removed")


def classify(status, path):
    """A: fork-only additions. B: seam changes to files upstream also has.
    C: vendored tooling and project scaffolding.

    Class is an inventory aid, not a conflict filter: conflict detection runs over
    the whole tree and is never scoped by class."""
    if status[0] in ("M", "D", "R"):
        return "B"
    if path.startswith(SCAFFOLD_PREFIXES) or path in SCAFFOLD_FILES:
        return "C"
    return "A"


def _split_z(text):
    """NUL-delimited fields, without the trailing empty one."""
    fields = text.split("\0")
    if fields and fields[-1] == "":
        fields.pop()
    return fields


def _name_status(root, tree):
    """path -> (status, previous path or None).

    Records are `<status>\0<path>`, except a rename or copy, which is
    `R<score>\0<old>\0<new>`. Record length depends on the status letter, which is
    why this cannot be parsed line-wise."""
    fields = _split_z(
        git(root, "diff-index", "--cached", "--name-status", "-M", "-z", tree)
    )
    result = {}
    index = 0
    while index < len(fields):
        status = fields[index]
        if status[:1] in ("R", "C"):
            result[fields[index + 2]] = (status, fields[index + 1])
            index += 3
        else:
            result[fields[index + 1]] = (status, None)
            index += 2
    return result


def _numstat(root, tree):
    """path -> (added, removed).

    Records are `<added>\t<removed>\t<path>`, except a rename, where the third
    tab-separated field is EMPTY and the old and new paths follow as two further
    NUL-terminated fields. This encoding differs from --name-status, so the two must
    be joined on the new path rather than zipped."""
    fields = _split_z(
        git(root, "diff-index", "--cached", "--numstat", "-M", "-z", tree)
    )
    result = {}
    index = 0
    while index < len(fields):
        added, removed, path = fields[index].split("\t", 2)
        if path == "":
            path = fields[index + 2]  # old at index + 1, new at index + 2
            index += 3
        else:
            index += 1
        # "-" is git's marker for a binary file.
        result[path] = (
            0 if added == "-" else int(added),
            0 if removed == "-" else int(removed),
        )
    return result


def inventory(root, tree):
    """Rows for every path differing between the baseline tree and the index,
    excluding the tool's own files, sorted by path.

    `previous` is the path the baseline knew, or None. It is carried all the way to
    the drift analysis on purpose: upstream never saw our rename, so a seam file the
    fork renamed must be looked up upstream under its OLD name. Dropping it reported
    "0 upstream commits" for a file upstream was actively editing.

    A key present in one diff format and not the other is an ERROR, never a default.
    An earlier draft used `statuses.get(path, "M")`, and that silent fallback hid a
    real bug: the two formats encode renames differently, so a rename produced the
    nonexistent path `src/{old.rs => new.rs}` with a fabricated "M" status."""
    statuses = _name_status(root, tree)
    numbers = _numstat(root, tree)
    if set(statuses) != set(numbers):
        raise ReportError(
            "diff-index --name-status and --numstat disagree on paths: "
            f"{sorted(set(statuses) ^ set(numbers))[:5]}"
        )
    rows = [
        Row(statuses[path][0], path, statuses[path][1], *numbers[path])
        for path in statuses
        if path not in SELF_PATHS
    ]
    return sorted(rows, key=lambda row: row.path)


def render_local(record, rows):
    """The generated local block: derived from the index alone, no network."""
    counts = {"A": 0, "B": 0, "C": 0}
    for row in rows:
        counts[classify(row.status, row.path)] += 1

    lines = [
        f"Baseline `{record['tag']}` (`{record['tag_commit'][:12]}`), "
        f"tree `{record['tree'][:12]}`, carrying {len(record['carried'])} patch(es).",
        "",
        f"{len(rows)} paths differ: {counts['B']} class B (seam), "
        f"{counts['A']} class A (fork-only), {counts['C']} class C (scaffolding).",
        "",
        "| Path | Class | Status | +/- |",
        "| --- | --- | --- | --- |",
    ]
    for row in rows:
        kind = classify(row.status, row.path)
        if kind == "A":
            continue
        renamed = f" (was `{row.previous}`)" if row.previous else ""
        lines.append(
            f"| `{row.path}`{renamed} | {kind} | {row.status} "
            f"| +{row.added}/-{row.removed} |"
        )
    lines.append("")
    lines.append(
        "Class A paths are counted, not listed: fork-only additions with no "
        "upstream counterpart. Class C is listed because `tools/tt` has an "
        "external source of truth in ops."
    )
    return "\n".join(lines)
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `python3 -m unittest discover -s tools -v`
Expected: PASS.

- [ ] **Step 5: Sanity-check against the real repository**

Run:

```bash
python3 - <<'PY'
import importlib.machinery, importlib.util, pathlib, subprocess
s = importlib.util.spec_from_loader("r", importlib.machinery.SourceFileLoader("r", "tools/upstream-report"))
r = importlib.util.module_from_spec(s); s.loader.exec_module(r)
root = pathlib.Path(".")
rec = r.resolve_baseline(root, r.load_baseline(root))
rows = r.inventory(root, rec["tree"])
from collections import Counter
print(Counter(r.classify(row.status, row.path) for row in rows))
PY
```

Expected: class B is **38** and total is **176** minus whichever of the tool's own files are staged. If B is not 38, the classification or the `-M` handling is wrong — stop and fix before continuing.

- [ ] **Step 6: Commit**

```bash
tasks done material-7d0d29 "index-backed inventory with -z rename-safe parsing; class B reproduces the measured 38"
git add tools/upstream-report tools/test_upstream_report.py tasks/
git commit -m "feat(tools): inventory the seam from the index against the baseline tree"
```

---

### Task 3: Marker splice and the `--check` freshness contract

**Files:**
- Modify: `tools/upstream-report`
- Modify: `tools/test_upstream_report.py`

**Interfaces:**
- Consumes: everything from Tasks 1-2.
- Produces: `splice(text, begin, end, body) -> str`; `render_report(root, record, text) -> str`; `check(root, record) -> list[str]`; constants `LOCAL_BEGIN`, `LOCAL_END`, `DRIFT_BEGIN`, `DRIFT_END`.

- [ ] **Step 1: Write the failing test**

Append to `tools/test_upstream_report.py`:

```python
class Splice(unittest.TestCase):
    def document(self, body):
        return f"# Doc\n\n{report.LOCAL_BEGIN}\n{body}\n{report.LOCAL_END}\n\ntail\n"

    def test_splice_replaces_only_between_markers(self):
        out = report.splice(self.document("old"), report.LOCAL_BEGIN, report.LOCAL_END, "new")
        self.assertIn("# Doc", out)
        self.assertIn("tail", out)
        self.assertIn("new", out)
        self.assertNotIn("old", out)

    def test_splice_is_idempotent(self):
        once = report.splice(self.document("old"), report.LOCAL_BEGIN, report.LOCAL_END, "new")
        twice = report.splice(once, report.LOCAL_BEGIN, report.LOCAL_END, "new")
        self.assertEqual(once, twice)

    def test_missing_markers_are_an_error(self):
        with self.assertRaises(report.ReportError):
            report.splice("# Doc\nno markers\n", report.LOCAL_BEGIN, report.LOCAL_END, "new")

    def test_markers_out_of_order_are_an_error(self):
        text = f"{report.LOCAL_END}\n{report.LOCAL_BEGIN}\n"
        with self.assertRaises(report.ReportError):
            report.splice(text, report.LOCAL_BEGIN, report.LOCAL_END, "new")


class Freshness(Fixture):
    def stage_document(self, body="stale"):
        tag_commit, tree, patched = self.baseline_repo()
        self.write_baseline(tag_commit, tree, patched, self.carried_ids(patched, 2))
        self.write("src/lib.rs", "fn upstream() { changed(); }\n")
        self.write(report.CONFLICTS_PATH, "acknowledged = []\n")
        self.write(
            report.REPORT_PATH,
            f"# Upstream divergence\n\n{report.LOCAL_BEGIN}\n{body}\n{report.LOCAL_END}\n",
        )
        self.git("add", "-A")
        return tree

    def regenerate(self, record):
        target = self.root / report.REPORT_PATH
        target.write_text(report.render_report(self.root, record, target.read_text()))

    def test_stale_report_is_a_finding(self):
        self.stage_document()
        record = report.resolve_baseline(self.root, report.load_baseline(self.root))
        self.assertTrue(report.check(self.root, record))

    def test_regenerated_and_staged_report_passes(self):
        self.stage_document()
        record = report.resolve_baseline(self.root, report.load_baseline(self.root))
        self.regenerate(record)
        self.git("add", report.REPORT_PATH)
        self.assertEqual(report.check(self.root, record), [])

    def test_unstaged_source_change_does_not_affect_the_verdict(self):
        self.stage_document()
        record = report.resolve_baseline(self.root, report.load_baseline(self.root))
        self.regenerate(record)
        self.git("add", report.REPORT_PATH)
        (self.root / "src" / "lib.rs").write_text("fn upstream() { changed_again(); }\n")
        self.assertEqual(report.check(self.root, record), [])

    def test_unstaged_baseline_edit_is_ignored(self):
        """Configuration comes from the index too. An edit left in the working tree
        must not change the verdict — this is the test that catches reading config
        from the working tree while reading sources from the index.

        The baseline is RELOADED after the edit; reusing the record loaded before it
        would pass even if load_baseline read the working tree."""
        self.stage_document()
        record = report.resolve_baseline(self.root, report.load_baseline(self.root))
        self.regenerate(record)
        self.git("add", report.REPORT_PATH)
        (self.root / report.BASELINE_PATH).write_text('tag = "bogus"\n')
        reloaded = report.resolve_baseline(self.root, report.load_baseline(self.root))
        self.assertEqual(reloaded["tag"], "v1.0")
        self.assertEqual(report.check(self.root, reloaded), [])

    def test_fresh_checkout_with_empty_staging_area_validates_the_commit(self):
        self.stage_document()
        record = report.resolve_baseline(self.root, report.load_baseline(self.root))
        self.regenerate(record)
        self.git("add", report.REPORT_PATH)
        self.commit("divergence")
        # Nothing staged now; the index still describes the checked-out commit.
        self.assertEqual(self.git("diff", "--cached", "--name-only").strip(), "")
        self.assertEqual(report.check(self.root, record), [])

    def test_generation_is_a_fixpoint(self):
        self.stage_document()
        record = report.resolve_baseline(self.root, report.load_baseline(self.root))
        target = self.root / report.REPORT_PATH
        once = report.render_report(self.root, record, target.read_text())
        target.write_text(once)
        self.git("add", report.REPORT_PATH)
        self.assertEqual(report.render_report(self.root, record, once), once)

    def test_regeneration_preserves_unstaged_prose(self):
        """Editing the feature table and regenerating must not discard the edit."""
        self.stage_document()
        record = report.resolve_baseline(self.root, report.load_baseline(self.root))
        target = self.root / report.REPORT_PATH
        target.write_text(target.read_text() + "\nhand-written prose\n")
        self.regenerate(record)
        self.assertIn("hand-written prose", target.read_text())
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `python3 -m unittest discover -s tools -v`
Expected: FAIL with `AttributeError: module 'upstream_report' has no attribute 'LOCAL_BEGIN'`.

- [ ] **Step 3: Implement splice, generate, and check**

Add to `tools/upstream-report`:

```python
LOCAL_BEGIN = "<!-- BEGIN GENERATED: local -->"
LOCAL_END = "<!-- END GENERATED: local -->"
DRIFT_BEGIN = "<!-- BEGIN GENERATED: drift -->"
DRIFT_END = "<!-- END GENERATED: drift -->"


def splice(text, begin, end, body):
    """Replace the content between two markers, leaving the markers in place."""
    start = text.find(begin)
    stop = text.find(end)
    if start < 0 or stop < 0 or stop < start:
        raise ReportError(f"{begin} / {end} not found in order")
    return text[: start + len(begin)] + "\n" + body.rstrip("\n") + "\n" + text[stop:]


def render_report(root, record, text):
    """Splice the local block into a GIVEN document text.

    The data comes from the index; the target text is the caller's choice. That
    separation matters: regeneration splices into the WORKING TREE document, so an
    unstaged edit to the feature table or the rebase log survives, while --check
    splices into the STAGED document and compares. An earlier draft read the staged
    document and wrote the result over the working tree, silently discarding
    unstaged prose."""
    body = render_local(record, inventory(root, record["tree"]))
    return splice(text, LOCAL_BEGIN, LOCAL_END, body)


def check(root, record):
    """Findings, empty when the staged report is fresh. The drift block is not
    checked: it needs a fetch, and a pre-commit hook must not touch the network."""
    staged = read_index(root, REPORT_PATH)
    if render_report(root, record, staged) != staged:
        return [
            f"{REPORT_PATH} is stale against the staged tree; "
            f"run `just upstream-report` and stage the result"
        ]
    return []
```

Extend `main` to accept `--check` and to regenerate by default:

```python
    parser.add_argument("--check", action="store_true",
                        help="fail if the staged report is stale; write nothing")
```

and extend `run` — inside the error boundary, never in `main` — after baseline resolution:

```python
    if args.check:
        findings = check(root, record)
        for finding in findings:
            print(f"upstream-report: {finding}", file=sys.stderr)
        return 1 if findings else 0

    target = root / REPORT_PATH
    target.write_text(render_report(root, record, target.read_text()))
    print(f"upstream-report: wrote {REPORT_PATH}")
    return 0
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `python3 -m unittest discover -s tools -v`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
tasks done material-13b933 "marker splice, index-sourced generation that preserves unstaged prose, and the --check contract"
git add tools/upstream-report tools/test_upstream_report.py tasks/
git commit -m "feat(tools): generate and verify the local block from the index"
```

---

### Task 4: Drift, conflicts, and acknowledgments

**Files:**
- Create: `docs/materials/upstream-conflicts.toml`
- Modify: `tools/upstream-report`
- Modify: `tools/test_upstream_report.py`

**Interfaces:**
- Consumes: everything from Tasks 1-3.
- Produces: `merge_tree(root, base, upstream, fork) -> tuple[int, list[str], list[tuple[str, list[str], str]]]`; `load_acknowledged(root) -> set[str]`; `verdict(status, paths, messages, acknowledged) -> list[str]`; `seam_paths(rows) -> dict[str, str]`; `_upstream_status(root, tag_commit, upstream) -> dict[str, tuple[str, str]]`; `churn(root, tag_commit, upstream, watched) -> dict[str, tuple[int, str, str, str]]`; `render_drift(...) -> str`.

**Note on the informational-record format.** With `-z`, `merge-tree` emits: the tree OID, then conflicted-file-info entries `"<mode> <oid> <stage>\t<path>"`, then an empty field, then informational records of the form `"<n>"`, `n` paths, `"<kind>"`, `"<message>"`. The `kind` field is enumerated (`Auto-merging`, `CONFLICT (contents)`, …) and machine-readable; the free-text `message` is not, and is reported verbatim without being parsed. Verified on git 2.55.0 against this repository: 20 `Auto-merging` and 6 `CONFLICT (contents)` records.

- [ ] **Step 1: Write the failing test**

Append to `tools/test_upstream_report.py`:

```python
class Conflicts(Fixture):
    def diverged(self):
        """base, ours, theirs: one file conflicting, one merging cleanly."""
        self.write("shared.txt", "base\n")
        self.write("quiet.txt", "base\n")
        base = self.commit("base")

        self.git("checkout", "-q", "-b", "ours", base)
        self.write("shared.txt", "ours\n")
        self.write("quiet.txt", "base\nours appended\n")
        ours = self.commit("ours")

        self.git("checkout", "-q", "-b", "theirs", base)
        self.write("shared.txt", "theirs\n")
        theirs = self.commit("theirs")
        return base, ours, theirs

    def test_clean_merge_reports_no_conflicts(self):
        self.write("a.txt", "base\n")
        base = self.commit("base")
        self.git("checkout", "-q", "-b", "ours", base)
        self.write("b.txt", "ours\n")
        ours = self.commit("ours")
        status, paths, messages = report.merge_tree(self.root, base, base, ours)
        self.assertEqual(status, 0)
        self.assertEqual(paths, [])

    def test_conflict_reports_exit_one_and_the_exact_paths(self):
        base, ours, theirs = self.diverged()
        status, paths, messages = report.merge_tree(self.root, base, ours, theirs)
        self.assertEqual(status, 1)
        self.assertEqual(paths, ["shared.txt"])
        kinds = {kind for kind, _, _ in messages}
        self.assertIn("Auto-merging", kinds)
        self.assertTrue(any(kind.startswith("CONFLICT") for kind in kinds))

    def test_bad_ref_is_an_error_not_a_clean_result(self):
        base, ours, _ = self.diverged()
        with self.assertRaises(report.ReportError):
            report.merge_tree(self.root, base, ours, "no-such-ref")

    def test_unacknowledged_path_is_a_finding(self):
        problems = report.verdict(
            1, ["src/a.rs"], [("CONFLICT (contents)", ["src/a.rs"], "…")], set()
        )
        self.assertEqual(len(problems), 1)
        self.assertIn("src/a.rs", problems[0])

    def test_acknowledged_path_passes(self):
        problems = report.verdict(
            1, ["src/a.rs"], [("CONFLICT (contents)", ["src/a.rs"], "…")], {"src/a.rs"}
        )
        self.assertEqual(problems, [])

    def test_unattributed_conflict_fails(self):
        """Exit 1 with no conflicted-file entries is a legitimate conflict — git
        names directory-rename conflicts as a case — not a tool error and not a pass."""
        problems = report.verdict(1, [], [], set())
        self.assertEqual(len(problems), 1)
        self.assertIn("unattributed", problems[0])

    def test_unattributed_conflict_is_not_hidden_by_an_acknowledged_path(self):
        """A conflict notice naming no path must fail even when every listed path
        is acknowledged."""
        messages = [
            ("CONFLICT (contents)", ["src/a.rs"], "…"),
            ("CONFLICT (directory rename split)", [], "…"),
        ]
        problems = report.verdict(1, ["src/a.rs"], messages, {"src/a.rs"})
        self.assertEqual(len(problems), 1)
        self.assertIn("unattributed", problems[0])

    def test_conflict_notice_on_an_unlisted_path_fails(self):
        messages = [("CONFLICT (rename/delete)", ["src/moved.rs"], "…")]
        problems = report.verdict(1, [], messages, set())
        self.assertTrue(any("src/moved.rs" in p for p in problems))

    def test_clean_status_never_produces_findings(self):
        self.assertEqual(report.verdict(0, [], [], set()), [])
```

Also add, to the same class, a fixture that actually exercises the empty-list case rather than asserting it from a hand-built tuple:

```python
    def directory_rename_split(self):
        """A divergence that really does yield exit 1 with NO conflicted-file entries.

        Ours splits old/ across two directories; theirs adds a file into old/. Git
        cannot decide where the new file goes and reports
        `CONFLICT(directory rename unclear split)` naming the DIRECTORY `old`, with
        an empty conflicted-file info section. Verified on git 2.55.0.

        Note the kind string has no space after CONFLICT, unlike
        `CONFLICT (contents)`; matching on the `CONFLICT` prefix covers both."""
        for name in "abcdef":
            self.write(f"old/{name}.txt", f"{name}\n")
        base = self.commit("base")

        self.git("checkout", "-q", "-b", "ours", base)
        (self.root / "dir-a").mkdir()
        (self.root / "dir-b").mkdir()
        self.git("mv", "old/a.txt", "old/b.txt", "old/c.txt", "dir-a/")
        self.git("mv", "old/d.txt", "old/e.txt", "old/f.txt", "dir-b/")
        ours = self.commit("ours splits the directory")

        self.git("checkout", "-q", "-b", "theirs", base)
        self.write("old/new.txt", "new\n")
        theirs = self.commit("theirs adds into old/")
        return base, ours, theirs

    def test_directory_rename_split_has_no_conflicted_file_entries(self):
        base, ours, theirs = self.directory_rename_split()
        status, paths, messages = report.merge_tree(self.root, base, ours, theirs)
        self.assertEqual(status, 1)
        self.assertEqual(paths, [])
        kinds = [kind for kind, _, _ in messages]
        self.assertTrue(any(kind.startswith("CONFLICT") for kind in kinds), kinds)

    def test_directory_rename_split_fails_the_verdict(self):
        """The required regression: exit 1 with an empty conflicted-file list must
        never be read as a clean merge."""
        base, ours, theirs = self.directory_rename_split()
        status, paths, messages = report.merge_tree(self.root, base, ours, theirs)
        self.assertNotEqual(report.verdict(status, paths, messages, set()), [])

    def test_directory_rename_split_fails_beside_an_acknowledged_conflict(self):
        """A non-empty, fully acknowledged path list must not conceal it."""
        base, ours, theirs = self.directory_rename_split()
        status, paths, messages = report.merge_tree(self.root, base, ours, theirs)
        messages = messages + [("CONFLICT (contents)", ["src/a.rs"], "…")]
        problems = report.verdict(
            status, paths + ["src/a.rs"], messages, {"src/a.rs"}
        )
        self.assertNotEqual(problems, [])

    def test_a_directory_cannot_be_acknowledged_away(self):
        """The notice names the DIRECTORY `old`, which has no conflicted-file entry.
        Acknowledging it must not silence the conflict — otherwise one line in
        upstream-conflicts.toml hides every future conflict under that directory."""
        base, ours, theirs = self.directory_rename_split()
        status, paths, messages = report.merge_tree(self.root, base, ours, theirs)
        involved = {path for _, paths_, _ in messages for path in paths_}
        self.assertIn("old", involved)
        self.assertEqual(paths, [])
        self.assertNotEqual(report.verdict(status, paths, messages, {"old"}), [])

    def test_a_directory_cannot_be_acknowledged_beside_a_real_one(self):
        base, ours, theirs = self.directory_rename_split()
        status, paths, messages = report.merge_tree(self.root, base, ours, theirs)
        messages = messages + [("CONFLICT (contents)", ["src/a.rs"], "…")]
        problems = report.verdict(
            status, paths + ["src/a.rs"], messages, {"old", "src/a.rs"}
        )
        self.assertNotEqual(problems, [])
        self.assertTrue(any("not acknowledgeable" in p for p in problems), problems)


class Cli(Fixture):
    """Subprocess-level tests. The helper tests never exercise the exit-code
    contract, and CI branches on it: 0 success, 1 finding, 2 error."""

    def stage_working_document(self):
        tag_commit, tree, patched = self.baseline_repo()
        self.write_baseline(tag_commit, tree, patched, self.carried_ids(patched, 2))
        self.write("src/lib.rs", "fn upstream() { changed(); }\n")
        self.write(report.CONFLICTS_PATH, "acknowledged = []\n")
        self.write(
            report.REPORT_PATH,
            f"# Upstream divergence\n\n{report.LOCAL_BEGIN}\n{report.LOCAL_END}\n"
            f"\n{report.DRIFT_BEGIN}\n{report.DRIFT_END}\n",
        )
        self.git("add", "-A")

    def cli(self, *args):
        tool = pathlib.Path(__file__).with_name("upstream-report")
        return subprocess.run(
            ["python3", str(tool), *args],
            cwd=self.root, capture_output=True, text=True,
        )

    def test_stale_report_exits_one(self):
        self.stage_working_document()
        result = self.cli("--check")
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertIn("stale", result.stderr)

    def test_fresh_report_exits_zero(self):
        self.stage_working_document()
        self.assertEqual(self.cli().returncode, 0)
        self.git("add", report.REPORT_PATH)
        self.assertEqual(self.cli("--check").returncode, 0)

    def test_malformed_toml_exits_two_without_a_traceback(self):
        self.stage_working_document()
        self.write(report.BASELINE_PATH, "tag = [unterminated\n")
        self.git("add", report.BASELINE_PATH)
        result = self.cli("--check")
        self.assertEqual(result.returncode, 2, result.stderr)
        self.assertNotIn("Traceback", result.stderr)
        self.assertIn("upstream-report:", result.stderr)

    def test_wrong_typed_config_exits_two_without_a_traceback(self):
        """Valid TOML with the wrong shape. `carried = 1` parses, then explodes on
        len() with a traceback and exit 1 unless the shape is validated."""
        self.stage_working_document()
        base = read = (self.root / report.BASELINE_PATH).read_text()
        for broken in ("carried = 1", 'carried = "two"', "carried = [1, 2]"):
            with self.subTest(broken=broken):
                body = "\n".join(
                    line for line in base.splitlines()
                    if not line.startswith("[[carried]]")
                    and not line.startswith("subject")
                    and not line.startswith("patch_id")
                )
                self.write(report.BASELINE_PATH, body + "\n" + broken + "\n")
                self.git("add", report.BASELINE_PATH)
                result = self.cli("--check")
                self.assertEqual(result.returncode, 2, result.stderr)
                self.assertNotIn("Traceback", result.stderr)

    def test_truncated_hash_exits_two(self):
        """A short hash copied out of the design doc must not validate."""
        self.stage_working_document()
        text = (self.root / report.BASELINE_PATH).read_text()
        lines = []
        for line in text.splitlines():
            if line.startswith("tree = "):
                line = 'tree = "7b010d1b"'
            lines.append(line)
        self.write(report.BASELINE_PATH, "\n".join(lines) + "\n")
        self.git("add", report.BASELINE_PATH)
        result = self.cli("--check")
        self.assertEqual(result.returncode, 2, result.stderr)
        self.assertIn("40-character", result.stderr)

    def test_missing_markers_exit_two(self):
        self.stage_working_document()
        self.write(report.REPORT_PATH, "# Upstream divergence\n\nno markers\n")
        self.git("add", report.REPORT_PATH)
        result = self.cli("--check")
        self.assertEqual(result.returncode, 2, result.stderr)
        self.assertNotIn("Traceback", result.stderr)

    def test_bad_upstream_ref_exits_two_not_one(self):
        """An unfetched or misspelled upstream is an ERROR. Reporting it as a finding
        would let CI open a 'conflicts found' issue for a broken fetch."""
        self.stage_working_document()
        self.assertEqual(self.cli().returncode, 0)
        self.git("add", report.REPORT_PATH)
        result = self.cli("--drift", "--against", "no-such-remote/main")
        self.assertEqual(result.returncode, 2, result.stderr)
        self.assertNotIn("Traceback", result.stderr)
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `python3 -m unittest discover -s tools -v`
Expected: FAIL with `AttributeError: module 'upstream_report' has no attribute 'merge_tree'`.

- [ ] **Step 3: Implement merge-tree parsing, acknowledgments, verdict, and churn**

Add to `tools/upstream-report`:

```python
def merge_tree(root, base, upstream, fork):
    """(status, conflicted paths, informational records).

    status is git's: 0 clean, 1 conflicts. Anything else is a ReportError — a bad
    ref, a missing object, an unfetched upstream — and must never be reported as a
    clean result. An empty path list with status 1 is a legitimate conflict, not a
    tool error: git names directory-rename conflicts as a case with no conflicted
    file entries."""
    result = subprocess.run(
        ["git", "-C", str(root), "merge-tree", "-z",
         f"--merge-base={base}", upstream, fork],
        capture_output=True, text=True,
    )
    if result.returncode not in (0, 1):
        raise ReportError(
            f"git merge-tree failed ({result.returncode}): "
            f"{result.stderr.strip() or result.stdout.strip()}"
        )

    fields = result.stdout.split("\0")
    paths = []
    index = 1
    while index < len(fields) and fields[index] != "":
        # "<mode> <oid> <stage>\t<path>"
        paths.append(fields[index].split("\t", 1)[1])
        index += 1
    index += 1  # the empty field separating the sections

    messages = []
    while index + 1 < len(fields):
        try:
            count = int(fields[index])
        except ValueError:
            break
        index += 1
        involved = fields[index:index + count]
        index += count
        if index + 1 >= len(fields):
            break
        kind = fields[index]
        text = fields[index + 1]
        index += 2
        messages.append((kind, involved, text.strip()))

    return result.returncode, sorted(set(paths)), messages


def load_acknowledged(root):
    """Paths whose conflicts we accept and resolve by hand each cycle."""
    record = tomllib.loads(read_index(root, CONFLICTS_PATH))
    entries = record.get("acknowledged", [])
    if not isinstance(entries, list):
        raise ReportError(
            f"{CONFLICTS_PATH}: acknowledged must be an array of tables, "
            f"got {entries!r}"
        )
    paths = set()
    for index, entry in enumerate(entries):
        if not isinstance(entry, dict) or not isinstance(entry.get("path"), str):
            raise ReportError(
                f"{CONFLICTS_PATH}: acknowledged[{index}] needs a string path"
            )
        paths.add(entry["path"])
    return paths


def verdict(status, paths, messages, acknowledged):
    """Findings. The enumerated `kind` field is used only to WIDEN the failure set,
    never to narrow it, and conflict types are never inferred.

    Acknowledgment applies only to paths git listed in the conflicted-file info
    section. A conflict notice naming something with no stage entry behind it — the
    directory `old` in a rename split, for instance — is structural and is NEVER
    acknowledgeable: an earlier draft merged those names into the acknowledgeable
    set, so a single `path = "old"` entry silenced the whole directory-rename case
    and returned a clean verdict."""
    if status == 0:
        return []

    backed = set(paths)
    problems = [
        f"unacknowledged conflict: {path}" for path in sorted(backed - acknowledged)
    ]

    notices = [message for message in messages if message[0].startswith("CONFLICT")]
    for kind, involved, text in notices:
        unbacked = [path for path in involved if path not in backed]
        if not involved or unbacked:
            problems.append(
                f"unattributed conflict, not acknowledgeable ({kind}): {text}"
            )
    if not notices and not backed:
        problems.append(
            "unattributed conflict: merge-tree exited 1 with no conflicted-file "
            "entries and no conflict notices"
        )
    return problems


def seam_paths(rows):
    """{fork path: baseline path} for class-B paths: where upstream movement can
    reach us, and the name upstream still knows each one by."""
    return {
        row.path: (row.previous or row.path)
        for row in rows
        if classify(row.status, row.path) == "B"
    }


def _upstream_status(root, tag_commit, upstream):
    """baseline path -> (status letter, upstream's current path).

    Renames are keyed by the OLD path, because that is the name the baseline — and
    therefore our inventory — knows the file by."""
    fields = _split_z(
        git(root, "diff", "--name-status", "-M", "-z", f"{tag_commit}..{upstream}")
    )
    result = {}
    index = 0
    while index < len(fields):
        status = fields[index]
        if status[:1] in ("R", "C"):
            result[fields[index + 1]] = (status, fields[index + 2])
            index += 3
        else:
            result[fields[index + 1]] = (status, fields[index + 1])
            index += 2
    return result


def churn(root, tag_commit, upstream, watched):
    """{fork path: baseline path} -> fork path -> (commits, upstream status, baseline
    path, upstream path).

    Upstream history is walked under the BASELINE path, never the fork's current
    one: upstream never saw our rename, so asking it about `src/new.rs` returns zero
    commits while it is busily editing `src/old.rs`."""
    statuses = _upstream_status(root, tag_commit, upstream)
    result = {}
    for path, baseline in watched.items():
        count = len(git(
            root, "log", "--oneline", f"{tag_commit}..{upstream}", "--", baseline
        ).splitlines())
        letter, upstream_path = statuses.get(baseline, ("", baseline))
        result[path] = (
            count,
            letter if letter[:1] in ("R", "D") else "",
            baseline,
            upstream_path,
        )
    return result


def render_drift(record, upstream_ref, upstream_sha, fork_sha, when,
                 status, paths, messages, churns, acknowledged):
    """The generated drift block. Stamped with the REPORT GENERATION date — not the
    fork's commit date, which says nothing about when the comparison was run."""
    lines = [
        f"As of **{when}**: baseline tree `{record['tree'][:12]}`, "
        f"fork `{fork_sha[:12]}`, upstream `{upstream_ref}` at `{upstream_sha[:12]}`.",
        "",
        "This is a three-way merge of final trees. It does not predict a rebase, "
        "which replays commits individually, and it says nothing about whether the "
        "result compiles or behaves correctly.",
        "",
    ]
    lines.append(
        "**No conflicts.**" if status == 0
        else f"**{len(paths)} conflicting path(s).**"
    )
    lines.append("")

    # The seam table is rendered ALWAYS, over every class-B path, not only over
    # conflicting ones. Upstream churn in a seam file that still merges cleanly —
    # including a clean upstream rename or delete — is exactly the early warning
    # this block exists to give, and it is invisible in a conflict-only table.
    lines.append(
        "| Seam path | Upstream name | Upstream commits since baseline "
        "| Upstream status | Conflict | Acknowledged |"
    )
    lines.append("| --- | --- | --- | --- | --- | --- |")
    conflicting = set(paths)
    for path in sorted(churns):
        count, letter, baseline, upstream_path = churns[path]
        # Show the fork's current path, but say which name upstream knows it by
        # whenever either side has renamed it.
        name = "same" if upstream_path == path else f"`{upstream_path}`"
        conflict = "yes" if path in conflicting else "-"
        mark = "yes" if path in acknowledged else ("**NO**" if path in conflicting else "-")
        lines.append(
            f"| `{path}` | {name} | {count} | {letter or '-'} | {conflict} | {mark} |"
        )

    unlisted = sorted(conflicting - set(churns))
    if unlisted:
        lines.append("")
        lines.append(
            "Conflicting paths outside the class-B inventory (upstream and this fork "
            "changed the same path independently, or a directory moved): "
            + ", ".join(f"`{path}`" for path in unlisted)
        )
    notices = [m for m in messages if m[0].startswith("CONFLICT")]
    if notices:
        lines.append("")
        lines.append("Conflict notices, verbatim and unparsed:")
        lines.append("")
        # An INDENTED code block, not a fenced one. The report is markdown that
        # gets embedded in other markdown — issue bodies, job summaries — and a
        # nested fence breaks whichever fence encloses it.
        for kind, involved, text in notices:
            lines.append(f"    {text}")
    return "\n".join(lines)
```

Extend `main` with the drift options:

```python
    parser.add_argument("--drift", action="store_true",
                        help="also regenerate the drift block; needs a fetched upstream")
    parser.add_argument("--against", default="upstream/main",
                        help="upstream side of the comparison (default: upstream/main)")
```

and, in `run`'s regeneration path, after writing the local block:

```python
    if args.drift:
        upstream_sha = rev(root, args.against)
        fork_sha = rev(root, "HEAD")
        status, paths, messages = merge_tree(
            root, record["tag_commit"], args.against, "HEAD"
        )
        acknowledged = load_acknowledged(root)
        # Churn over every seam path plus anything that conflicted, so a cleanly
        # merging seam file that upstream is actively rewriting still shows up.
        watched = seam_paths(inventory(root, record["tree"]))
        # A conflicting path outside the seam set is watched under its own name.
        for path in paths:
            watched.setdefault(path, path)
        body = render_drift(
            record, args.against, upstream_sha, fork_sha,
            datetime.datetime.now(datetime.UTC).strftime("%Y-%m-%d"),
            status, paths, messages,
            churn(root, record["tag_commit"], args.against, watched),
            acknowledged,
        )
        text = splice(
            (root / REPORT_PATH).read_text(), DRIFT_BEGIN, DRIFT_END, body
        )
        (root / REPORT_PATH).write_text(text)
        findings = verdict(status, paths, messages, acknowledged)
        for finding in findings:
            print(f"upstream-report: {finding}", file=sys.stderr)
        if findings:
            return 1
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `python3 -m unittest discover -s tools -v`
Expected: PASS.

- [ ] **Step 5: Write the acknowledgment file**

Create `docs/materials/upstream-conflicts.toml` with the six paths the canary currently reports:

```toml
# Conflicts we accept and resolve by hand each rebase. Hand-edited; the tool never
# writes this file, so regenerating the report can never approve a new conflict.
#
# LIMITATION: acknowledgment is path-granular. A NEW conflict inside an already
# acknowledged file is not detected — acknowledging a path accepts every conflict in
# it, now and later. The drift block's per-file upstream churn count is the mitigation:
# it rises visibly when upstream works in a file we have accepted.
#
# Acknowledgment is not approval of upstream's change. It records that we expect to
# resolve this conflict by hand.

[[acknowledged]]
path = "src/layout/floating.rs"
note = "material state on floating tiles"

[[acknowledged]]
path = "src/layout/monitor.rs"
note = "material render pass threaded through the monitor"

[[acknowledged]]
path = "src/layout/tile.rs"
note = "signal level, focus crossfade, and the material element live here"

[[acknowledged]]
path = "src/layout/workspace.rs"
note = "carries the #4147 IPC field as well as material state"

[[acknowledged]]
path = "src/protocols/foreign_toplevel.rs"
note = "signal sources hook the toplevel protocol"

[[acknowledged]]
path = "src/render_helpers/blur.rs"
note = "we insert the material pass before blur; expected to conflict every cycle"
```

- [ ] **Step 6: Commit**

```bash
tasks done material-e62c55 "merge-tree canary with structured -z parsing, path acknowledgments, seam churn, and the six current conflicts"
git add tools/upstream-report tools/test_upstream_report.py docs/materials/upstream-conflicts.toml tasks/
git commit -m "feat(tools): detect upstream drift with merge-tree and acknowledged paths"
```

---

### Task 5: The divergence document

**Files:**
- Create: `docs/materials/upstream-divergence.md`

**Interfaces:**
- Consumes: `LOCAL_BEGIN`/`LOCAL_END`/`DRIFT_BEGIN`/`DRIFT_END` from Task 3; the tool from Tasks 1-4.
- Produces: the document every later task points at.

- [ ] **Step 1: Write the document skeleton with markers**

Create `docs/materials/upstream-divergence.md`. Write the prose in full; leave the two generated regions empty between their markers.

```markdown
# Upstream divergence

**Posture:** the fork is permanent. The material system is not going upstream.
"Upstream candidate" here means a *seam*: a small change to an upstream file that,
if accepted, would shrink our diff and make every rebase cheaper.

The baseline is pinned in `upstream-baseline.toml` and validated on every run by
comparing **trees**. This repository's history was rewritten: `git merge-base
materials-26.04 v26.04` returns a 2023 commit, and a diff from it reports 5743 files
against a true 176. Nothing here consults ancestry.

## Divergence classes

- **A — additive fork-only files.** Lower textual conflict risk, not zero cost:
  upstream can rename a directory we occupy, or add a file at a path we also add.
- **B — seam edits to upstream files.** Where conflicts are expected as a matter of
  course, and the only class where "upstream candidate" is meaningful.
- **C — vendored tooling and project scaffolding.** `tools/`, `.githooks/`,
  `justfile`, `packaging/`. Fork-only by construction, but tracked because
  `tools/tt` has an external source of truth in ops.
- **D — carried upstream patches.** The #4147 IPC commits, listed in
  `upstream-baseline.toml` as `carried`. Retired when their content is present in
  the release tag we move to — which is not the same moment upstream merges them.

**Class is an inventory aid, not a conflict filter.** Conflict detection runs over
the whole tree and is never scoped by class.

## Feature table

| Feature | Where | Class | Posture | Status |
| --- | --- | --- | --- | --- |
| Material rendering and glass shader | `src/render_helpers/material.rs`, `shaders/material.frag` | A | fork-only | — |
| Material and glass configuration | `niri-config/src/material.rs` | A | fork-only | — |
| Per-window signals | `src/render_helpers/signal.rs`, `src/window/signal.rs` | A | fork-only | — |
| Workspace IPC field | `niri-ipc/`, `src/ipc/`, `src/layout/workspace.rs` | D | carried | submitted #4147 |
| Material pass ordering hook | `src/render_helpers/blur.rs`, `effect_buffer.rs` | B | seam | unfiled |
| Tile material state | `src/layout/tile.rs` | B | fork-only | — |
| Test front door and timing | `justfile`, `tools/` | C | fork-only | — |

Posture is `seam`, `fork-only`, or `carried`. Status tracks seam candidates through
`unfiled` → `submitted #NNNN` → `merged <tag>` / `declined: <reason>`.

## Seam inventory

Generated from the git index against the baseline tree. Do not edit by hand; run
`just upstream-report`.

<!-- BEGIN GENERATED: local -->
<!-- END GENERATED: local -->

## Upstream drift

Generated against a fetched upstream. Refreshed weekly by
`.github/workflows/upstream-drift.yml` and at every rebase; the stamp says how stale
it is. Do not edit by hand; run `just upstream-report --drift`.

<!-- BEGIN GENERATED: drift -->
<!-- END GENERATED: drift -->

## Rebase procedure

Triggered by a new upstream release tag, or quarterly, whichever comes first. The
authoritative version, with the reasoning for each step, is
`../specs/2026-09-06-upstream-divergence-design.md`.

1. `git fetch upstream --tags`, then `just upstream-report --drift --against <newtag>`.
2. Check whether #4147's content is present **in the target tag**, not merely merged
   to `main`. `git patch-id --stable` is evidence; read the tag's tree and decide.
3. Create `patched-<newtag>`; record the release tree, release commit, new patched
   commit, and carried patch-ids in `upstream-baseline.toml`.
4. Rebase in a fresh `.worktrees/` worktree with an explicit old boundary:
   `git rebase --onto patched-<newtag> patched-<oldtag> materials-<newtag>`.
   Flatten the merges — but first `git show --remerge-diff` each one in
   `git log --merges patched-<oldtag>..materials-<oldtag>`, record every non-empty
   result, and verify afterwards that each resolution survived.
5. Regenerate and stage the baseline record and the report, then `just gate`.
6. Nested GLES smoke on the headless host.
7. Physical DRM acceptance on real hardware, with a human watching scanout. Not
   interchangeable with step 6.
8. Prune stale acknowledgments, re-review the posture column, add a rebase-log line.
9. Re-pin packaging: recompute the commit count, rewrite `pkgver` and the `source=`
   commit in `packaging/arch/PKGBUILD`, push before building.
10. Keep the old branches as archives, then change the repository's default branch on
    GitHub (`gh repo edit --default-branch materials-<newtag>`) — the weekly workflow
    only schedules from the default branch. Moving local `origin/HEAD` does not do this.

## Rebase log

| Date | From | To | Conflicts resolved | Merge resolutions carried | Evidence |
| --- | --- | --- | --- | --- | --- |
| — | — | `v26.04` | — | — | baseline established 2026-09-06 |
```

- [ ] **Step 2: Generate both blocks and verify**

```bash
git add docs/materials/upstream-divergence.md
python3 tools/upstream-report
git fetch upstream --tags
python3 tools/upstream-report --drift
```

Expected: both exit **0**. The first writes the local block. The second writes the drift block and finds six conflicts, all of them acknowledged in Task 4, so it reports no findings.

If the second exits 1, read the path it names. Either the acknowledgment file has a typo, or upstream moved past the cached tip `3439d4ef` the design measured and a genuinely new conflict appeared. In the second case add it to `upstream-conflicts.toml` **with a note explaining why it conflicts** — never silence it with a bare entry — and say so in the commit message.

- [ ] **Step 3: Verify the generated numbers match the design's measurements**

Run: `grep -c '| B |' docs/materials/upstream-divergence.md`
Expected: **38**, matching the design's Current-build evidence. A different number means Task 2's classification drifted from the design; reconcile before committing.

- [ ] **Step 4: Verify the freshness check passes on the staged result**

```bash
git add docs/materials/upstream-divergence.md
python3 tools/upstream-report --check && echo FRESH
```

Expected: `FRESH`.

- [ ] **Step 5: Commit**

```bash
tasks done material-46c3be "the divergence document: classes, feature table, rebase procedure, log, and both generated blocks populated"
git add tasks/
python3 tools/upstream-report
git add docs/materials/upstream-divergence.md
git commit -m "docs(material): add the upstream divergence document"
```

The regenerate-after-staging step matters from here on: `tasks done` rewrote a file under `tasks/`, which is in the inventory, so the report written in Step 2 is already stale.

---

### Task 6: `just` wiring

**Files:**
- Modify: `justfile:17` (`check_cmd`), and a new recipe

**Interfaces:**
- Consumes: `tools/upstream-report --check` from Task 3.
- Produces: `just upstream-report [args...]`; `check_cmd` gains the freshness check.

- [ ] **Step 1: Add the recipe**

Add to `justfile`, next to the other tooling recipes:

```just
# Regenerate the upstream divergence report. Pass --drift for the drift block
# (needs `git fetch upstream --tags` first), --check to verify staged freshness.
upstream-report *args:
    python3 tools/upstream-report {{args}}
```

- [ ] **Step 2: Extend `check_cmd`**

`justfile:17` currently reads:

```just
check_cmd := "cargo fmt --all -- --check && cargo clippy --all --all-targets && python3 -m unittest discover -s tools 2>&1 && tasks check"
```

Append the freshness check as the last command:

```just
check_cmd := "cargo fmt --all -- --check && cargo clippy --all --all-targets && python3 -m unittest discover -s tools 2>&1 && tasks check && python3 tools/upstream-report --check"
```

It goes last deliberately: it is the cheapest to run and the most likely to fail during ordinary work, and putting it after `tasks check` keeps the existing failure ordering unchanged.

- [ ] **Step 3: Verify both directions**

```bash
just upstream-report --check && echo FRESH
just check && echo CHECK-PASSES
```

Expected: both succeed.

Now prove the gate actually bites:

```bash
printf '\n// drift\n' >> src/render_helpers/material.rs
git add src/render_helpers/material.rs
just upstream-report --check; echo "exit=$?"
git restore --staged --worktree src/render_helpers/material.rs
```

Expected: `exit=1` with `docs/materials/upstream-divergence.md is stale against the staged tree`. If it exits 0, the check is reading the working tree instead of the index — stop and fix Task 3.

- [ ] **Step 4: Commit**

```bash
tasks done material-331614 "just upstream-report recipe and the freshness check in check_cmd, verified to fail on a staged source change"
git add justfile tasks/
python3 tools/upstream-report
git add docs/materials/upstream-divergence.md
git commit -m "build(just): enforce upstream report freshness in check"
```

`justfile` is a class-C path in the inventory and its line counts just changed, so without the regeneration this commit fails the gate it is itself installing.

---

### Task 7: Weekly drift workflow, index entries, closure

**Files:**
- Create: `.github/workflows/upstream-drift.yml`
- Modify: `docs/materials/README.md`
- Modify: `AGENTS.md`
- Modify: `docs/specs/2026-09-06-upstream-divergence-design.md` (status header)

**Interfaces:**
- Consumes: `just upstream-report --drift` and `python3 -m unittest discover -s tools`.
- Produces: the scheduled canary; the pointers agents read.

- [ ] **Step 1: Write the workflow**

Create `.github/workflows/upstream-drift.yml`. A new file — `ci.yml` is already a class-B seam and must not grow.

```yaml
name: upstream drift

on:
  schedule:
    # Weekly. Scheduled workflows only run from the file on the DEFAULT branch,
    # so a rebase that leaves the default behind silently stops this check.
    - cron: "17 6 * * 1"
  pull_request:
  workflow_dispatch:

permissions:
  contents: read
  issues: write

jobs:
  drift:
    name: upstream drift
    runs-on: ubuntu-24.04
    steps:
      # python3 and git only. Deliberately NOT `just check`: two of check_cmd's
      # commands need the Rust toolchain and apt dependencies, and `tasks check`
      # needs the tasks CLI, which CI does not have. ci.yml's rustfmt and clippy
      # jobs already cover format and lint; they are not duplicated here.
      - uses: actions/checkout@v6
        with:
          fetch-depth: 0
          show-progress: false

      - name: Tooling tests
        run: python3 -m unittest discover -s tools

      - name: Report freshness
        run: python3 tools/upstream-report --check

      - name: Fetch upstream
        # A fetch failure fails the job. It must never be reported as clean drift.
        run: |
          git remote add upstream https://github.com/niri-wm/niri
          git fetch --tags --no-recurse-submodules upstream

      - name: Drift
        if: github.event_name != 'pull_request'
        id: drift
        run: |
          set +e
          python3 tools/upstream-report --drift 2>drift.err
          echo "status=$?" >> "$GITHUB_OUTPUT"
          set -e
          cat drift.err

      # ANY status other than 0 or 1 is an error, not a conflict finding: 2 from the
      # tool's own boundary, but also 137 from an OOM kill, 124 from a timeout, or a
      # shell failure. Fail loudly and open no issue. Testing `== '2'` would let every
      # other failure fall through to publication as though the analysis had run.
      - name: Fail on analysis error
        if: >-
          github.event_name != 'pull_request'
          && steps.drift.outputs.status != '0'
          && steps.drift.outputs.status != '1'
        run: |
          echo "::error::upstream-report exited ${{ steps.drift.outputs.status }};\
            this is an analysis failure, not a drift finding"
          cat drift.err || true
          exit 1

      # Publish on EVERY completed analysis, findings or not. The committed document
      # is a snapshot from the last local run; this workflow does not commit, so
      # without this the refreshed table exists only inside the dead runner.
      - name: Publish the report
        if: >-
          github.event_name != 'pull_request'
          && contains(fromJSON('["0", "1"]'), steps.drift.outputs.status)
        run: |
          {
            echo "## Upstream drift"
            echo
            sed -n "/BEGIN GENERATED: drift/,/END GENERATED: drift/p" \
              docs/materials/upstream-divergence.md
          } >> "$GITHUB_STEP_SUMMARY"

      - uses: actions/upload-artifact@v4
        if: >-
          github.event_name != 'pull_request'
          && contains(fromJSON('["0", "1"]'), steps.drift.outputs.status)
        with:
          name: upstream-divergence
          path: docs/materials/upstream-divergence.md

      - name: Report drift
        if: github.event_name != 'pull_request' && steps.drift.outputs.status == '1'
        env:
          GH_TOKEN: ${{ github.token }}
        run: |
          marker="<!-- upstream-drift-canary -->"
          body="$marker
          The weekly upstream drift canary found conflicts that
          \`docs/materials/upstream-conflicts.toml\` does not acknowledge.

          \`\`\`
          $(cat drift.err)
          \`\`\`

          The refreshed report is attached to this run as the \`upstream-divergence\`
          artifact and rendered in the run summary.

          Run \`just upstream-report --drift\` locally, then either resolve the seam
          or acknowledge the path with a note explaining why it conflicts."
          existing=$(gh issue list --state open --search "$marker in:body" \
            --json number --jq '.[0].number')
          if [ -n "$existing" ]; then
            gh issue edit "$existing" --body "$body"
          else
            gh issue create --title "Upstream drift: unacknowledged conflicts" \
              --body "$body"
          fi
          exit 1
```

Baseline resolution in this job works because `upstream-baseline.toml` pins SHAs: `actions/checkout` fetches full history at `fetch-depth: 0` but creates no local `patched-26.04`, so a name-based lookup would fail here.

- [ ] **Step 2: Validate the workflow parses**

Validate the workflow with a real parser, not substring matching. Substring checks
cannot work here: the file legitimately contains the string `just check` in a comment
explaining why it is not run, and `just upstream-report` in the issue body it writes.

`actionlint` is the right tool — it parses the YAML *and* checks Actions semantics
(expression syntax, `needs`, context names, shell issues):

```bash
actionlint .github/workflows/upstream-drift.yml
```

Expected: no output, exit 0.

If `actionlint` is not on PATH, fall back to a YAML parse plus structural assertions:

```bash
python3 - <<'GUARD'
import pathlib, sys
try:
    import yaml
except ImportError:
    sys.exit("install actionlint or PyYAML to validate the workflow")

doc = yaml.safe_load(pathlib.Path(".github/workflows/upstream-drift.yml").read_text())

# `on` is parsed by YAML 1.1 as the boolean True, not the string "on".
triggers = doc.get(True, doc.get("on"))
assert "schedule" in triggers, "no schedule trigger"
assert doc["permissions"]["issues"] == "write", "issues: write missing"

steps = doc["jobs"]["drift"]["steps"]
runs = "\n".join(step.get("run", "") for step in steps)
uses = [step.get("uses", "") for step in steps]

assert any("upload-artifact" in u for u in uses), "report is never published"
assert "GITHUB_STEP_SUMMARY" in runs, "no job summary"
assert "upstream-report --check" in runs
assert "upstream-report --drift" in runs

# The toolchain claim, checked against the COMMANDS rather than the whole file.
for step in steps:
    command = step.get("run", "")
    for line in command.splitlines():
        line = line.strip()
        if line.startswith("#") or not line:
            continue
        assert not line.startswith("just "), line
        assert not line.startswith("cargo "), line
        assert not line.startswith("tasks "), line
        assert "apt-get" not in line, line
print("workflow invariants ok")
GUARD
```

Expected: `workflow invariants ok`.

Note the `on:` key: PyYAML follows YAML 1.1 and parses a bare `on` as the boolean
`True`, so `doc["on"]` raises `KeyError` on a perfectly valid workflow. The check
above reads `doc.get(True, ...)` first for that reason.

- [ ] **Step 3: Add the index entry and the agent pointer**

In `docs/materials/README.md`, add after the `upstream-divergence-design` line:

```markdown
- `upstream-divergence.md`: what this fork changes relative to upstream, which changes are seam candidates, and the rebase procedure. Generated blocks come from `just upstream-report`; `upstream-baseline.toml` pins the baseline and `upstream-conflicts.toml` acknowledges expected conflicts.
```

In `AGENTS.md`, extend the session protocol list with:

```markdown
- Before rebasing onto a new upstream release, read `docs/materials/upstream-divergence.md`:
  it carries the baseline, the acknowledged conflicts, and the rebase procedure. Never
  use `git merge-base` against upstream — this repository's history was rewritten and it
  returns a 2023 commit.
```

- [ ] **Step 4: Correct the spec's status**

The design doc still reads `**Status:** awaiting review 2026-09-06; not implemented.`
A merge is the moment a status goes stale. Update the header of
`docs/specs/2026-09-06-upstream-divergence-design.md`:

```markdown
**Status:** implemented 2026-09-06; plan
`../plans/2026-09-06-upstream-divergence.md`. The document it specifies is
`../materials/upstream-divergence.md`, the baseline it pins is
`../materials/upstream-baseline.toml`, and the canary is
`.github/workflows/upstream-drift.yml`.
```

**No commit SHA here.** The header lives in the same commit that completes the work,
and writing that commit's own hash into it is impossible — inserting the hash changes
the hash. The date and the artifact links carry the same information and are stable.
Other design docs in `docs/specs/` cite a SHA because their status was corrected in a
*later* commit than the one it names; this one is not.

Then correct the same claim where it propagated, in `docs/materials/README.md`:
change `awaiting review` to `implemented`. Grep before committing:

```bash
grep -rn "awaiting review" docs/ | grep upstream-divergence
```

Expected: no output.

- [ ] **Step 5: Full verification**

```bash
just gate
```

Expected: PASS — rustfmt, clippy, tooling tests, `tasks check`, the freshness check, and the test suite.

- [ ] **Step 6: Commit and close**

```bash
tasks done material-d467fb "weekly drift workflow on python3+git only, index entry, and the AGENTS pointer"
tasks done material-a9447f "upstream divergence document, generated inventory, drift canary, and rebase procedure landed"
git add .github/workflows/upstream-drift.yml docs/materials/README.md AGENTS.md \
        docs/specs/2026-09-06-upstream-divergence-design.md tasks/
python3 tools/upstream-report
git add docs/materials/upstream-divergence.md
git commit -m "ci: add the weekly upstream drift canary"
```

- [ ] **Step 7: Confirm GitHub accepts the workflow**

`actionlint` proves the file is well-formed, but only GitHub decides whether it
registers the workflow, and a workflow it rejects is **silently ignored** rather than
reported. This must happen after the commit — before it, there is nothing on the
remote to register.

```bash
git push -u origin docs/upstream-divergence
gh workflow run upstream-drift.yml --ref docs/upstream-divergence
gh run list --workflow upstream-drift.yml --branch docs/upstream-divergence --limit 1
```

Expected: `workflow_dispatch` is accepted and a run appears for the branch. If
`gh workflow run` reports the workflow does not exist, GitHub rejected the file —
`gh workflow list` alone would not have told you, since it neither validates YAML nor
scopes to a branch.

Then read the run's summary and artifact to confirm the report actually published:

```bash
gh run view --log | sed -n '/Upstream drift/,+20p'
```

Note the schedule itself will not fire from this branch. Scheduled workflows run only
from the file on the **default** branch, so weekly runs begin when this merges.

Three staged paths in the previous step are in the inventory — the workflow and `AGENTS.md` are class B (both already exist upstream or in the fork's seam set), and `tasks/` is class A — so the regeneration is required, not optional.

---

## Self-review notes

**Spec coverage.** Baseline identity → Task 1. Seam inventory and classes → Task 2. Marker blocks and freshness enforcement → Task 3. Canary invocation, parsing, acknowledgments, drift block → Task 4. Document structure, feature table, rebase procedure, rebase log → Task 5. `just` wiring → Task 6. Workflow contract, index, pointer, and the spec's own status correction → Task 7. The spec's Verification section maps onto the fixtures in Tasks 1-4; the "generated numbers match the hand-measured figures" acceptance is Task 5 Step 3.

**Drift coverage.** The drift block's seam table is rendered over **every class-B path**, not only conflicting ones, and unconditionally rather than only when conflicts exist. Upstream churn in a seam file that still merges cleanly — a clean upstream rename or delete especially — is the earliest warning available, and a conflict-only table hides it. Conflicting paths outside the class-B set are listed separately rather than dropped.

**Not covered by code, by design.** The rebase procedure is a written human procedure (Task 5), not automation — the spec's non-goals forbid automated rebasing. `tasks check` in CI is the spec's named deliberate gap.

**Type consistency.** `inventory` returns 4-tuples `(status, path, added, removed)` throughout; `render_local`, `seam_paths`, the Task 2 tests, and Task 5's Step 3 grep all assume that shape. `_name_status` returns `path -> (status, previous)` and `_numstat` returns `path -> (added, removed)`, joined on the new path. `merge_tree` returns `(status, paths, messages)` with `messages` as `(kind, involved, text)` 3-tuples, consumed identically by `verdict` and `render_drift`. `resolve_baseline` returns the record it was given, so `load_baseline` → `resolve_baseline` composes. `render_report(root, record, text)` takes its target text as a parameter — regeneration passes the working tree, `check` passes the index.

**Exit-code contract, asserted end to end.** `run()` returns 0 or 1; every operational failure raises and is converted to 2 by the single boundary in `main()`. The `Cli` test class exercises this through `subprocess`, because the helper tests cannot: they call functions that raise, never a process that exits. CI branches on `== '2'` (error, fail loudly, open no issue) and `== '1'` (finding, open the deduplicated issue).

**Verified constructions, not assumed ones.** Three facts in this plan were measured on git 2.55.0 rather than reasoned about, because each had already produced a wrong answer once:
- `--name-status -z` and `--numstat -z` encode renames differently. Joining them line-wise yields the nonexistent path `src/{old.rs => new.rs}`; Task 2 parses both with `-z` and joins on the new path, and errors rather than defaulting when the key sets disagree.
- A directory-rename **split** (ours splits `old/` across two directories, theirs adds a file into `old/`) really does produce exit 1 with an empty conflicted-file list and a `CONFLICT(directory rename unclear split)` record — note no space after `CONFLICT`. Plain both-sides-rename does not; it produces 15 stage entries. Task 4's fixture uses the construction that works.
- `git branch --format` prints `(HEAD detached at abc1234)` under a detached HEAD, so Task 1's branchless fixture uses `for-each-ref refs/heads/`.

**Known ordering constraints.**
- Task 6 must not land before Task 5: `check_cmd` gains `upstream-report --check`, which fails until `upstream-divergence.md` exists with its markers.
- Within every task, `tasks done` runs before staging and the report is regenerated after staging. `tasks/` is a class-A inventory path, so closing a record changes the inventory; from Task 6 on, staging without regenerating makes the commit fail its own freshness gate.
