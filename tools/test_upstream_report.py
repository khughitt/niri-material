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
        # The baseline path survives, so Task 4's drift analysis can ask upstream
        # about the old name. seam_paths and churn are asserted in Task 4, which is
        # where they are implemented — Task 2 must pass its own gate on its own.
        self.assertEqual(row.previous, "src/lib.rs")

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

    def test_local_block_counts_classes(self):
        tree = self.stage_divergence()
        record = report.resolve_baseline(self.root, report.load_baseline(self.root))
        body = report.render_local(record, report.inventory(self.root, tree))
        self.assertIn("| `src/lib.rs` | B |", body)
        self.assertIn("| `tools/tt` | C |", body)
        self.assertNotIn("src/material.rs", body)  # class A is counted, not listed
        self.assertIn("v1.0", body)


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


class Drift(Fixture):
    def test_seam_paths_map_to_the_baseline_name(self):
        tag_commit, tree, patched = self.baseline_repo()
        self.write_baseline(tag_commit, tree, patched, self.carried_ids(patched, 2))
        self.git("mv", "src/lib.rs", "src/renamed.rs")
        self.git("add", "-A")
        watched = report.seam_paths(report.inventory(self.root, tree))
        self.assertEqual(watched["src/renamed.rs"], "src/lib.rs")

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
        churns = report.churn(self.root, tag_commit, "upstream-main", watched)
        count, letter, baseline, upstream_path = churns["src/renamed.rs"]
        self.assertEqual(baseline, "src/lib.rs")
        self.assertEqual(count, 1, "upstream churn must follow the baseline path")

    def test_resolve_baseline_exposes_the_fork_baseline_commit(self):
        tag_commit, tree, patched = self.baseline_repo()
        self.write_baseline(tag_commit, tree, patched, self.carried_ids(patched, 2))
        record = report.resolve_baseline(self.root, report.load_baseline(self.root))
        expected = self.git("rev-parse", f"{patched}~2").strip()
        self.assertEqual(record["baseline_commit"], expected)
        self.assertEqual(
            self.git("rev-parse", f"{record['baseline_commit']}^{{tree}}").strip(),
            tree,
        )

    def test_churn_counts_from_the_fork_baseline_commit_not_the_tag(self):
        """Mirrors the real repository: the local v26.04 tag is a rewritten copy of
        the release and is not an ancestor of upstream/main, so `git log A..B`
        cannot walk from it. Anchoring churn at the tag produces a different
        (inflated) count than anchoring it at the fork's baseline commit, which
        upstream's real history line passes through."""
        tag_commit, tree, patched = self.baseline_repo()
        self.write_baseline(tag_commit, tree, patched, self.carried_ids(patched, 2))
        record = report.resolve_baseline(self.root, report.load_baseline(self.root))
        baseline_commit = record["baseline_commit"]

        # Upstream branches off the fork's BASELINE commit (the rewritten, parentless
        # copy) -- not off the local tag, which is a different, unrelated commit
        # with the same tree.
        self.git("checkout", "-q", "-b", "upstream-main", baseline_commit)
        self.write("src/lib.rs", "fn upstream() { moved_on(); }\n")
        self.commit("upstream edits src/lib.rs")

        watched = {"src/lib.rs": "src/lib.rs"}
        from_baseline = report.churn(self.root, baseline_commit, "upstream-main", watched)
        from_tag = report.churn(self.root, tag_commit, "upstream-main", watched)

        self.assertEqual(from_baseline["src/lib.rs"][0], 1)
        self.assertEqual(from_tag["src/lib.rs"][0], 2)
        self.assertNotEqual(
            from_baseline["src/lib.rs"][0], from_tag["src/lib.rs"][0]
        )


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
        base = (self.root / report.BASELINE_PATH).read_text()
        for broken in ("carried = 1", 'carried = "two"', "carried = [1, 2]"):
            with self.subTest(broken=broken):
                body = "\n".join(
                    line for line in base.splitlines()
                    if not line.strip().startswith(("[[carried]]", "subject", "patch_id"))
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
        lines, replaced = [], 0
        for line in text.splitlines():
            if line.strip().startswith("tree = "):
                line, replaced = 'tree = "7b010d1b"', replaced + 1
            lines.append(line)
        # Assert the edit landed. Without this the test passes for the wrong reason:
        # an unmodified config yields a stale-report finding, exit 1, not exit 2.
        self.assertEqual(replaced, 1, text)
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


if __name__ == "__main__":
    unittest.main()
