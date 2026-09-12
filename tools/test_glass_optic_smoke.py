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


class CaptureMetaAdoptionTest(unittest.TestCase):
    LIB = (Path(__file__).resolve().parents[1] / 'docs/materials/scripts/glass-optic-smoke-lib.sh').read_text()

    def function(self, name):
        return re.search(r'^' + name + r'\(\) \{.*?^}', self.LIB, re.S | re.M).group()

    def run_bash(self, script, out):
        return subprocess.run(['bash', '-eu', '-c', script, 'test', out], text=True, capture_output=True)

    def test_settle_before_launch_passes_config_and_name(self):
        with tempfile.TemporaryDirectory() as out:
            (Path(out) / 'A.kdl').write_text('glass')
            script = ('capture_meta() { printf "%s\\n" "$*" > "$OUT/call"; }\n' + self.function('settle_before_launch') +
                      '\nOUT=$1; settle_before_launch "$OUT/A.kdl"; cat "$OUT/call"; settle_before_launch "$OUT/A.kdl" A-move-1; cat "$OUT/call"')
            result = self.run_bash(script, out)
            self.assertEqual(result.returncode, 0, result.stderr)
            lines = result.stdout.splitlines()
            self.assertEqual(lines[0], f'settle {out} --sub-run A --input {out}/A.kdl')
            self.assertEqual(lines[1], f'settle {out} --sub-run A-move-1 --input {out}/A.kdl')

    def test_start_nested_settles_first_and_lib_never_preflights(self):
        body = self.function('start_nested')
        first = body.splitlines()[1].strip()
        self.assertEqual(first, 'settle_before_launch "$2" "${3-}"')
        top_level = re.sub(r'^\w+\(\) \{.*?^}', '', self.LIB, flags=re.S | re.M)
        code = '\n'.join(line for line in top_level.splitlines() if not line.lstrip().startswith('#'))
        self.assertNotIn('capture_preflight', code)
        self.assertNotIn('capture-meta preflight', code)

    def test_finish_hashes_every_file_but_the_manifest(self):
        with tempfile.TemporaryDirectory() as out:
            for name in ('a.png', 'b.kdl', 'c.tracy', 'capture.json', 'niri.log'):
                (Path(out) / name).write_text(name)
            (Path(out) / 'sub').mkdir(); (Path(out) / 'sub' / 'd.csv').write_text('d')
            script = ('rg() { return 1; }\n' + self.function('finish') + '\nOUT=$1; finish >/dev/null; cut -d" " -f3- "$OUT/SHA256SUMS" | sort')
            result = self.run_bash(script, out)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(result.stdout.split(), ['./a.png', './b.kdl', './c.tracy', './capture.json', './niri.log', './sub/d.csv'])

    def test_smokes_preflight_after_sourcing_and_identify_after_build(self):
        for smoke in ('glass-aurora-smoke.sh', 'glass-iridescence-smoke.sh'):
            text = (Path(__file__).resolve().parents[1] / 'docs/materials/scripts' / smoke).read_text()
            source = text.index('glass-optic-smoke-lib.sh')
            pre = text.index('capture_preflight headless')
            build = text.index('build_binaries')
            ident = text.index('capture_identity')
            self.assertTrue(source < pre < build < ident, smoke)

    def test_preflight_and_identity_pass_complete_capture_arguments(self):
        with tempfile.TemporaryDirectory() as out:
            script = ('fail() { echo "$*" >&2; exit 1; }\n'
                      'capture_meta() { printf "%s\\n" "$*"; }\n' +
                      self.function('capture_preflight') + '\n' + self.function('capture_identity') +
                      '\nOUT=$1; ROOT=/source; CAPTURE_TASK=material-task; NIRI=/bin/niri; NIRI_TRACY=; '
                      'capture_preflight headless; capture_identity --config preset=aurora')
            result = self.run_bash(script, out)
            self.assertEqual(result.returncode, 0, result.stderr)
            preflight, identity = result.stdout.splitlines()
            self.assertRegex(preflight, rf'^preflight {re.escape(out)} --lane headless --task material-task '
                             r'--fixture test --owner-pid [0-9]+ --tool weston --tool kitty --tool tracy=0\.13\.1$')
            self.assertEqual(identity, f'identity {out} --source /source --binary /bin/niri '
                             '--input /source/docs/materials/scripts/glass-optic-smoke-lib.sh '
                             '--input test --config preset=aurora')

    def test_cleanup_ignores_capture_release_refusal(self):
        with tempfile.TemporaryDirectory() as out:
            script = ('capture_meta() { printf "%s" "$*" > "$OUT/release-call"; return 1; }\n'
                      'stop_weston() { :; }\nremove_runtime_dir() { :; }\n' + self.function('cleanup') +
                      '\nOUT=$1; CAP_PID=; NIRI_PID=; cleanup')
            result = self.run_bash(script, out)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual((Path(out) / 'release-call').read_text(), f'release {out}')
