"""Cheap contracts for tooling execution; synthetic suites never run lifecycle cases."""
import ast
import json
import os
import re
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import time
import unittest
from unittest import mock

from scripts.testing.hermetic_git import isolate
from scripts.testing.tooling_tests import fast_mode, flatten, module_jobs, worker_limit

ROOT = Path(__file__).resolve().parents[2]


class ModeTests(unittest.TestCase):
    def test_modes_fail_early(self):
        for value, expected in ((None, False), ('0', False), ('1', True)):
            with self.subTest(value=value), mock.patch.dict(os.environ, {}, clear=True):
                if value is not None:
                    os.environ['NIRI_TOOLING_FAST'] = value
                self.assertEqual(fast_mode(), expected)
        for value in ('', 'true', '2', ' 1', '１'):
            with self.subTest(value=value), mock.patch.dict(os.environ, NIRI_TOOLING_FAST=value):
                with self.assertRaisesRegex(ValueError, 'NIRI_TOOLING_FAST'):
                    fast_mode()

    def test_budget_caps_total_children_and_rejects_malformed_values(self):
        for value, expected in ((None, 10), ('1', 1), ('10', 10), ('12', 12), ('16', 16), ('999', 16)):
            with self.subTest(value=value), mock.patch.dict(os.environ, {}, clear=True):
                if value is not None:
                    os.environ['NEXTEST_TEST_THREADS'] = value
                self.assertEqual(worker_limit(), expected)
        for value in ('', '0', '-1', '1.5', ' 1', '1 ', '１', 'many'):
            with self.subTest(value=value), mock.patch.dict(os.environ, NEXTEST_TEST_THREADS=value):
                with self.assertRaisesRegex(ValueError, 'NEXTEST_TEST_THREADS'):
                    worker_limit()

    def test_raw_imports_validate_before_skipping(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            home = root / 'home'
            xdg = root / 'xdg'
            home.mkdir()
            xdg.mkdir()
            env = {
                'PATH': os.environ['PATH'],
                'HOME': str(home),
                'XDG_CONFIG_HOME': str(xdg),
                'PYTHONPATH': str(ROOT),
                'NIRI_TOOLING_FAST': 'typo',
            }
            isolate(env)
            for module in ('tests.capture.test_optic_settling', 'tests.capture.test_vt_lib'):
                run = subprocess.run([sys.executable, '-c', f'import {module}'], cwd=ROOT,
                                     env=env, capture_output=True, text=True, timeout=15)
                self.assertNotEqual(run.returncode, 0)
                self.assertIn('NIRI_TOOLING_FAST', run.stderr)

    def test_module_jobs_groups_whole_modules_largest_first(self):
        self.assertEqual(module_jobs([]), [])
        small_cls = type('Small', (unittest.TestCase,), {
            '__module__': 'test_small',
            'test_only': lambda self: None,
        })
        big_cls = type('Big', (unittest.TestCase,), {
            '__module__': 'test_big',
            'test_first': lambda self: None,
            'test_second': lambda self: None,
            'test_third': lambda self: None,
        })
        mid_a = type('MidA', (unittest.TestCase,), {
            '__module__': 'test_mid',
            'test_one': lambda self: None,
        })
        mid_b = type('MidB', (unittest.TestCase,), {
            '__module__': 'test_mid',
            'test_two': lambda self: None,
        })
        cases = [
            small_cls('test_only'),
            mid_a('test_one'),
            big_cls('test_first'),
            big_cls('test_second'),
            big_cls('test_third'),
            mid_b('test_two'),
        ]
        self.assertEqual(module_jobs(cases), [
            ['test_big.Big.test_first', 'test_big.Big.test_second', 'test_big.Big.test_third'],
            ['test_mid.MidA.test_one', 'test_mid.MidB.test_two'],
            ['test_small.Small.test_only'],
        ])

    def test_flatten_rejects_duplicate_native_ids(self):
        case = unittest.FunctionTestCase(lambda: None)
        with self.assertRaisesRegex(ValueError, 'duplicate'):
            flatten(unittest.TestSuite([case, unittest.TestSuite([case])]))


EVENT_HELPERS = '''def _emit_event(kind):
    fd = os.open('events.log', os.O_WRONLY | os.O_CREAT | os.O_APPEND, 0o600)
    try:
        os.write(fd, f'{kind}:{os.getpid()}\\n'.encode())
    finally:
        os.close(fd)
def _await_wave():
    if 'PROBE_WAVE' not in os.environ:
        return
    _emit_event('start')
    target = int(os.environ['PROBE_WAVE'])
    deadline = time.monotonic() + 5.0
    while time.monotonic() < deadline:
        with open('events.log') as log:
            if sum(1 for line in log if line.startswith('start:')) >= target:
                break
        time.sleep(0.005)
    time.sleep(0.03)
def _finish_wave():
    if 'PROBE_WAVE' in os.environ:
        _emit_event('end')
'''


class CoordinatorTests(unittest.TestCase):
    def test_nested_tests_keep_qualified_ids_in_workers(self):
        for relative in ('tests', 'tests/capture'):
            package = self.root / relative
            package.mkdir(exist_ok=True)
            (package / '__init__.py').write_text('')
        (self.root / 'tests/capture/test_probe.py').write_text(
            "import unittest\nclass Probe(unittest.TestCase):\n"
            "    def test_one(self):\n"
            "        with open('seen', 'a') as out: out.write(self.id() + '\\n')\n")
        run = self.run_suite()
        self.assertEqual(run.returncode, 0, run.stdout + run.stderr)
        self.assertIn('Ran 1 tests', run.stdout)
        self.assertEqual((self.root / 'seen').read_text().splitlines(),
                         ['tests.capture.test_probe.Probe.test_one'])

    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        (self.root / 'tests').mkdir()
        (self.root / 'tests/__init__.py').write_text('')
        home = self.root / 'home'
        xdg = self.root / 'xdg'
        home.mkdir()
        xdg.mkdir()
        self.env = {
            'PATH': os.environ['PATH'],
            'HOME': str(home),
            'XDG_CONFIG_HOME': str(xdg),
            'PYTHONPATH': str(ROOT),
            'NIRI_TOOLING_FAST': '1',
            'NEXTEST_TEST_THREADS': '10',
        }
        isolate(self.env)
        self.command = [sys.executable, '-m', 'scripts.testing.tooling_tests', '--full', '--ci']

    def fixture(self, body, module='test_probe.py'):
        directory = self.root / 'tests'
        if module == 'test_glass_optic_smoke.py':
            directory /= 'capture'
            directory.mkdir(exist_ok=True)
            (directory / '__init__.py').write_text('')
        (directory / module).write_text('import unittest, os, subprocess, time\n' + body)

    def run_suite(self, **env):
        run_env = self.env.copy()
        run_env.update(env)
        return subprocess.run(self.command, cwd=self.root, env=run_env,
                              capture_output=True, text=True, timeout=30)

    def assert_worker_events(self, expected_workers, max_concurrency):
        raw = (self.root / 'events.log').read_text().splitlines()
        events = []
        active = set()
        started = []
        ended = []
        peak = 0
        for line in raw:
            kind, pid = line.split(':')
            self.assertIn(kind, ('start', 'end'))
            events.append((kind, pid))
            if kind == 'start':
                self.assertNotIn(pid, active)
                self.assertNotIn(pid, started)
                active.add(pid)
                started.append(pid)
                peak = max(peak, len(active))
            else:
                self.assertIn(pid, active)
                active.remove(pid)
                ended.append(pid)
        self.assertEqual(active, set())
        self.assertEqual(len(started), expected_workers)
        self.assertEqual(set(started), set(ended))
        self.assertEqual(peak, max_concurrency)
        return events, started

    def test_full_overrides_fast_and_budget_one_preserves_all_ids(self):
        self.fixture('''class Remainder(unittest.TestCase):
    def test_first(self): self.record()
    def test_second(self): self.record()
    def record(self):
        with open('seen', 'a') as out: out.write(self.id() + ':' + os.environ['NIRI_TOOLING_FAST'] + '\\n')
class DriverCleanupTests(Remainder):
    test_second = None
''')
        run = self.run_suite(NEXTEST_TEST_THREADS='1')
        self.assertEqual(run.returncode, 0, run.stdout + run.stderr)
        self.assertIn('Ran 3 tests', run.stdout)
        lines = (self.root / 'seen').read_text().splitlines()
        self.assertEqual(set(lines), {
            'tests.test_probe.Remainder.test_first:0', 'tests.test_probe.Remainder.test_second:0',
            'tests.test_probe.DriverCleanupTests.test_first:0'})
        self.assertEqual(len(lines), 3)

    def test_full_orders_lifecycle_jobs_before_module_jobs_and_caps_children(self):
        hooks = EVENT_HELPERS + 'def setUpModule(): _await_wave()\ndef tearDownModule(): _finish_wave()\n'
        cases = '''    def record(self):
        with open(self.id() + '.pid', 'a') as out:
            out.write(os.environ['NIRI_TOOLING_FAST'] + ':' + str(os.getpid()) + '\\n')
    def test_one(self): self.record()
    def test_two(self): self.record()
'''
        modules = ('alpha', 'beta', 'gamma', 'delta')
        for name in modules:
            self.fixture(hooks + 'class Cases(unittest.TestCase):\n' + cases, f'test_{name}.py')
        self.fixture(
            hooks + 'class Cases(unittest.TestCase):\n' + cases +
            'class DriverCleanupTests(unittest.TestCase):\n'
            '    def test_one(self):\n'
            '        with open(self.id() + ".pid", "a") as out:\n'
            '            out.write(os.environ["NIRI_TOOLING_FAST"] + ":" + str(os.getpid()) + "\\n")\n',
            'test_mixed.py',
        )
        self.fixture(hooks + 'class DriverCleanupTests(unittest.TestCase):\n' + cases, 'test_life.py')
        all_modules = (*modules, 'mixed')
        lifecycle = [
            'tests.test_life.DriverCleanupTests.test_one',
            'tests.test_life.DriverCleanupTests.test_two',
            'tests.test_mixed.DriverCleanupTests.test_one',
        ]
        expected = {f'tests.test_{name}.Cases.test_{case}' for name in all_modules for case in ('one', 'two')}
        expected.update(lifecycle)
        for budget, children in (('10', 8), ('3', 3), ('1', 1)):
            with self.subTest(budget=budget):
                (self.root / 'events.log').unlink(missing_ok=True)
                for path in self.root.glob('*.pid'):
                    path.unlink()
                run = self.run_suite(NEXTEST_TEST_THREADS=budget, PROBE_WAVE=str(children))
                self.assertEqual(run.returncode, 0, run.stdout + run.stderr)
                self.assertIn(f'Full tooling: 13 cases, {children} children', run.stdout)
                self.assertIn('Ran 13 tests', run.stdout)
                runs = {path.name.removesuffix('.pid'): path.read_text().splitlines()
                        for path in self.root.glob('*.pid')}
                self.assertEqual(set(runs), expected)
                self.assertTrue(all(len(lines) == 1 for lines in runs.values()), runs)
                pid = {case_id: lines[0].split(':')[1] for case_id, lines in runs.items()}
                self.assertEqual({lines[0].split(':')[0] for lines in runs.values()}, {'0'})
                for name in all_modules:
                    self.assertEqual(pid[f'tests.test_{name}.Cases.test_one'], pid[f'tests.test_{name}.Cases.test_two'])
                remainder = {pid[case_id] for case_id in expected if '.Cases.' in case_id}
                life_pids = {pid[case_id] for case_id in lifecycle}
                self.assertEqual(len(remainder), 5)
                self.assertEqual(len(life_pids), 3)
                self.assertFalse(remainder & life_pids)
                events, started_pids = self.assert_worker_events(8, children)
                by_pid = {}
                for case_id, worker_pid in pid.items():
                    by_pid.setdefault(worker_pid, set()).add(case_id)
                if budget == '1':
                    self.assertEqual(
                        events,
                        [pair for worker_pid in started_pids for pair in (('start', worker_pid), ('end', worker_pid))],
                    )
                    self.assertEqual(
                        {cid for worker_pid in started_pids[:3] for cid in by_pid[worker_pid]},
                        set(lifecycle),
                    )

    def test_fast_preserves_native_inventory_and_module_fixtures(self):
        body = EVENT_HELPERS + '''def setUpModule():
    with open(__name__ + '.module', 'a') as out: out.write(str(os.getpid()) + '\\n')
    _await_wave()
def tearDownModule():
    _finish_wave()
class Cases(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        with open(__name__ + '.class', 'a') as out: out.write(str(os.getpid()) + '\\n')
    def record(self):
        with open(__name__ + '.seen', 'a') as out:
            out.write(self.id() + ':' + os.environ['NIRI_TOOLING_FAST'] + ':' + str(os.getpid()) + '\\n')
    def test_one(self): self.record()
    def test_two(self): self.record()
'''
        self.fixture(body + '''    @unittest.expectedFailure
    def test_known(self): self.record(); self.fail('known defect')
''', 'test_alpha.py')
        self.fixture(body + '''@unittest.skipIf(os.environ['NIRI_TOOLING_FAST']=='1', 'fast lifecycle omission')
class DriverCleanupTests(unittest.TestCase):
    def test_slow(self): self.fail('must be omitted in fast mode')
''', 'test_beta.py')
        self.fixture(body, 'test_gamma.py')
        self.fixture(body, 'test_delta.py')
        self.fixture(body, 'test_epsilon.py')
        native_env = self.env.copy()
        native_env['NIRI_TOOLING_FAST'] = '1'
        native = subprocess.run([sys.executable, '-m', 'unittest', 'discover', '-s', 'tests', '-t', '.'],
                                cwd=self.root, env=native_env,
                                capture_output=True, text=True, timeout=30)
        self.assertEqual(native.returncode, 0, native.stdout + native.stderr)
        self.assertIn('Ran 12 tests', native.stderr)
        self.assertIn('skipped=1, expected failures=1', native.stderr)
        native_seen = {path.name: [line.rsplit(':', 1)[0] for line in path.read_text().splitlines()]
                       for path in self.root.glob('*.seen')}
        expected = {
            'tests.test_alpha.Cases.test_known', 'tests.test_alpha.Cases.test_one', 'tests.test_alpha.Cases.test_two',
            'tests.test_beta.Cases.test_one', 'tests.test_beta.Cases.test_two',
            'tests.test_beta.DriverCleanupTests.test_slow',
            'tests.test_gamma.Cases.test_one', 'tests.test_gamma.Cases.test_two',
            'tests.test_delta.Cases.test_one', 'tests.test_delta.Cases.test_two',
            'tests.test_epsilon.Cases.test_one', 'tests.test_epsilon.Cases.test_two',
        }
        self.command = [sys.executable, '-m', 'scripts.testing.tooling_tests', '--fast']
        for budget, children in (('10', 5), ('2', 2), ('1', 1)):
            with self.subTest(budget=budget):
                (self.root / 'events.log').unlink(missing_ok=True)
                for suffix in ('seen', 'class', 'module'):
                    for path in self.root.glob('*.' + suffix):
                        path.unlink()
                run = self.run_suite(NIRI_TOOLING_FAST='0', NEXTEST_TEST_THREADS=budget,
                                     PROBE_WAVE=str(children))
                self.assertEqual(run.returncode, 0, run.stdout + run.stderr)
                self.assertIn(f'Fast tooling: 12 cases, {children} children', run.stdout)
                self.assertIn('OK (skipped=1, expected failures=1)', run.stdout)
                ids = [line[5:].rsplit(': ', 1)[0] for line in run.stdout.splitlines()
                       if line.startswith('case ')]
                self.assertEqual(set(ids), expected)
                self.assertEqual(len(ids), 12)
                pids = set()
                for path in self.root.glob('*.seen'):
                    lines = path.read_text().splitlines()
                    self.assertEqual([line.rsplit(':', 1)[0] for line in lines], native_seen[path.name])
                    worker_pids = {line.rsplit(':', 1)[1] for line in lines}
                    self.assertEqual(len(worker_pids), 1)
                    pids.update(worker_pids)
                    for suffix in ('class', 'module'):
                        self.assertEqual(path.with_suffix('.' + suffix).read_text().splitlines(),
                                         list(worker_pids))
                self.assertEqual(len(pids), 5)
                self.assert_worker_events(5, children)

    def test_empty_discovery_succeeds_without_spawning_workers(self):
        for args, label in ((['--full', '--ci'], 'Full'), (['--fast'], 'Fast')):
            with self.subTest(args=args):
                self.command = [sys.executable, '-m', 'scripts.testing.tooling_tests', *args]
                run = self.run_suite()
                self.assertEqual(run.returncode, 0, run.stdout + run.stderr)
                self.assertIn(f'{label} tooling: 0 cases, 0 children', run.stdout)
                self.assertIn('Ran 0 tests', run.stdout)

    def test_fast_rejects_ci_and_public_worker_mode_before_discovery(self):
        self.fixture("open('imported', 'w').close()\nclass C(unittest.TestCase):\n    def test_ok(self): pass\n")
        for args, message in ((['--fast', '--ci'], '--ci requires --full'),
                              (['--full', '--worker-mode', '1'], '--worker-mode requires --worker')):
            with self.subTest(args=args):
                self.command = [sys.executable, '-m', 'scripts.testing.tooling_tests', *args]
                run = self.run_suite()
                self.assertNotEqual(run.returncode, 0)
                self.assertIn(message, run.stderr)
                self.assertFalse((self.root / 'imported').exists())

    def test_fast_malformed_budget_fails_before_discovery(self):
        self.fixture("open('imported', 'w').close()\nclass C(unittest.TestCase):\n    def test_ok(self): pass\n")
        self.command = [sys.executable, '-m', 'scripts.testing.tooling_tests', '--fast']
        run = self.run_suite(NEXTEST_TEST_THREADS='many')
        self.assertNotEqual(run.returncode, 0)
        self.assertIn('NEXTEST_TEST_THREADS', run.stderr)
        self.assertFalse((self.root / 'imported').exists())

    def test_malformed_budget_fails_before_discovery_or_spawn(self):
        self.fixture("open('imported', 'w').close()\nclass C(unittest.TestCase):\n    def test_ok(self): pass\n")
        run = self.run_suite(NEXTEST_TEST_THREADS='0')
        self.assertNotEqual(run.returncode, 0)
        self.assertIn('NEXTEST_TEST_THREADS', run.stderr)
        self.assertFalse((self.root / 'imported').exists())

    def test_ci_rejects_required_class_skip_before_execution(self):
        self.fixture("@unittest.skip('missing GI binding')\nclass Required(unittest.TestCase):\n    def test_binding(self): pass\n")
        run = self.run_suite()
        self.assertNotEqual(run.returncode, 0)
        self.assertIn('tests.test_probe.Required.test_binding', run.stdout + run.stderr)
        self.assertIn('missing GI binding', run.stdout + run.stderr)

    def test_dynamic_skip_failure_import_error_and_crash_fail(self):
        for body, diagnostic in (
            ("class C(unittest.TestCase):\n    def test_bad(self): self.skipTest('unexpected dependency')\n", 'unexpected dependency'),
            ("class C(unittest.TestCase):\n    def test_bad(self): self.fail('lifecycle assertion')\n", 'lifecycle assertion'),
            ("raise ImportError('missing required module')\n", 'missing required module'),
            ("class C(unittest.TestCase):\n    def test_bad(self): os._exit(7)\n", '7'),
        ):
            with self.subTest(diagnostic=diagnostic):
                self.fixture(body)
                run = self.run_suite()
                self.assertNotEqual(run.returncode, 0, run.stdout)
                self.assertIn(diagnostic, run.stdout + run.stderr)

    def test_optional_skip_is_permitted(self):
        self.fixture("class RenderOrderBehindMatrixTest(unittest.TestCase):\n"
                     "    def test_positive_face_gate_requires_more_than_one_code(self): self.skipTest('no magick')\n",
                     'test_glass_optic_smoke.py')
        run = self.run_suite()
        self.assertEqual(run.returncode, 0, run.stdout + run.stderr)
        self.assertIn('skipped=1', run.stdout)

    def test_expected_failure_preserves_native_reporting(self):
        self.fixture('''class C(unittest.TestCase):
    @unittest.expectedFailure
    def test_expected(self): self.fail('known defect')
''')
        run = self.run_suite()
        self.assertEqual(run.returncode, 0, run.stdout + run.stderr)
        self.assertIn('EXPECTED FAILURE tests.test_probe.C.test_expected', run.stdout)
        self.assertIn('expected failures=1', run.stdout)

    def test_unexpected_success_preserves_native_failure(self):
        self.fixture('''class C(unittest.TestCase):
    @unittest.expectedFailure
    def test_unexpected(self): pass
''')
        for mode in ('--full', '--fast'):
            with self.subTest(mode=mode):
                self.command = [sys.executable, '-m', 'scripts.testing.tooling_tests', mode]
                run = self.run_suite()
                self.assertNotEqual(run.returncode, 0, run.stdout)
                self.assertIn('UNEXPECTED SUCCESS tests.test_probe.C.test_unexpected', run.stdout)
                self.assertIn('unexpected successes=1', run.stdout)

    def test_interrupt_reaps_a_child_from_an_unwound_case(self):
        self.fixture('''class DriverCleanupTests(unittest.TestCase):
    def test_wait(self):
        child = subprocess.Popen(['sleep', '60'], start_new_session=True)
        self.addCleanup(child.wait)
        self.addCleanup(child.kill)
        with open('child.pid', 'w') as out: out.write(str(child.pid))
        time.sleep(60)
''')
        self.check_interrupt('child.pid', 0.02)

    def test_interrupt_during_normal_cleanup_finishes_reaping(self):
        self.fixture('''class DriverCleanupTests(unittest.TestCase):
    def test_cleanup(self):
        child = subprocess.Popen(['sleep', '60'], start_new_session=True,
                                 stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        def cleanup():
            with open('cleanup.started', 'w') as out: out.write(str(child.pid))
            time.sleep(0.5)
            child.kill()
            child.wait()
        self.addCleanup(cleanup)
''')
        self.check_interrupt('cleanup.started', 0.01)

    def check_interrupt(self, marker_name, poll_s):
        for mode in ('--full', '--fast'):
            with self.subTest(mode=mode):
                (self.root / marker_name).unlink(missing_ok=True)
                command = [sys.executable, '-m', 'scripts.testing.tooling_tests', mode]
                parent = subprocess.Popen(command, cwd=self.root, env=self.env,
                                          stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
                self.addCleanup(self.reap, parent)
                marker = self.root / marker_name
                deadline = time.monotonic() + 15
                while not marker.exists():
                    self.assertIsNone(parent.poll())
                    self.assertLess(time.monotonic(), deadline)
                    time.sleep(poll_s)
                pid = int(marker.read_text())
                # Keep the deliberately failing red run from leaking its fixture child.
                def cleanup_probe(pid=pid):
                    try:
                        os.kill(pid, signal.SIGKILL)
                    except ProcessLookupError:
                        pass
                self.addCleanup(cleanup_probe)
                parent.send_signal(signal.SIGTERM)
                stdout, stderr = parent.communicate(timeout=20)
                self.assertNotEqual(parent.returncode, 0, stdout + stderr)
                with self.assertRaises(ProcessLookupError):
                    os.kill(pid, 0)

    def reap(self, process):
        if process.poll() is None:
            process.kill()
        process.wait()
        process.stdout.close()
        process.stderr.close()

    def test_workers_run_with_hermetic_git_in_both_modes(self):
        # The synthetic module never calls isolate: the coordinator must.
        self.fixture("import json\nclass C(unittest.TestCase):\n"
                     "    def test_env(self):\n"
                     "        with open(os.environ['PROBE_OUT'], 'w') as out:\n"
                     "            json.dump({k: v for k, v in os.environ.items() if k.startswith('GIT_')}, out)\n")
        poisoned = dict(GIT_DIR=str(self.root / 'decoy.git'), GIT_INDEX_FILE=str(self.root / 'index'),
                        GIT_CONFIG_GLOBAL=str(self.root / 'host-gitconfig'))
        for command in (['--full', '--ci'], ['--fast']):
            with self.subTest(command=command):
                out = self.root / f'probe-{command[0][2:]}.json'
                self.command = [sys.executable, '-m', 'scripts.testing.tooling_tests', *command]
                run = self.run_suite(PROBE_OUT=str(out), **poisoned)
                self.assertEqual(run.returncode, 0, run.stdout + run.stderr)
                self.assertEqual(json.loads(out.read_text()),
                                 {'GIT_CONFIG_GLOBAL': os.devnull, 'GIT_CONFIG_NOSYSTEM': '1'})


class StaticCoverageTests(unittest.TestCase):
    def test_literal_root_and_here_helpers_are_required(self):
        entry = self.root / 'scripts/capture/optic-settling-smoke.sh'
        helper = self.root / 'scripts/capture/extra-client.py'
        original = entry.read_text()
        for prefix in ('$ROOT/scripts/capture/', '${ROOT}/scripts/capture/',
                       '$HERE/', '${HERE}/'):
            for command in ('python3 "{path}"\n',
                            'CONSUMER=${{OVERRIDE:-{path}}}\n',
                            'args+=(--input "{path}")\n'):
                with self.subTest(prefix=prefix, command=command):
                    helper.write_text('')
                    entry.write_text(original + command.format(path=prefix + helper.name))
                    with self.assertRaisesRegex(ValueError, 'extra-client.py'):
                        self.check()
                    helper.unlink()
                    with self.assertRaisesRegex(ValueError, 'extra-client.py'):
                        self.check(self.patterns + ['scripts/capture/extra-client.py'])
        entry.write_text(original)

    def test_quoted_python_imports_are_declared_seeds(self):
        from scripts.testing import tooling_tests
        driver = (ROOT / 'scripts/capture/optic-settling-smoke.sh').read_text()
        imports = []
        for body in re.findall(r"<<'PY'\n(.*?)^PY$", driver, re.M | re.S):
            for node in ast.walk(ast.parse(body)):
                if isinstance(node, ast.ImportFrom) and node.module and node.module.startswith(('scripts.', 'tests.')):
                    imports.append(node.module.replace('.', '/') + '.py')
        self.assertTrue(imports, 'no local heredoc import was checked')
        seeds = getattr(tooling_tests, 'LIFECYCLE_PYTHON_SEEDS', ())
        for path in imports:
            self.assertIn(path, seeds)

    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.files = {
            'scripts/capture/optic-settling-smoke.sh': '. "$HERE/shared.sh"\npython3 "$ROOT/scripts/capture/optic_settling.py"\n',
            'scripts/capture/shared.sh': '. "$HERE/nested-lib.sh"\ncapture() { ${CAPTURE_META:-python3 "$ROOT/scripts/testing/stubbed-fallback"}; }\n',
            'scripts/capture/nested-lib.sh': 'value=$((1 + (2 * 3)))\n',
            'scripts/capture/vt-lib.sh': '',
            'scripts/capture/class-subject.sh': '',
            'tests/capture/test_optic_settling.py': "ROOT=Path(__file__).resolve().parents[2]\nclass DriverCleanupTests:\n    script=ROOT / 'scripts/capture/class-subject.sh'\n",
            'tests/capture/test_vt_lib.py': "LIB=Path(__file__).resolve().parents[2] / 'scripts/capture/vt-lib.sh'\nclass VtLibTests:\n    lib=LIB\n",
            'scripts/capture/optic_settling.py': 'import scripts.testing.other\nfrom scripts.testing import extra\n',
            'scripts/testing/other.py': 'from . import leaf\n',
            'scripts/testing/extra.py': '', 'scripts/testing/leaf.py': '',
            'scripts/capture/screencast_consumer.py': '', 'tests/fixtures/capture/fake_screencast.py': '',
        }
        for filename, source in self.files.items():
            path = self.root / filename
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(source)
        self.patterns = list(self.files)

    def check(self, patterns=None):
        from scripts.testing.tooling_tests import assert_static_coverage
        assert_static_coverage(self.root, self.patterns if patterns is None else patterns)

    def test_sources_class_paths_and_transitive_imports_must_be_routed(self):
        self.check()
        for path in ('scripts/capture/nested-lib.sh',
                     'scripts/capture/class-subject.sh',
                     'scripts/testing/other.py', 'scripts/testing/extra.py', 'scripts/testing/leaf.py'):
            with self.subTest(path=path), self.assertRaisesRegex(ValueError, path):
                self.check([pattern for pattern in self.patterns if pattern != path])

    def test_package_initializers_and_their_imports_must_be_routed(self):
        package = self.root / 'scripts/testing/pkg'
        package.mkdir()
        (package / 'leaf.py').write_text('')
        initializer = package / '__init__.py'
        initializer.write_text('import scripts.testing.hidden\n')
        (self.root / 'scripts/testing/hidden.py').write_text('')
        (self.root / 'scripts/testing/other.py').write_text('import scripts.testing.pkg.leaf\n')
        patterns = self.patterns + ['scripts/testing/pkg/leaf.py']
        with self.assertRaisesRegex(ValueError, 'scripts/testing/pkg/__init__.py'):
            self.check(patterns)
        patterns.append('scripts/testing/pkg/__init__.py')
        with self.assertRaisesRegex(ValueError, 'scripts/testing/hidden.py'):
            self.check(patterns)
        self.check(patterns + ['scripts/testing/hidden.py'])
        # Without the initializer, pkg is a namespace and imports no hidden code.
        initializer.unlink()
        self.check(self.patterns + ['scripts/testing/pkg/leaf.py'])
        seed_initializer = self.root / 'tests/capture/__init__.py'
        seed_initializer.write_text('')
        with self.assertRaisesRegex(ValueError, 'tests/capture/__init__.py'):
            self.check(self.patterns + ['scripts/testing/pkg/leaf.py'])

    def test_missing_source_cycle_dynamic_operand_and_missing_module_fail(self):
        cases = [
            ('scripts/capture/shared.sh', '. "$HERE/missing.sh"\n', 'missing.sh'),
            ('scripts/capture/nested-lib.sh', '. "$HERE/shared.sh"\n', 'cycle'),
            ('scripts/capture/shared.sh', '. "$HERE/$DYNAMIC"\n', 'unsupported'),
            ('scripts/testing/other.py', 'import scripts.testing.missing\n', 'scripts.testing.missing'),
            ('tests/capture/test_optic_settling.py', "ROOT=Path(__file__).resolve().parents[2]\nclass DriverCleanupTests:\n    script=ROOT / VARIABLE\n", 'dynamic'),
            ('tests/capture/test_optic_settling.py', "class DriverCleanupTests:\n    script=Path(__file__).parent.parent / 'scripts/testing/extra.py'\n", 'unsupported'),
            ('tests/capture/test_optic_settling.py', "ROOT=Path(__file__).resolve().parents[2]\nclass DriverCleanupTests:\n    script=ROOT.joinpath('scripts/testing/extra.py')\n", 'unsupported'),
        ]
        for filename, source, diagnostic in cases:
            with self.subTest(diagnostic=diagnostic):
                path = self.root / filename
                path.write_text(source)
                with self.assertRaisesRegex(ValueError, diagnostic):
                    self.check()
                path.write_text(self.files[filename])

    def test_nested_command_substitution_in_arithmetic_is_rejected_in_helpers(self):
        path = self.root / 'scripts/capture/nested-lib.sh'
        path.write_text('# ordinary arithmetic is fine\nvalue=$((1 + $(clock)))\n')
        with self.assertRaisesRegex(ValueError, 'nested-lib.sh:2.*command substitution.*arithmetic'):
            self.check()

    def test_sourced_helper_inherits_driver_here_and_repository_cwd(self):
        entry = self.root / 'scripts/capture/optic-settling-smoke.sh'
        entry.write_text('. "$ROOT/scripts/testing/helper.sh"\n')
        (self.root / 'scripts/testing/helper.sh').write_text(
            '. "$HERE/nested-lib.sh"\n. scripts/testing/root-helper.sh\n')
        (self.root / 'scripts/testing/root-helper.sh').write_text('')
        self.check(self.patterns + ['scripts/testing/helper.sh', 'scripts/testing/root-helper.sh'])
