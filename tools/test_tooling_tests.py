"""Cheap contracts for tooling execution; synthetic suites never run lifecycle cases."""
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import time
import unittest
from unittest import mock

from tools.tooling_tests import fast_mode, flatten, worker_limit

ROOT = Path(__file__).resolve().parents[1]


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
        for value, expected in ((None, 10), ('1', 1), ('10', 10), ('999', 10)):
            with self.subTest(value=value), mock.patch.dict(os.environ, {}, clear=True):
                if value is not None:
                    os.environ['NEXTEST_TEST_THREADS'] = value
                self.assertEqual(worker_limit(), expected)
        for value in ('', '0', '-1', '1.5', ' 1', '1 ', '１', 'many'):
            with self.subTest(value=value), mock.patch.dict(os.environ, NEXTEST_TEST_THREADS=value):
                with self.assertRaisesRegex(ValueError, 'NEXTEST_TEST_THREADS'):
                    worker_limit()

    def test_raw_imports_validate_before_skipping(self):
        for module in ('tools.test_optic_settling', 'tools.test_vt_lib'):
            run = subprocess.run([sys.executable, '-c', f'import {module}'], cwd=ROOT,
                                 env=dict(os.environ, NIRI_TOOLING_FAST='typo'),
                                 capture_output=True, text=True, timeout=15)
            self.assertNotEqual(run.returncode, 0)
            self.assertIn('NIRI_TOOLING_FAST', run.stderr)

    def test_flatten_rejects_duplicate_native_ids(self):
        case = unittest.FunctionTestCase(lambda: None)
        with self.assertRaisesRegex(ValueError, 'duplicate'):
            flatten(unittest.TestSuite([case, unittest.TestSuite([case])]))


class CoordinatorTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        (self.root / 'tools').mkdir()
        self.env = dict(os.environ, PYTHONPATH=str(ROOT), NIRI_TOOLING_FAST='1',
                        NEXTEST_TEST_THREADS='10')
        self.command = [sys.executable, '-m', 'tools.tooling_tests', '--full', '--ci']

    def fixture(self, body, module='test_probe.py'):
        (self.root / 'tools' / module).write_text('import unittest, os, subprocess, time\n' + body)

    def run_suite(self, **env):
        return subprocess.run(self.command, cwd=self.root, env=dict(self.env, **env),
                              capture_output=True, text=True, timeout=30)

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
            'test_probe.Remainder.test_first:0', 'test_probe.Remainder.test_second:0',
            'test_probe.DriverCleanupTests.test_first:0'})
        self.assertEqual(len(lines), 3)

    def test_fast_preserves_native_inventory_and_module_fixtures(self):
        body = '''def setUpModule():
    with open(__name__ + '.module', 'a') as out: out.write(str(os.getpid()) + '\\n')
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
        native = subprocess.run([sys.executable, '-m', 'unittest', 'discover', '-s', 'tools'],
                                cwd=self.root, env=dict(self.env, NIRI_TOOLING_FAST='1'),
                                capture_output=True, text=True, timeout=30)
        self.assertEqual(native.returncode, 0, native.stdout + native.stderr)
        self.assertIn('Ran 10 tests', native.stderr)
        self.assertIn('skipped=1, expected failures=1', native.stderr)
        native_seen = {path.name: [line.rsplit(':', 1)[0] for line in path.read_text().splitlines()]
                       for path in self.root.glob('*.seen')}
        expected = {
            'test_alpha.Cases.test_known', 'test_alpha.Cases.test_one', 'test_alpha.Cases.test_two',
            'test_beta.Cases.test_one', 'test_beta.Cases.test_two',
            'test_beta.DriverCleanupTests.test_slow',
            'test_gamma.Cases.test_one', 'test_gamma.Cases.test_two',
            'test_delta.Cases.test_one', 'test_delta.Cases.test_two',
        }
        self.command = [sys.executable, '-m', 'tools.tooling_tests', '--fast']
        for budget, children in (('10', 4), ('1', 1)):
            with self.subTest(budget=budget):
                for suffix in ('seen', 'class', 'module'):
                    for path in self.root.glob('*.' + suffix):
                        path.unlink()
                run = self.run_suite(NIRI_TOOLING_FAST='0', NEXTEST_TEST_THREADS=budget)
                self.assertEqual(run.returncode, 0, run.stdout + run.stderr)
                self.assertIn(f'Fast tooling: 10 cases, {children} children', run.stdout)
                self.assertIn('OK (skipped=1, expected failures=1)', run.stdout)
                ids = [line[5:].rsplit(': ', 1)[0] for line in run.stdout.splitlines()
                       if line.startswith('case ')]
                self.assertEqual(set(ids), expected)
                self.assertEqual(len(ids), 10)
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
                self.assertEqual(len(pids), children)

    def test_fast_rejects_ci_and_public_worker_mode_before_discovery(self):
        self.fixture("open('imported', 'w').close()\nclass C(unittest.TestCase):\n    def test_ok(self): pass\n")
        for args, message in ((['--fast', '--ci'], '--ci requires --full'),
                              (['--full', '--worker-mode', '1'], '--worker-mode requires --worker')):
            with self.subTest(args=args):
                self.command = [sys.executable, '-m', 'tools.tooling_tests', *args]
                run = self.run_suite()
                self.assertNotEqual(run.returncode, 0)
                self.assertIn(message, run.stderr)
                self.assertFalse((self.root / 'imported').exists())

    def test_fast_malformed_budget_fails_before_discovery(self):
        self.fixture("open('imported', 'w').close()\nclass C(unittest.TestCase):\n    def test_ok(self): pass\n")
        self.command = [sys.executable, '-m', 'tools.tooling_tests', '--fast']
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
        self.assertIn('test_probe.Required.test_binding', run.stdout + run.stderr)
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
        self.assertIn('EXPECTED FAILURE test_probe.C.test_expected', run.stdout)
        self.assertIn('expected failures=1', run.stdout)

    def test_unexpected_success_preserves_native_failure(self):
        self.fixture('''class C(unittest.TestCase):
    @unittest.expectedFailure
    def test_unexpected(self): pass
''')
        for mode in ('--full', '--fast'):
            with self.subTest(mode=mode):
                self.command = [sys.executable, '-m', 'tools.tooling_tests', mode]
                run = self.run_suite()
                self.assertNotEqual(run.returncode, 0, run.stdout)
                self.assertIn('UNEXPECTED SUCCESS test_probe.C.test_unexpected', run.stdout)
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
                command = [sys.executable, '-m', 'tools.tooling_tests', mode]
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


class StaticCoverageTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.files = {
            'docs/materials/scripts/optic-settling-smoke.sh': '. "$HERE/shared.sh"\npython3 "$ROOT/tools/optic_settling.py"\n',
            'docs/materials/scripts/shared.sh': '. "$HERE/nested-lib.sh"\ncapture() { ${CAPTURE_META:-python3 "$ROOT/tools/stubbed-fallback"}; }\n',
            'docs/materials/scripts/nested-lib.sh': 'value=$((1 + (2 * 3)))\n',
            'docs/materials/scripts/vt-lib.sh': '',
            'docs/materials/scripts/class-subject.sh': '',
            'tools/test_optic_settling.py': "ROOT=Path(__file__).resolve().parents[1]\nclass DriverCleanupTests:\n    script=ROOT / 'docs/materials/scripts/class-subject.sh'\n",
            'tools/test_vt_lib.py': "LIB=Path(__file__).resolve().parents[1] / 'docs/materials/scripts/vt-lib.sh'\nclass VtLibTests:\n    lib=LIB\n",
            'tools/optic_settling.py': 'import tools.other\nfrom tools import extra\n',
            'tools/other.py': 'from . import leaf\n',
            'tools/extra.py': '', 'tools/leaf.py': '',
            'tools/screencast_consumer.py': '', 'tools/fake_screencast.py': '',
        }
        for filename, source in self.files.items():
            path = self.root / filename
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(source)
        self.patterns = list(self.files)

    def check(self, patterns=None):
        from tools.tooling_tests import assert_static_coverage
        assert_static_coverage(self.root, self.patterns if patterns is None else patterns)

    def test_sources_class_paths_and_transitive_imports_must_be_routed(self):
        self.check()
        for path in ('docs/materials/scripts/nested-lib.sh',
                     'docs/materials/scripts/class-subject.sh',
                     'tools/other.py', 'tools/extra.py', 'tools/leaf.py'):
            with self.subTest(path=path), self.assertRaisesRegex(ValueError, path):
                self.check([pattern for pattern in self.patterns if pattern != path])

    def test_package_initializers_and_their_imports_must_be_routed(self):
        package = self.root / 'tools/pkg'
        package.mkdir()
        (package / 'leaf.py').write_text('')
        initializer = package / '__init__.py'
        initializer.write_text('import tools.hidden\n')
        (self.root / 'tools/hidden.py').write_text('')
        (self.root / 'tools/other.py').write_text('import tools.pkg.leaf\n')
        patterns = self.patterns + ['tools/pkg/leaf.py']
        with self.assertRaisesRegex(ValueError, 'tools/pkg/__init__.py'):
            self.check(patterns)
        patterns.append('tools/pkg/__init__.py')
        with self.assertRaisesRegex(ValueError, 'tools/hidden.py'):
            self.check(patterns)
        self.check(patterns + ['tools/hidden.py'])
        # Without the initializer, pkg is a namespace and imports no hidden code.
        initializer.unlink()
        self.check(self.patterns + ['tools/pkg/leaf.py'])

    def test_missing_source_cycle_dynamic_operand_and_missing_module_fail(self):
        cases = [
            ('docs/materials/scripts/shared.sh', '. "$HERE/missing.sh"\n', 'missing.sh'),
            ('docs/materials/scripts/nested-lib.sh', '. "$HERE/shared.sh"\n', 'cycle'),
            ('docs/materials/scripts/shared.sh', '. "$HERE/$DYNAMIC"\n', 'unsupported'),
            ('tools/other.py', 'import tools.missing\n', 'tools.missing'),
            ('tools/test_optic_settling.py', "ROOT=Path(__file__).resolve().parents[1]\nclass DriverCleanupTests:\n    script=ROOT / VARIABLE\n", 'dynamic'),
            ('tools/test_optic_settling.py', "class DriverCleanupTests:\n    script=Path(__file__).parent.parent / 'tools/extra.py'\n", 'unsupported'),
            ('tools/test_optic_settling.py', "ROOT=Path(__file__).resolve().parents[1]\nclass DriverCleanupTests:\n    script=ROOT.joinpath('tools/extra.py')\n", 'unsupported'),
        ]
        for filename, source, diagnostic in cases:
            with self.subTest(diagnostic=diagnostic):
                path = self.root / filename
                path.write_text(source)
                with self.assertRaisesRegex(ValueError, diagnostic):
                    self.check()
                path.write_text(self.files[filename])

    def test_nested_command_substitution_in_arithmetic_is_rejected_in_helpers(self):
        path = self.root / 'docs/materials/scripts/nested-lib.sh'
        path.write_text('# ordinary arithmetic is fine\nvalue=$((1 + $(clock)))\n')
        with self.assertRaisesRegex(ValueError, 'nested-lib.sh:2.*command substitution.*arithmetic'):
            self.check()

    def test_sourced_helper_inherits_driver_here_and_repository_cwd(self):
        entry = self.root / 'docs/materials/scripts/optic-settling-smoke.sh'
        entry.write_text('. "$ROOT/tools/helper.sh"\n')
        (self.root / 'tools/helper.sh').write_text(
            '. "$HERE/nested-lib.sh"\n. tools/root-helper.sh\n')
        (self.root / 'tools/root-helper.sh').write_text('')
        self.check(self.patterns + ['tools/helper.sh', 'tools/root-helper.sh'])
