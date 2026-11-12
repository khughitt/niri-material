"""Exercise the gate recipes and Git LFS handoff without pushing or building Rust."""
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile
import unittest

from tools.hermetic_git import isolate

# Fixture repositories here must not see a hook's GIT_INDEX_FILE or GIT_DIR, nor host git config.
isolate(os.environ)

ROOT = Path(__file__).resolve().parent.parent

BROAD_TARGETS = [
    "check-code",
    "check-code-full",
    "ci-test",
    "ci-test-release",
    "ci-tooling-test",
    "test",
    "test-fast",
    "test-push-fast",
]


class Gates(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        base = Path(self.tmp.name)
        self.home = base / "home"
        self.xdg = base / "xdg"
        self.bin = base / "bin"
        self.root = base / "repo"
        for directory in (self.home, self.xdg, self.bin, self.root,
                          self.root / "tools", self.root / "tasks"):
            directory.mkdir(parents=True)
        shutil.copy2(ROOT / "justfile", self.root / "justfile")
        shutil.copy2(ROOT / "tools" / "tt", self.root / "tools" / "tt")
        (self.root / "tasks" / ".config.toml").write_text('prefix = "fixture"\n', encoding="utf-8")
        # Fixtures provide host-budget themselves and never inherit host credentials or HOME.
        self.env = {
            "PATH": f"{self.bin}:/usr/local/bin:/usr/bin:/bin",
            "HOME": str(self.home),
            "XDG_CONFIG_HOME": str(self.xdg),
            "TT_LOG": str(self.root / "timings.jsonl"),
            "GATE_TMP": str(self.root),
        }
        isolate(self.env)
        self.git = shutil.which("git")
        self.just = shutil.which("just")
        self.real_python = shutil.which("python3")
        self.stub('host-budget', '[ "$1" = run ] || exit 2\nshift\n'
                  '[ "$1" = -- ] || exit 2\nshift\nexec "$@"')
        self.stub(
            "ssh",
            'printf "%s\\n" "$*" >> "$GATE_TMP/ssh-argv"\n'
            'printf "serveraliveinterval %s\\n" "${FAKE_SSH_ALIVE:-60}"',
        )
        self.run_git("init", "-q")
        self.stub(
            "just",
            'if [ "$1" = --evaluate ] && [ "$2" = "${EVAL_FAIL:-}" ]; then printf %s origin; exit 1; fi\n'
            f'if [ "$1" = --evaluate ]; then exec "{self.just}" --justfile "{self.root / "justfile"}" "$@"; fi\n'
            'printf %s "$1" > "$GATE_TMP/recipe"\n'
            f'exec "{self.real_python}" -c \'import json, os, pathlib; '
            'keys = ("GIT_INDEX_FILE", "GIT_DIR", "GIT_LITERAL_PATHSPECS", '
            '"GIT_GLOB_PATHSPECS", "GIT_NOGLOB_PATHSPECS", "GIT_ICASE_PATHSPECS"); '
            'pathlib.Path(os.environ["GATE_TMP"], "just-env.json").write_text('
            'json.dumps({k: os.environ.get(k) for k in keys}))\'',
        )
        self.stub("git", 'if [ "$1" = lfs ]; then cat > "$GATE_TMP/lfs-refs"; exit "${LFS_EXIT:-0}"; fi\n'
                  f'exec "{self.git}" "$@"')

    def stub(self, name, body):
        path = self.bin / name
        path.write_text(f"#!/bin/sh\n{body}\n")
        path.chmod(0o755)

    def run_git(self, *args, env=None):
        subprocess.run([self.git, *args], cwd=self.root,
                       env=self.env if env is None else env,
                       check=True, capture_output=True)

    def hook(self, name, *args, refs="", env=None):
        return subprocess.run(["sh", str(ROOT / ".githooks" / name), *args], cwd=self.root,
                              env=self.env if env is None else env, input=refs,
                              capture_output=True, text=True)

    def write_policy(self, *, automatic_broad=False, broad_targets=BROAD_TARGETS):
        rendered = ", ".join(json.dumps(target) for target in broad_targets)
        value = "true" if automatic_broad else "false"
        (self.root / ".tt.toml").write_text(
            f"automatic_broad = {value}\nbroad_targets = [{rendered}]\n",
            encoding="utf-8",
        )

    def timings(self):
        path = self.root / "timings.jsonl"
        if not path.exists():
            return []
        return [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines()]

    def reset_gate_logs(self):
        (self.root / "events").unlink(missing_ok=True)
        (self.root / "timings.jsonl").unlink(missing_ok=True)

    def run_recipe(self, *args, env=None):
        return subprocess.run(
            [self.just, *args],
            cwd=self.root,
            env=self.env if env is None else env,
            capture_output=True,
            text=True,
        )

    def test_hygiene_checks_guides_after_ops_check(self):
        # Every route runs hygiene_cmd, so a stale identity region fails each of them.
        value = subprocess.run([self.just, "--justfile", str(self.root / "justfile"), "--evaluate",
                                "hygiene_cmd"], cwd=self.root, env=self.env,
                               capture_output=True, text=True, check=True).stdout
        self.assertEqual(value.split(" && "),
                         ["python3 tools/ops-check", "python3 tools/ops-docs check"])

    def test_staged_paths_select_docs_only_conservatively(self):
        for name, expected in (("README.md", "hook-pre-commit-docs"),
                               ("CLAUDE.md", "hook-pre-commit-docs"),
                               ("identity.toml", "hook-pre-commit-docs"),
                               ("CONTRIBUTING.md", "hook-pre-commit-docs"),
                               ("docs/specs/a.md", "hook-pre-commit"),
                               ("scripts/capture/a.py", "hook-pre-commit"),
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
        result = self.run_recipe("--set", "one_cmd", f"python3 {argv}", "test-one", *args)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(json.loads(result.stdout.splitlines()[0]), args)
        record = self.timings()[-1]
        self.assertEqual((record["target"], record["tests"]), ("test-one", 1))

    def test_focused_recipe_requires_arguments(self):
        result = self.run_recipe("test-one")
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse((self.root / "timings.jsonl").exists())

    def test_focused_recipe_records_failure_through_host_budget(self):
        script = self.root / 'fail.py'
        script.write_text("import sys\nprint('Ran 1 test in 0.001s', file=sys.stderr)\nsys.exit(7)\n")
        result = self.run_recipe('--set', 'one_cmd', f'python3 {script}', 'test-one', 'ignored')
        self.assertNotEqual(result.returncode, 0)
        record = self.timings()[-1]
        self.assertEqual((record['exit'], record['tests']), (7, 1))

    def stage(self, *names):
        self.run_git('read-tree', '--empty')
        for name in names:
            path = self.root / name
            if not path.exists():
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
                'scripts/capture/vt-lib.sh', 'scripts/capture/optic-settling-smoke.sh',
                'scripts/capture/glass-optic-smoke-lib.sh', 'scripts/capture/window-client.c',
                '.githooks/pre-commit', 'justfile', '.github/workflows/ci.yml', '.tt.toml']
        fast = ['tools/upstream-report', 'tools/package-pin', 'tools/test-affected', 'tools/tt',
                'scripts/capture/unrelated-capture.sh', 'src/layout/mod.rs']
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
        marker = self.bin / 'marker'
        # Native shell markers avoid starting Python for each stage. NUL fields
        # with an argument count preserve argv without quoting or JSON escaping.
        self.stub('marker', '''kind=$1
shift
{
    printf '%s\\0%s\\0%s\\0' "$kind" "${NIRI_TOOLING_FAST-unset}" "$#"
    for arg do printf '%s\\0' "$arg"; done
} >> "$GATE_TMP/events"
case "$kind" in
    fast) printf 'Ran 2 tests in 0.001s\\n' >&2 ;;
    full) printf 'Ran 3 tests in 0.001s\\n' >&2 ;;
    affected) printf 'test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\\n' ;;
    rust|push) printf 'test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\\n' ;;
esac
if [ "${FAIL_EVENT-}" = "$kind" ]; then exit 7; fi''')
        self.stub('python3', 'if [ "$1" = -m ]; then\n'
                  f'  if [ "$2" = unittest ]; then shift 2; exec "{marker}" fast "$@"; fi\n'
                  '  if [ "$2" = tools.tooling_tests ]; then shift 2;\n'
                  f'    if [ "$1" = --fast ]; then exec "{marker}" fast "$@"; fi\n'
                  f'    if [ "$1" = --full ]; then exec "{marker}" full "$@"; fi\n'
                  f'    if [ "$1" = --check-paths ]; then exec "{marker}" paths "$@"; fi\n'
                  '  fi\nfi\n'
                  f'exec "{self.real_python}" "$@"')
        self.stub('just', f'exec "{self.just}" "$@"')
        replacements = {
            'hygiene_cmd': f'"{marker} hygiene"',
            'target_dir_cmd': f'"{marker} target-dir"',
            'identity_cmd': f'"{marker} identity"',
            'check_after_cmd': f'"{marker} after"',
            'check_code_base_cmd': f'"{marker} lint"',
            'stage_report_cmd': f'"{marker} stage"',
            'fast_cmd': f'"{marker} affected"',
            'test_cmd': f'"{marker} rust"',
            'push_fast_cmd': f'"{marker} push"',
        }
        justfile_path = self.root / 'justfile'
        lines = []
        for line in justfile_path.read_text(encoding='utf-8').splitlines():
            match = re.match(r'^([a-z_]+)\s*:=\s*.*$', line)
            if match and match.group(1) in replacements:
                line = f'{match.group(1)} := {replacements[match.group(1)]}'
            lines.append(line)
        justfile_path.write_text('\n'.join(lines) + '\n', encoding='utf-8')
        self.env['NIRI_TOOLING_FAST'] = '1'

    def recipe_events(self):
        events_path = self.root / 'events'
        if not events_path.exists():
            return []
        fields = events_path.read_bytes().decode().split('\0')
        self.assertEqual(fields.pop(), '')
        events = []
        while fields:
            kind, mode, count = fields[:3]
            count = int(count)
            events.append(dict(kind=kind, mode=None if mode == 'unset' else mode,
                               args=fields[3:3 + count]))
            del fields[:3 + count]
        return events

    def test_recipe_composition_modes_counts_and_route_hook_targets(self):
        self.recipe_fixture()
        hygiene_events = ['hygiene', 'target-dir', 'paths', 'after']
        code_fast_events = ['lint', 'fast', 'identity']
        code_full_events = ['lint', 'full', 'identity']
        cases = [
            ('check-hygiene', hygiene_events, [('check-hygiene', None)]),
            ('check-code', code_fast_events, [('check-code', 2)]),
            ('check-code-full', code_full_events, [('check-code-full', 3)]),
            ('check', hygiene_events + code_fast_events, [('check-hygiene', None), ('check-code', 2)]),
            ('check-full', hygiene_events + code_full_events, [('check-hygiene', None), ('check-code-full', 3)]),
            ('gate', hygiene_events + code_full_events + ['rust'],
             [('check-hygiene', None), ('check-code-full', 3), ('test', 5)]),
            ('hook-pre-commit', ['stage'] + hygiene_events + code_fast_events,
             [('hook-pre-commit-fast', None), ('check-code', 2)]),
            ('hook-pre-commit-full', ['stage'] + hygiene_events + code_full_events,
             [('hook-pre-commit', None), ('check-code-full', 3)]),
            ('hook-pre-commit-docs', ['stage'] + hygiene_events, [('hook-pre-commit-docs', None)]),
            ('hook-pre-push-fast', hygiene_events + code_fast_events + ['push'],
             [('hook-pre-push-fast', None), ('check-code', 2), ('test-push-fast', 5)]),
            ('hook-pre-push', hygiene_events + code_full_events + ['rust'],
             [('hook-pre-push', None), ('check-code-full', 3), ('test', 5)]),
            ('test-fast', ['affected'], [('test-fast', 4)]),
            ('test-push-fast', ['push'], [('test-push-fast', 5)]),
            ('test', ['rust'], [('test', 5)]),
            ('test-full', ['rust'], [('test', 5)]),
            ('ci-tooling-test', ['full'], [('ci-tooling-test', 3)]),
            ('ci-test', ['rust'], [('ci-test', 5)]),
            ('ci-test-release', ['rust'], [('ci-test-release', 5)]),
        ]
        for recipe, expected_events, expected_rows in cases:
            with self.subTest(recipe=recipe):
                self.reset_gate_logs()
                run = self.run_recipe(recipe)
                self.assertEqual(run.returncode, 0, run.stderr)
                events = self.recipe_events()
                self.assertEqual([event['kind'] for event in events], expected_events)
                for event in events:
                    if event['kind'] in ('fast', 'full', 'paths'):
                        self.assertEqual(event['mode'], '1' if event['kind'] == 'fast' else '0')
                    if event['kind'] == 'fast':
                        self.assertIn('--fast', event['args'])
                records = self.timings()
                self.assertEqual([(r['target'], r['tests']) for r in records], expected_rows)
                expected_reason = 'forced' if recipe == 'test-full' else 'default'
                for record in records:
                    self.assertEqual((record['execution'], record['execution_reason']), ('ran', expected_reason))
                if recipe == 'ci-tooling-test':
                    self.assertIn('--ci', events[0]['args'])
                if recipe == 'ci-test-release':
                    self.assertIn('--release', events[0]['args'])

    def test_mandatory_hygiene_or_broad_stage_failure_stops_before_following_stages(self):
        self.recipe_fixture()
        for recipe, expected_target, expected_events in (
            ('check', 'check-hygiene', ['hygiene']),
            ('gate', 'check-hygiene', ['hygiene']),
            ('hook-pre-commit-full', 'hook-pre-commit', ['stage', 'hygiene']),
            ('hook-pre-push', 'hook-pre-push', ['hygiene']),
        ):
            with self.subTest(stage='hygiene', recipe=recipe):
                self.reset_gate_logs()
                run = self.run_recipe(recipe, env={**self.env, 'FAIL_EVENT': 'hygiene'})
                self.assertEqual(run.returncode, 7, run.stderr)
                self.assertEqual([e['kind'] for e in self.recipe_events()], expected_events)
                records = self.timings()
                self.assertEqual([(r['target'], r['exit']) for r in records], [(expected_target, 7)])

        for recipe, expected_events, expected_rows in (
            ('hook-pre-commit-full', ['stage', 'hygiene', 'target-dir', 'paths', 'after', 'lint', 'full'],
             [('hook-pre-commit', 0, None), ('check-code-full', 7, 3)]),
            ('hook-pre-push', ['hygiene', 'target-dir', 'paths', 'after', 'lint', 'full'],
             [('hook-pre-push', 0, None), ('check-code-full', 7, 3)]),
        ):
            with self.subTest(stage='broad', recipe=recipe):
                self.reset_gate_logs()
                run = self.run_recipe(recipe, env={**self.env, 'FAIL_EVENT': 'full'})
                self.assertEqual(run.returncode, 7, run.stderr)
                self.assertEqual([e['kind'] for e in self.recipe_events()], expected_events)
                records = self.timings()
                self.assertEqual([(r['target'], r['exit'], r['tests']) for r in records], expected_rows)

    def test_policy_disabled_skips_broad_targets_preserves_mandatory_hygiene_and_supports_rollback(self):
        self.recipe_fixture()
        self.write_policy(automatic_broad=False)

        for target in BROAD_TARGETS:
            with self.subTest(broad=target):
                self.reset_gate_logs()
                run = self.run_recipe(target)
                self.assertEqual(run.returncode, 0, run.stderr)
                self.assertEqual(self.recipe_events(), [])
                records = self.timings()
                self.assertEqual(len(records), 1)
                self.assertEqual(
                    (records[0]['target'], records[0]['execution'],
                     records[0]['execution_reason'], records[0]['exit'], records[0]['tests']),
                    (target, 'skipped', 'policy-disabled', None, None),
                )

        hygiene_events = ['hygiene', 'target-dir', 'paths', 'after']
        composite_cases = [
            ('hook-pre-commit', ['stage'] + hygiene_events, 'hook-pre-commit-fast', ['check-code']),
            ('hook-pre-commit-full', ['stage'] + hygiene_events, 'hook-pre-commit', ['check-code-full']),
            ('hook-pre-commit-docs', ['stage'] + hygiene_events, 'hook-pre-commit-docs', []),
            ('check', hygiene_events, 'check-hygiene', ['check-code']),
            ('check-full', hygiene_events, 'check-hygiene', ['check-code-full']),
            ('gate', hygiene_events, 'check-hygiene', ['check-code-full', 'test']),
            ('hook-pre-push-fast', hygiene_events, 'hook-pre-push-fast', ['check-code', 'test-push-fast']),
            ('hook-pre-push', hygiene_events, 'hook-pre-push', ['check-code-full', 'test']),
        ]
        for recipe, expected_events, mandatory_target, skipped_children in composite_cases:
            with self.subTest(composite=recipe):
                self.reset_gate_logs()
                run = self.run_recipe(recipe)
                self.assertEqual(run.returncode, 0, run.stderr)
                self.assertEqual([e['kind'] for e in self.recipe_events()], expected_events)
                records = self.timings()
                self.assertEqual(
                    [(r['target'], r['execution'], r['execution_reason']) for r in records],
                    [(mandatory_target, 'ran', 'default')]
                    + [(child, 'skipped', 'policy-disabled') for child in skipped_children],
                )
                self.reset_gate_logs()
                fail_run = self.run_recipe(recipe, env={**self.env, 'FAIL_EVENT': 'hygiene'})
                self.assertEqual(fail_run.returncode, 7, fail_run.stderr)
                fail_records = self.timings()
                self.assertEqual([(r['target'], r['exit']) for r in fail_records], [(mandatory_target, 7)])

        self.reset_gate_logs()
        one_script = self.root / 'one.py'
        one_script.write_text("import sys\nprint('Ran 1 test in 0.001s', file=sys.stderr)\n")
        one_run = self.run_recipe('--set', 'one_cmd', f'{self.real_python} {one_script}', 'test-one', 'focused')
        self.assertEqual(one_run.returncode, 0, one_run.stderr)
        one_records = self.timings()
        self.assertEqual(
            [(r['target'], r['execution'], r['execution_reason'], r['tests']) for r in one_records],
            [('test-one', 'ran', 'default', 1)],
        )

        self.reset_gate_logs()
        full_run = self.run_recipe('test-full')
        self.assertEqual(full_run.returncode, 0, full_run.stderr)
        self.assertEqual([e['kind'] for e in self.recipe_events()], ['rust'])
        self.assertEqual(
            [(r['target'], r['execution'], r['execution_reason'], r['tests']) for r in self.timings()],
            [('test', 'ran', 'forced', 5)],
        )
        self.reset_gate_logs()
        full_fail = self.run_recipe('test-full', env={**self.env, 'FAIL_EVENT': 'rust'})
        self.assertEqual(full_fail.returncode, 7, full_fail.stderr)
        self.assertEqual([(r['target'], r['exit']) for r in self.timings()], [('test', 7)])

        # Rollback toggle: automatic_broad = true resumes broad execution with policy-enabled.
        self.write_policy(automatic_broad=True)
        self.reset_gate_logs()
        rollback_test = self.run_recipe('test')
        self.assertEqual(rollback_test.returncode, 0, rollback_test.stderr)
        self.assertEqual([e['kind'] for e in self.recipe_events()], ['rust'])
        self.assertEqual(
            [(r['target'], r['execution'], r['execution_reason']) for r in self.timings()],
            [('test', 'ran', 'policy-enabled')],
        )
        self.reset_gate_logs()
        rollback_check = self.run_recipe('check')
        self.assertEqual(rollback_check.returncode, 0, rollback_check.stderr)
        self.assertEqual(
            [(r['target'], r['execution'], r['execution_reason']) for r in self.timings()],
            [('check-hygiene', 'ran', 'default'), ('check-code', 'ran', 'policy-enabled')],
        )

    def test_hosted_env_refuses_broad_skips_without_tt_force_but_allows_docs_and_hygiene_only(self):
        self.recipe_fixture()
        self.write_policy(automatic_broad=False)
        hosted_env = {**self.env, 'GITHUB_ACTIONS': 'true'}

        for recipe in [
            *BROAD_TARGETS,
            'check',
            'check-full',
            'gate',
            'hook-pre-commit',
            'hook-pre-commit-full',
            'hook-pre-push-fast',
            'hook-pre-push',
        ]:
            with self.subTest(refused=recipe):
                self.reset_gate_logs()
                run = self.run_recipe(recipe, env=hosted_env)
                self.assertEqual(run.returncode, 2, run.stderr)
                self.assertIn('hosted verification would be skipped', run.stderr)

        one_script = self.root / 'one.py'
        one_script.write_text("import sys\nprint('Ran 1 test in 0.001s', file=sys.stderr)\n")
        for args in (
            ('hook-pre-commit-docs',),
            ('check-hygiene',),
            ('--set', 'one_cmd', f'{self.real_python} {one_script}', 'test-one', 'focused'),
            ('test-full',),
        ):
            with self.subTest(allowed=args[-1]):
                self.reset_gate_logs()
                run = self.run_recipe(*args, env=hosted_env)
                self.assertEqual(run.returncode, 0, run.stderr)

    def test_hooks_clear_pathspec_env_vars_and_pre_commit_preserves_git_index_file(self):
        self.stage('tools/optic_settling.py')
        alt_index = self.root / 'alt.index'
        readme = self.root / 'README.md'
        readme.write_text('docs only\n', encoding='utf-8')
        alt_env = {**self.env, 'GIT_INDEX_FILE': str(alt_index)}
        self.run_git('read-tree', '--empty', env=alt_env)
        self.run_git('add', '--', 'README.md', env=alt_env)

        pathspec_vars = {
            'GIT_LITERAL_PATHSPECS': '1',
            'GIT_GLOB_PATHSPECS': '1',
            'GIT_NOGLOB_PATHSPECS': '1',
            'GIT_ICASE_PATHSPECS': '1',
        }
        commit_run = self.hook('pre-commit', env={**alt_env, **pathspec_vars})
        self.assertEqual(commit_run.returncode, 0, commit_run.stderr)
        self.assertEqual((self.root / 'recipe').read_text(), 'hook-pre-commit-docs')
        commit_just_env = json.loads((self.root / 'just-env.json').read_text())
        self.assertEqual(commit_just_env['GIT_INDEX_FILE'], str(alt_index))
        for key in pathspec_vars:
            self.assertIsNone(commit_just_env[key], key)

        branch = f"refs/heads/topic {'1' * 40} refs/heads/topic {'2' * 40}\n"
        push_run = self.hook(
            'pre-push', 'origin', 'https://example.invalid/repo', refs=branch,
            env={**self.env, 'GIT_DIR': str(self.root / '.git'), **pathspec_vars},
        )
        self.assertEqual(push_run.returncode, 0, push_run.stderr)
        push_just_env = json.loads((self.root / 'just-env.json').read_text())
        self.assertIsNone(push_just_env['GIT_DIR'])
        for key in pathspec_vars:
            self.assertIsNone(push_just_env[key], key)

    def test_pre_push_enforces_ssh_keepalive_and_warns_on_uncovered_policy_skip(self):
        branch = f"refs/heads/topic {'1' * 40} refs/heads/topic {'2' * 40}\n"

        https_run = self.hook('pre-push', 'origin', 'https://example.invalid/repo', refs=branch)
        self.assertEqual(https_run.returncode, 0, https_run.stderr)
        self.assertFalse((self.root / 'ssh-argv').exists())

        for url in ('git@example.invalid:org/repo', 'ssh://git@example.invalid:2222/org/repo'):
            with self.subTest(url=url):
                for name in ('ssh-argv', 'lfs-refs', 'recipe'):
                    (self.root / name).unlink(missing_ok=True)
                refused = self.hook('pre-push', 'origin', url, refs=branch,
                                    env={**self.env, 'FAKE_SSH_ALIVE': '0'})
                self.assertEqual(refused.returncode, 1)
                self.assertIn('ServerAliveInterval 60', refused.stderr)
                self.assertFalse((self.root / 'lfs-refs').exists())
                self.assertFalse((self.root / 'recipe').exists())

                for name in ('ssh-argv', 'lfs-refs', 'recipe'):
                    (self.root / name).unlink(missing_ok=True)
                allowed = self.hook('pre-push', 'origin', url, refs=branch,
                                    env={**self.env, 'FAKE_SSH_ALIVE': '60'})
                self.assertEqual(allowed.returncode, 0, allowed.stderr)
                self.assertEqual((self.root / 'recipe').read_text(), 'hook-pre-push-fast')

        self.write_policy(automatic_broad=False)
        (self.root / 'recipe').unlink(missing_ok=True)
        uncovered = self.hook('pre-push', 'backup', 'https://example.invalid/repo', refs=branch)
        self.assertEqual(uncovered.returncode, 0, uncovered.stderr)
        self.assertEqual((self.root / 'recipe').read_text(), 'hook-pre-push')
        self.assertIn(
            'pre-push: local test skipped by policy; remote full-suite coverage is not declared '
            'for this push. Use TT_FORCE=1 for deliberate local verification.',
            uncovered.stderr,
        )

        (self.root / 'recipe').unlink(missing_ok=True)
        covered = self.hook('pre-push', 'origin', 'https://example.invalid/repo', refs=branch)
        self.assertEqual(covered.returncode, 0, covered.stderr)
        self.assertEqual((self.root / 'recipe').read_text(), 'hook-pre-push-fast')
        self.assertEqual(covered.stderr, '')

        (self.root / '.tt.toml').write_text('automatic_broad = "broken"\n', encoding='utf-8')
        (self.root / 'recipe').unlink(missing_ok=True)
        malformed = self.hook('pre-push', 'backup', 'https://example.invalid/repo', refs=branch)
        self.assertEqual(malformed.returncode, 2)
        self.assertIn('cannot inspect test execution policy', malformed.stderr)
        self.assertFalse((self.root / 'recipe').exists())


class CiWorkflowTests(unittest.TestCase):
    VERIFICATION_JOBS = (
        'tooling-test',
        'test',
        'build',
        'build-musl',
        'randomized-tests',
        'visual-tests',
        'msrv',
        'clippy',
        'rustfmt',
        'fedora',
        'freebsd',
        'nix',
    )
    PUBLISH_JOBS = ('publish-wiki', 'publish-docs')
    EXPECTED_TIMEOUTS = {
        'rustfmt': '10',
        'tooling-test': '15',
        'build': '20',
        'build-musl': '20',
        'visual-tests': '20',
        'msrv': '20',
        'clippy': '20',
        'fedora': '20',
        'publish-docs': '20',
        'publish-wiki': '20',
        'test': '25',
        'freebsd': '25',
        'randomized-tests': '30',
        'nix': '30',
    }

    @staticmethod
    def _strip_comment(raw: str) -> str:
        in_single = False
        in_double = False
        for index, char in enumerate(raw):
            if char == "'" and not in_double:
                in_single = not in_single
            elif char == '"' and not in_single:
                in_double = not in_double
            elif char == '#' and not in_single and not in_double and (index == 0 or raw[index - 1].isspace()):
                return raw[:index].rstrip()
        return raw.rstrip()

    @classmethod
    def _parse_ci_workflow(cls):
        text = (ROOT / '.github/workflows/ci.yml').read_text(encoding='utf-8')
        top_sections = {}
        current_top = None
        for raw_line in text.splitlines():
            stripped = raw_line.strip()
            if not stripped or stripped.startswith('#'):
                continue
            indent = len(raw_line) - len(raw_line.lstrip(' '))
            if indent == 0:
                key, _, rest = cls._strip_comment(raw_line).partition(':')
                current_top = key.strip()
                top_sections[current_top] = []
                if rest.strip():
                    top_sections[current_top].append((0, rest.strip()))
            elif current_top is not None:
                top_sections[current_top].append((indent, cls._strip_comment(raw_line.lstrip(' '))))

        def mapping_at(entries, target_indent):
            result = {}
            for indent, content in entries:
                if indent == target_indent and ':' in content and not content.startswith('- '):
                    key, _, value = content.partition(':')
                    result[key.strip()] = value.strip()
            return result

        jobs = {}
        current_job = None
        for indent, content in top_sections.get('jobs', []):
            if indent == 2 and content.endswith(':'):
                current_job = content[:-1].strip()
                jobs[current_job] = []
            elif current_job is not None and indent >= 4:
                jobs[current_job].append((indent, content))

        parsed_jobs = {}
        for job_name, entries in jobs.items():
            scalars = mapping_at(entries, 4)
            permissions = {}
            in_permissions = False
            for indent, content in entries:
                if indent == 4:
                    in_permissions = content == 'permissions:'
                elif in_permissions and indent == 6 and ':' in content:
                    k, _, v = content.partition(':')
                    permissions[k.strip()] = v.strip()
            steps = []
            current_step = None
            in_steps = False
            for indent, content in entries:
                if indent == 4:
                    in_steps = content == 'steps:'
                    if not in_steps:
                        current_step = None
                    continue
                if not in_steps:
                    continue
                if indent == 6 and content.startswith('- '):
                    current_step = [content[2:]]
                    steps.append(current_step)
                elif current_step is not None and indent > 6:
                    current_step.append(content)
            parsed_jobs[job_name] = {
                'scalars': scalars,
                'permissions': permissions,
                'steps': ['\n'.join(step) for step in steps],
            }

        return text, {
            'permissions': mapping_at(top_sections.get('permissions', []), 2),
            'concurrency': mapping_at(top_sections.get('concurrency', []), 2),
            'env': mapping_at(top_sections.get('env', []), 2),
            'jobs': parsed_jobs,
        }

    def test_workflow_hardening_deduplication_timeouts_and_tooling_hygiene(self):
        text, workflow = self._parse_ci_workflow()
        self.assertNotIn('pull_request_target', text)
        self.assertEqual(workflow['permissions'], {'contents': 'read'})
        self.assertEqual(
            workflow['concurrency'],
            {
                'group': '${{ github.workflow }}-${{ github.event_name }}-${{ github.event.pull_request.number || github.ref }}',
                'cancel-in-progress': "${{ github.ref != 'refs/heads/main' }}",
            },
        )
        self.assertIn(workflow['env'].get('TT_FORCE'), ('"1"', "'1'"))

        jobs = workflow['jobs']
        self.assertEqual(set(jobs), set(self.VERIFICATION_JOBS) | set(self.PUBLISH_JOBS))

        expected_pr_guard = (
            "github.event_name != 'pull_request' || "
            "github.event.pull_request.head.repo.full_name != github.repository"
        )
        for name in self.VERIFICATION_JOBS:
            with self.subTest(verification_job=name):
                scalars = jobs[name]['scalars']
                self.assertEqual(scalars.get('if'), expected_pr_guard)
                self.assertEqual(scalars.get('runs-on'), 'ubuntu-24.04')
                self.assertEqual(scalars.get('timeout-minutes'), self.EXPECTED_TIMEOUTS[name])
                self.assertEqual(jobs[name]['permissions'], {})

        for name in self.PUBLISH_JOBS:
            with self.subTest(publish_job=name):
                scalars = jobs[name]['scalars']
                self.assertEqual(
                    scalars.get('if'),
                    "github.event_name == 'push' && github.ref == 'refs/heads/main'",
                )
                self.assertEqual(scalars.get('runs-on'), 'ubuntu-24.04')
                self.assertEqual(scalars.get('timeout-minutes'), self.EXPECTED_TIMEOUTS[name])
                self.assertEqual(jobs[name]['permissions'], {'contents': 'write'})

        tooling_steps = jobs['tooling-test']['steps']
        checkout_steps = [s for s in tooling_steps if 'uses: actions/checkout@v6' in s]
        self.assertEqual(len(checkout_steps), 1)
        self.assertIn('fetch-depth: 0', checkout_steps[0])

        hygiene_chain = (
            'python3 tools/ops-check && python3 tools/ops-docs check && '
            'python3 tools/identity-check && python3 tools/upstream-report --check && '
            'python3 tools/package-pin --check'
        )
        hygiene_indices = [i for i, s in enumerate(tooling_steps) if hygiene_chain in s]
        tooling_indices = [i for i, s in enumerate(tooling_steps) if 'just ci-tooling-test' in s]
        self.assertEqual(len(hygiene_indices), 1)
        self.assertEqual(len(tooling_indices), 1)
        self.assertLess(hygiene_indices[0], tooling_indices[0])
