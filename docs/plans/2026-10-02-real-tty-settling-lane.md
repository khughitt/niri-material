# Real-TTY Settling Lane Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Run the `tty-resume`, `unlock` and `screencast` optic-settling acceptance cases on real DRM from a TTY, through the existing settling driver and offline analyzer.

**Architecture:** `optic-settling-smoke.sh --lane dedicated` launches niri on DRM instead of nested under Weston; everything else (manifest, journal, Tracy capture, export, analyzer) is shared with the headless lane. Three new owned helpers carry the hardware-specific parts: a VT library (switch, verify, restore on every exit), a PAM-free session-lock client in C, and a screencast consumer (D-Bus ScreenCast session + GStreamer) that samples the next frame after a request. The analyzer gains three generic checks: `edge_in`, consumer frames and cast samples.

**Tech Stack:** bash, Python 3 (unittest, PyGObject: Gio, Gst, GstVideo), C (libwayland-client, wayland-scanner), GStreamer `pipewiresrc`, niri IPC, Tracy 0.13.1 tools.

**Spec:** `docs/specs/2026-10-02-real-tty-settling-lane-design.md` (accepted, owner review round 3).

## Global Constraints

- Every host item is an owner action; no task installs packages, edits sudoers or grants device access (spec §3, `docs/materials/capture-host-setup.md`).
- VT switches go through `sudo -n chvt <n>` only, the command bounded to 2 s (kill 1 s later); each is verified by reading `/sys/class/tty/tty0/active` back within 2 s (spec §4).
- VT restoration on every exit: up to three verified attempts; outcome `not-needed`, `restored` or `failed` in `vt-restore.json`; a failed restoration exits nonzero even when every case passed; it runs before the capture lock is released (spec §4).
- niri on DRM runs with a private `dbus-daemon` per case, `debug { dbus-interfaces-in-non-session-instances; }`, and `PIPEWIRE_RUNTIME_DIR` at the user's daemon; no compositor interface reaches the user's bus (spec §4).
- DP-1's mode and `scale 1` are pinned in every dedicated case config (spec §4).
- Thresholds and protocol are the headless lane's: `idle-after-ms 5000`, hold 5 s, flush ≤ 2 redraws in 2 s, stimulus window ≤ 6 s (`tools/optic_settling.py` constants).
- A cast sample is the first frame arriving after its armed request; a sample with no frame inside its window fails the case and is never retried; pixel equality is exact on decoded bytes (spec §5).
- Pilot only after the development check (`drm-aurora`, `tty-resume`, `screencast`) passes; matrix only after a passed pilot with identical binary and configs (spec §7).
- Dimming (`display-dim`, `ops-a1715a`) never gates a capture (spec §6).

## Review Focus

1. niri's DMA-BUFs (its only buffer type) fail to import through EGL on this GPU: the consumer must exit nonzero with GStreamer's error, not hang. Task 3 Step 5b probes the import against a real niri before any driver work depends on it.
2. A frame with row padding (stride ≠ width × 3): saved samples must be tightly packed so crops align (Task 3 test).
3. A frame that arrived before the request was armed must never be saved as the sample (Task 3 and Task 4 tests).
4. A resume edge outside the VT switch or unlock window, for example from a stray pointer event, must fail the case (Task 4 test).
5. A refused return switch must leave a `failed` record and a nonzero exit, not a silent success (Task 1 test).

**Plan-level correction to the spec.** Spec §5 says crops are "computed from the window geometry niri reports". niri's IPC gives no on-screen position for tiled windows (`tile_pos_in_workspace_view` is unset; `src/layout/tests.rs` pins this), so the dedicated lane calibrates the probe rectangle from a screenshot of an opaque geometry probe, exactly as `glass-optic-smoke-lib.sh`'s `calibrate_probe_rect` does for the headless lanes. Task 6 updates the spec sentence.

**Second plan-level correction (buffer path).** Spec §4 says the consumer
"requests the linear modifier" when niri offers no CPU-mappable buffers.
niri offers DMA-BUFs only (`src/screencasting/pw_utils.rs`, dataType
`DmaBuf`), and a system-memory `videoconvert` cannot take them. The consumer
instead imports whatever modifier niri fixates through GStreamer GL on
headless EGL (`glupload ! glcolorconvert ! gldownload`). Probed on
2026-10-02 against the owner's desktop niri (owner-approved): node 106
negotiated `XR24:0x0300000000606012` / `XR24:0x0300000000e08014` (NVIDIA
tiled), imported, and delivered RGB to EOS with exit 0. Task 6 updates the
spec sentence.

---

### Task 1: VT library with verified switching and restoration

**Files:**
- Create: `docs/materials/scripts/vt-lib.sh`
- Test: `tools/test_vt_lib.py`

**Interfaces:**
- Consumes: a `fail` function defined by the sourcing script.
- Produces (sourced by Task 5's driver):
  - `VT_HOME` (string, set by `vt_record_home`), `VT_ACTIVE_FILE` (default `/sys/class/tty/tty0/active`), `VT_CHVT` (test override; default `sudo -n chvt`), `VT_LOGINCTL` (test override; default `loginctl`).
  - `vt_active` → prints the active VT number.
  - `vt_record_home` → sets `VT_HOME` or fails.
  - `vt_spare` → prints the lowest VT in 2..12 that is not `VT_HOME` and has no logind session; returns 1 if none.
  - `vt_switch N` → 0 when VT N is active within 2 s, else 1.
  - `vt_restore OUT_JSON` → writes the outcome; returns 1 only on `failed`; does nothing when `VT_HOME` is empty.

- [ ] **Step 1: Write the failing tests**

```python
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
LOGINCTL = ('#!/bin/sh\ncase $1 in\n    list-sessions) printf "1 1000 keith seat0 tty1\\n7 1000 keith seat0 tty2\\n" ;;\n'
            '    show-session) case $2 in 1) echo 1 ;; 7) echo 2 ;; esac ;;\nesac\n')


class VtLibTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.dir = Path(temporary.name)
        self.active = self.dir / 'active'
        self.active.write_text('tty1\n')
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
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `just --set one_cmd 'python3 -m unittest' test-one tools.test_vt_lib`
Expected: FAIL (every test; `vt-lib.sh` does not exist).

- [ ] **Step 3: Write the library**

```bash
# docs/materials/scripts/vt-lib.sh
# vt-lib.sh: verified VT switching and restoration for the dedicated capture
# lane (docs/specs/2026-10-02-real-tty-settling-lane-design.md §4). Sourced;
# the caller defines fail(). libseat's logind backend releases the seat on
# exit without switching back, so the caller's exit handler must call
# vt_restore. Tests override VT_ACTIVE_FILE, VT_CHVT and VT_LOGINCTL.
VT_ACTIVE_FILE=${VT_ACTIVE_FILE:-/sys/class/tty/tty0/active}
VT_HOME=
vt_active() { local name; name=$(cat "$VT_ACTIVE_FILE"); echo "${name#tty}"; }
# chvt blocks until the VT activates (kbd's chvt waits on VT_WAITACTIVE), so
# the command itself is bounded: 2 s, then KILL 1 s later.
vt_chvt() {
    if [ -n "${VT_CHVT:-}" ]; then timeout -k 1 2 "$VT_CHVT" "$1"
    else timeout -k 1 2 sudo -n chvt "$1"; fi
}
vt_switch() {   # N: 0 when VT N is active within 2 s of a bounded chvt
    vt_chvt "$1" || return 1
    local _
    for _ in $(seq 20); do [ "$(vt_active)" = "$1" ] && return 0; sleep 0.1; done
    return 1
}
vt_record_home() {
    VT_HOME=$(vt_active)
    [[ $VT_HOME =~ ^[0-9]+$ ]] || fail "cannot read the active VT from $VT_ACTIVE_FILE"
}
vt_spare() {   # the lowest VT in 2..12 that is not home and has no logind session
    local login=${VT_LOGINCTL:-loginctl} used session n
    used=$("$login" list-sessions --no-legend | while read -r session _; do
        "$login" show-session "$session" -p VTNr --value
    done)
    for n in $(seq 2 12); do
        [ "$n" = "$VT_HOME" ] && continue
        grep -qx "$n" <<< "$used" || { echo "$n"; return 0; }
    done
    return 1
}
vt_restore() {   # OUT_JSON: put VT_HOME back; 1 only when that failed
    local out=$1 from attempt
    [ -n "$VT_HOME" ] || return 0
    from=$(vt_active)
    if [ "$from" = "$VT_HOME" ]; then
        printf '{"outcome": "not-needed", "home": %s}\n' "$VT_HOME" > "$out"
        return 0
    fi
    for attempt in 1 2 3; do
        if vt_switch "$VT_HOME"; then
            printf '{"outcome": "restored", "home": %s, "from": %s, "attempts": %s}\n' \
                "$VT_HOME" "$from" "$attempt" > "$out"
            echo "VT restored to $VT_HOME from $from" >&2
            return 0
        fi
    done
    printf '{"outcome": "failed", "home": %s, "observed": %s}\n' "$VT_HOME" "$(vt_active)" > "$out"
    echo "FAIL: VT restoration failed: home $VT_HOME, active $(vt_active)" >&2
    return 1
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `just --set one_cmd 'python3 -m unittest' test-one tools.test_vt_lib`
Expected: PASS (6 tests). Then `bash -n docs/materials/scripts/vt-lib.sh`.

- [ ] **Step 5: Commit**

```bash
git add docs/materials/scripts/vt-lib.sh tools/test_vt_lib.py
git commit -m "feat(material): verified VT switching and restoration for the TTY lane (material-f7eb0b)"
```

---

### Task 2: PAM-free session-lock client

**Files:**
- Create: `docs/materials/scripts/session-lock-client.c`

**Interfaces:**
- Produces: an executable (built per run by Task 5's `build_client`) that locks on start, prints exactly `locked` (one line, flushed) once the compositor sends `locked`, and on SIGUSR1 calls `unlock_and_destroy`, round-trips and exits 0. SIGTERM or SIGINT exits 2 without unlocking; a `finished` event exits 1. Build inputs: `ext-session-lock-v1.xml` from `/usr/share/wayland-protocols/staging/ext-session-lock/`.

- [ ] **Step 1: Write the client**

```c
// session-lock-client.c: a PAM-free ext_session_lock_v1 client for the
// optic-settling TTY lane (docs/specs/2026-10-02-real-tty-settling-lane-design.md
// §4). Locks on start, gives every output a solid lock surface, prints
// "locked" when the compositor confirms, and on SIGUSR1 unlocks and exits 0.
// It never authenticates: niri's SessionLockHandler::unlock runs exactly as
// for any lock client. SIGTERM/SIGINT exit 2 without unlocking; a refused or
// ended lock ("finished") exits 1.
#define _GNU_SOURCE
#include <errno.h>
#include <poll.h>
#include <signal.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>
#include <unistd.h>

#include <wayland-client.h>

#include "ext-session-lock-v1-client-protocol.h"

enum { MAX_OUTPUTS = 8 };

struct lock_output {
    struct wl_output *output;
    struct wl_surface *surface;
    struct ext_session_lock_surface_v1 *lock_surface;
    struct wl_buffer *buffer;
};

static struct wl_compositor *compositor;
static struct wl_shm *shm;
static struct ext_session_lock_manager_v1 *manager;
static struct lock_output outputs[MAX_OUTPUTS];
static int n_outputs;
static int locked;
static volatile sig_atomic_t unlock_requested, stop_requested;

static void die(int status, const char *what) {
    fprintf(stderr, "session-lock-client: %s\n", what);
    exit(status);
}

static void on_usr1(int sig) {
    (void)sig;
    unlock_requested = 1;
}

static void on_stop(int sig) {
    (void)sig;
    stop_requested = 1;
}

static void registry_global(void *data, struct wl_registry *registry, uint32_t name,
                            const char *interface, uint32_t version) {
    (void)data;
    (void)version;
    if (strcmp(interface, wl_compositor_interface.name) == 0)
        compositor = wl_registry_bind(registry, name, &wl_compositor_interface, 4);
    else if (strcmp(interface, wl_shm_interface.name) == 0)
        shm = wl_registry_bind(registry, name, &wl_shm_interface, 1);
    else if (strcmp(interface, ext_session_lock_manager_v1_interface.name) == 0)
        manager = wl_registry_bind(registry, name, &ext_session_lock_manager_v1_interface, 1);
    else if (strcmp(interface, wl_output_interface.name) == 0 && n_outputs < MAX_OUTPUTS)
        outputs[n_outputs++].output = wl_registry_bind(registry, name, &wl_output_interface, 1);
}

static void registry_remove(void *data, struct wl_registry *registry, uint32_t name) {
    (void)data;
    (void)registry;
    (void)name;
}

static const struct wl_registry_listener registry_listener = {
    .global = registry_global,
    .global_remove = registry_remove,
};

static struct wl_buffer *solid_buffer(uint32_t width, uint32_t height) {
    int stride = (int)width * 4, size = stride * (int)height;
    int fd = memfd_create("session-lock-client", MFD_CLOEXEC);
    if (fd < 0 || ftruncate(fd, size) < 0)
        die(1, "shm file");
    uint32_t *pixels = mmap(NULL, size, PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0);
    if (pixels == MAP_FAILED)
        die(1, "mmap");
    for (uint32_t i = 0; i < width * height; i++)
        pixels[i] = 0xff202428;
    munmap(pixels, size);
    struct wl_shm_pool *pool = wl_shm_create_pool(shm, fd, size);
    struct wl_buffer *buffer = wl_shm_pool_create_buffer(pool, 0, (int)width, (int)height, stride,
                                                         WL_SHM_FORMAT_XRGB8888);
    wl_shm_pool_destroy(pool);
    close(fd);
    return buffer;
}

static void lock_surface_configure(void *data, struct ext_session_lock_surface_v1 *lock_surface,
                                   uint32_t serial, uint32_t width, uint32_t height) {
    struct lock_output *out = data;
    ext_session_lock_surface_v1_ack_configure(lock_surface, serial);
    if (out->buffer)
        wl_buffer_destroy(out->buffer);
    out->buffer = solid_buffer(width, height);
    wl_surface_attach(out->surface, out->buffer, 0, 0);
    wl_surface_damage_buffer(out->surface, 0, 0, (int32_t)width, (int32_t)height);
    wl_surface_commit(out->surface);
}

static const struct ext_session_lock_surface_v1_listener lock_surface_listener = {
    .configure = lock_surface_configure,
};

static void lock_locked(void *data, struct ext_session_lock_v1 *lock) {
    (void)data;
    (void)lock;
    locked = 1;
    printf("locked\n");
    fflush(stdout);
}

static void lock_finished(void *data, struct ext_session_lock_v1 *lock) {
    (void)data;
    (void)lock;
    die(1, "the compositor refused or ended the lock");
}

static const struct ext_session_lock_v1_listener lock_listener = {
    .locked = lock_locked,
    .finished = lock_finished,
};

int main(void) {
    struct sigaction usr1 = {.sa_handler = on_usr1}, stop = {.sa_handler = on_stop};
    sigaction(SIGUSR1, &usr1, NULL);
    sigaction(SIGTERM, &stop, NULL);
    sigaction(SIGINT, &stop, NULL);

    struct wl_display *display = wl_display_connect(NULL);
    if (!display)
        die(1, "cannot connect to the Wayland display");
    struct wl_registry *registry = wl_display_get_registry(display);
    wl_registry_add_listener(registry, &registry_listener, NULL);
    wl_display_roundtrip(display);
    if (!compositor || !shm || !manager || n_outputs == 0)
        die(1, "missing wl_compositor, wl_shm, ext_session_lock_manager_v1 or an output");

    struct ext_session_lock_v1 *lock = ext_session_lock_manager_v1_lock(manager);
    ext_session_lock_v1_add_listener(lock, &lock_listener, NULL);
    for (int i = 0; i < n_outputs; i++) {
        outputs[i].surface = wl_compositor_create_surface(compositor);
        outputs[i].lock_surface =
            ext_session_lock_v1_get_lock_surface(lock, outputs[i].surface, outputs[i].output);
        ext_session_lock_surface_v1_add_listener(outputs[i].lock_surface, &lock_surface_listener,
                                                 &outputs[i]);
    }

    struct pollfd fd = {.fd = wl_display_get_fd(display), .events = POLLIN};
    while (!unlock_requested && !stop_requested) {
        while (wl_display_prepare_read(display) != 0)
            wl_display_dispatch_pending(display);
        if (wl_display_flush(display) < 0 && errno != EAGAIN) {
            wl_display_cancel_read(display);
            die(1, "flush failed");
        }
        if (poll(&fd, 1, -1) < 0) {
            wl_display_cancel_read(display);
            if (errno == EINTR)
                continue;
            die(1, "poll failed");
        }
        if (wl_display_read_events(display) < 0)
            die(1, "the compositor closed the connection");
        wl_display_dispatch_pending(display);
    }
    if (stop_requested)
        die(2, "stopped without unlocking");
    if (!locked)
        die(1, "unlock requested before the compositor confirmed the lock");

    ext_session_lock_v1_unlock_and_destroy(lock);
    wl_display_roundtrip(display);
    wl_display_disconnect(display);
    return 0;
}
```

- [ ] **Step 2: Build it with the driver's flags**

```bash
B=$(mktemp -d)
X=/usr/share/wayland-protocols/staging/ext-session-lock/ext-session-lock-v1.xml
wayland-scanner client-header "$X" "$B/ext-session-lock-v1-client-protocol.h"
wayland-scanner private-code "$X" "$B/ext-session-lock-v1-protocol.c"
cc -std=c11 -Wall -Wextra -Werror -O2 -I"$B" -o "$B/session-lock-client" \
    docs/materials/scripts/session-lock-client.c "$B/ext-session-lock-v1-protocol.c" -lwayland-client
```

Expected: builds with no warnings.

- [ ] **Step 3: Smoke it on headless Weston (offscreen, never the desktop)**

Write a scratch script (not committed) in the session scratchpad that owns its cleanup: a `systemd-run --user --unit=lock-smoke-weston --collect weston --backend=headless --renderer=gl --shell=kiosk-shell.so --width=1280 --height=720 --socket=lks-weston`, a short runtime dir `$XDG_RUNTIME_DIR/lks` with the Weston socket symlinked in, a debug build of this worktree's niri (`cargo build -p niri`, copied to the scratchpad) started with `XDG_RUNTIME_DIR=$RT WAYLAND_DISPLAY=lks-weston`, and a `trap` on EXIT/TERM/INT that kills niri and the client and stops the unit. In it: start the client against niri's socket with stdout to a file; wait up to 5 s for the line `locked`; send SIGUSR1; `wait` it.

Expected: `locked` appears, exit status 0, niri's log shows no panic, `systemctl --user is-active lock-smoke-weston` reports `inactive` afterwards. Also send SIGTERM to a second client instance after `locked`: exit status 2.

- [ ] **Step 4: Commit**

```bash
git add docs/materials/scripts/session-lock-client.c
git commit -m "feat(material): PAM-free session-lock client for the TTY lane (material-f7eb0b)"
```

---

### Task 3: Screencast consumer with armed frame sampling

**Files:**
- Move: `docs/materials/scripts/screencast-consumer.py` → `tools/screencast_consumer.py` (`git mv`)
- Modify: `tools/screencast_consumer.py`
- Create: `tools/fake_screencast.py` (a minimal ScreenCast D-Bus service for the end-to-end test)
- Test: `tools/test_screencast_consumer.py`

**Interfaces:**
- Produces (used by Task 5's driver):
  - CLI: `python3 tools/screencast_consumer.py CONNECTOR FRAMES SUMMARY REQUEST_FILE` with `DBUS_SESSION_BUS_ADDRESS` set to the case's private bus.
  - stdout: `ready <node>` once the pipeline is PLAYING (not on the first frame: niri sends no frame without damage).
  - On SIGUSR1: reads the target path from `REQUEST_FILE`, writes `<target>.armed` containing the request time, then saves the first frame arriving after it as tightly packed RGB to `<target>` and `<target>.json` (`request_mono_ns`, `frame_mono_ns`, `width`, `height`), in that order.
  - `FRAMES`: one CLOCK_MONOTONIC ns per received frame. `SUMMARY`: `consumer`, `gstreamer`, `pipewiresrc`, `connector`, `node`, `frames`, `first_mono_ns`, `last_mono_ns`, `stopped_by_signal`.
  - Module API (tested): `Sampler.request(path, now_ns)`, `Sampler.offer(data, width, height, arrival_ns) -> bool`, `Sampler.pending`, `packed_rgb(data, width, height, stride) -> bytes`, `wait_for(subscribe, timeout_s) -> value | None`, `PIPELINE` (format string with `{node}`).
  - Test seam: `SCREENCAST_CONSUMER_PIPELINE` replaces `PIPELINE` (offline tests only; the driver never sets it).

- [ ] **Step 1: Move the consumer**

```bash
git mv docs/materials/scripts/screencast-consumer.py tools/screencast_consumer.py
```

- [ ] **Step 2: Write the failing tests**

```python
# tools/test_screencast_consumer.py
"""The consumer's sampling logic without GStreamer: a sample is the first
frame after an armed request, tightly packed (spec 2026-10-02 real-TTY §4)."""

import json
import os
import shutil
import signal
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path

from tools.screencast_consumer import PIPELINE, Sampler, packed_rgb, wait_for

ROOT = Path(__file__).resolve().parents[1]


class SamplerTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.target = Path(temporary.name) / 'sample-1.raw'

    def test_saves_the_first_frame_after_the_request(self):
        sampler = Sampler()
        sampler.request(self.target, 1_000)
        self.assertEqual(Path(f'{self.target}.armed').read_text(), '1000\n')
        self.assertFalse(sampler.offer(b'old', 1, 1, 1_000))       # not after the request
        self.assertFalse(self.target.exists())
        self.assertTrue(sampler.offer(b'new', 1, 1, 1_001))
        self.assertEqual(self.target.read_bytes(), b'new')
        self.assertEqual(json.loads(Path(f'{self.target}.json').read_text()),
                         {'request_mono_ns': 1_000, 'frame_mono_ns': 1_001, 'width': 1, 'height': 1})
        self.assertIsNone(sampler.pending)
        self.assertFalse(sampler.offer(b'later', 1, 1, 1_002))     # one frame per request

    def test_refuses_a_second_request_while_one_is_pending(self):
        sampler = Sampler()
        sampler.request(self.target, 1)
        with self.assertRaises(RuntimeError):
            sampler.request(self.target.with_name('sample-2.raw'), 2)

    def test_nothing_is_saved_without_a_request(self):
        self.assertFalse(Sampler().offer(b'frame', 1, 1, 5))


try:
    import gi
    gi.require_version('GLib', '2.0')
    from gi.repository import GLib
except (ImportError, ValueError):
    GLib = None


@unittest.skipIf(GLib is None, 'PyGObject is not installed')
class WaitForTests(unittest.TestCase):
    def test_the_discovery_deadline_cannot_stop_consumption(self):
        def subscribe(deliver):
            GLib.timeout_add(50, lambda: deliver(42) or GLib.SOURCE_REMOVE)
            return lambda: None

        self.assertEqual(wait_for(subscribe, 0.3), 42)
        # The consumption loop outlives the discovery deadline (0.3 s).
        loop = GLib.MainLoop()
        GLib.timeout_add(700, lambda: loop.quit() or GLib.SOURCE_REMOVE)
        started = time.monotonic()
        loop.run()
        self.assertGreater(time.monotonic() - started, 0.65)

    def test_times_out_without_a_node(self):
        started = time.monotonic()
        self.assertIsNone(wait_for(lambda deliver: (lambda: None), 0.2))
        self.assertLess(time.monotonic() - started, 1)


class PipelineTests(unittest.TestCase):
    def test_imports_dma_bufs_through_gl(self):
        description = PIPELINE.format(node=7)
        self.assertIn('pipewiresrc path=7 ! video/x-raw(memory:DMABuf),format=DMA_DRM ! glupload', description)
        self.assertIn('gldownload ! videoconvert ! video/x-raw,format=RGB ! appsink', description)


# Startup through summary on a private bus, with a fake ScreenCast service and
# a test source in place of PipeWire. 6x3-byte rows are padded to 20 bytes by
# GStreamer, so the sample's size also proves packing.
def consumer_bindings():
    """PyGObject with the introspection data the consumer and fake service load."""
    try:
        import gi
        for namespace, version in (('Gio', '2.0'), ('GLib', '2.0'), ('Gst', '1.0'), ('GstVideo', '1.0')):
            gi.require_version(namespace, version)
        from gi.repository import Gio, GLib, Gst, GstVideo  # noqa: F401
    except (ImportError, ValueError):
        return False
    return True


@unittest.skipUnless(shutil.which('dbus-daemon') and shutil.which('gst-launch-1.0') and consumer_bindings(),
                     'needs dbus-daemon, GStreamer and PyGObject with Gio, Gst and GstVideo')
class ConsumerEndToEndTests(unittest.TestCase):
    def test_startup_sampling_and_summary(self):
        tmp = Path(tempfile.mkdtemp()); self.addCleanup(shutil.rmtree, tmp)
        bus = subprocess.Popen(['dbus-daemon', '--session', '--nofork', f'--address=unix:path={tmp}/bus'])
        self.addCleanup(lambda: (bus.kill(), bus.wait()))
        for _ in range(50):
            if (tmp / 'bus').exists(): break
            time.sleep(0.1)
        env = dict(os.environ, DBUS_SESSION_BUS_ADDRESS=f'unix:path={tmp}/bus',
                   SCREENCAST_CONSUMER_PIPELINE='videotestsrc is-live=true ! video/x-raw,width=6,height=4 '
                   '! videoconvert ! video/x-raw,format=RGB ! appsink name=sink emit-signals=true sync=false')
        service = subprocess.Popen([sys.executable, str(ROOT / 'tools/fake_screencast.py'), '7'], env=env,
                                   stdout=subprocess.PIPE, text=True)
        self.addCleanup(lambda: (service.kill(), service.wait()))
        time.sleep(0.5)
        request = tmp / 'request'; target = tmp / 'sample-1.raw'
        consumer = subprocess.Popen([sys.executable, str(ROOT / 'tools/screencast_consumer.py'), 'DP-1',
                                     str(tmp / 'frames'), str(tmp / 'summary.json'), str(request)],
                                    env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        self.addCleanup(lambda: consumer.poll() is None and consumer.kill())
        self.assertEqual(consumer.stdout.readline().strip(), 'ready 7')
        request.write_text(f'{target}\n')
        consumer.send_signal(signal.SIGUSR1)
        for _ in range(50):
            if Path(f'{target}.json').exists(): break
            time.sleep(0.1)
        consumer.send_signal(signal.SIGTERM)
        _, stderr = consumer.communicate(timeout=10)
        self.assertEqual(consumer.returncode, 0, stderr)
        summary = json.loads((tmp / 'summary.json').read_text())
        self.assertEqual((summary['node'], summary['connector'], summary['stopped_by_signal']), (7, 'DP-1', True))
        self.assertGreater(summary['frames'], 0)
        sample = json.loads(Path(f'{target}.json').read_text())
        self.assertEqual((sample['width'], sample['height']), (6, 4))
        self.assertEqual(len(target.read_bytes()), 6 * 4 * 3)
        self.assertLess(sample['request_mono_ns'], sample['frame_mono_ns'])


class PackedRgbTests(unittest.TestCase):
    def test_strips_row_padding(self):
        # 2x2 RGB, stride 8: two pad bytes per row.
        data = bytes([1, 2, 3, 4, 5, 6, 0, 0, 7, 8, 9, 10, 11, 12, 0, 0])
        self.assertEqual(packed_rgb(data, 2, 2, 8), bytes(range(1, 13)))

    def test_packed_input_is_unchanged(self):
        data = bytes(range(12))
        self.assertEqual(packed_rgb(data, 2, 2, 6), data)


if __name__ == '__main__':
    unittest.main()
```

- [ ] **Step 2a: Write the fake ScreenCast service**

```python
# tools/fake_screencast.py
"""A minimal org.gnome.Mutter.ScreenCast service for consumer tests."""
import sys
import gi
gi.require_version('Gio', '2.0')
from gi.repository import Gio, GLib

NODE = int(sys.argv[1])
XML = '''<node>
<interface name="org.gnome.Mutter.ScreenCast"><method name="CreateSession"><arg type="a{sv}" direction="in"/><arg type="o" direction="out"/></method></interface>
<interface name="org.gnome.Mutter.ScreenCast.Session"><method name="RecordMonitor"><arg type="s" direction="in"/><arg type="a{sv}" direction="in"/><arg type="o" direction="out"/></method><method name="Start"/><method name="Stop"/></interface>
<interface name="org.gnome.Mutter.ScreenCast.Stream"><signal name="PipeWireStreamAdded"><arg type="u"/></signal></interface>
</node>'''
info = Gio.DBusNodeInfo.new_for_xml(XML)
loop = GLib.MainLoop()
calls = []

def handle(connection, sender, path, interface, method, params, invocation):
    calls.append(method)
    print(method, flush=True)
    if method == 'CreateSession':
        invocation.return_value(GLib.Variant('(o)', ('/s',)))
    elif method == 'RecordMonitor':
        invocation.return_value(GLib.Variant('(o)', ('/st',)))
    elif method == 'Start':
        invocation.return_value(None)
        connection.emit_signal(None, '/st', 'org.gnome.Mutter.ScreenCast.Stream', 'PipeWireStreamAdded',
                               GLib.Variant('(u)', (NODE,)))
    else:
        invocation.return_value(None)

def on_bus(connection, name):
    for path, iface in (('/org/gnome/Mutter/ScreenCast', 0), ('/s', 1)):
        connection.register_object(path, info.interfaces[iface], handle, None, None)

Gio.bus_own_name(Gio.BusType.SESSION, 'org.gnome.Mutter.ScreenCast', 0, on_bus, None, lambda *a: loop.quit())
loop.run()
```

- [ ] **Step 3: Run the tests to verify they fail**

Run: `just --set one_cmd 'python3 -m unittest' test-one tools.test_screencast_consumer`
Expected: FAIL with `ImportError: cannot import name 'Sampler'` (or a module error, since `gi` is imported at module level today).

- [ ] **Step 4: Rewrite the consumer**

Replace the whole file with:

```python
#!/usr/bin/env python3
"""A real screencast consumer for the optic-settling TTY lane (material-3acc86,
docs/specs/2026-10-02-real-tty-settling-lane-design.md §4).

Opens an org.gnome.Mutter.ScreenCast session on the session bus (the case's
private bus, where niri on DRM serves it), records one monitor with the
cursor hidden, and consumes the PipeWire node through GStreamer
(pipewiresrc ! glupload ! gldownload ! RGB appsink: niri sends DMA-BUFs only,
imported through EGL without a display). Every received frame is
journalled as one CLOCK_MONOTONIC line in FRAMES, the clock of niri's optic
edges. "ready <node>" goes to stdout once the pipeline plays: niri sends no
frame without damage, so readiness cannot wait for one.

SIGUSR1 arms a sample: the target path is read from REQUEST_FILE, the
request time is written to <target>.armed, and the first frame arriving
after it is saved as packed RGB to <target>, then its times and size to
<target>.json. SIGTERM or SIGINT stops the session and writes SUMMARY.

    screencast_consumer.py CONNECTOR FRAMES SUMMARY REQUEST_FILE
"""

import json
import os
import signal
import sys
import time
from pathlib import Path

BUS_NAME = 'org.gnome.Mutter.ScreenCast'
NODE_TIMEOUT_S = 10
CURSOR_HIDDEN = 0
PIPELINE = ('pipewiresrc path={node} ! video/x-raw(memory:DMABuf),format=DMA_DRM '
            '! glupload ! glcolorconvert ! gldownload ! videoconvert ! video/x-raw,format=RGB '
            '! appsink name=sink emit-signals=true sync=false')
# A TTY has no display server: GStreamer GL uses EGL with a pbuffer surface.
GL_ENV = {'GST_GL_PLATFORM': 'egl', 'GST_GL_WINDOW': 'surfaceless'}


def wait_for(subscribe, timeout_s):
    """Run a private main loop until subscribe's deliver(value) or the timeout.

    subscribe(deliver) starts the wait and returns an unsubscribe callable.
    The timeout source is removed before returning, so it can never stop a
    later loop: the consumer's frames run long after this deadline."""
    from gi.repository import GLib

    loop = GLib.MainLoop()
    found = []

    def deliver(value):
        if not found:
            found.append(value)
            loop.quit()

    def expire():
        loop.quit()
        return GLib.SOURCE_REMOVE

    unsubscribe = subscribe(deliver)
    timer = GLib.timeout_add(int(timeout_s * 1000), expire)
    loop.run()
    if found:
        GLib.source_remove(timer)
    unsubscribe()
    return found[0] if found else None


def packed_rgb(data, width, height, stride):
    """RGB rows without GStreamer's row padding."""
    row = width * 3
    if stride == row:
        return bytes(data[:row * height])
    return b''.join(bytes(data[y * stride:y * stride + row]) for y in range(height))


class Sampler:
    """Saves the first frame that arrives after an armed request."""

    def __init__(self):
        self.pending = None

    def request(self, path, now_ns):
        if self.pending is not None:
            raise RuntimeError('a sample is already pending')
        self.pending = (Path(path), now_ns)
        Path(f'{path}.armed').write_text(f'{now_ns}\n')

    def offer(self, data, width, height, arrival_ns):
        if self.pending is None:
            return False
        path, requested = self.pending
        if arrival_ns <= requested:
            return False
        path.write_bytes(data)
        Path(f'{path}.json').write_text(json.dumps(dict(
            request_mono_ns=requested, frame_mono_ns=arrival_ns, width=width, height=height)) + '\n')
        self.pending = None
        return True


def main():
    import gi
    gi.require_version('Gst', '1.0')
    gi.require_version('GstVideo', '1.0')
    gi.require_version('Gio', '2.0')
    from gi.repository import Gio, GLib, Gst, GstVideo

    connector, frames_path, summary_path, request_file = sys.argv[1:5]
    os.environ.update(GL_ENV)
    Gst.init(None)
    factory = Gst.ElementFactory.find('pipewiresrc')
    if factory is None:
        sys.exit('screencast-consumer: no pipewiresrc element (gst-plugin-pipewire)')

    bus = Gio.bus_get_sync(Gio.BusType.SESSION, None)

    def call(path, interface, method, args, reply):
        return bus.call_sync(BUS_NAME, path, interface, method, args, GLib.VariantType(reply),
                             Gio.DBusCallFlags.NONE, 5000, None).unpack()

    (session,) = call('/org/gnome/Mutter/ScreenCast', 'org.gnome.Mutter.ScreenCast',
                      'CreateSession', GLib.Variant('(a{sv})', ({},)), '(o)')
    properties = {'cursor-mode': GLib.Variant('u', CURSOR_HIDDEN)}
    (stream,) = call(session, 'org.gnome.Mutter.ScreenCast.Session', 'RecordMonitor',
                     GLib.Variant('(sa{sv})', (connector, properties)), '(o)')

    def subscribe(deliver):
        handle = bus.signal_subscribe(
            BUS_NAME, 'org.gnome.Mutter.ScreenCast.Stream', 'PipeWireStreamAdded', stream, None,
            Gio.DBusSignalFlags.NONE, lambda *args: deliver(args[-1].unpack()[0]))
        call(session, 'org.gnome.Mutter.ScreenCast.Session', 'Start', None, '()')
        return lambda: bus.signal_unsubscribe(handle)

    node = wait_for(subscribe, NODE_TIMEOUT_S)
    if node is None:
        sys.exit(f'screencast-consumer: no PipeWireStreamAdded within {NODE_TIMEOUT_S} s')

    # niri offers DMA-BUFs only (pw_utils: dataType DmaBuf) with the modifier
    # it fixates from the consumer's list. glupload imports any of them
    # through EGL; gldownload brings the frame to system memory.
    # SCREENCAST_CONSUMER_PIPELINE: offline tests substitute a test source.
    description = os.environ.get('SCREENCAST_CONSUMER_PIPELINE', PIPELINE)
    pipeline = Gst.parse_launch(description.format(node=node))
    sink = pipeline.get_by_name('sink')
    loop = GLib.MainLoop()
    frames = open(frames_path, 'w', buffering=1)
    times = []
    sampler = Sampler()

    def on_sample(appsink):
        sample = appsink.emit('pull-sample')
        now = time.monotonic_ns()
        times.append(now)
        frames.write(f'{now}\n')
        if sampler.pending is not None:
            info = GstVideo.VideoInfo.new_from_caps(sample.get_caps())
            buffer = sample.get_buffer()
            ok, mapped = buffer.map(Gst.MapFlags.READ)
            if not ok:
                sys.stderr.write('screencast-consumer: cannot map a frame\n')
                loop.quit()
                return Gst.FlowReturn.ERROR
            try:
                data = packed_rgb(mapped.data, info.width, info.height, info.stride[0])
            finally:
                buffer.unmap(mapped)
            sampler.offer(data, info.width, info.height, now)
        return Gst.FlowReturn.OK

    def on_error(_bus, message):
        error, debug = message.parse_error()
        sys.stderr.write(f'screencast-consumer: {error.message} ({debug})\n')
        loop.quit()

    def on_request():
        sampler.request(Path(request_file).read_text().strip(), time.monotonic_ns())
        return GLib.SOURCE_CONTINUE

    stopping = []

    def stop():
        stopping.append(True)
        loop.quit()
        return GLib.SOURCE_REMOVE

    sink.connect('new-sample', on_sample)
    pipeline_bus = pipeline.get_bus()
    pipeline_bus.add_signal_watch()
    pipeline_bus.connect('message::error', on_error)
    GLib.unix_signal_add(GLib.PRIORITY_HIGH, signal.SIGUSR1, on_request)
    GLib.unix_signal_add(GLib.PRIORITY_HIGH, signal.SIGTERM, stop)
    GLib.unix_signal_add(GLib.PRIORITY_HIGH, signal.SIGINT, stop)
    if pipeline.set_state(Gst.State.PLAYING) == Gst.StateChangeReturn.FAILURE:
        sys.exit('screencast-consumer: the pipeline did not start')
    print(f'ready {node}', flush=True)
    loop.run()

    pipeline.set_state(Gst.State.NULL)
    call(session, 'org.gnome.Mutter.ScreenCast.Session', 'Stop', None, '()')
    frames.close()
    summary = dict(
        consumer='gstreamer pipewiresrc ! glupload ! gldownload ! videoconvert ! appsink',
        gstreamer=Gst.version_string(),
        pipewiresrc=f'{factory.get_plugin_name()} {factory.get_plugin().get_version()}',
        connector=connector, node=node, frames=len(times),
        first_mono_ns=times[0] if times else None, last_mono_ns=times[-1] if times else None,
        stopped_by_signal=bool(stopping),
    )
    Path(summary_path).write_text(json.dumps(summary, indent=2) + '\n')
    if not stopping:
        sys.exit('screencast-consumer: the pipeline stopped before a stop signal')


if __name__ == '__main__':
    main()
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `just --set one_cmd 'python3 -m unittest' test-one tools.test_screencast_consumer`
Expected: PASS (9 tests; `WaitForTests` skip where PyGObject is absent, the end-to-end test where `dbus-daemon` or GStreamer is).

- [ ] **Step 5a: Verify the GL path offline on this host**

```bash
env -u WAYLAND_DISPLAY -u DISPLAY GST_GL_PLATFORM=egl GST_GL_WINDOW=surfaceless \
    GST_DEBUG='glcontext:4' gst-launch-1.0 -q videotestsrc num-buffers=3 ! glupload ! glcolorconvert \
    ! gldownload ! videoconvert ! video/x-raw,format=RGB ! fakesink 2>&1 | grep GL_RENDERER
gst-inspect-1.0 glupload | sed -n '/SINK template/,/SRC template/p' | grep -c 'format: DMA_DRM'
```

Expected: a `GL_RENDERER` line naming the GPU (no display server involved), and a nonzero count. The import of niri's own DMA-BUFs is verified in Step 5b.

- [ ] **Step 5b: Probe the import of niri's own DMA-BUFs**

Only a niri on DRM has a GBM device, and the one available outside a TTY session is the owner's desktop. This step is host use and needs the owner's yes at the time it runs: a 5-second cast of the desktop output into a fakesink, no pixels kept, no window opened.

```bash
NODE_PROBE=$(mktemp -d)
python3 - "$NODE_PROBE" <<'PY'
import sys, time
sys.path.insert(0, '.')
import gi
gi.require_version('Gio', '2.0')
from gi.repository import Gio, GLib
from tools.screencast_consumer import BUS_NAME, wait_for
bus = Gio.bus_get_sync(Gio.BusType.SESSION, None)
call = lambda p, i, m, a, r: bus.call_sync(BUS_NAME, p, i, m, a, GLib.VariantType(r), 0, 5000, None).unpack()
(session,) = call('/org/gnome/Mutter/ScreenCast', 'org.gnome.Mutter.ScreenCast', 'CreateSession',
                  GLib.Variant('(a{sv})', ({},)), '(o)')
try:
    (stream,) = call(session, 'org.gnome.Mutter.ScreenCast.Session', 'RecordMonitor',
                     GLib.Variant('(sa{sv})', ('DP-1', {'cursor-mode': GLib.Variant('u', 0)})), '(o)')
    def subscribe(deliver):
        h = bus.signal_subscribe(BUS_NAME, 'org.gnome.Mutter.ScreenCast.Stream', 'PipeWireStreamAdded',
                                 stream, None, 0, lambda *a: deliver(a[-1].unpack()[0]))
        call(session, 'org.gnome.Mutter.ScreenCast.Session', 'Start', None, '()')
        return lambda: bus.signal_unsubscribe(h)
    node = wait_for(subscribe, 10)
    open(sys.argv[1] + '/node', 'w').write(f'{node}\n')
    time.sleep(6)
finally:
    call(session, 'org.gnome.Mutter.ScreenCast.Session', 'Stop', None, '()')
PY
```

Run that in the background, then, within its 6 s, with `N=$(cat "$NODE_PROBE/node")`:

```bash
rc=0
GST_GL_PLATFORM=egl GST_GL_WINDOW=surfaceless timeout 5 gst-launch-1.0 -v \
    pipewiresrc path=$N num-buffers=1 ! 'video/x-raw(memory:DMABuf),format=DMA_DRM' \
    ! glupload ! glcolorconvert ! gldownload ! videoconvert ! video/x-raw,format=RGB ! fakesink \
    > "$NODE_PROBE/gst.log" 2>&1 || rc=$?
echo "gst-launch exit $rc"          # 124 is a timeout: no frame reached the sink
grep -oE 'drm-format=\(string\)[A-Za-z0-9:x]+|format=\(string\)RGB|ERROR.*|not-negotiated' \
    "$NODE_PROBE/gst.log" | sort | uniq -c
```

Expected: `gst-launch exit 0`, which with `num-buffers=1` means a frame was imported, converted and reached EOS; a `drm-format` caps line (niri's fourcc and modifier); an RGB caps line after `gldownload`; no `ERROR` or `not-negotiated`. Caps lines alone are not a pass: any nonzero exit, a timeout (124) included, fails the step. (Already observed once during planning: `XR24` with NVIDIA tiled modifiers, exit 0. Rerun it at execution, since the GStreamer or driver stack may have changed.) Record the negotiated `drm-format` in a task note. A negotiation or import error stops the plan here: note the exact error and return the spec to the owner, since the accepted design depends on this path.

- [ ] **Step 6: Commit**

```bash
git add tools/screencast_consumer.py tools/fake_screencast.py tools/test_screencast_consumer.py
git commit -m "feat(material): screencast consumer samples the next frame after an armed request (material-3acc86)"
```

---

### Task 4: Analyzer checks for edge windows, cast frames and samples

**Files:**
- Modify: `tools/optic_settling.py` (`analyze_case`, after the per-stimulus `messages` loop)
- Test: `tools/test_optic_settling.py` (`RunTests`)

**Interfaces:**
- Consumes: case manifest fields written by Task 5:
  - `edge_in`: list of `[edge_index, stimulus_label]`.
  - `consumer`: `{"frames_in": [labels], "samples": [labels]}`; files in the case dir: `cast-frames.tsv` (one mono ns per line), `cast-summary.json`, and `<label>.raw.json` per sample.
- Produces: `ValueError` messages `"{name}: edge {i} is not inside stimulus {label}"`, `"{name}: no cast frame inside stimulus {label}"`, `"{name}: the screencast consumer did not stop on a signal"`, `"{name}: sample {label} is stale or outside its window"`, and `"missing or invalid {label}.raw.json"` (from `read_json`).

- [ ] **Step 1: Write the failing tests**

Add to `RunTests` in `tools/test_optic_settling.py` (the fixture's `damage` stimulus is journalled at monotonic 1014–1015 s, trace 14–15 s; the resume edge is at trace 20 s, monotonic 1020 s):

```python
    def test_edge_must_fall_inside_its_named_stimulus(self):
        # A second stimulus around the resume edge (trace 20 s); the damage
        # stimulus and its draws at 14.5 s stay where they are. It ends the
        # settled quiet span at 19.5 s (4.5 s after damage), so the declared
        # hold drops to 4 s for this case.
        self.manifest['cases'][0]['hold_ns'] = 4 * S
        self.manifest['cases'][0]['stimuli'].append({'label': 'resume'})
        self.observation['journal'].append({'label': 'resume', 'start_mono_ns': 1019 * S + S // 2,
                                            'end_mono_ns': 1020 * S + S // 2})
        self.manifest['cases'][0]['edge_in'] = [[1, 'damage']]
        self.save()
        self.rejects('edge 1 is not inside stimulus damage')
        self.manifest['cases'][0]['edge_in'] = [[1, 'resume']]
        self.save()
        analyze_run(self.run)
        self.manifest['cases'][0]['edge_in'] = [[2, 'resume']]   # no such edge
        self.save()
        self.rejects('edge 2 is not inside stimulus resume')

    def cast(self, frames, stopped=True, samples=()):
        (self.case / 'cast-frames.tsv').write_text(''.join(f'{t}\n' for t in frames))
        (self.case / 'cast-summary.json').write_text(json.dumps({'stopped_by_signal': stopped}))
        for label, request, frame in samples:
            (self.case / f'{label}.raw.json').write_text(json.dumps(
                {'request_mono_ns': request, 'frame_mono_ns': frame, 'width': 1, 'height': 1}))

    def test_consumer_frames_and_samples(self):
        self.manifest['cases'][0]['consumer'] = {'frames_in': ['damage'], 'samples': ['damage']}
        self.save()
        inside = 1014 * S + S // 2
        self.cast([1013 * S], samples=[('damage', 1014 * S + S // 4, inside)])
        self.rejects('no cast frame inside stimulus damage')
        self.cast([inside], samples=[('damage', 1014 * S + S // 4, inside)])
        analyze_run(self.run)
        self.cast([inside], stopped=False, samples=[('damage', 1014 * S + S // 4, inside)])
        self.rejects('did not stop on a signal')

    def test_rejects_stale_missing_and_late_samples(self):
        self.manifest['cases'][0]['consumer'] = {'frames_in': [], 'samples': ['damage']}
        self.save()
        inside = 1014 * S + S // 2
        for label, request, frame in (
            ('stale', inside, inside),                         # the frame did not follow the request
            ('early request', 1013 * S, inside),               # armed before the stimulus began
            ('late frame', 1014 * S + S // 4, 1016 * S),       # answered after the window
        ):
            with self.subTest(label):
                self.cast([inside], samples=[('damage', request, frame)])
                self.rejects('sample damage is stale or outside its window')
        (self.case / 'damage.raw.json').unlink()
        self.rejects('damage.raw.json')

    def test_a_redraw_between_samples_breaks_the_quiet_interval(self):
        # The settled-segment check is what keeps the gap before sample-3 quiet.
        self.cpu += [['Niri::redraw', 17 * S, 1000]]
        self.write_tables()
        self.rejects('while settled')
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `just --set one_cmd 'python3 -m unittest' test-one tools.test_optic_settling.RunTests`
Expected: the three new consumer/edge tests FAIL (no such checks); `test_a_redraw_between_samples_breaks_the_quiet_interval` already PASSES (it pins existing behavior).

- [ ] **Step 3: Implement the checks**

In `analyze_case`, directly after the `for stimulus, (start, end) in zip(declared, stimuli):` loop (which ends with the `messages` check), add:

```python
    windows = {stimulus['label']: interval for stimulus, interval in zip(declared, stimuli)}

    def window(label):
        if label not in windows:
            raise ValueError(f'{name}: no declared stimulus {label}')
        return windows[label]

    # A resume a stimulus must cause (a VT switch, an unlock) falls inside it.
    for index, label in case.get('edge_in', []):
        start, end = window(label)
        if not index < len(edges) or not start <= edges[index]['trace_ns'] < end:
            raise ValueError(f'{name}: edge {index} is not inside stimulus {label}')

    # A real screencast consumer: frames arrive where damage was caused, and
    # each sample is the first frame after its armed request, inside its window.
    consumer = case.get('consumer')
    if consumer:
        offset = monotonic_offset(edges)
        frames = [int(line) - offset for line in (directory / 'cast-frames.tsv').read_text().split()]
        if read_json(directory / 'cast-summary.json').get('stopped_by_signal') is not True:
            raise ValueError(f'{name}: the screencast consumer did not stop on a signal')
        for label in consumer.get('frames_in', []):
            start, end = window(label)
            if not any(start <= t < end for t in frames):
                raise ValueError(f'{name}: no cast frame inside stimulus {label}')
        for label in consumer.get('samples', []):
            start, end = window(label)
            sidecar = read_json(directory / f'{label}.raw.json')
            request = integer(sidecar.get('request_mono_ns'), 'sample request') - offset
            frame = integer(sidecar.get('frame_mono_ns'), 'sample frame') - offset
            if not start <= request < frame < end:
                raise ValueError(f'{name}: sample {label} is stale or outside its window')
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `just --set one_cmd 'python3 -m unittest' test-one tools.test_optic_settling.RunTests`
Expected: PASS (all `RunTests`).

- [ ] **Step 5: Commit**

```bash
git add tools/optic_settling.py tools/test_optic_settling.py
git commit -m "feat(material): settling analyzer checks edge windows, cast frames and samples (material-3acc86)"
```

---

### Task 5: The dedicated lane in the settling driver

> Later change: `tty-resume`'s single `vt-away` stimulus was split into `vt-out` and `vt-return`, with the resume edge required inside `vt-return`; the spec (`docs/specs/2026-10-02-real-tty-settling-lane-design.md`) is the authority.

**Files:**
- Modify: `docs/materials/scripts/optic-settling-smoke.sh` (header comment; lane refusal stub; `on_exit`; configs; case table; matrix repeats; client builds; host start/stop; `run_case`; new drive/post functions)
- Test: `tools/test_optic_settling.py` (`PrepareTests`, `DriverCleanupTests`, `STUBS`, `SERVE`)

**Interfaces:**
- Consumes: Task 1 `vt-lib.sh` (`vt_record_home`, `vt_spare`, `vt_switch`, `vt_restore`, `VT_HOME`); Task 2 `session-lock-client.c`; Task 3 `tools/screencast_consumer.py` CLI and file contract; Task 4 manifest fields `edge_in`, `consumer`.
- Produces: `--lane dedicated` runs with required `DRM_OUTPUT` (e.g. `DP-1`) and `DRM_MODE` (e.g. `3440x1440@59.999`); artifacts per run `vt.json`, `vt-restore.json`, `probe-rect-drm.txt`, `clients/`; per screencast case `cast-frames.tsv`, `cast-summary.json`, `sample-{1,2,3}.raw[.json]`, `client-{1,2,3}.rgb`, `aurora-{1,2,3}.rgb`.

- [ ] **Step 1: Write the failing prepare test**

Add to `PrepareTests`:

```python
    def test_prepare_the_dedicated_lane(self):
        root = Path(__file__).resolve().parents[1]
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            bin_dir = base / 'bin'; bin_dir.mkdir()
            niri = bin_dir / 'niri'
            niri.write_text('#!/bin/sh\n[ "$1" = validate ]\n')
            magick = bin_dir / 'magick'
            magick.write_text('#!/bin/sh\nfor arg do :; done\nprintf image > "$arg"\n')
            niri.chmod(0o755); magick.chmod(0o755)
            source = subprocess.check_output(['git', '-C', str(root), 'rev-parse', 'HEAD'], text=True).strip()
            identity = dict(source_commit=source, features=['profile-with-tracy'],
                            binary_sha256=hashlib.sha256(niri.read_bytes()).hexdigest())
            Path(str(niri) + '.identity.json').write_text(json.dumps(identity))
            env = dict(os.environ, OUT=str(base / 'out'), NIRI_BIN=str(niri),
                       NIRI_MATERIAL_WORK_ROOT=str(base), XDG_RUNTIME_DIR=str(base),
                       CAPTURE_META='/bin/true', DRM_OUTPUT='DP-1', DRM_MODE='3440x1440@59.999',
                       PATH=f'{bin_dir}:{os.environ["PATH"]}')
            script = root / 'docs/materials/scripts/optic-settling-smoke.sh'
            run = subprocess.run(['bash', str(script), 'prepare', '--lane', 'dedicated'], env=env,
                                 capture_output=True, text=True)
            self.assertEqual(run.returncode, 0, run.stderr)
            manifest = json.loads((base / 'out/manifest.json').read_text())
            by_name = {case['name']: case for case in manifest['cases']}
            dedicated = sorted(name for name, case in by_name.items() if case['lane'] == 'dedicated')
            self.assertEqual(dedicated, ['drm-aurora', 'screencast', 'tty-resume', 'unlock'])
            self.assertEqual(by_name['tty-resume']['edge_in'], [[1, 'vt-away']])
            self.assertEqual(by_name['unlock']['edge_in'], [[1, 'unlock']])
            self.assertEqual(by_name['screencast']['consumer'],
                             {'frames_in': ['sample-1', 'sample-2', 'sample-3'],
                              'samples': ['sample-1', 'sample-2', 'sample-3']})
            for name in dedicated:
                self.assertIn({'label': 'collect', 'min_redraws': 1}, by_name[name]['stimuli'], name)
                kdl = (base / 'out' / name / 'case.kdl').read_text()
                self.assertIn('output "DP-1" { mode "3440x1440@59.999"; scale 1; }', kdl)
                self.assertIn('dbus-interfaces-in-non-session-instances', kdl)
            self.assertFalse((base / 'out/aurora-full').exists())      # headless configs not written
            missing = subprocess.run(['bash', str(script), 'prepare', '--lane', 'dedicated'],
                                     env=dict(env, OUT=str(base / 'out2'), DRM_MODE=''),
                                     capture_output=True, text=True)
            self.assertNotEqual(missing.returncode, 0)
            self.assertIn('DRM_MODE', missing.stderr)
```

Also extend the existing `test_prepare_validates_inventory_and_cleans_runtime`: replace its two screencast assertions with

```python
            self.assertEqual((by_name['screencast']['lane'], by_name['screencast']['required']),
                             ('dedicated', False))
```

- [ ] **Step 2: Run it to verify it fails**

Run: `just --set one_cmd 'python3 -m unittest' test-one tools.test_optic_settling.PrepareTests`
Expected: FAIL (`dedicated capture requires a real TTY fixture` for the dedicated prepare; the screencast lane assertion).

- [ ] **Step 3: Lane setup, prerequisites and the VT library**

In the driver header comment, replace the sentence about cases outside the lane with:

```bash
# The dedicated lane (--lane dedicated; docs/specs/2026-10-02-real-tty-settling-lane-design.md)
# runs niri on DRM from a TTY with the desktop stopped: drm-aurora, tty-resume,
# unlock and screencast. It needs DRM_OUTPUT and DRM_MODE, switches VTs through
# vt-lib.sh, and restores the starting VT on every exit. A second output stays
# unverified, as does every case outside the lane being run.
```

Replace the refusal stub

```bash
[ "$MODE" = prepare ] || [ "$LANE" = headless ] \
    || fail 'dedicated capture requires a real TTY fixture; no TTY capture is implemented'
```

with

```bash
. "$HERE/vt-lib.sh"
if [ "$LANE" = dedicated ]; then
    : "${DRM_OUTPUT:?physical output name, e.g. DP-1}" "${DRM_MODE:?fixed WIDTHxHEIGHT@REFRESH}"
    [ -n "$DRM_OUTPUT" ] && [ -n "$DRM_MODE" ] || fail 'DRM_OUTPUT and DRM_MODE must be non-empty'
fi
# Host items are owner actions (docs/materials/capture-host-setup.md); refuse naming the missing one.
dedicated_prerequisites() {
    [ -z "${WAYLAND_DISPLAY:-}" ] || fail 'the dedicated lane runs from a TTY login, not inside a Wayland session'
    ! pgrep -x niri > /dev/null || fail 'a niri is already running: stop the desktop session first'
    sudo -n -l /usr/bin/chvt > /dev/null 2>&1 \
        || fail 'no NOPASSWD rule for /usr/bin/chvt (docs/materials/capture-host-setup.md)'
    gst-inspect-1.0 pipewiresrc > /dev/null 2>&1 || fail 'no pipewiresrc element: install gst-plugin-pipewire'
}
```

Note: `${DRM_MODE:?}` already fails on an empty value; the explicit line documents it.

After the `OPTIC_SETTLING_STUB_TOOLS` / `CAPTURE_META` check, add the stub-only overrides the cleanup tests use to reach casting, a held lock and a switched VT within seconds:

```bash
# Offline stub runs only: a time scale for the drive schedule and stand-ins
# for the consumer and lock client. A recorded run never uses them.
TIMESCALE=${OPTIC_SETTLING_STUB_TIMESCALE:-1}
CAST_CONSUMER=${OPTIC_SETTLING_STUB_CONSUMER:-$ROOT/tools/screencast_consumer.py}
if [ -z "${OPTIC_SETTLING_STUB_TOOLS:-}" ]; then
    [ "$TIMESCALE" = 1 ] && [ -z "${OPTIC_SETTLING_STUB_CONSUMER:-}${OPTIC_SETTLING_STUB_LOCK:-}" ] \
        || fail 'stub timescale, consumer and lock client need OPTIC_SETTLING_STUB_TOOLS'
fi
```

In `at()`, scale the target: pass `"$TIMESCALE"` as a third argument to its Python and read
`t0, s = int(sys.argv[1]), float(sys.argv[2]) * float(sys.argv[3])`.

Directly after `[ "$MODE" = prepare ] || capture_preflight "$LANE"`, add:

```bash
if [ "$MODE" != prepare ] && [ "$LANE" = dedicated ]; then
    dedicated_prerequisites
    vt_record_home
    VT_SPARE=$(vt_spare) || fail "no spare VT without a logind session"
    printf '{"home": %s, "spare": %s}\n' "$VT_HOME" "$VT_SPARE" > "$OUT/vt.json"
fi
```

- [ ] **Step 4: Ownership and restoration in `on_exit`**

Extend the owned-process variables and `on_exit`:

```bash
PROBE_PID=; OTHER_PID=; EXPORT_PID=; BG_PID=; SLEEP_PID=; INHIBIT_PID=; BUS_PID=; CAST_PID=; LOCK_PID=
```

In `on_exit`, add `"$CAST_PID" "$LOCK_PID"` before `"$NIRI_PID"` and `"$BUS_PID"` after it in the reap loop, and after `remove_runtime_dir || rc=1` add:

```bash
    vt_restore "$OUT/vt-restore.json" || rc=1
```

so restoration runs after every owned process is gone and before `write_sums` and `capture_meta release`.

- [ ] **Step 5: Configs and the case table**

After `SIGNAL5=...`, add:

```bash
DRM_TOP=
[ "$LANE" != dedicated ] || DRM_TOP="output \"$DRM_OUTPUT\" { mode \"$DRM_MODE\"; scale 1; }
debug { dbus-interfaces-in-non-session-instances; }"
```

In `write_case_configs`, add a branch before the outside-lane line, and drop the four names from that line:

```bash
        drm-aurora|tty-resume|unlock|screencast) case_config "$2" "$AURORA" "$SIGNAL5
$DRM_TOP" ;;
        output-removal) : ;;   # outside every lane this driver runs
```

In the manifest heredoc, give `case()` a passthrough and add/replace the dedicated cases:

```python
def case(name, family, edges, segments, stimuli=(), pixels=(), capture_s=30, lane='headless', why=None, **extra):
    return dict(name=name, family=family, lane=lane, edges=edges, segments=segments,
                stimuli=list(stimuli), pixels=list(pixels), capture_s=capture_s, why=why, **extra)
```

Replace the `screencast`, `tty-resume` and `unlock` entries with:

```python
    # Dedicated lane: niri on DRM from a TTY. drm-aurora is the lane's cadence control.
    case('drm-aurora', 'active-idle-resume', [0, 1], [A4, HELD, A4], pixels=held_pair, lane='dedicated'),
    case('tty-resume', 'session-activation', [0, 1, 0], [A4, HELD, A4, HELD],
         [dict(label='vt-away', min_redraws=1)], capture_s=45, lane='dedicated',
         edge_in=[[1, 'vt-away']]),
    case('unlock', 'session-activation', [0, 1, 0], [A4, HELD, A4, HELD],
         [dict(label='lock', min_redraws=1), dict(label='unlock', min_redraws=1)], capture_s=45,
         lane='dedicated', edge_in=[[1, 'unlock']]),
    # Each sample arms first, then causes bounded damage: niri sends no cast
    # frame without damage. The thief's lines stay outside both crops.
    case('screencast', 'session-activation', [0], [A4, HELD],
         [dict(label='cast-start'), dict(label='sample-1', min_redraws=1),
          dict(label='sample-2', min_redraws=1, min_draws=1), dict(label='sample-3', min_redraws=1),
          dict(label='cast-stop')],
         [dict(before='client-1.rgb', after='client-2.rgb', expect='different'),
          dict(before='client-2.rgb', after='client-3.rgb', expect='equal'),
          dict(before='aurora-1.rgb', after='aurora-2.rgb', expect='equal'),
          dict(before='aurora-2.rgb', after='aurora-3.rgb', expect='equal')],
         capture_s=45, lane='dedicated',
         consumer=dict(frames_in=['sample-1', 'sample-2', 'sample-3'],
                       samples=['sample-1', 'sample-2', 'sample-3'])),
```

Make the `collect` stimulus apply to every runnable lane: change `if item['lane'] == 'headless' and item['edges']:` to `if item['lane'] in ('headless', 'dedicated') and item['edges']:`.

Replace the matrix repetition block so repeats follow the run's lane:

```python
if mode == 'matrix':
    # Three cycles of each repeated case in this lane; the headless lane also
    # holds aurora-full for 600 s once.
    extra = []
    for base in ('aurora-full', 'aurora-reduced', 'tty-resume', 'unlock'):
        template = next(c for c in cases if c['name'] == base)
        if template['lane'] == lane:
            extra += [dict(template, name=f'{base}-r{k}') for k in (2, 3)]
    cases += extra
    if lane == 'headless':
        first = next(c for c in cases if c['name'] == 'aurora-full')
        first['hold_ns'] = 600 * S
        first['capture_s'] = 30 + 600 - 5   # resume at 621 s: 603 s settled
```

- [ ] **Step 6: Run the prepare tests**

Run: `just --set one_cmd 'python3 -m unittest' test-one tools.test_optic_settling.PrepareTests`
Expected: PASS.

- [ ] **Step 7: Client builds (shared with the idle inhibitor)**

Replace `build_inhibit_client` and its `IDENTITY_EXTRA` block with:

```bash
# Per-run Wayland clients, built against the installed protocols and
# identified with their sources like the binary.
IDENTITY_EXTRA=()
PROTOCOLS=/usr/share/wayland-protocols
build_client() {   # name, protocol XML...: docs/materials/scripts/<name>.c into $OUT/clients
    local name=$1 dir=$OUT/clients xml base sources=()
    shift
    mkdir -p "$dir"
    for xml in "$@"; do
        base=$(basename "$xml" .xml)
        wayland-scanner client-header "$xml" "$dir/$base-client-protocol.h"
        wayland-scanner private-code "$xml" "$dir/$base-protocol.c"
        sources+=("$dir/$base-protocol.c")
    done
    cc -std=c11 -Wall -Wextra -Werror -O2 -I"$dir" -o "$dir/$name" "$HERE/$name.c" "${sources[@]}" \
        -lwayland-client > "$dir/$name.build.log" 2>&1 || fail "$name did not build; see $dir/$name.build.log"
    IDENTITY_EXTRA+=(--binary "$dir/$name" --input "$HERE/$name.c")
}
selected() { printf '%s\n' "${RUN_CASES[@]}" | grep -qE "^$1(-r[0-9]+)?$"; }
INHIBIT_BIN=$OUT/clients/idle-inhibit-client
LOCK_BIN=$OUT/clients/session-lock-client
if selected idle-inhibitor; then
    build_client idle-inhibit-client "$PROTOCOLS/stable/xdg-shell/xdg-shell.xml" \
        "$PROTOCOLS/unstable/idle-inhibit/idle-inhibit-unstable-v1.xml"
fi
if [ -n "${OPTIC_SETTLING_STUB_LOCK:-}" ]; then
    LOCK_BIN=$OPTIC_SETTLING_STUB_LOCK
elif selected unlock; then
    build_client session-lock-client "$PROTOCOLS/staging/ext-session-lock/ext-session-lock-v1.xml"
fi
if selected screencast; then IDENTITY_EXTRA+=(--input "$ROOT/tools/screencast_consumer.py"); fi
capture_identity --config threshold-ms=5000 --config cases="${RUN_CASES[*]}" --config lane="$LANE" \
    ${DRM_OUTPUT:+--config drm-output="$DRM_OUTPUT"} ${DRM_MODE:+--config drm-mode="$DRM_MODE"} \
    "${IDENTITY_EXTRA[@]}"
```

- [ ] **Step 8: Host start/stop and calibration**

Add after the client builds:

```bash
# niri on DRM from this TTY (as idle-budget's start_drm), with a private
# session bus so no compositor interface reaches the user's bus.
start_drm() {   # $1 niri, $2 config, $3 sub-run name
    settle_before_launch "$2" "${3-}"
    "$1" validate -c "$2" || fail "config $2 does not validate with $1"
    dbus-daemon --session --nofork --address="unix:path=$RT/bus" >> "$OUT/dbus.log" 2>&1 &
    BUS_PID=$!
    for _ in $(seq 50); do [ -S "$RT/bus" ] && break; sleep 0.1; done
    [ -S "$RT/bus" ] || fail "no private session bus (see $OUT/dbus.log)"
    env -u WAYLAND_DISPLAY -u WAYLAND_SOCKET -u DISPLAY -u NIRI_SOCKET \
        XDG_RUNTIME_DIR="$RT" DBUS_SESSION_BUS_ADDRESS="unix:path=$RT/bus" \
        PIPEWIRE_RUNTIME_DIR="$XDG_RUNTIME_DIR" "$1" -c "$2" >> "$OUT/niri.log" 2>&1 &
    NIRI_PID=$!
    for _ in $(seq 100); do
        compgen -G "$RT/niri.*.sock" > /dev/null && break
        kill -0 "$NIRI_PID" 2>/dev/null || fail 'DRM niri exited (see niri.log)'
        sleep 0.1
    done
    NIRI_SOCKET=$(ls "$RT"/niri.*.sock) || fail 'no DRM niri socket'
    export NIRI_SOCKET
    # Stub runs (offline tests) launch a script, not the snapshot.
    [ -n "${OPTIC_SETTLING_STUB_TOOLS:-}" ] || cmp -s "$1" "/proc/$NIRI_PID/exe" \
        || fail 'running executable differs from the snapshot'
    sleep 1
}
stop_drm() {
    kill "$NIRI_PID" 2>/dev/null || true; wait "$NIRI_PID" 2>/dev/null || true; NIRI_PID=
    [ -z "$BUS_PID" ] || reap "$BUS_PID"
    BUS_PID=
    rm -f "$RT"/niri.*.sock "$RT/bus"; sleep 0.5
}
start_host() { if [ "$LANE" = dedicated ]; then start_drm "$@"; else start_nested "$@"; fi; }
stop_host() { if [ "$LANE" = dedicated ]; then stop_drm; else stop_nested; fi; }
topology() {
    if [ "$LANE" = dedicated ]; then msg "$NIRI" -j outputs | jq -c 'keys'; else echo '["headless-1"]'; fi
}
# IPC gives no on-screen position for tiled windows, so the screencast crops
# come from an opaque geometry probe in the same two-window layout.
calibrate_drm_probe() {
    write_geometry_config "$OUT/geometry-drm.kdl"
    printf '%s\n' "$DRM_TOP" >> "$OUT/geometry-drm.kdl"
    start_host "$NIRI" "$OUT/geometry-drm.kdl" geometry-drm
    spawn_geometry_probe "$NIRI"
    steal_focus "$NIRI"
    sleep 2
    shot "$NIRI" geometry-drm
    measure_rect "$OUT/geometry-drm.png" > "$OUT/probe-rect-drm.txt"
    stop_host
    await_gpu_rest
}
```

In `run_case`, replace `start_nested "$NIRI" "$CASE_DIR/live.kdl" "$CASE"` with `start_host "$NIRI" "$CASE_DIR/live.kdl" "$CASE"`, `stop_nested` with `stop_host`, extend the per-case reap list to `"$BG_PID" "$PROBE_PID" "$OTHER_PID" "$INHIBIT_PID" "$CAST_PID" "$LOCK_PID"` and reset `CAST_PID=; LOCK_PID=` with the others, and write the observation's topology from the host: change the observation call to `python3 - "$CASE_DIR" "$T0" "$(topology)" <<'PY'` and in its body use `topology=json.loads(sys.argv[3])`. Call `topology` before `stop_host`.

Before the case loop (`for name in "${RUN_CASES[@]}"`), add:

```bash
if [ "$LANE" = dedicated ] && selected screencast; then calibrate_drm_probe; fi
```

- [ ] **Step 9: The focus thief reads a FIFO in the screencast case**

In `run_case`, replace the focus-thief launch with:

```bash
    local other_cmd=$IDLE
    OTHER_FIFO=
    if [ "$base" = screencast ]; then
        OTHER_FIFO=$RT/other-$CASE.fifo
        mkfifo "$OTHER_FIFO"
        other_cmd="printf '\033[?25l'; exec 3<>'$OTHER_FIFO'; while read -r line <&3; do printf '%s\n' \"\$line\"; done"
    fi
    XDG_RUNTIME_DIR=$RT WAYLAND_DISPLAY=$DISPLAY_NAME \
        kitty --config NONE --class gos-other -o cursor_blink_interval=0 -o mouse_hide_wait=0 sh -c "$other_cmd" >> "$OUT/other.log" 2>&1 &
```

and add `[ -z "$OTHER_FIFO" ] || rm -f "$OTHER_FIFO"` next to `rm -f "$FIFO"`.

- [ ] **Step 10: Drive and post functions**

Add after `drive_idle_inhibitor`:

```bash
drive_drm_aurora() { drive_cycle; }
post_drm_aurora() { post_held; }
# Out to the spare VT and back: niri's session pauses, then activation calls
# notify_activity. Both switches are verified; the window stays under 6 s.
vt_away() {
    vt_switch "$VT_SPARE" || fail "$CASE: did not reach spare VT $VT_SPARE"
    nap 3   # backgrounded, so TERM while away is handled at once
    vt_switch "$VT_HOME" || fail "$CASE: the return to VT $VT_HOME did not land"
}
drive_tty_resume() { keepalive; at 26; stim vt-away 1.5 vt_away; }
lock_start() {
    XDG_RUNTIME_DIR=$RT WAYLAND_DISPLAY=$DISPLAY_NAME "$LOCK_BIN" > "$CASE_DIR/lock.out" 2>> "$OUT/lock.log" &
    LOCK_PID=$!
    for _ in $(seq 50); do grep -qx locked "$CASE_DIR/lock.out" && return; alive "$LOCK_PID" || break; sleep 0.1; done
    fail "$CASE: the session never locked (see lock.log)"
}
unlock_now() {
    local rc=0
    kill -USR1 "$LOCK_PID"
    wait "$LOCK_PID" || rc=$?
    LOCK_PID=
    [ "$rc" -eq 0 ] || fail "$CASE: the lock client exited $rc"
}
drive_unlock() { keepalive; at 26; stim lock 2 lock_start; at 30; stim unlock 1.5 unlock_now; }
cast_start() {
    : > "$CASE_DIR/cast.out"
    DBUS_SESSION_BUS_ADDRESS="unix:path=$RT/bus" python3 "$CAST_CONSUMER" \
        "$DRM_OUTPUT" "$CASE_DIR/cast-frames.tsv" "$CASE_DIR/cast-summary.json" "$CASE_DIR/sample-request" \
        > "$CASE_DIR/cast.out" 2>> "$OUT/cast.log" &
    CAST_PID=$!
    for _ in $(seq 100); do grep -q '^ready ' "$CASE_DIR/cast.out" && return; alive "$CAST_PID" || break; sleep 0.1; done
    fail "$CASE: the screencast consumer never became ready (see cast.log)"
}
cast_stop() {
    local rc=0
    kill -TERM "$CAST_PID"
    wait "$CAST_PID" || rc=$?
    CAST_PID=
    [ "$rc" -eq 0 ] || fail "$CASE: the screencast consumer exited $rc (see cast.log)"
}
thief_line() { echo "thief $1" > "$OTHER_FIFO"; }
# Arm a sample, cause its damage, wait for the frame that damage produced.
# A request no frame answers fails the case; it is never retried.
cast_sample() {   # label, damage command...
    local label=$1 target=$CASE_DIR/$1.raw
    shift
    printf '%s\n' "$target" > "$CASE_DIR/sample-request"
    kill -USR1 "$CAST_PID"
    for _ in $(seq 20); do [ -e "$target.armed" ] && break; sleep 0.1; done
    [ -e "$target.armed" ] || fail "$CASE: $label was never armed"
    "$@"
    for _ in $(seq 30); do [ -e "$target.json" ] && return; sleep 0.1; done
    fail "$CASE: no cast frame answered $label"
}
# sample-2 to sample-3 is the held interval observed while casting (> 5 s).
drive_screencast() {
    keepalive
    at 20; stim cast-start 1 cast_start
    at 24; stim sample-1 1 cast_sample sample-1 thief_line 1
    at 28; stim sample-2 1 cast_sample sample-2 print_line 1
    at 36; stim sample-3 1 cast_sample sample-3 thief_line 2
    at 40; stim cast-stop 1 cast_stop
}
# Crops in output pixels (scale 1): the probe's top text rows, and glass
# below its few lines with Aurora alone.
post_screencast() {
    local k w h px py pw ph
    read -r px py pw ph < "$OUT/probe-rect-drm.txt"
    for k in 1 2 3; do
        w=$(jq -r .width "$CASE_DIR/sample-$k.raw.json"); h=$(jq -r .height "$CASE_DIR/sample-$k.raw.json")
        magick -size "${w}x${h}" -depth 8 "rgb:$CASE_DIR/sample-$k.raw" \
            -crop "$((pw - 20))x80+$((px + 10))+$((py + 10))" +repage -depth 8 rgb:- > "$CASE_DIR/client-$k.rgb"
        magick -size "${w}x${h}" -depth 8 "rgb:$CASE_DIR/sample-$k.raw" \
            -crop "400x200+$((px + 40))+$((py + ph - 240))" +repage -depth 8 rgb:- > "$CASE_DIR/aurora-$k.rgb"
    done
}
```

- [ ] **Step 11: Write the failing cleanup test for the dedicated lane**

In `tools/test_optic_settling.py`, extend `SERVE` so a `dbus-daemon` stub binds the path from `--address=unix:path=...`:

```python
if sys.argv[1] == 'weston':
    names = [arg.split('=', 1)[1] for arg in sys.argv[2:] if arg.startswith('--socket=')]
elif sys.argv[1] == 'dbus-daemon':
    names = [arg.split('path=', 1)[1] for arg in sys.argv[2:] if arg.startswith('--address=')]
else:
    names = ['stub-1', f'niri.stub-1.{os.getpid()}.sock']
```

Add stubs to `STUBS`:

```python
    'bin/dbus-daemon': 'echo "dbus-daemon $$" >> "$STUB_DIR/pids"\nexec python3 "$STUB_DIR/serve.py" dbus-daemon "$@"\n',
    'bin/sudo': 'exit 0\n',
    'bin/pgrep': 'exit 1\n',
    'bin/gst-inspect-1.0': 'exit 0\n',
    'vt/chvt': ('printf "tty%s\\n" "$1" > "$VT_ACTIVE_FILE"\necho "chvt $1" >> "$STUB_DIR/meta.log"\n'
                '[ "$1" = 1 ] || : > "$STUB_DIR/away"\n'),
    'vt/loginctl': 'exit 0\n',
    'tools/lock': ('echo "lock $$" >> "$STUB_DIR/pids"\ntrap \'exit 0\' USR1\necho locked\n'
                   ': > "$STUB_DIR/locked"\nwhile :; do sleep 0.1; done\n'),
```

Change the `bin/magick` stub so calibration can measure a rectangle, and let the `bin/niri` stub write screenshots:

```python
    'bin/magick': 'case "$*" in *info:*) echo 100x100+10+10; exit 0 ;; esac\nfor arg do :; done\nprintf image > "$arg"\n',
```

and in the `bin/niri` stub's `msg)` branch, before `exit 0`, add
`for a do case $prev in --path) printf png > "$a" ;; esac; prev=$a; done; `.

Add a stub consumer next to `SERVE`:

```python
CONSUMER = """
import json, os, pathlib, signal, sys, time
stub = pathlib.Path(os.environ['STUB_DIR'])
with open(stub / 'pids', 'a') as pids:
    pids.write(f'consumer {os.getpid()}\\n')
def stop(*_):
    pathlib.Path(sys.argv[3]).write_text(json.dumps({'stopped_by_signal': True}))
    sys.exit(0)
signal.signal(signal.SIGTERM, stop)
print('ready 1', flush=True)
(stub / 'cast-ready').touch()
while True:
    time.sleep(0.1)
"""
```

written in `setUp` with `(self.stubs / 'tools/consumer.py').write_text(CONSUMER)`.

Add to `DriverCleanupTests`:

```python
    def dedicated(self, case, marker):
        """TERM a stub dedicated run once `marker` appears; everything is reaped."""
        active = self.base / 'active'
        active.write_text('tty1\n')
        driver = self.start(case, STUB_CAPTURE_S='600', DRM_OUTPUT='DP-1', DRM_MODE='3440x1440@59.999',
                            VT_ACTIVE_FILE=str(active), VT_CHVT=str(self.stubs / 'vt/chvt'),
                            VT_LOGINCTL=str(self.stubs / 'vt/loginctl'),
                            OPTIC_SETTLING_STUB_TIMESCALE='0.05',
                            OPTIC_SETTLING_STUB_CONSUMER=str(self.stubs / 'tools/consumer.py'),
                            OPTIC_SETTLING_STUB_LOCK=str(self.stubs / 'tools/lock'),
                            LANE_ARGS='--lane dedicated')
        self.await_marker(driver, marker, 60)
        elapsed, stderr = self.terminate(driver)
        self.assertEqual(driver.returncode, 143, stderr)
        self.assertLess(elapsed, self.BOUND_S, stderr)
        self.assert_cleaned_up(case)
        return active

    def started(self, name):
        return [line for line in (self.stubs / 'pids').read_text().splitlines() if line.startswith(name)]

    def test_term_while_casting_reaps_the_consumer_and_the_bus(self):
        self.dedicated('screencast', 'cast-ready')
        self.assertTrue(self.started('consumer'))
        self.assertTrue(self.started('dbus-daemon'))

    def test_term_with_the_session_locked_reaps_the_lock_client(self):
        self.dedicated('unlock', 'locked')
        self.assertTrue(self.started('lock'))

    def test_term_while_switched_away_restores_the_vt_before_release(self):
        active = self.dedicated('tty-resume', 'away')
        self.assertEqual(active.read_text(), 'tty1\n')
        record = json.loads((self.out / 'vt-restore.json').read_text())
        self.assertEqual((record['outcome'], record['home'], record['from']), ('restored', 1, 2))
        meta = (self.stubs / 'meta.log').read_text().splitlines()
        self.assertLess(meta.index('chvt 1'), max(i for i, line in enumerate(meta) if line.startswith('release ')))

    def test_stub_overrides_need_stub_tools(self):
        env = dict(self.env, OPTIC_SETTLING_STUB_TIMESCALE='0.05')
        env.pop('OPTIC_SETTLING_STUB_TOOLS')
        script = self.root / 'docs/materials/scripts/optic-settling-smoke.sh'
        run = subprocess.run(['bash', str(script), 'pilot'], cwd=self.root, env=env,
                             capture_output=True, text=True, timeout=60)
        self.assertNotEqual(run.returncode, 0)
        self.assertIn('need OPTIC_SETTLING_STUB_TOOLS', run.stderr)

    def test_term_during_a_dedicated_capture_reaps_the_bus_and_records_the_vt(self):
        active = self.base / 'active'
        active.write_text('tty1\n')
        driver = self.start('drm-aurora', STUB_CAPTURE_S='600', DRM_OUTPUT='DP-1',
                            DRM_MODE='3440x1440@59.999', VT_ACTIVE_FILE=str(active),
                            VT_CHVT=str(self.stubs / 'vt/chvt'), VT_LOGINCTL=str(self.stubs / 'vt/loginctl'),
                            LANE_ARGS='--lane dedicated')
        self.await_marker(driver, 'capture-started', 60)
        time.sleep(0.5)
        elapsed, stderr = self.terminate(driver)
        self.assertEqual(driver.returncode, 143, stderr)
        self.assertLess(elapsed, self.BOUND_S, stderr)
        self.assertIn('dbus-daemon', (self.stubs / 'pids').read_text())
        self.assert_cleaned_up('drm-aurora')
        self.assertEqual(json.loads((self.out / 'vt-restore.json').read_text())['outcome'], 'not-needed')
        self.assertEqual(json.loads((self.out / 'vt.json').read_text()), {'home': 1, 'spare': 2})
```

    def test_dedicated_prerequisites_name_the_missing_item(self):
        (self.stubs / 'bin/sudo').write_text('#!/bin/sh\nexit 1\n')
        driver = self.start('drm-aurora', DRM_OUTPUT='DP-1', DRM_MODE='3440x1440@59.999',
                            VT_ACTIVE_FILE=str(self.base / 'active'), LANE_ARGS='--lane dedicated')
        _, stderr = driver.communicate(timeout=60)
        self.assertEqual(driver.returncode, 1)
        self.assertIn('no NOPASSWD rule for /usr/bin/chvt', stderr)
        self.assertEqual(self.pids(), [])                     # nothing was launched
        self.assertEqual(list(self.runtime.glob('gos.*')), [])

and make `start` pass the lane through:

```python
    def start(self, case, **env):
        script = self.root / 'docs/materials/scripts/optic-settling-smoke.sh'
        lane = env.pop('LANE_ARGS', '').split()
        return subprocess.Popen(['bash', str(script), 'pilot', *lane], cwd=self.root, start_new_session=True,
                                env=dict(self.env, CASES=case, **env),
                                stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
```

- [ ] **Step 12: Run the driver tests**

Run: `just --set one_cmd 'python3 -m unittest' test-one tools.test_optic_settling.DriverCleanupTests tools.test_optic_settling.PrepareTests`
Expected: PASS.

- [ ] **Step 13: Run every tooling test and the shell syntax check**

Run: `bash -n docs/materials/scripts/optic-settling-smoke.sh && just --set fast_cmd 'python3 -m unittest discover -s tools 2>&1' test-fast`
Expected: all tooling tests pass.

- [ ] **Step 14: Commit**

```bash
git add docs/materials/scripts/optic-settling-smoke.sh tools/test_optic_settling.py
git commit -m "feat(material): dedicated real-TTY lane in the optic-settling driver (material-f7eb0b, material-3acc86)"
```

---

### Task 6: Documentation and the quiet-session dimming protocol

**Files:**
- Modify: `docs/specs/2026-10-02-real-tty-settling-lane-design.md` (§5 crop geometry sentence)
- Modify: `docs/materials/2026-09-30-optic-settling-evidence.md` (Unverified section)
- Modify: `AGENTS.md` (quiet-session bullet)
- Modify: `docs/materials/capture-host-setup.md` (lane run command)

**Interfaces:**
- Consumes: Task 5's command line (`DRM_OUTPUT`, `DRM_MODE`, `--lane dedicated`); `ops-a1715a`'s `display-dim` (may not exist yet).
- Produces: documented run command and protocol used by Task 7.

- [ ] **Step 1: Correct the spec's crop sentence**

In §5, replace "computed from the window geometry niri reports:" with:

```markdown
calibrated once per run from an opaque geometry probe in the same
two-window layout (niri's IPC reports no on-screen position for tiled
windows), as the headless lanes' `calibrate_probe_rect` does:
```

In §4's screencast-consumer paragraph, replace the sentence beginning "If niri's stream offers no CPU-mappable buffers" with:

```markdown
niri offers DMA-BUFs only, so the consumer imports whatever modifier niri
fixates through GStreamer GL on headless EGL (`glupload ! glcolorconvert !
gldownload`) and converts to RGB; a probe on 2026-10-02 imported niri's
NVIDIA-tiled `XR24` buffers this way. The development check (§7) proves
sampling works before the pilot.
```

and its pipeline to `pipewiresrc ! glupload ! glcolorconvert ! gldownload ! videoconvert ! video/x-raw,format=RGB ! appsink`.

- [ ] **Step 2: Evidence document**

Under "### Unverified" in `docs/materials/2026-09-30-optic-settling-evidence.md`, append:

```markdown
The dedicated real-TTY lane is implemented
([design](../specs/2026-10-02-real-tty-settling-lane-design.md)): `drm-aurora`,
`tty-resume`, `unlock` and `screencast` run on DP-1 from a TTY through
`optic-settling-smoke.sh --lane dedicated`. They stay unverified until its
development check, pilot and matrix pass.
```

- [ ] **Step 3: Quiet-session protocol in `AGENTS.md`**

After the sentence ending "give the commands and known pitfalls." add:

```markdown
  A TTY has no display sleep: when ops's `display-dim` is installed
  (`ops-a1715a`), run `display-dim status` first (a stale record means an
  earlier session did not restore; restore it), `display-dim set 0.25`
  before the first run and `display-dim restore` after the last, before the
  final report. Dimming never gates a run; a monitor that rejects DDC stays
  undimmed and the report says so.
```

- [ ] **Step 4: Run command in the setup record**

Append to `docs/materials/capture-host-setup.md`:

````markdown
## Running the dedicated lane

From a TTY login with the desktop stopped, in the execution worktree, with
an identified Tracy snapshot (`niri-tracy` plus its `.identity.json`):

```sh
OUT=$NIRI_MATERIAL_WORK_ROOT/optic-settling/tty-dev-$(date +%Y%m%d)-1 \
  CASES='drm-aurora tty-resume screencast' CAPTURE_TASK=material-f7eb0b \
  NIRI_BIN=<snapshot>/niri-tracy DRM_OUTPUT=DP-1 DRM_MODE=3440x1440@59.999 \
  docs/materials/scripts/optic-settling-smoke.sh pilot --lane dedicated
```

Drop `CASES` for the pilot; the matrix adds `PILOT_DIR` and uses `matrix`.
````

- [ ] **Step 5: Check and commit**

Run: `tasks check` (no output expected).

```bash
git add AGENTS.md docs/specs/2026-10-02-real-tty-settling-lane-design.md \
    docs/materials/2026-09-30-optic-settling-evidence.md docs/materials/capture-host-setup.md
git commit -m "docs(material): dedicated lane run command and quiet-session dimming (material-f7eb0b)"
```

---

### Task 7: Development check, pilot and matrix on the TTY

This task needs the host from a TTY with the desktop stopped: it is parked
`tasks park <id> "<steps>" --reason quiet --waiting-on user --needs headless --minutes 30`
until the owner hands the host over. Every attempt ends with a `run:` note.

**Files:**
- Modify: `docs/materials/2026-09-30-optic-settling-evidence.md`

**Interfaces:**
- Consumes: Tasks 1–6; an identified Tracy snapshot at this worktree's HEAD.
- Produces: `analysis.json` verdicts for the development run, pilot and matrix; the evidence document's dedicated-lane table.

- [ ] **Step 1: Dim, if available**

Run `display-dim status` and `display-dim set 0.25` when `ops-a1715a` has landed; otherwise note "undimmed: display-dim not installed".

- [ ] **Step 2: Build and snapshot the binary at HEAD**

```bash
cargo build --release --features profile-with-tracy
H=$(git rev-parse --short HEAD); D=$NIRI_MATERIAL_WORK_ROOT/optic-settling/bin-$H
mkdir -p "$D"
cp "$(cargo metadata --format-version 1 --no-deps | jq -r .target_directory)/release/niri" "$D/niri-tracy"
python3 - "$D/niri-tracy" "$(git rev-parse HEAD)" <<'PY'
import hashlib, json, pathlib, sys
binary = pathlib.Path(sys.argv[1])
identity = dict(source_commit=sys.argv[2], features=['profile-with-tracy'],
                binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest())
pathlib.Path(f'{binary}.identity.json').write_text(json.dumps(identity, indent=2) + '\n')
PY
```

Wait for `load1 < 1.0` before the run. Commit nothing from here until the
matrix finishes: the driver refuses a snapshot whose source commit is not
HEAD, so `run:` notes stay uncommitted until Step 6.

- [ ] **Step 3: Development check**

Run the command in `capture-host-setup.md` ("Running the dedicated lane") with `CASES='drm-aurora tty-resume screencast'`.
Expected: exit 0, `analysis.json` verdict `development-passed`; read it: `tty-resume` edges `[0, 1, 0]` with edge 1 inside `vt-away`; `screencast` three samples with `client-1`≠`client-2`, `client-2`=`client-3`, Aurora crops equal; `vt-restore.json` outcome `not-needed`. A consumer error here, after Step 5b passed, is a regression in the TTY environment (for example no EGL device without a display server): record the exact error, do not change crops or thresholds, and investigate before the pilot.

- [ ] **Step 4: Pilot**

Same command without `CASES`, fresh `OUT` (`tty-pilot-…`). Expected: verdict `lane-passed`.

- [ ] **Step 5: Matrix**

`matrix` mode, fresh `OUT`, `PILOT_DIR=<the pilot>`. Expected: verdict `lane-passed` with `tty-resume-r2/r3` and `unlock-r2/r3` passed.

- [ ] **Step 6: Restore the display and publish the evidence**

Run `display-dim restore` if Step 1 dimmed. In the evidence document add a "Dedicated lane" section: identity table (source, binary sha256, `DRM_OUTPUT`/`DRM_MODE`, lock-client and consumer hashes from `capture.json`, consumer identity from `cast-summary.json`), the verdict table for development, pilot and matrix, and move `tty-resume`, `unlock` and `screencast` out of Unverified. Close `material-f7eb0b` and `material-3acc86` in the same commit.

```bash
git add docs/materials/2026-09-30-optic-settling-evidence.md tasks/
git commit -m "docs(material): dedicated-lane acceptance evidence (material-f7eb0b, material-3acc86)"
```
