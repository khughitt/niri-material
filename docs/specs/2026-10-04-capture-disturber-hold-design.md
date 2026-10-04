# Hold host disturbers for the length of a quiet capture

**Status:** draft for owner review, 2026-10-04.
**Task:** `material-188aaa`, under `material-2834d7`.
**Extends:** [capture protocol design](2026-09-11-material-capture-protocol-design.md)
(`tools/capture-meta`: preflight, settle, release).

## 1. Intent

`capture-meta preflight` samples the host once, before the run. Anything the
host schedules after that sample lands inside the measurement unseen. Three
captures have been lost or put at risk this way:

- **2026-09-24 16:06:58** — `wali-rotate.timer` (every 15 min) rotated the
  wallpaper, ran Prism's apply and reloaded the desktop niri; the view-tilt
  smoke's nested kitty never mapped and the smoke failed after 13 minutes.
- **2026-09-24 16:54:05** — the desktop idle lock raised the GPU's P8 floor
  from 9.9 W to about 11.5 W mid-run; the settle gate refused
  `material-4241c3` at case 12 of 24 (22 minutes lost). A monitor wake at
  about 05:08 that morning would have done the same to a running capture.
- **2026-10-03** — with the desktop stopped, `dropbox.service` crash-loops
  (its override sets `DISPLAY=:0`; xwayland-satellite panics without a
  compositor) and restarts every ~14 s; one restart put a P5 sample in a
  settle window and refused the run.

This design makes preflight hold the host's known disturbers for the whole
run, record what it held and how to restore it, and restore it on every exit.
Release then scans the journal over the run window, so a result taken while
something fired anyway is marked as disturbed rather than silently accepted.

Success: on this host, a capture started from a TTY or from the desktop runs
with every user timer, the declared services and (on the desktop) idle and
monitor power held; `capture.json` lists each held item; every item is back
in its prior state after the run, including after a refused preflight or a
killed fixture; a timer that fires during the run is named in `capture.json`
and makes `release` exit non-zero.

Out of scope: system timers (stopping them needs root; §5 detects them
instead), changing any fixture's own pass/fail logic, and lanes on other
hosts beyond what §3.2's config file allows.

## 2. Decisions

| Question | Decision | Rejected |
| --- | --- | --- |
| Which timers | Every *active user* timer, discovered at hold time. | A curated list: it misses the next timer someone installs (this host has 13; `familiar-reap` fires every minute) and puts host unit names in the repository. |
| Which services | Units named in a per-host file (§3.2). Reference host: `dropbox.service`. | Holding every user service: most are the session itself. An environment variable: easily missing in a fresh TTY login. |
| Desktop idle | `noctalia msg caffeine-enable` for the run, `caffeine-disable` after. Idle then neither locks nor blanks mid-run. | Locking first: the lock surface is itself rendering and its state still changes when it times out. A Wayland idle-inhibit client: it needs a visible surface on the desktop. |
| Monitors | `niri msg action power-off-monitors` after caffeine, `power-on-monitors` on restore. | Leaving them on: the desktop's redraws failed the GPU P8/IQR gate on 2026-09-24 until the monitors were off. |
| Where | Inside `capture-meta preflight` and `release`, always on. Every fixture already calls both, so no fixture changes. | A wrapper fixtures opt into: each fixture would need editing and one forgotten fixture is an unheld run. |
| Disturbed run | `release` restores, records, then exits 1 with the disturbances named. | Exit 0 with a warning: fixtures that collect `release`'s status (`optic-settling-smoke.sh`) would pass a disturbed run. |

## 3. What is held

### 3.1 User timers

At hold time, `systemctl --user list-timers --output=json` lists the active
timers. Each is stopped with `systemctl --user stop <timer>`; restore runs
`systemctl --user start <timer>`. A stopped timer does not fire; on start, a
`Persistent=true` timer that missed its elapse fires once, after the run.
A service a timer activated that is still running at hold time is recorded
(`active_at_hold`) but not stopped: the preflight's own samples judge
whether it disturbs.

### 3.2 Declared services

`${XDG_CONFIG_HOME:-~/.config}/niri-material/capture-hold` lists one unit
per line (`#` comments and blank lines allowed). Each listed unit that is
`active` or `activating` is stopped and started again on restore; a listed
unit that is inactive is recorded as `not_running` and left alone. A missing
file holds no services and is recorded as `config: absent`; a listed unit
that does not exist refuses preflight (CannotRun), since the file names it
on purpose. [Capture host setup](../materials/capture-host-setup.md) records
the reference host's file.

### 3.3 Desktop idle and monitors

The desktop's socket is the inherited `NIRI_SOCKET`; without one (a TTY
login while the desktop still runs) it is the one `niri.*.sock` in
`$XDG_RUNTIME_DIR` on which `niri msg version` answers. More than one live
socket refuses preflight (CannotRun) rather than guessing. When a socket
answers, a desktop is up (the headless lane run from the desktop). The hold
then runs
`noctalia msg caffeine-enable` and `niri msg action power-off-monitors`,
recording the socket path so `release` (or a later recovery) reaches the same
compositor whatever its own environment says. Restore runs
`niri msg action power-on-monitors` and `noctalia msg caffeine-disable`.
Noctalia has no query for caffeine, so the prior state is recorded as
`unknown` and restore always disables it; caffeine is not left on by habit
on this host.

Without a desktop (the dedicated lane from a TTY, or a TTY session with the
desktop stopped) this part is recorded as `desktop: absent`. With a desktop
but no `noctalia` on `PATH`, idle is recorded as `not_held` and monitors are
still powered off; §5's scan does not see idle locks, so `show` prints the
gap.

### 3.4 Not held

System timers (`man-db`, `plocate-updatedb`, `logrotate`, …) need root to
stop. Their names and next elapse at hold time are recorded
(`system_timers`), and §5 flags any that fire.

## 4. Lifecycle

### 4.1 Preflight

The order becomes: write `run` → acquire the lock → **recover a stale hold**
(§4.3) → **hold** → `environment` → sample → judge. Holding before sampling
means the quiet verdict is taken on the held host, which is the host the run
measures (and a crash-looping service is already stopped).

The hold writes the host hold file (§4.3) before its first action and
rewrites it after each, so a kill at any point leaves a file naming exactly
what was changed. The `hold` section of `capture.json` is written once the
hold is complete:

```json
"hold": {
  "started": "2026-10-04T22:10:03-04:00",
  "started_epoch_us": 1791180603000000,
  "items": [
    {"kind": "timer", "unit": "wali-rotate.timer", "action": "stopped", "restore": "systemctl --user start wali-rotate.timer"},
    {"kind": "service", "unit": "dropbox.service", "action": "stopped", "restore": "systemctl --user start dropbox.service"},
    {"kind": "idle", "action": "caffeine-enabled", "prior": "unknown", "restore": "noctalia msg caffeine-disable"},
    {"kind": "monitors", "socket": "/run/user/1000/niri.wayland-1.571310.sock", "action": "powered-off", "restore": "niri msg action power-on-monitors"}
  ],
  "config": "~/.config/niri-material/capture-hold",
  "desktop": "present",
  "system_timers": [{"unit": "plocate-updatedb.timer", "next": "2026-10-05T00:23:41-04:00"}],
  "recovered": null
}
```

A failing hold action (a `systemctl stop` that errors) restores what was
already held and fails preflight with CannotRun (exit 2). A refused preflight
restores the hold before releasing the lock, as it already releases the lock.

### 4.2 Release

`release` restores every item in reverse order, then scans (§5), writes a
`hold_end` section, removes the host hold file, and releases the lock:

```json
"hold_end": {
  "restored": "2026-10-04T23:02:41-04:00",
  "failures": [],
  "disturbances": [
    {"at": "2026-10-04T22:23:41-04:00", "manager": "system", "unit": "plocate-updatedb.service", "kind": "timer-fired"}
  ],
  "verdict": "disturbed"
}
```

Restore is best-effort per item and continues past a failure. Any failure
keeps the host hold file holding the unrestored items, prints each with its
`restore` command, and exits 2. Otherwise a `disturbed` verdict exits 1 after
everything is restored and the lock released; a `clean` one exits 0.
`capture-meta show` prints the hold and its verdict above the sub-runs.

### 4.3 Recovery after a killed run

The host hold file is `$XDG_RUNTIME_DIR/capture-meta.hold.json`, beside the
lock: `{run_id, owner_pid, run_dir, items}`. `$XDG_RUNTIME_DIR` is cleared at
logout, which is also when the user manager's timers stop on their own.

- Preflight, after taking the lock, restores any hold file it finds (its
  owner is necessarily dead: a live owner would still hold the lock) and
  records `recovered: {run_id, items, failures}` in the new run's `hold`.
  A failed recovery refuses preflight with each unrestored item named.
- `capture-meta restore` restores the hold file by hand, refusing while the
  lock's owner is alive. This is the command a park note or a person uses
  after a fixture was killed and nothing has run since.

## 5. Disturbance scan

Over the window from the hold's completion to the start of restore, release
reads `journalctl --user` and `journalctl --system` (`-o json`, `--since
@<start>`, `--until @<end>`; this host's user is in `adm`, which reads the
system journal). It flags:

| Kind | Journal evidence |
| --- | --- |
| `timer-fired` | A job start (`MESSAGE_ID=7d4958e842da4a758f6c1cdc7b36dcc5`) of a unit some timer activates, by the timer→service map taken at hold time from both managers' `list-timers --all`. |
| `restart` | A scheduled restart (`MESSAGE_ID=5eb03494b6584870a536b337290809b3`) of any unit: a crash-looping service, declared or not. |
| `held-started` | A job start of a unit the hold stopped. |

The fixture's own transient units (headless Weston, per-run clients) are not
timer-activated and do not restart, so they are not flagged. An unreadable
journal is a scan failure: `hold_end.verdict` is `unscanned`, release exits 2
after restoring, and the run is not evidence until rescanned by hand.

Monitor wakes and idle locks have no journal entry this scan reads; holding
them (§3.3) is the protection.

## 6. Testing and the first live run

Unit tests in `tools/test_capture_meta.py` drive the hold through an injected
host adapter (systemctl, journal, niri and noctalia calls) and cover: timer
and service discovery, a hold action failing midway, refused-preflight
restore, release ordering, each scan kind, a restore failure keeping the hold
file, recovery by the next preflight and by `restore`, and the
`hold`/`hold_end` record validation.

Live, from a TTY with the desktop stopped (the first quiet run):

1. **Round trip** (about 2 min): `preflight` then `release` on a scratch
   run directory; user timers stopped during, started after; `dropbox` held;
   verdict `clean`.
2. **Positive control** (about 2 min): the same, with a transient user timer
   started inside the window (`systemd-run --user --on-active=30s true`);
   release names it `timer-fired` and exits 1.
3. **Kill recovery** (about 1 min): preflight, then kill the owner; the next
   preflight restores and records `recovered`.
4. **Evidence run:** the dedicated-lane development check
   (`optic-settling-smoke.sh pilot`, three cases; §"Running the dedicated
   lane" in capture host setup) with the hold in place.

The desktop part (§3.3) is exercised the next time a headless-lane run is
taken from the desktop; until then it is tested only through the adapter.
