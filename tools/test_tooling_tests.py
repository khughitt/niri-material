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

    def test_interrupt_reaps_a_child_from_an_unwound_case(self):
        self.fixture('''class DriverCleanupTests(unittest.TestCase):
    def test_wait(self):
        child = subprocess.Popen(['sleep', '60'], start_new_session=True)
        self.addCleanup(child.wait)
        self.addCleanup(child.kill)
        with open('child.pid', 'w') as out: out.write(str(child.pid))
        time.sleep(60)
''')
        parent = subprocess.Popen(self.command, cwd=self.root, env=self.env,
                                  stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        self.addCleanup(self.reap, parent)
        deadline = time.monotonic() + 15
        while not (self.root / 'child.pid').exists():
            self.assertIsNone(parent.poll())
            self.assertLess(time.monotonic(), deadline)
            time.sleep(0.02)
        pid = int((self.root / 'child.pid').read_text())
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
