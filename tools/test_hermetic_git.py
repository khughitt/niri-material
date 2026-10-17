"""isolate() keeps the host's git environment and configuration out of tooling tests."""
import os
import pathlib
import re
import subprocess
import sys
import tempfile
import unittest

from tools.hermetic_git import isolate

isolate(os.environ)

ROOT = pathlib.Path(__file__).resolve().parents[1]
IDENTITY = ('-c', 'user.name=Fixture', '-c', 'user.email=fixture@example.invalid')


def git(cwd, env, *args):
    return subprocess.run(['git', '-C', str(cwd), *args], env=env, check=True,
                          capture_output=True, text=True).stdout.strip()


def git_version():
    text = git(ROOT, os.environ, '--version')
    return tuple(int(part) for part in re.match(r'git version (\d+)\.(\d+)', text).groups())


class IsolateTests(unittest.TestCase):
    def test_scrubs_git_keys_and_pins_config_sources(self):
        environ = {'PATH': '/bin', 'GIT_DIR': 'd', 'GIT_WORK_TREE': 'w', 'GIT_INDEX_FILE': 'i',
                   'GIT_CONFIG_GLOBAL': 'g', 'GIT_CONFIG_COUNT': '1'}
        isolate(environ)
        expected = {'PATH': '/bin', 'GIT_CONFIG_GLOBAL': os.devnull, 'GIT_CONFIG_NOSYSTEM': '1'}
        self.assertEqual(environ, expected)
        isolate(environ)
        self.assertEqual(environ, expected)


class SuppressionTests(unittest.TestCase):
    """Each host hook form fires in a control commit, then not after isolate."""

    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.base = pathlib.Path(temporary.name)
        self.marker = self.base / 'marker'
        self.repo = self.base / 'repo'
        self.clean = dict(os.environ)
        isolate(self.clean)
        git(self.base, self.clean, 'init', '-q', '-b', 'main', str(self.repo))

    def host(self, config):
        path = self.base / 'host-gitconfig'
        path.write_text(config)
        return dict(self.clean, GIT_CONFIG_GLOBAL=str(path))

    def commit(self, env, message):
        (self.repo / 'f').write_text(message)
        git(self.repo, env, 'add', 'f')
        git(self.repo, env, *IDENTITY, 'commit', '-q', '-m', message)

    def control_then_isolated(self, env):
        self.commit(env, 'control')
        fired = self.marker.exists()
        self.marker.unlink(missing_ok=True)
        isolate(env)
        self.commit(env, 'isolated')
        return fired, self.marker.exists()

    def test_core_hooks_path_in_host_config(self):
        hooks = self.base / 'hooks'
        hooks.mkdir()
        script = hooks / 'pre-commit'
        script.write_text(f"#!/bin/sh\ntouch '{self.marker}'\n")
        script.chmod(0o755)
        fired, after = self.control_then_isolated(self.host(f'[core]\n\thooksPath = {hooks}\n'))
        self.assertTrue(fired, 'control commit did not run the core.hooksPath hook')
        self.assertFalse(after, 'the host core.hooksPath hook ran after isolate')

    def test_config_declared_hook_in_host_config(self):
        env = self.host(f"[hook \"probe\"]\n\tevent = pre-commit\n\tcommand = touch '{self.marker}'\n")
        fired, after = self.control_then_isolated(env)
        if git_version() < (2, 54):
            # Config hooks arrived in Git 2.54; on older Git (CI's packaged one) the form is inert.
            self.assertFalse(fired, 'a config hook fired on Git older than 2.54')
        else:
            self.assertTrue(fired, 'control commit did not run the config-declared hook')
        self.assertFalse(after, 'the host config hook ran after isolate')


PREPARE = 'tools.test_optic_settling.PrepareTests.test_prepare_validates_inventory_and_cleans_runtime'


class FocusedRunTests(unittest.TestCase):
    def test_optic_settling_prepare_records_the_checkout_head_under_a_decoy_git_dir(self):
        clean = dict(os.environ)
        isolate(clean)
        head = git(ROOT, clean, 'rev-parse', 'HEAD')
        with tempfile.TemporaryDirectory() as temporary:
            decoy = pathlib.Path(temporary) / 'decoy'
            git(temporary, clean, 'init', '-q', '-b', 'main', str(decoy))
            git(decoy, clean, *IDENTITY, 'commit', '-q', '--allow-empty', '-m', 'decoy')
            self.assertNotEqual(git(decoy, clean, 'rev-parse', 'HEAD'), head)
            run = subprocess.run([sys.executable, '-m', 'unittest', PREPARE], cwd=ROOT,
                                 env=dict(os.environ, GIT_DIR=str(decoy / '.git')),
                                 capture_output=True, text=True, timeout=120)
            self.assertEqual(run.returncode, 0, run.stdout + run.stderr)


if __name__ == '__main__':
    unittest.main()
