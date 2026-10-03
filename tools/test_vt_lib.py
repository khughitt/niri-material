# tools/test_vt_lib.py
"""vt-lib.sh on a stub VT: verified switching, spare-VT choice and
restoration on every exit (docs/specs/2026-10-02-real-tty-settling-lane-design.md §4)."""

import json
import os
import signal
import subprocess
import tempfile
import time
import unittest
from pathlib import Path

LIB = Path(__file__).resolve().parents[1] / 'docs/materials/scripts/vt-lib.sh'
CHVT = ('#!/bin/sh\necho "$1" >> "$STUB_DIR/chvt.log"\n[ -z "${STUB_HANG:-}" ] || exec sleep 60\n'
        '[ -z "${STUB_STUCK:-}" ] || exit 0\nprintf "tty%s\\n" "$1" > "$VT_ACTIVE_FILE"\n')
LOGINCTL = ('#!/bin/sh\ncase $1 in\n'
            '    list-sessions) [ -z "${STUB_LIST_FAIL:-}" ] || exit 1; while read -r id vt; do echo "$id 1000 keith seat0 tty$vt"; done < "$STUB_DIR/sessions" ;;\n'
            '    show-session) [ -z "${STUB_SHOW_FAIL:-}" ] || exit 1; while read -r id vt; do if [ "$id" = "$2" ]; then echo "$vt"; fi; done < "$STUB_DIR/sessions" ;;\nesac\n')


class VtLibTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.dir = Path(temporary.name)
        self.active = self.dir / 'active'
        self.active.write_text('tty1\n')
        (self.dir / 'sessions').write_text('1 1\n7 2\n')
        for name, body in (('chvt', CHVT), ('loginctl', LOGINCTL)):
            (self.dir / name).write_text(body)
            (self.dir / name).chmod(0o755)
        self.env = dict(os.environ, STUB_DIR=str(self.dir), VT_ACTIVE_FILE=str(self.active),
                        VT_CHVT=str(self.dir / 'chvt'), VT_LOGINCTL=str(self.dir / 'loginctl'))

    def kill_group(self, process):
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except OSError:
            pass
        process.wait()

    def script(self, body):
        return f'fail() {{ echo "FAIL: $*" >&2; exit 1; }}\n. "{LIB}"\n{body}\n'

    def bash(self, body, **env):
        return subprocess.run(['bash', '-c', self.script(body)], env=dict(self.env, **env),
                              capture_output=True, text=True, timeout=30)

    def test_switch_verifies_that_the_vt_landed(self):
        self.assertEqual(self.bash('vt_switch 5').returncode, 0)
        self.assertEqual(self.active.read_text(), 'tty5\n')
        stuck = self.bash('vt_switch 6', STUB_STUCK='1')
        self.assertEqual(stuck.returncode, 1)
        self.assertEqual(self.active.read_text(), 'tty5\n')

    def test_a_hanging_chvt_is_bounded(self):
        # chvt waits for the VT to activate; a blocked switch must not stall
        # the lane or its restoration.
        started = time.monotonic()
        run = self.bash('vt_switch 5', STUB_HANG='1')
        self.assertEqual(run.returncode, 1)
        self.assertLess(time.monotonic() - started, 6)
        out = self.dir / 'vt-restore.json'
        self.active.write_text('tty5\n')
        started = time.monotonic()
        run = self.bash(f'VT_HOME=1; vt_restore "{out}"', STUB_HANG='1')
        self.assertEqual(run.returncode, 1)
        self.assertLess(time.monotonic() - started, 20)
        self.assertEqual(json.loads(out.read_text())['outcome'], 'failed')

    def test_spare_skips_home_and_logind_sessions(self):
        run = self.bash('vt_record_home; vt_spare')
        self.assertEqual(run.stdout.strip(), '3', run.stderr)

    def set_sessions(self, *vts):
        (self.dir / 'sessions').write_text(''.join(f'{i + 1} {vt}\n' for i, vt in enumerate(vts)))

    def test_spare_skips_the_home_vt(self):
        # Range starts at 2, so home 1 never proves the skip: home 3, VT 2 used.
        self.active.write_text('tty3\n')
        self.set_sessions(2)
        run = self.bash('vt_record_home; vt_spare')
        self.assertEqual(run.stdout.strip(), '4', run.stderr)

    def test_spare_without_a_free_vt_fails(self):
        self.set_sessions(*range(2, 13))
        run = self.bash('vt_record_home; vt_spare')
        self.assertEqual(run.returncode, 1)
        self.assertEqual(run.stdout, '')

    def test_spare_fails_loudly_when_loginctl_fails(self):
        for failing in ('STUB_LIST_FAIL', 'STUB_SHOW_FAIL'):
            run = self.bash('vt_record_home; vt_spare', **{failing: '1'})
            self.assertEqual(run.returncode, 1, failing)
            self.assertEqual(run.stdout, '', failing)
            self.assertIn('loginctl', run.stderr, failing)

    def test_restore_with_an_unreadable_active_vt_still_restores(self):
        out = self.dir / 'vt-restore.json'
        run = self.bash(f'vt_record_home; vt_switch 5; mv "$VT_ACTIVE_FILE" "$VT_ACTIVE_FILE.gone"; '
                        f'vt_restore "{out}"')
        self.assertEqual(run.returncode, 0, run.stderr)
        record = json.loads(out.read_text())
        self.assertEqual((record['outcome'], record['home']), ('restored', 1))
        self.assertIsNone(record['from'])
        self.assertEqual(self.active.read_text(), 'tty1\n')

    def test_failed_restore_with_an_unreadable_active_vt_is_valid_json(self):
        out = self.dir / 'vt-restore.json'
        run = self.bash(f'vt_record_home; vt_switch 5; mv "$VT_ACTIVE_FILE" "$VT_ACTIVE_FILE.gone"; '
                        f'STUB_STUCK=1 vt_restore "{out}"', STUB_STUCK='')
        self.assertEqual(run.returncode, 1)
        record = json.loads(out.read_text())
        self.assertEqual((record['outcome'], record['home']), ('failed', 1))
        self.assertIsNone(record['observed'])
        self.assertIn('unreadable', run.stderr)

    def test_restore_not_needed_when_never_switched(self):
        out = self.dir / 'vt-restore.json'
        run = self.bash(f'vt_record_home; vt_restore "{out}"')
        self.assertEqual(run.returncode, 0, run.stderr)
        self.assertEqual(json.loads(out.read_text())['outcome'], 'not-needed')

    def test_term_while_away_restores_home(self):
        out = self.dir / 'vt-restore.json'
        # The away wait is a recorded child the trap reaps, as the driver's
        # SLEEP_PID is: an unreaped sleep would hold the pipes open.
        body = (f'vt_record_home\n'
                f'trap \'kill "$AWAY" 2>/dev/null; wait "$AWAY" 2>/dev/null; '
                f'vt_restore "{out}" || exit 3; exit 143\' TERM\n'
                f'vt_switch 5\nsleep 60 & AWAY=$!\n: > "$STUB_DIR/away"\nwait "$AWAY"')
        driver = subprocess.Popen(['bash', '-c', self.script(body)], env=self.env, start_new_session=True,
                                  stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        self.addCleanup(self.kill_group, driver)
        deadline = time.monotonic() + 10
        while not (self.dir / 'away').exists():
            self.assertLess(time.monotonic(), deadline, 'never switched away')
            time.sleep(0.05)
        driver.send_signal(signal.SIGTERM)
        _, stderr = driver.communicate(timeout=30)
        self.assertEqual(driver.returncode, 143, stderr)
        self.assertEqual(self.active.read_text(), 'tty1\n')
        record = json.loads(out.read_text())
        self.assertEqual((record['outcome'], record['home'], record['from']), ('restored', 1, 5))

    def test_failed_restore_reports_the_observed_vt(self):
        out = self.dir / 'vt-restore.json'
        run = self.bash(f'vt_record_home; vt_switch 5; STUB_STUCK=1 vt_restore "{out}"')
        self.assertEqual(run.returncode, 1)
        self.assertIn('VT restoration failed', run.stderr)
        record = json.loads(out.read_text())
        self.assertEqual((record['outcome'], record['home'], record['observed']), ('failed', 1, 5))
        self.assertEqual((self.dir / 'chvt.log').read_text().split(), ['5', '1', '1', '1'])


if __name__ == '__main__':
    unittest.main()
