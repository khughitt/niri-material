"""Unit tests for tools/identity-check: `python3 -m unittest discover -s tools`."""
import os
import pathlib
import subprocess
import sys
import tempfile
import unittest

# Git exports GIT_DIR, GIT_WORK_TREE and GIT_INDEX_FILE into hook processes and they
# OVERRIDE `git -C <dir>`; check_cmd runs this suite from the pre-commit hook, so a
# fixture's `git config` would otherwise land in THIS repository's .git/config. Same
# scrub as test_upstream_report.py.
for _name in [_key for _key in os.environ if _key.startswith("GIT_")]:
    del os.environ[_name]

TOOL = pathlib.Path(__file__).with_name("identity-check")


class IdentityCheckTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        base = pathlib.Path(self.tmp.name).resolve()
        self.repo = base / "repo"
        self.repo.mkdir()
        # A global identity the check must accept, kept out of the user's real config.
        self.env = {**os.environ, "GIT_CONFIG_GLOBAL": str(base / "global"),
                    "GIT_CONFIG_NOSYSTEM": "1"}
        self.git(self.repo, "init", "-q", "-b", "main")
        toplevel = self.git(self.repo, "rev-parse", "--show-toplevel").strip()
        self.assertEqual(pathlib.Path(toplevel), self.repo, "fixture git escaped its temp dir")
        self.git(self.repo, "config", "--global", "user.name", "Owner")
        self.git(self.repo, "config", "--global", "user.email", "owner@example.invalid")

    def git(self, cwd, *args):
        return subprocess.run(("git", *args), cwd=cwd, env=self.env, check=True,
                              capture_output=True, text=True).stdout

    def check(self, cwd, env=None):
        return subprocess.run((sys.executable, str(TOOL)), cwd=cwd, env=env or self.env,
                              capture_output=True, text=True, check=False)

    def test_global_identity_alone_passes(self):
        result = self.check(self.repo)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stderr, "")

    def test_local_identity_is_refused(self):
        self.git(self.repo, "config", "user.name", "Fixture")
        self.git(self.repo, "config", "user.email", "fixture@example.invalid")
        result = self.check(self.repo)
        self.assertEqual(result.returncode, 1)
        self.assertIn("user.name=Fixture (local)", result.stderr)
        self.assertIn("user.email=fixture@example.invalid (local)", result.stderr)
        self.assertIn("git config --local --unset user.name", result.stderr)

    def test_worktree_identity_is_refused(self):
        self.git(self.repo, "config", "extensions.worktreeConfig", "true")
        self.git(self.repo, "config", "--worktree", "user.email", "fixture@example.invalid")
        result = self.check(self.repo)
        self.assertEqual(result.returncode, 1)
        self.assertIn("user.email=fixture@example.invalid (worktree)", result.stderr)

    def test_linked_worktree_sees_the_shared_local_config(self):
        self.git(self.repo, "commit", "-q", "--allow-empty", "-m", "root")
        linked = self.repo.parent / "linked"
        self.git(self.repo, "worktree", "add", "-q", str(linked), "-b", "linked")
        self.git(self.repo, "config", "user.name", "Fixture")
        self.assertEqual(self.check(linked).returncode, 1)

    def test_hook_environment_checks_the_repository_git_names(self):
        # Inside a hook GIT_DIR names the repository being committed to, whatever
        # the working directory: that repository is the one checked.
        self.git(self.repo, "config", "user.name", "Fixture")
        elsewhere = self.repo.parent / "elsewhere"
        elsewhere.mkdir()
        env = {**self.env, "GIT_DIR": str(self.repo / ".git")}
        self.assertEqual(self.check(elsewhere, env).returncode, 1)

    def test_outside_a_repository_cannot_run(self):
        elsewhere = self.repo.parent / "elsewhere"
        elsewhere.mkdir()
        env = {**self.env, "GIT_CEILING_DIRECTORIES": str(self.repo.parent)}
        result = self.check(elsewhere, env)
        self.assertEqual(result.returncode, 2)
        self.assertIn("identity-check:", result.stderr)


if __name__ == "__main__":
    unittest.main()
