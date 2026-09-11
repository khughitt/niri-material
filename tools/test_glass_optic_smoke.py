"""A quiet window needs a complete trace before zero redraws mean anything."""
import csv
import re
import subprocess
import tempfile
import unittest
from pathlib import Path


class TraceCoverageTest(unittest.TestCase):
    def count(self, rows):
        lib = (Path(__file__).resolve().parents[1] /
               'docs/materials/scripts/glass-optic-smoke-lib.sh').read_text()
        functions = '\n'.join(re.search(r'^' + name + r'\(\) \{.*?^}', lib,
                                       re.S | re.M).group()
                              for name in ('col', 'trace_end', 'count_last20'))
        with tempfile.TemporaryDirectory() as out:
            with open(Path(out) / 'case.csv', 'w') as file:
                writer = csv.writer(file, lineterminator='\n')
                writer.writerow(['name', 'ns_since_start'])
                writer.writerows(rows)
            return subprocess.run(['bash', '-eu', '-c',
                                   'export_cpu() { :; }\n'
                                   'fail() { echo "$*" >&2; exit 1; }\n' + functions +
                                   '\nOUT=$1; count_last20 case', 'test', out],
                                  text=True, capture_output=True)

    def test_complete_idle_and_four_hz_traces(self):
        heartbeat = [('Niri::refresh_idle_inhibit', t * 10**9) for t in range(5, 37)]
        idle = self.count(heartbeat)
        self.assertEqual(idle.returncode, 0, idle.stderr)
        self.assertEqual(idle.stdout, '0')
        moving = self.count(heartbeat + [('Niri::redraw', t * 250_000_000)
                                         for t in range(20, 144)])
        self.assertEqual(moving.returncode, 0, moving.stderr)
        self.assertEqual(moving.stdout, '80')

    def test_empty_truncated_and_stalled_traces_fail(self):
        for rows in ([], [('Niri::refresh_idle_inhibit', 5 * 10**9)],
                     [('Niri::refresh_idle_inhibit', t * 10**9) for t in (5, 36)]):
            with self.subTest(rows=rows):
                self.assertNotEqual(self.count(rows).returncode, 0)
