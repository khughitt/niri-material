# Terminal transport probe — material-07bac9

Bounded reproduction recorded 2026-10-11. Requires installed kitty 0.49.2,
Weston, a niri binary with signal IPC, and a working user systemd manager.
Uses a headless Weston unit and private sockets/config; the Python controller
cleans up children and its unit in `finally`, including TERM/INT.

Copy the Python block to `probe.py` in a new private temporary directory,
then run `python3 probe.py`. `PROBE_NIRI` selects an explicit alternative
binary; the successful run used `/usr/bin/niri` 26.04 (`99196eba`), SHA-256
`4b13e677894b228cf4dedee5e6bad91fa826d3ed9d5019ed11b1c400886dfab3`.
The directory must be writable. This is a throwaway assert-based probe,
not an installed watcher or a portable lifecycle harness. It records only
its fixture terminals; do not point its remote-control socket at a live terminal.

Ghostty 1.3.1-arch2 was inspected, but not launched in this reproduction.

## Controller and isolated kitty hooks

```python
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parent
RT = ROOT / 'runtime'
RT.mkdir(mode=0o700, exist_ok=True)
CFG = ROOT / 'config'
(CFG / 'kitty').mkdir(parents=True, exist_ok=True)
EVENTS = ROOT / 'events.jsonl'
EVENTS.write_text('')
UNIT = f'material-07bac9-{os.getpid()}-weston'
HOST = f'material-07bac9-{os.getpid()}'
TOKEN = f'material-07bac9-{os.getpid()}'
NIRI = os.environ.get('PROBE_NIRI', '/usr/bin/niri')
processes = []
files = []
report = {'niri': NIRI, 'checks': [], 'windows': [], 'signals': []}

def run(*args, env=None, check=True):
    return subprocess.run(args, env=env, check=check, capture_output=True, text=True, timeout=10)

def spawn(args, env, name):
    log = (ROOT / f'{name}.log').open('w')
    files.append(log)
    proc = subprocess.Popen(args, env=env, stdout=log, stderr=log, start_new_session=True)
    processes.append(proc)
    return proc

def wait_for(fn, description):
    deadline = time.monotonic() + 15
    while time.monotonic() < deadline:
        value = fn()
        if value:
            return value
        time.sleep(0.1)
    raise AssertionError(f'timed out: {description}; inspect {ROOT}')

def interrupted(signum, frame):
    raise SystemExit(f'interrupted by signal {signum}')

signal.signal(signal.SIGTERM, interrupted)
signal.signal(signal.SIGINT, interrupted)
watcher = ROOT / 'watcher.py'
watcher.write_text('''import json, os
from kitty.fast_data_types import get_boss
identities = {}

def on_load(boss, data):
    from kitty.constants import config_dir
    assert config_dir == os.environ['KITTY_CONFIG_DIRECTORY'], config_dir

def emit(kind, window, data):
    boss = get_boss()
    if kind != 'close':
        identities[window.os_window_id] = boss.os_window_map[window.os_window_id].wm_class
    row = dict(kind=kind, pid=os.getpid(), kitty_id=window.id,
               os_window_id=window.os_window_id,
               app_id=identities[window.os_window_id], data=data)
    with open(os.environ['PROBE_EVENTS'], 'a') as out:
        out.write(json.dumps(row) + '\\n')

def on_cmd_startstop(boss, window, data):
    emit('command', window, data)

def on_close(boss, window, data):
    emit('close', window, {})

def on_tab_bar_dirty(boss, window, data):
    for tab in data['tab_manager']:
        for w in tab:
            emit('progress', w, dict(state=w.progress.state.value, percent=w.progress.percent))
''')
(CFG / 'kitty' / 'notifications.py').write_text('''from kitty.fast_data_types import get_boss
import runpy, os

def main(nc):
    window = get_boss().window_id_map[nc.channel_id]
    runpy.run_path(os.environ['PROBE_WATCHER'])['emit']('notification', window, dict(title=nc.title))
    return True  # Keep probe notifications off the host desktop.
''')
(ROOT / 'niri.kdl').write_text('input { keyboard { xkb { layout "us"; }; }; }\n')
env = os.environ.copy()
for name in ('DISPLAY', 'WAYLAND_DISPLAY', 'WAYLAND_SOCKET', 'NIRI_SOCKET'):
    env.pop(name, None)
env.update(XDG_RUNTIME_DIR=str(RT), XDG_CONFIG_HOME=str(CFG),
           KITTY_CONFIG_DIRECTORY=str(CFG / 'kitty'), PROBE_EVENTS=str(EVENTS),
           PROBE_WATCHER=str(watcher), KITTY_SHELL_INTEGRATION='enabled')
try:
    run('systemd-run', '--user', f'--unit={UNIT}', '--collect', 'weston', '--backend=headless',
        '--renderer=gl', '--shell=kiosk-shell.so', '--width=1280', '--height=720', f'--socket={HOST}')
    host_socket = Path(os.environ['XDG_RUNTIME_DIR']) / HOST
    wait_for(host_socket.exists, 'headless Weston socket')
    (RT / HOST).symlink_to(host_socket)
    env['WAYLAND_DISPLAY'] = HOST
    spawn([NIRI, '-c', str(ROOT / 'niri.kdl')], env, 'niri')
    sock = wait_for(lambda: next(RT.glob('niri.*.sock'), None), 'private niri socket')
    env['NIRI_SOCKET'] = str(sock)
    env['WAYLAND_DISPLAY'] = sock.name.split('.')[1]
    def msg(*args, check=True):
        return run(NIRI, 'msg', *args, env=env, check=check)
    def windows():
        return json.loads(msg('-j', 'windows').stdout)
    def events():
        return [json.loads(line) for line in EVENTS.read_text().splitlines()]
    def signal_for(wid):
        return next(w['signal'] for w in windows() if w['id'] == wid)
    def rc(*args):
        return run('kitten', '@', '--to', f'unix:{ROOT}/kitty.sock', *args, env=env)
    def start_kitty(suffix):
        return spawn(['kitty', '--config', 'NONE', '--class', TOKEN + suffix,
                      '--listen-on', f'unix:{ROOT}/kitty.sock', '-o', 'allow_remote_control=socket-only',
                      '-o', f'watcher={watcher}', '-o', 'linux_display_server=wayland',
                      '-o', 'enable_audio_bell=no', '-o', 'shell_integration=enabled',
                      'bash', '--noprofile', '--norc', '-i'], env, 'kitty' + suffix)
    kitty = start_kitty('-a')
    wait_for(lambda: len(windows()) == 1 and (ROOT / 'kitty.sock').exists(), 'first kitty window')
    second = int(rc('launch', '--type=os-window', '--os-window-class', TOKEN + '-b',
                    '--watcher', str(watcher), 'bash', '--noprofile', '--norc', '-i').stdout)
    wait_for(lambda: len(windows()) == 2, 'second kitty OS window')
    before = windows()
    assert before[0]['pid'] == before[1]['pid'] == kitty.pid, before
    assert len({w['app_id'] for w in before}) == 2
    report['windows'] = [{k: w[k] for k in ('id', 'app_id', 'pid', 'signal')} for w in before]
    report['checks'].append('two OS windows have one PID and distinct app IDs')
    ls = json.loads(rc('ls').stdout)
    report['kitty_os_windows'] = [{k: w[k] for k in ('id', 'wm_class', 'platform_window_id')} for w in ls]
    for osw in ls:
        matches = [w for w in before if w['app_id'] == osw['wm_class'] and w['pid'] == kitty.pid]
        assert len(matches) == 1, matches
        wid = matches[0]['id']
        kwid = osw['tabs'][0]['windows'][0]['id']
        for command, code, kind in [('true', 0, 'Done'), ('false', 1, 'Error')]:
            offset = len(events())
            rc('send-text', '--match', f'id:{kwid}', 'sleep 1; ' + command + '\n')
            starts = wait_for(lambda: [e for e in events()[offset:] if e['kind'] == 'command'
                                      and e['kitty_id'] == kwid and e['data']['is_start']], 'command start')
            msg('set-window-signal', '--id', str(wid), '--source', 'terminal-probe', '--level', 'active')
            assert signal_for(wid)['level'] == 'Active'
            assert not any(e['kind'] == 'command' and e['kitty_id'] == kwid
                           and not e['data']['is_start'] for e in events()[offset:])
            report['signals'].append({'id': wid, 'phase': 'start', 'signal': signal_for(wid)})
            rows = wait_for(lambda: [e for e in events()[offset:] if e['kind'] == 'command'
                                    and e['kitty_id'] == kwid and not e['data']['is_start']], 'command end')
            stop = rows[-1]
            assert stop['data']['exit_status'] == code, stop
            assert all(e['app_id'] == osw['wm_class'] for e in starts + rows)
            msg('set-window-signal', '--id', str(wid), '--source', 'terminal-probe', '--level', 'notice' if code == 0 else 'demand')
            msg('pulse-window-signal', '--id', str(wid), '--source', 'terminal-probe', '--kind', kind.lower())
            sig = signal_for(wid)
            assert sig['impulses'][-1]['kind'] == kind, sig
            report['signals'].append({'id': wid, 'phase': command, 'signal': sig})
            assert all(w['signal'] is None for w in windows() if w['id'] != wid)
            msg('clear-window-signal', '--id', str(wid), '--source', 'terminal-probe')
            assert signal_for(wid) is None
        report['checks'].append(f'{osw["wm_class"]}: start, exit 0/1, Done/Error, clear, no cross-window writes')
    first_id = next(w['id'] for w in before if w['app_id'] == ls[0]['wm_class'])
    assert msg('pulse-window-signal', '--id', str(first_id), '--source', 'terminal-probe', '--kind', 'done', check=False).returncode != 0
    assert signal_for(first_id) is None
    msg('set-window-signal', '--id', str(first_id), '--source', 'terminal-probe', '--level', 'active',
        '--ttl-ms', '150', '--after-level', 'quiet', '--after-motion', 'static')
    wait_for(lambda: signal_for(first_id)['level'] == 'Quiet', 'slot decay')
    assert signal_for(first_id)['sources'] == ['terminal-probe']
    msg('clear-window-signal', '--id', str(first_id), '--source', 'terminal-probe')
    report['checks'].append('pulse before set refuses; TTL demotes but retains source; clear removes it')
    first_kwid = ls[0]['tabs'][0]['windows'][0]['id']
    offset = len(events())
    osc_input = ROOT / 'osc-input.txt'
    osc_input.write_text("printf '\\e]9;probe-nine\\a\\e]99;;probe-ninety-nine\\a\\e]9;4;1;42\\a'\n")
    rc('send-text', '--match', f'id:{first_kwid}', '--from-file', str(osc_input))
    wait_for(lambda: len([e for e in events()[offset:] if e['kind'] == 'notification']) >= 2, 'OSC 9/99 notification filters')
    wait_for(lambda: any(e['kind'] == 'progress' and e['data']['percent'] == 42 for e in events()[offset:]), 'OSC 9;4 progress')
    report['checks'].append('OSC 9/99 use notification filter; OSC 9;4 progress sampled by tab-bar watcher')
    offset = len(events())
    rc('close-window', '--match', f'id:{second}')
    wait_for(lambda: len(windows()) == 1, 'second window closes')
    wait_for(lambda: any(e['kind'] == 'close' and e['kitty_id'] == second for e in events()[offset:]), 'close watcher')
    closed_id = next(w['id'] for w in before if w['app_id'] == TOKEN + '-b')
    assert msg('set-window-signal', '--id', str(closed_id), '--source', 'terminal-probe', check=False).returncode != 0
    rc('close-window', '--match', f'id:{first_kwid}')
    kitty.wait(timeout=10)
    wait_for(lambda: not windows(), 'terminal exits')
    start_kitty('-restart')
    restarted = wait_for(lambda: next((w for w in windows() if w['app_id'] == TOKEN + '-restart'), None), 'terminal restarts')
    assert restarted['id'] not in {w['id'] for w in before} and restarted['signal'] is None
    assert msg('set-window-signal', '--id', str(first_id), '--source', 'terminal-probe', check=False).returncode != 0
    report['restart'] = {k: restarted[k] for k in ('id', 'app_id', 'pid', 'signal')}
    report['checks'].append('close callback observed; stale IDs refuse writes; restart gets fresh ID and empty signal')
    (ROOT / 'verdict.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report, indent=2))
finally:
    for proc in reversed(processes):
        if proc.poll() is None:
            os.killpg(proc.pid, signal.SIGTERM)
        try:
            proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            os.killpg(proc.pid, signal.SIGKILL)
            proc.wait(timeout=5)
    run('systemctl', '--user', 'stop', UNIT, check=False)
    for file in files:
        file.close()

```

## Successful verdict

```json
{
  "niri": "/usr/bin/niri",
  "checks": [
    "two OS windows have one PID and distinct app IDs",
    "material-07bac9-2933004-a: start, exit 0/1, Done/Error, clear, no cross-window writes",
    "material-07bac9-2933004-b: start, exit 0/1, Done/Error, clear, no cross-window writes",
    "pulse before set refuses; TTL demotes but retains source; clear removes it",
    "OSC 9/99 use notification filter; OSC 9;4 progress sampled by tab-bar watcher",
    "close callback observed; stale IDs refuse writes; restart gets fresh ID and empty signal"
  ],
  "windows": [
    {
      "id": 2,
      "app_id": "material-07bac9-2933004-a",
      "pid": 2933031,
      "signal": null
    },
    {
      "id": 3,
      "app_id": "material-07bac9-2933004-b",
      "pid": 2933031,
      "signal": null
    }
  ],
  "signals": [
    {
      "id": 2,
      "phase": "start",
      "signal": {
        "level": "Active",
        "motion": "Static",
        "accent": null,
        "tag": null,
        "sources": [
          "terminal-probe"
        ],
        "impulses": []
      }
    },
    {
      "id": 2,
      "phase": "true",
      "signal": {
        "level": "Notice",
        "motion": "Static",
        "accent": null,
        "tag": null,
        "sources": [
          "terminal-probe"
        ],
        "impulses": [
          {
            "source": "terminal-probe",
            "kind": "Done",
            "accent": null,
            "at": {
              "secs": 79777,
              "nanos": 479808955
            },
            "expires_at": {
              "secs": 79778,
              "nanos": 979808955
            }
          }
        ]
      }
    },
    {
      "id": 2,
      "phase": "start",
      "signal": {
        "level": "Active",
        "motion": "Static",
        "accent": null,
        "tag": null,
        "sources": [
          "terminal-probe"
        ],
        "impulses": []
      }
    },
    {
      "id": 2,
      "phase": "false",
      "signal": {
        "level": "Demand",
        "motion": "Static",
        "accent": null,
        "tag": null,
        "sources": [
          "terminal-probe"
        ],
        "impulses": [
          {
            "source": "terminal-probe",
            "kind": "Error",
            "accent": null,
            "at": {
              "secs": 79778,
              "nanos": 673123686
            },
            "expires_at": {
              "secs": 79780,
              "nanos": 173123686
            }
          }
        ]
      }
    },
    {
      "id": 3,
      "phase": "start",
      "signal": {
        "level": "Active",
        "motion": "Static",
        "accent": null,
        "tag": null,
        "sources": [
          "terminal-probe"
        ],
        "impulses": []
      }
    },
    {
      "id": 3,
      "phase": "true",
      "signal": {
        "level": "Notice",
        "motion": "Static",
        "accent": null,
        "tag": null,
        "sources": [
          "terminal-probe"
        ],
        "impulses": [
          {
            "source": "terminal-probe",
            "kind": "Done",
            "accent": null,
            "at": {
              "secs": 79779,
              "nanos": 858321133
            },
            "expires_at": {
              "secs": 79781,
              "nanos": 358321133
            }
          }
        ]
      }
    },
    {
      "id": 3,
      "phase": "start",
      "signal": {
        "level": "Active",
        "motion": "Static",
        "accent": null,
        "tag": null,
        "sources": [
          "terminal-probe"
        ],
        "impulses": []
      }
    },
    {
      "id": 3,
      "phase": "false",
      "signal": {
        "level": "Demand",
        "motion": "Static",
        "accent": null,
        "tag": null,
        "sources": [
          "terminal-probe"
        ],
        "impulses": [
          {
            "source": "terminal-probe",
            "kind": "Error",
            "accent": null,
            "at": {
              "secs": 79781,
              "nanos": 173502649
            },
            "expires_at": {
              "secs": 79782,
              "nanos": 673502649
            }
          }
        ]
      }
    }
  ],
  "kitty_os_windows": [
    {
      "id": 1,
      "wm_class": "material-07bac9-2933004-a",
      "platform_window_id": null
    },
    {
      "id": 2,
      "wm_class": "material-07bac9-2933004-b",
      "platform_window_id": null
    }
  ],
  "restart": {
    "id": 4,
    "app_id": "material-07bac9-2933004-restart",
    "pid": 2933584,
    "signal": null
  }
}

```

## Selected actual terminal events

```jsonl
{"kind": "command", "pid": 2933031, "kitty_id": 1, "os_window_id": 1, "app_id": "material-07bac9-2933004-a", "data": {"is_start": true, "time": 0.60038225, "cmdline": "sleep 1; true", "exit_status": 0}}
{"kind": "command", "pid": 2933031, "kitty_id": 1, "os_window_id": 1, "app_id": "material-07bac9-2933004-a", "data": {"is_start": false, "time": 1.603513251, "cmdline": "sleep 1; true", "exit_status": 0}}
{"kind": "command", "pid": 2933031, "kitty_id": 1, "os_window_id": 1, "app_id": "material-07bac9-2933004-a", "data": {"is_start": true, "time": 1.795314063, "cmdline": "sleep 1; false", "exit_status": 0}}
{"kind": "command", "pid": 2933031, "kitty_id": 1, "os_window_id": 1, "app_id": "material-07bac9-2933004-a", "data": {"is_start": false, "time": 2.798431027, "cmdline": "sleep 1; false", "exit_status": 1}}
{"kind": "command", "pid": 2933031, "kitty_id": 2, "os_window_id": 2, "app_id": "material-07bac9-2933004-b", "data": {"is_start": true, "time": 2.972331853, "cmdline": "sleep 1; true", "exit_status": 0}}
{"kind": "command", "pid": 2933031, "kitty_id": 2, "os_window_id": 2, "app_id": "material-07bac9-2933004-b", "data": {"is_start": false, "time": 3.976080461, "cmdline": "sleep 1; true", "exit_status": 0}}
{"kind": "command", "pid": 2933031, "kitty_id": 2, "os_window_id": 2, "app_id": "material-07bac9-2933004-b", "data": {"is_start": true, "time": 4.284174441, "cmdline": "sleep 1; false", "exit_status": 0}}
{"kind": "command", "pid": 2933031, "kitty_id": 2, "os_window_id": 2, "app_id": "material-07bac9-2933004-b", "data": {"is_start": false, "time": 5.287476292, "cmdline": "sleep 1; false", "exit_status": 1}}
{"kind": "command", "pid": 2933031, "kitty_id": 1, "os_window_id": 1, "app_id": "material-07bac9-2933004-a", "data": {"is_start": true, "time": 5.814030998, "cmdline": "printf '\\e]9;probe-nine\\a\\e]99;;probe-ninety-nine\\a\\e]9;4;1;42\\a'", "exit_status": 0}}
{"kind": "notification", "pid": 2933031, "kitty_id": 1, "os_window_id": 1, "app_id": "material-07bac9-2933004-a", "data": {"title": "probe-nine"}}
{"kind": "notification", "pid": 2933031, "kitty_id": 1, "os_window_id": 1, "app_id": "material-07bac9-2933004-a", "data": {"title": "probe-ninety-nine"}}
{"kind": "progress", "pid": 2933031, "kitty_id": 1, "os_window_id": 1, "app_id": "material-07bac9-2933004-a", "data": {"state": 1, "percent": 42}}
{"kind": "command", "pid": 2933031, "kitty_id": 1, "os_window_id": 1, "app_id": "material-07bac9-2933004-a", "data": {"is_start": false, "time": 5.816718889, "cmdline": "printf '\\e]9;probe-nine\\a\\e]99;;probe-ninety-nine\\a\\e]9;4;1;42\\a'", "exit_status": 0}}
{"kind": "progress", "pid": 2933031, "kitty_id": 1, "os_window_id": 1, "app_id": "material-07bac9-2933004-a", "data": {"state": 1, "percent": 42}}
{"kind": "close", "pid": 2933031, "kitty_id": 2, "os_window_id": 2, "app_id": "material-07bac9-2933004-b", "data": {}}
{"kind": "progress", "pid": 2933031, "kitty_id": 1, "os_window_id": 1, "app_id": "material-07bac9-2933004-a", "data": {"state": 1, "percent": 42}}
{"kind": "close", "pid": 2933031, "kitty_id": 1, "os_window_id": 1, "app_id": "material-07bac9-2933004-a", "data": {}}
{"kind": "close", "pid": 2933584, "kitty_id": 1, "os_window_id": 1, "app_id": "material-07bac9-2933004-restart", "data": {}}
```
