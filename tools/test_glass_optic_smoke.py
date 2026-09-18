"""A quiet window needs a complete trace before zero redraws mean anything."""
import csv
import os
import re
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

# Git exports repository variables into hooks, where they override subprocess cwd.
for _name in [_key for _key in os.environ if _key.startswith("GIT_")]:
    del os.environ[_name]


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
            script = ('rg() { return 1; }\n' + self.function('finish') + '\nOUT=$1; finish >/dev/null; cut -d" " -f3- "$OUT/SHA256SUMS" | LC_ALL=C sort')
            result = self.run_bash(script, out)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(result.stdout.split(), ['./a.png', './b.kdl', './c.tracy', './capture.json', './niri.log', './sub/d.csv'])

    def test_smokes_preflight_after_sourcing_and_identify_after_build(self):
        for smoke in ('glass-aurora-smoke.sh', 'glass-iridescence-smoke.sh',
                      'glass-render-order-smoke.sh'):
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


class RenderOrderBehindMatrixTest(unittest.TestCase):
    SCRIPT = (Path(__file__).resolve().parents[1] /
              'docs/materials/scripts/glass-render-order-smoke.sh').read_text()
    LIB = (Path(__file__).resolve().parents[1] /
           'docs/materials/scripts/glass-optic-smoke-lib.sh').read_text()

    def function(self, name):
        match = re.search(r'^' + name + r'\(\) \{.*?^}', self.SCRIPT,
                          re.S | re.M)
        self.assertIsNotNone(match, f'missing shell function {name}')
        return match.group()

    def functions(self, *names):
        return '\n'.join(self.function(name) for name in names)

    def lib_function(self, name):
        match = re.search(r'^' + name + r'\(\) \{.*?^}', self.LIB, re.S | re.M)
        self.assertIsNotNone(match, f'missing library shell function {name}')
        return match.group()

    def test_verify_runs_the_focused_pixel_matrix_in_order(self):
        cases = ('neutral_identity grain_preservation additive_matrix '
                 'dense_grain signed_transfer_probes opaque_bypass').split()
        stubs = ''.join(f'{case}() {{ echo {case}; }}\n' for case in cases)
        result = subprocess.run(
            ['bash', '-eu', '-c', stubs + self.function('pixel_matrix') +
             '\npixel_matrix'], text=True, capture_output=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.splitlines(), cases)

    def test_matrix_pins_required_fixture_values(self):
        for value in ('noise 0.02', 'noise 0.5', 'noise 1', 'saturation 0',
                      'saturation 3', 'ior 1', 'attenuation-color "#ffffff"',
                      'attenuation-color "#222436"', 'thickness 80',
                      'attenuation-distance 30', 'aurora 0.5',
                      'iridescence 0.8', 'focus "ring-light"'):
            self.assertIn(value, self.SCRIPT)
        self.assertIn('for type in white fine lightness', self.SCRIPT)
        self.assertIn('for opacity in 1.0 0.7', self.SCRIPT)
        self.assertIn('for c in baseline neutral active', self.SCRIPT)

    def test_additive_rejects_shader_fallback_before_accepting_metrics(self):
        capture = self.function('capture')
        additive = self.function('additive_case')
        self.assertIn('assert_clean_log', capture)
        self.assertLess(additive.index('capture '),
                        additive.index('glass-render-order-metrics.py" additive'))

    def test_capture_serializes_default_opacity_as_kdl_float(self):
        with tempfile.TemporaryDirectory() as out:
            script = (
                'write_config() { printf "window-rule {\\n}\\n" > "$1"; }\n'
                'start_nested() { :; }\nspawn_client() { :; }\nsleep() { :; }\n'
                'probe_rect() { :; }\nshot() { :; }\nae() { METRIC=0; }\n'
                'assert_zero() { :; }\nstop_nested() { :; }\nassert_clean_log() { :; }\n' +
                self.function('capture') +
                '\nOUT=$1; NIRI=/niri; capture default "noise 0"; '
                'grep -Fx "    opacity 1.0" "$OUT/default.kdl"')
            result = subprocess.run(['bash', '-eu', '-c', script, 'test', out],
                                    text=True, capture_output=True)
            self.assertEqual(result.returncode, 0, result.stderr)

    def test_additive_uses_face_for_aurora_and_chamfer_for_surface_lights(self):
        with tempfile.TemporaryDirectory() as out:
            script = (
                'capture() { :; }\nrg() { return 1; }\n'
                'python3() { echo "$*"; }\nretain_failure() { :; }\n' +
                self.function('additive_case') +
                '\nOUT=$1; HERE=/fixture; NIRI=/niri; PX=40; PY=40; '
                'additive_case "$2" off on none')
            for name, rect in (('aurora', '--rect 100 160 200 400'),
                               ('ring', '--rect 36 160 20 400'),
                               ('iridescence', '--rect 36 160 20 400')):
                with self.subTest(name=name):
                    result = subprocess.run(['bash', '-eu', '-c', script, 'test', out, name],
                                            text=True, capture_output=True)
                    self.assertEqual(result.returncode, 0, result.stderr)
                    self.assertIn(rect, result.stdout)

    @unittest.skipUnless(shutil.which('magick'), 'ImageMagick is required for pixel probes')
    def test_signed_transfer_helpers_reject_black_clipped_and_wrong_solid_output(self):
        with tempfile.TemporaryDirectory() as out:
            out = Path(out)
            pixels = {
                'off': bytes([0, 0, 0]) * 400,
                'black': bytes([0, 0, 0]) * 400,
                'informative': (bytes([0, 0, 0, 64, 64, 64]) * 200),
                'clipped': (bytes([0, 0, 0, 255, 255, 255]) * 200),
                'solid': bytes([255, 0, 73]) * 400,
                'wrong': bytes([255, 0, 76]) * 400,
            }
            for name, data in pixels.items():
                (out / f'{name}.png').write_bytes(b'P6\n20 20\n255\n' + data)
            script = (
                'fail() { echo "$*" >&2; exit 1; }\n'
                'is_number() { [[ $1 =~ ^-?[0-9]+([.][0-9]+)?([eE][-+]?[0-9]+)?$ ]]; }\n'
                'assert_greater() { awk -v a="$2" -v b="$3" "BEGIN { exit !(a > b) }" || fail "$1"; }\n'
                'assert_positive() { awk -v a="$2" "BEGIN { exit !(a > 0) }" || fail "$1"; }\n' +
                self.lib_function('one_code') + '\n' +
                self.functions('metric_grain', 'midrange_share',
                               'assert_informative_grain', 'uniform_rgb_error',
                               'assert_uniform_rgb') +
                '\nOUT=$1; HERE=$2; case=$3; quantum_code=$(one_code); '
                'awk -v q="$quantum_code" "BEGIN { exit !(q > 0.01) }" || fail "one_code unexpectedly normalized"; '
                'case $case in '
                'informative) assert_informative_grain ok informative off 0 0 20 20 ;; '
                'black) assert_informative_grain bad black off 0 0 20 20 ;; '
                'clipped) assert_informative_grain bad clipped off 0 0 20 20 ;; '
                'solid) assert_uniform_rgb solid solid 0 0 20 20 255 0 73.488 ;; '
                'wrong) assert_uniform_rgb wrong wrong 0 0 20 20 255 0 73.488 ;; esac')
            metrics = str(Path(__file__).resolve().parents[1] / 'docs/materials/scripts')
            for case, expected in (('informative', 0), ('black', 1), ('clipped', 1),
                                   ('solid', 0), ('wrong', 1)):
                with self.subTest(case=case):
                    result = subprocess.run(['bash', '-eu', '-c', script, 'test', str(out),
                                             metrics, case], text=True, capture_output=True)
                    self.assertEqual(result.returncode == 0, expected == 0,
                                     result.stdout + result.stderr)

    def test_gpu_trace_waits_before_strict_settle_then_checks_log_and_median(self):
        with tempfile.TemporaryDirectory() as out:
            script = (
                'fail() { exit 1; }\n'
                'date() { echo 2026-09-18T12:00:00-04:00; }\n'
                'sleep() { echo "sleep:$1" >> "$OUT/order"; }\n'
                'trace_run() { echo trace >> "$OUT/order"; }\n'
                'assert_clean_log() { echo clean >> "$OUT/order"; }\n'
                'gpu_median_ns() { echo median >> "$OUT/order"; echo 1; }\n' +
                self.function('gpu_case') +
                '\nOUT=$1; BASE_NIRI_TRACY=/base; NIRI_TRACY=/candidate; TICK=tick; '
                'gpu_case active 1')
            result = subprocess.run(['bash', '-eu', '-c', script, 'test', out],
                                    text=True, capture_output=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual((Path(out) / 'order').read_text().splitlines(),
                             ['sleep:30', 'trace', 'clean', 'median'])
            self.assertEqual((Path(out) / 'cost-cooldown.log').read_text().splitlines(), [
                '2026-09-18T12:00:00-04:00 gpu-active-1 cooldown start (30s)',
                '2026-09-18T12:00:00-04:00 gpu-active-1 cooldown complete',
            ])

    def test_gpu_trace_propagates_strict_settle_refusal_after_wait(self):
        with tempfile.TemporaryDirectory() as out:
            script = (
                'date() { echo 2026-09-18T12:00:00-04:00; }\n'
                'sleep() { echo "sleep:$1" >> "$OUT/order"; }\n'
                'trace_run() { echo trace >> "$OUT/order"; return 23; }\n'
                'assert_clean_log() { echo unexpected-clean >> "$OUT/order"; }\n'
                'gpu_median_ns() { echo unexpected-median >> "$OUT/order"; echo 1; }\n' +
                self.function('gpu_case') +
                '\nOUT=$1; BASE_NIRI_TRACY=/base; NIRI_TRACY=/candidate; TICK=tick; '
                'gpu_case active 2')
            result = subprocess.run(['bash', '-eu', '-c', script, 'test', out],
                                    text=True, capture_output=True)
            self.assertEqual(result.returncode, 23, result.stderr)
            self.assertEqual((Path(out) / 'order').read_text().splitlines(),
                             ['sleep:30', 'trace'])
            self.assertFalse((Path(out) / 'gpu-active.medians').exists())

    def test_scope_dispatch_never_runs_cost_from_pixel_scope(self):
        script = (
            'pixel_matrix() { echo pixels; }\ncost_matrix() { echo cost; }\n'
            'behind_matrix() { pixel_matrix; cost_matrix; }\n'
            'additive_case() { echo red; }\n' + self.function('run_scope') +
            '\nMODE=verify; SCOPE=$1; run_scope')
        for scope, expected in (('pixels', ['pixels']), ('cost', ['cost']),
                                ('all', ['pixels', 'cost'])):
            with self.subTest(scope=scope):
                result = subprocess.run(['bash', '-eu', '-c', script, 'test', scope],
                                        text=True, capture_output=True)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual(result.stdout.splitlines(), expected)

    def test_cost_scope_rejects_capture_override_before_preflight(self):
        validate = self.function('validate_scope')
        self.assertLess(self.SCRIPT.index('validate_scope || exit'),
                        self.SCRIPT.index('capture_preflight headless'))
        script = validate + '\nSCOPE=$1; CAPTURE_META=${2-}; validate_scope'
        refused = subprocess.run(['bash', '-eu', '-c', script, 'test', 'cost', '/waiver'],
                                 text=True, capture_output=True)
        self.assertNotEqual(refused.returncode, 0)
        allowed = subprocess.run(['bash', '-eu', '-c', script, 'test', 'pixels', '/waiver'],
                                 text=True, capture_output=True)
        self.assertEqual(allowed.returncode, 0, allowed.stderr)

    def test_explicit_retained_candidate_inputs_skip_build(self):
        with tempfile.TemporaryDirectory() as directory:
            directory = Path(directory)
            for name in ('niri', 'niri-tracy'):
                path = directory / name
                path.write_text(name)
                path.chmod(0o755)
            record = directory / 'capture.json'
            record.write_text('{}')
            script = (
                'build_binaries() { echo unexpected-build >&2; exit 9; }\n'
                'fail() { echo "$*" >&2; exit 1; }\n' + self.function('select_binaries') +
                '\nMODE=verify; CANDIDATE_NIRI=$1; CANDIDATE_NIRI_TRACY=$2; '
                'CANDIDATE_BUILD_RECORD=$3; select_binaries; '
                'printf "%s|%s|%s\\n" "$NIRI" "$NIRI_TRACY" "$CANDIDATE_BUILD_RECORD"')
            result = subprocess.run(['bash', '-eu', '-c', script, 'test',
                                     str(directory / 'niri'), str(directory / 'niri-tracy'),
                                     str(record)], text=True, capture_output=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(result.stdout.strip(),
                             f'{directory / "niri"}|{directory / "niri-tracy"}|{record}')
