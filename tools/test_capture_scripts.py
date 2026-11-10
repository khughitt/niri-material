# tools/test_capture_scripts.py
"""The live capture harnesses live under scripts/capture/ and nothing names the old home."""

import os
import re
import subprocess
import unittest
from pathlib import Path

from tools.hermetic_git import isolate

isolate(os.environ)

ROOT = Path(__file__).resolve().parents[1]
CAPTURE = ROOT / 'scripts/capture'
OLD = 'docs/materials' + '/scripts'

LIVE = """
drag-lag-clips.sh focus-glass-content.sh focus-ring-light.sh focus-swap-clips.sh
glass-aurora-smoke.sh glass-edge-compare.py glass-edge-sheet.sh glass-iridescence-smoke.sh
glass-noise-layers-smoke.sh glass-noise-site-smoke.sh glass-optic-smoke-lib.sh
glass-parameter-sweep.sh glass-render-order-metrics.py glass-render-order-smoke.sh
hidden-window-attribution.sh idle-inhibit-client.c material-signals-smoke.sh
noctalia-glass-palette.py noctalia-surfaces.sh noise-layers-bins.py noise-layers-cost.sh
noise-placement-cost.sh optic-settling-smoke.sh ring-motion-clips.sh session-lock-client.c
swing-measured-sheet.py vdrag.py view-tilt-sheet.py vt-lib.sh warp-calibration.py
""".split()

ROOT_ASSIGNMENT = re.compile(r'^(?:REPO|ROOT)=\$\(cd "\$(?:HERE|SCRIPTS)(/\.\.)+" && pwd\)\s*$', re.M)
HERE_ASSIGNMENT = re.compile(r'^(?:HERE|SCRIPTS)=.*$', re.M)


class CaptureScriptTests(unittest.TestCase):
    def test_live_scripts_live_under_scripts_capture(self):
        for name in LIVE:
            with self.subTest(name=name):
                self.assertTrue((CAPTURE / name).is_file(), f'{name} missing under scripts/capture')
                self.assertFalse((ROOT / OLD / name).exists(), f'{name} still under the old home')

    def test_capture_scripts_resolve_root(self):
        checked = 0
        for script in sorted(CAPTURE.glob('*.sh')):
            text = script.read_text()
            root_line = ROOT_ASSIGNMENT.search(text)
            if not root_line:
                continue
            here_line = HERE_ASSIGNMENT.search(text)
            self.assertIsNotNone(here_line, f'{script.name} climbs from an unset HERE')
            program = (f'set -e; cd /; {here_line.group(0).replace("$0", str(script))}\n'
                       f'{root_line.group(0)}\necho "${{REPO:-$ROOT}}"')
            out = subprocess.run(['bash', '-c', program], capture_output=True, text=True,
                                 env={'PATH': os.environ['PATH']}, check=True).stdout.strip()
            self.assertEqual(Path(out), ROOT, script.name)
            checked += 1
        self.assertGreater(checked, 0, 'no script computes a root; the test checks nothing')

    def test_no_old_script_paths(self):
        found = subprocess.run(
            ['git', '-C', str(ROOT), 'grep', '-n', '-F', OLD, '--', 'tools', 'src', 'niri-config',
             'justfile', '.githooks', 'scripts', ':!tools/test_capture_scripts.py'],
            capture_output=True, text=True, env=os.environ)
        self.assertEqual(found.returncode, 1, f'old script paths remain:\n{found.stdout}')


if __name__ == '__main__':
    unittest.main()
