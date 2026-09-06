"""Unit tests for tools/upstream-report: `python3 -m unittest discover -s tools`."""
import importlib.machinery
import importlib.util
import os
import pathlib
import shutil
import subprocess
import tempfile
import textwrap
import unittest

# Git exports GIT_DIR, GIT_WORK_TREE and GIT_INDEX_FILE into hook processes, and they
# OVERRIDE `git -C <dir>`. `check_cmd` runs this suite from the pre-commit hook, so
# without this scrub a fixture's `git commit` lands in THIS repository instead of its
# temporary one — which is exactly how an earlier run moved the branch to a bogus
# commit. Scrubbing the whole process also covers the report.* helpers the tests call.
for _name in [_key for _key in os.environ if _key.startswith("GIT_")]:
    del os.environ[_name]

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

        toplevel = self.git("rev-parse", "--show-toplevel").strip()
        self.assertEqual(
            os.path.realpath(toplevel), os.path.realpath(self.root),
            "fixture escaped its temporary repository",
        )

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
        """Flush-left TOML, built line by line.

        Do NOT use an indented triple-quoted string here. `write` runs
        textwrap.dedent, which strips the COMMON prefix across all non-blank lines —
        and interpolating the carried entries flush-left makes that prefix empty, so
        every other line silently keeps its indentation. TOML tolerates it, so the
        file still parses and nothing complains; the damage shows up in tests that
        match on `line.startswith("tree = ")` and quietly match nothing."""
        lines = [
            'tag = "v1.0"',
            f'tag_commit = "{tag_commit}"',
            f'tree = "{tree}"',
            f'patched_commit = "{patched_commit}"',
        ]
        for subject, patch in carried:
            lines += [
                "",
                "[[carried]]",
                f'subject = "{subject}"',
                f'patch_id = "{patch}"',
            ]
        self.write(report.BASELINE_PATH, "\n".join(lines) + "\n")
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
