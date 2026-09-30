"""Exercise the gate recipes and Git LFS handoff without pushing or building Rust."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parent.parent


class Gates(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.bin = self.root / "bin"
        self.bin.mkdir()
        self.env = dict(os.environ, PATH=f"{self.bin}:{os.environ['PATH']}",
                        TT_LOG=str(self.root / "timings.jsonl"), GATE_TMP=str(self.root))
        self.git = shutil.which("git")
        self.just = shutil.which("just")
        self.run_git("init", "-q")
        self.stub("just", 'if [ "$1" = --evaluate ] && [ "$2" = "${EVAL_FAIL:-}" ]; then printf %s origin; exit 1; fi\n'
                  f'if [ "$1" = --evaluate ]; then exec "{self.just}" --justfile "{ROOT / "justfile"}" "$@"; fi\n'
                  'printf %s "$1" > "$GATE_TMP/recipe"')
        self.stub("git", 'if [ "$1" = lfs ]; then cat > "$GATE_TMP/lfs-refs"; exit "${LFS_EXIT:-0}"; fi\n'
                  f'exec "{self.git}" "$@"')

    def stub(self, name, body):
        path = self.bin / name
        path.write_text(f"#!/bin/sh\n{body}\n")
        path.chmod(0o755)

    def run_git(self, *args):
        subprocess.run([self.git, *args], cwd=self.root, check=True, capture_output=True)

    def hook(self, name, *args, refs=""):
        return subprocess.run(["sh", str(ROOT / ".githooks" / name), *args], cwd=self.root,
                              env=self.env, input=refs, capture_output=True, text=True)

    def test_staged_paths_select_docs_only_conservatively(self):
        for name, expected in (("README.md", "hook-pre-commit-docs"),
                               ("tasks/material-123456.md", "hook-pre-commit-docs"),
                               ("docs/specs/a.md", "hook-pre-commit-docs"),
                               ("docs/materials/scripts/a.py", "hook-pre-commit"),
                               ("docs/materials/material-config.md", "hook-pre-commit"),
                               ("docs/wiki/examples/a.frag", "hook-pre-commit"),
                               ("docs/wiki/Configuration.md", "hook-pre-commit"),
                               ("src/a.rs", "hook-pre-commit"),
                               ("src/fixture.md", "hook-pre-commit"),
                               ("a.md\ndocs/b", "hook-pre-commit")):
            with self.subTest(name=name):
                self.run_git("read-tree", "--empty")
                path = self.root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("changed\n")
                self.run_git("add", "--", name)
                result = self.hook("pre-commit")
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual((self.root / "recipe").read_text(), expected)

    def test_push_selects_ci_gate_and_preserves_lfs_input(self):
        branch = f"refs/heads/topic {'1' * 40} refs/heads/topic {'2' * 40}\n"
        tag = f"refs/tags/v1 {'1' * 40} refs/tags/v1 {'0' * 40}\n"
        deletion = f"(delete) {'0' * 40} refs/heads/old {'2' * 40}\n"
        for remote, refs, expected in (("origin", branch, "hook-pre-push-fast"),
                                       ("origin", branch + tag, "hook-pre-push-fast"),
                                       ("origin", deletion, "hook-pre-push-fast"),
                                       ("backup", branch, "hook-pre-push"),
                                       ("origin", "bad input\n", "hook-pre-push"),
                                       ("origin", "", "hook-pre-push")):
            with self.subTest(remote=remote, refs=refs):
                result = self.hook("pre-push", remote, "https://example.invalid/repo", refs=refs)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual((self.root / "recipe").read_text(), expected)
                self.assertEqual((self.root / "lfs-refs").read_text(), refs)

    def test_failing_remote_evaluation_with_partial_output_takes_full_gate(self):
        self.env["EVAL_FAIL"] = "ci_remote"
        result = self.hook("pre-push", "origin", "https://example.invalid/repo",
                           refs=f"refs/heads/a {'1' * 40} refs/heads/a {'2' * 40}\n")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual((self.root / "recipe").read_text(), "hook-pre-push")

    def test_lfs_failure_stops_before_gate(self):
        self.env["LFS_EXIT"] = "7"
        result = self.hook("pre-push", "origin", "https://example.invalid/repo",
                           refs=f"refs/heads/a {'1' * 40} refs/heads/a {'2' * 40}\n")
        self.assertEqual(result.returncode, 7)
        self.assertFalse((self.root / "recipe").exists())

    def test_focused_recipe_preserves_arguments_and_counts_stderr(self):
        argv = self.root / "argv.py"
        argv.write_text("import json, sys\nprint(json.dumps(sys.argv[1:]), flush=True)\n"
                        "print('Ran 1 test in 0.001s', file=sys.stderr)\n")
        args = ["-E", "test(foo) | test(bar)", "a b", "it's", "$HOME", "*"]
        result = subprocess.run([self.just, "--set", "one_cmd", f"python3 {argv}",
                                 "test-one", *args], cwd=ROOT, env=self.env,
                                capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(json.loads(result.stdout.splitlines()[0]), args)
        record = json.loads((self.root / "timings.jsonl").read_text().splitlines()[-1])
        self.assertEqual((record["target"], record["tests"]), ("test-one", 1))

    def test_focused_recipe_requires_arguments(self):
        result = subprocess.run([self.just, "test-one"], cwd=ROOT, env=self.env,
                                capture_output=True, text=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse((self.root / "timings.jsonl").exists())
