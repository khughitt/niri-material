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
        # Fixtures provide host-budget themselves; CI has no shared ops tooling.
        self.env = dict(os.environ, PATH=f"{self.bin}:/usr/local/bin:/usr/bin:/bin",
                        TT_LOG=str(self.root / "timings.jsonl"), GATE_TMP=str(self.root))
        self.git = shutil.which("git")
        self.just = shutil.which("just")
        self.stub('host-budget', '[ "$1" = run ] || exit 2\nshift\n'
                  '[ "$1" = -- ] || exit 2\nshift\nexec "$@"')
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

    def test_focused_recipe_records_failure_through_host_budget(self):
        script = self.root / 'fail.py'
        script.write_text("import sys\nprint('Ran 1 test in 0.001s', file=sys.stderr)\nsys.exit(7)\n")
        result = subprocess.run([self.just, '--set', 'one_cmd', f'python3 {script}',
                                 'test-one', 'ignored'], cwd=ROOT, env=self.env,
                                capture_output=True, text=True)
        self.assertNotEqual(result.returncode, 0)
        record = json.loads((self.root / 'timings.jsonl').read_text().splitlines()[-1])
        self.assertEqual((record['exit'], record['tests']), (7, 1))

    def stage(self, *names):
        self.run_git('read-tree', '--empty')
        for name in names:
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text('changed\n')
        self.run_git('add', '--', *names)

    def assert_commit_route(self, expected):
        run = self.hook('pre-commit')
        self.assertEqual(run.returncode, 0, run.stderr)
        self.assertEqual((self.root / 'recipe').read_text(), expected)

    def test_narrow_subjects_select_full_and_unrelated_paths_select_fast(self):
        full = ['tools/optic_settling.py', 'tools/screencast_consumer.py', 'tools/fake_screencast.py',
                'tools/test_optic_settling.py', 'tools/test_screencast_consumer.py', 'tools/test_vt_lib.py',
                'tools/tooling_tests.py', 'tools/test_tooling_tests.py',
                'docs/materials/scripts/vt-lib.sh', 'docs/materials/scripts/optic-settling-smoke.sh',
                'docs/materials/scripts/glass-optic-smoke-lib.sh', 'docs/materials/scripts/window-client.c',
                '.githooks/pre-commit', 'justfile', '.github/workflows/ci.yml']
        fast = ['tools/upstream-report', 'tools/package-pin', 'tools/test-affected', 'tools/tt',
                'docs/materials/scripts/unrelated-capture.sh', 'src/layout/mod.rs']
        for paths, expected in ((full, 'hook-pre-commit-full'), (fast, 'hook-pre-commit')):
            for name in paths:
                with self.subTest(name=name):
                    self.stage(name)
                    self.assert_commit_route(expected)
        self.stage('README.md', 'tools/optic_settling.py')
        self.assert_commit_route('hook-pre-commit-full')

    def test_commit_classification_errors_and_empty_index_take_full(self):
        self.assert_commit_route('hook-pre-commit-full')
        self.stage('README.md')
        for variable in ('docs_paths', 'tooling_full_paths'):
            with self.subTest(variable=variable):
                self.env['EVAL_FAIL'] = variable
                self.assert_commit_route('hook-pre-commit-full')
        self.env.pop('EVAL_FAIL')
        for output, status in ((r'README.md\000', 7), (r'\377\000', 0)):
            with self.subTest(output=output):
                self.stub('git', f'printf "{output}"\nexit {status}')
                self.assert_commit_route('hook-pre-commit-full')

    def test_deleted_subject_and_both_rename_endpoints_take_full(self):
        for old, new in (('tools/optic_settling.py', None),
                         ('tools/optic_settling.py', 'tools/unrelated.py'),
                         ('tools/unrelated.py', 'tools/optic_settling.py')):
            with self.subTest(old=old, new=new):
                for name in ('tools/optic_settling.py', 'tools/unrelated.py'):
                    (self.root / name).unlink(missing_ok=True)
                self.stage(old)
                self.run_git('-c', 'user.name=fixture', '-c', 'user.email=fixture@example.invalid',
                             'commit', '--allow-empty', '-qm', 'test: snapshot')
                if new is None:
                    self.run_git('rm', old)
                else:
                    self.run_git('mv', old, new)
                self.assert_commit_route('hook-pre-commit-full')

    def recipe_fixture(self):
        marker = self.root / 'marker.py'
        marker.write_text('''import json, os, sys
kind=sys.argv[1]
with open(os.environ['GATE_TMP']+'/events', 'a') as out:
    out.write(json.dumps(dict(kind=kind, mode=os.environ.get('NIRI_TOOLING_FAST'), args=sys.argv[2:]))+'\\n')
if kind in ('fast', 'full'): print('Ran '+str(2 if kind=='fast' else 3)+' tests in 0.001s', file=sys.stderr)
if kind in ('rust', 'push'): print('test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s')
if os.environ.get('FAIL_EVENT')==kind: sys.exit(7)
''')
        real_python = shutil.which('python3')
        self.stub('python3', 'if [ "$1" = -m ]; then\n'
                  f'  if [ "$2" = unittest ]; then shift 2; exec "{real_python}" "{marker}" fast "$@"; fi\n'
                  '  if [ "$2" = tools.tooling_tests ]; then shift 2;\n'
                  f'    if [ "$1" = --full ]; then exec "{real_python}" "{marker}" full "$@"; fi\n'
                  f'    if [ "$1" = --check-paths ]; then exec "{real_python}" "{marker}" paths "$@"; fi\n'
                  '  fi\nfi\n'
                  f'exec "{real_python}" "$@"')
        overrides = []
        for name, kind in (('check_before_cmd', 'before'), ('check_after_cmd', 'after'),
                           ('hygiene_cmd', 'hygiene'), ('stage_report_cmd', 'stage'),
                           ('test_cmd', 'rust'), ('push_fast_cmd', 'push')):
            overrides += ['--set', name, f'python3 {marker} {kind}']
        self.env['NIRI_TOOLING_FAST'] = '1'
        return overrides

    def test_recipe_composition_modes_counts_and_shared_hook_target(self):
        overrides = self.recipe_fixture()
        cases = [
            ('check', ['before', 'fast', 'after']),
            ('check-full', ['before', 'full', 'after']),
            ('gate', ['before', 'full', 'after', 'rust']),
            ('hook-pre-commit', ['stage', 'before', 'fast', 'after']),
            ('hook-pre-commit-full', ['stage', 'before', 'full', 'after']),
            ('hook-pre-commit-docs', ['stage', 'hygiene', 'paths', 'after']),
            ('hook-pre-push-fast', ['before', 'fast', 'after', 'push']),
            ('hook-pre-push', ['before', 'full', 'after', 'rust']),
            ('test', ['rust']), ('ci-test', ['rust']), ('ci-test-release', ['rust']),
            ('ci-tooling-test', ['full']),
        ]
        for recipe, expected in cases:
            with self.subTest(recipe=recipe):
                (self.root / 'events').unlink(missing_ok=True)
                (self.root / 'timings.jsonl').unlink(missing_ok=True)
                run = subprocess.run([self.just, *overrides, recipe], cwd=ROOT, env=self.env,
                                     capture_output=True, text=True)
                self.assertEqual(run.returncode, 0, run.stderr)
                events = [json.loads(line) for line in (self.root / 'events').read_text().splitlines()]
                self.assertEqual([event['kind'] for event in events], expected)
                for event in events:
                    if event['kind'] in ('fast', 'full', 'paths'):
                        self.assertEqual(event['mode'], '1' if event['kind']=='fast' else '0')
                records = [json.loads(line) for line in (self.root / 'timings.jsonl').read_text().splitlines()]
                if recipe in ('hook-pre-commit', 'hook-pre-commit-full'):
                    self.assertEqual((records[-1]['target'], records[-1]['tests']),
                                     ('hook-pre-commit', 2 if recipe=='hook-pre-commit' else 3))
                if recipe == 'hook-pre-commit-docs':
                    self.assertIsNone(records[-1]['tests'])
                if recipe == 'ci-tooling-test':
                    self.assertIn('--ci', events[0]['args'])
                if recipe == 'ci-test-release':
                    self.assertIn('--release', events[0]['args'])

    def test_full_recipe_failure_stops_before_following_stages(self):
        overrides = self.recipe_fixture()
        self.env['FAIL_EVENT'] = 'full'
        run = subprocess.run([self.just, *overrides, 'hook-pre-commit-full'], cwd=ROOT,
                             env=self.env, capture_output=True, text=True)
        self.assertNotEqual(run.returncode, 0)
        events = [json.loads(line)['kind'] for line in (self.root / 'events').read_text().splitlines()]
        self.assertEqual(events, ['stage', 'before', 'full'])
        record = json.loads((self.root / 'timings.jsonl').read_text().splitlines()[-1])
        self.assertEqual((record['target'], record['exit'], record['tests']), ('hook-pre-commit', 7, 3))
