"""Unit tests for tools/target-dir-check: `python3 -m unittest discover -s tools`."""
import os
import pathlib
import subprocess
import sys
import tempfile
import unittest

# Git exports GIT_DIR, GIT_WORK_TREE and GIT_INDEX_FILE into hook processes and they
# OVERRIDE `git -C <dir>`; check_cmd runs this suite from the pre-commit hook, so a
# fixture's worktrees would otherwise land in THIS repository. Same scrub as
# test_upstream_report.py.
for _name in [_key for _key in os.environ if _key.startswith("GIT_")]:
    del os.environ[_name]

TOOL = pathlib.Path(__file__).with_name("target-dir-check")


class TargetDirCheckTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        base = pathlib.Path(self.tmp.name).resolve()
        self.main = base / "main"
        self.main.mkdir()
        self.git(self.main, "init", "-q", "-b", "main")
        self.git(self.main, "config", "user.email", "fixture@example.invalid")
        self.git(self.main, "config", "user.name", "Fixture")
        self.git(self.main, "config", "commit.gpgsign", "false")
        toplevel = self.git(self.main, "rev-parse", "--show-toplevel").strip()
        self.assertEqual(pathlib.Path(toplevel), self.main, "fixture git escaped its temp dir")
        (self.main / "Cargo.toml").write_text(
            '[package]\nname = "fixture"\nversion = "0.1.0"\nedition = "2021"\n')
        (self.main / "src").mkdir()
        (self.main / "src/lib.rs").write_text("")
        (self.main / ".gitignore").write_text("/target\n/.cargo\n")
        self.git(self.main, "add", ".")
        self.git(self.main, "commit", "-qm", "fixture")
        self.a = base / "wt-a"
        self.b = base / "wt-b"
        self.git(self.main, "worktree", "add", "-q", str(self.a), "-b", "a")
        self.git(self.main, "worktree", "add", "-q", str(self.b), "-b", "b")
        # A host's own cargo config or CARGO_* environment would make every fixture
        # checkout share; each run gets an empty CARGO_HOME and none of the variables.
        self.cargo_home = base / "cargo-home"
        self.cargo_home.mkdir()

    def git(self, cwd, *args):
        return subprocess.run(
            ("git", *args), cwd=cwd, check=True, capture_output=True, text=True).stdout

    def point(self, checkout, target, key="target-dir"):
        (checkout / ".cargo").mkdir(exist_ok=True)
        (checkout / ".cargo/config.toml").write_text(f'[build]\n{key} = "{target}"\n')

    def check(self, cwd, env=None):
        clean = {k: v for k, v in os.environ.items() if not k.startswith("CARGO_")}
        clean.pop("CARGO", None)
        return subprocess.run(
            (sys.executable, str(TOOL)), cwd=cwd, capture_output=True, text=True,
            env={**clean, "CARGO_HOME": str(self.cargo_home), **(env or {})})

    def test_worktrees_with_their_own_target_pass(self):
        for checkout in (self.main, self.a, self.b):
            result = self.check(checkout)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(result.stderr, "")

    def test_worktree_borrowing_the_main_target_fails(self):
        self.point(self.a, self.main / "target")
        result = self.check(self.a)
        self.assertEqual(result.returncode, 1)
        self.assertIn(str(self.main / "target"), result.stderr)
        self.assertIn(str(self.main), result.stderr)
        self.assertIn("build.target-dir", result.stderr)
        # The other worktree keeps its own target and is unaffected.
        self.assertEqual(self.check(self.b).returncode, 0)

    def test_main_checkout_warns_but_passes(self):
        self.point(self.a, self.main / "target")
        result = self.check(self.main)
        self.assertEqual(result.returncode, 0)
        self.assertIn(str(self.a), result.stderr)

    def test_two_worktrees_sharing_a_third_dir_both_fail(self):
        shared = pathlib.Path(self.tmp.name).resolve() / "shared-target"
        self.point(self.a, shared)
        self.point(self.b, shared)
        for checkout, other in ((self.a, self.b), (self.b, self.a)):
            result = self.check(checkout)
            self.assertEqual(result.returncode, 1)
            self.assertIn(str(other), result.stderr)
        self.assertEqual(self.check(self.main).returncode, 0)

    def test_same_dir_through_a_symlink_is_shared(self):
        link = pathlib.Path(self.tmp.name).resolve() / "link-to-main-target"
        (self.main / "target").mkdir()
        link.symlink_to(self.main / "target")
        self.point(self.a, link)
        self.assertEqual(self.check(self.a).returncode, 1)

    def test_cargo_target_dir_in_the_environment_is_named(self):
        result = self.check(self.a, {"CARGO_TARGET_DIR": str(self.main / "target")})
        self.assertEqual(result.returncode, 1)
        self.assertIn("CARGO_TARGET_DIR", result.stderr)

    def test_missing_worktree_is_skipped(self):
        self.point(self.a, self.main / "target")
        subprocess.run(("rm", "-rf", str(self.a)), check=True)
        result = self.check(self.main)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.check(self.b).returncode, 0)

    def test_locked_worktree_whose_directory_is_gone_is_skipped(self):
        # git never lists a locked worktree as prunable.
        self.git(self.main, "worktree", "lock", str(self.a))
        subprocess.run(("rm", "-rf", str(self.a)), check=True)
        for checkout in (self.main, self.b):
            result = self.check(checkout)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn(str(self.a), result.stderr)

    def test_another_checkout_with_a_broken_manifest_is_skipped(self):
        self.point(self.b, self.main / "target")
        (self.b / "Cargo.toml").write_text("<<<<<<< conflict\n")
        result = self.check(self.a)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn(str(self.b), result.stderr)

    def test_this_checkout_with_a_broken_manifest_cannot_run(self):
        (self.a / "Cargo.toml").write_text("<<<<<<< conflict\n")
        self.assertEqual(self.check(self.a).returncode, 2)

    def test_a_shared_build_dir_is_shared(self):
        shared = pathlib.Path(self.tmp.name).resolve() / "shared-build"
        self.point(self.a, shared, key="build-dir")
        self.point(self.b, shared, key="build-dir")
        result = self.check(self.a)
        self.assertEqual(result.returncode, 1)
        self.assertIn(str(self.b), result.stderr)

    def test_runs_from_a_subdirectory(self):
        self.point(self.a, self.main / "target")
        self.assertEqual(self.check(self.a / "src").returncode, 1)
        self.assertEqual(self.check(self.b / "src").returncode, 0)

    def test_cargo_build_target_dir_in_the_environment_is_named(self):
        result = self.check(self.a, {"CARGO_BUILD_TARGET_DIR": str(self.main / "target")})
        self.assertEqual(result.returncode, 1)
        self.assertIn("CARGO_BUILD_TARGET_DIR", result.stderr)


if __name__ == "__main__":
    unittest.main()
