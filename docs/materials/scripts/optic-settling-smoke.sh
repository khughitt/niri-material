#!/usr/bin/env bash
# Bounded Tracy capture for the active/idle Aurora control.
set -eu
MODE=${1:?usage: optic-settling-smoke.sh prepare|pilot|matrix [--lane headless|dedicated]}
shift
LANE=headless
if [ "$#" -gt 0 ]; then
    [ "$#" -eq 2 ] && [ "$1" = --lane ] || { echo 'expected --lane headless|dedicated' >&2; exit 2; }
    LANE=$2
fi
case $MODE in prepare|pilot|matrix) ;; *) echo "unknown mode: $MODE" >&2; exit 2 ;; esac
case $LANE in headless|dedicated) ;; *) echo "unknown lane: $LANE" >&2; exit 2 ;; esac
: "${OUT:?fresh artifact directory}"
: "${NIRI_BIN:?explicit profile-with-tracy binary}"
: "${NIRI_MATERIAL_WORK_ROOT:?evidence root}"
HERE=$(dirname "$(readlink -f "$0")")
. "$HERE/glass-optic-smoke-lib.sh"
[ "$MODE" = prepare ] || [ "$LANE" = headless ] \
    || fail 'dedicated capture requires a real TTY fixture; no TTY capture is implemented'

# The library owns Weston and niri. This driver also owns the capture timeout
# and its clients; a signal must stop all of them before releasing preflight.
PROBE_PID=; OTHER_PID=; EXPORT_PID=
on_exit() {
    local rc=$?
    trap - EXIT INT TERM
    for pid in "$CAP_PID" "$EXPORT_PID" "$PROBE_PID" "$OTHER_PID" "$NIRI_PID" "$WESTON_PID"; do
        [ -z "$pid" ] || { kill "$pid" 2>/dev/null || true; wait "$pid" 2>/dev/null || true; }
    done
    remove_runtime_dir || rc=1
    capture_meta release "$OUT" || rc=1
    exit "$rc"
}
trap 'exit 130' INT
trap 'exit 143' TERM
trap on_exit EXIT

# Keep exports interruptible; the shared helper's foreground export cannot
# be reaped promptly when the driver receives TERM.
csvexport() {
    timeout 120 "$TOOLS/tracy-csvexport" "$1" "$2" > "$3" & EXPORT_PID=$!
    local rc=0
    wait "$EXPORT_PID" || rc=$?
    EXPORT_PID=
    [ "$rc" -eq 0 ] || fail "tracy-csvexport $1 ${2##*/} failed or timed out"
}

[ -f "$NIRI_BIN.identity.json" ] || fail "missing $NIRI_BIN.identity.json"
python3 - "$NIRI_BIN" "$NIRI_BIN.identity.json" "$ROOT" <<'PY'
import hashlib, json, pathlib, subprocess, sys
binary, sidecar, root = map(pathlib.Path, sys.argv[1:])
identity = json.loads(sidecar.read_text())
source = subprocess.check_output(['git', '-C', str(root), 'rev-parse', 'HEAD'], text=True).strip()
assert identity.get('source_commit') == source, 'binary source differs from worktree HEAD'
assert identity.get('features') == ['profile-with-tracy'], 'binary lacks requested Tracy feature identity'
assert identity.get('binary_sha256') == hashlib.sha256(binary.read_bytes()).hexdigest(), 'binary hash differs from sidecar'
PY
[ "$MODE" = prepare ] || capture_preflight "$LANE"
cp "$NIRI_BIN" "$OUT/binary"
cp "$NIRI_BIN.identity.json" "$OUT/binary.identity.json"
NIRI=$OUT/binary

# Begin with one unfocused, lit Aurora window and a real virtual-pointer
# resume. The manifest keeps every available family required, so this
# partial driver cannot yield a passing pilot.
GLASS_EXTRA='aurora 0.5 { drift-hz 4; }'
TOP_EXTRA='signal { idle-after-ms 5000; }'
mkdir -p "$OUT/active-idle-resume"
write_config "$OUT/active-idle-resume/case.kdl"
"$NIRI" validate -c "$OUT/active-idle-resume/case.kdl" > "$OUT/validate.log" 2>&1 \
    || fail "generated config did not validate"
python3 - "$OUT" "$MODE" "$LANE" <<'PY'
import hashlib, json, pathlib, sys
out, mode, lane = pathlib.Path(sys.argv[1]), sys.argv[2], sys.argv[3]
identity = json.loads((out / 'binary.identity.json').read_text())
families = ('active-idle-resume', 'combined-motion', 'gate-policy', 'client-damage',
            'finite-attention', 'visibility', 'outputs-dpms', 'session-activation', 'reload-clock')
cases = []
for family in families:
    here = out / family
    cases.append(dict(name=family, family=family,
                      lane='dedicated' if family == 'session-activation' else 'headless',
                      config_sha256=hashlib.sha256((here / 'case.kdl').read_bytes()).hexdigest() if here.exists() else '0' * 64,
                      stimuli=['pointer'] if family == 'active-idle-resume' else [],
                      repetitions=1 if mode != 'matrix' else 3,
                      hold_ns=5_000_000_000 if mode != 'matrix' else 600_000_000_000,
                      required=(family != 'session-activation') if lane == 'headless' else family == 'session-activation'))
manifest = dict(schema=1, mode=mode, lane=lane, source_commit=identity['source_commit'],
                binary_sha256=identity['binary_sha256'], cases=cases)
(out / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
PY
if [ "$MODE" = matrix ]; then
    : "${PILOT_DIR:?passed pilot directory}"
    python3 - "$PILOT_DIR" "$OUT/manifest.json" <<'PY'
import json, pathlib, sys
pilot = pathlib.Path(sys.argv[1]); current = json.loads(pathlib.Path(sys.argv[2]).read_text())
analysis = json.loads((pilot / 'analysis.json').read_text())
planned = json.loads((pilot / 'manifest.json').read_text())
assert analysis['verdict'] == 'passed' and planned['mode'] == 'pilot' and planned['lane'] == current['lane'], 'matching passed lane pilot required'
assert planned['binary_sha256'] == current['binary_sha256'] and planned['source_commit'] == current['source_commit'], 'pilot binary mismatch'
old = {case['name']: case['config_sha256'] for case in planned['cases']}
new = {case['name']: case['config_sha256'] for case in current['cases']}
assert old == new, 'pilot case configs differ from matrix'
PY
fi
[ "$MODE" = prepare ] && exit 0
: "${CAPTURE_TASK:?task authorizing this capture}"
tools_ready
reserve_tracy_port
capture_identity --config threshold-ms=5000 --config case=active-idle-resume
start_nested "$NIRI" "$OUT/active-idle-resume/case.kdl" active-idle-resume
XDG_RUNTIME_DIR=$RT WAYLAND_DISPLAY=$(basename "$NIRI_SOCKET") \
    kitty --config NONE --class gos-probe -o background_opacity=0 \
    -o cursor_blink_interval=0 sh -c "$IDLE" >> "$OUT/probe.log" 2>&1 &
PROBE_PID=$!
for _ in $(seq 100); do [ "$(windows_with "$NIRI" gos-probe)" -ge 1 ] && break; sleep 0.1; done
[ "$(windows_with "$NIRI" gos-probe)" -eq 1 ] || fail 'probe never opened'
XDG_RUNTIME_DIR=$RT WAYLAND_DISPLAY=$(basename "$NIRI_SOCKET") \
    kitty --config NONE --class gos-other -o cursor_blink_interval=0 \
    sh -c "$IDLE" >> "$OUT/other.log" 2>&1 &
OTHER_PID=$!
for _ in $(seq 100); do [ "$(windows_with "$NIRI" gos-other)" -ge 1 ] && break; sleep 0.1; done
[ "$(windows_with "$NIRI" gos-other)" -eq 1 ] || fail 'focus thief never opened'
sleep 1
case_dir=$OUT/active-idle-resume
capture_seconds=18
[ "$MODE" = matrix ] && capture_seconds=610
timeout "$((capture_seconds + 60))" "$TOOLS/tracy-capture" -o "$case_dir/capture.tracy" \
    -a 127.0.0.1 -p "$TRACY_PORT" -s "$capture_seconds" > "$case_dir/capture.log" 2>&1 &
CAP_PID=$!
capture_ready active-idle-resume
[ "$MODE" = matrix ] && sleep 606 || sleep 14
msg "$NIRI" action spawn -- wlrctl pointer move 1 0
capture_wait
csvexport --messages "$case_dir/capture.tracy" "$case_dir/messages.csv"
csvexport --unwrap "$case_dir/capture.tracy" "$case_dir/cpu.csv"
csvexport --gpu "$case_dir/capture.tracy" "$case_dir/gpu.csv"
printf '{"messages":true,"cpu":true,"gpu":true}\n' > "$case_dir/export.json"
python3 - "$ROOT" "$case_dir" <<'PY'
import json, pathlib, sys
sys.path.insert(0, sys.argv[1])
from tools.optic_settling import csv_rows, parse_edges, zones, BEAT
directory = pathlib.Path(sys.argv[2])
edges = parse_edges(csv_rows(directory / 'messages.csv', ('MessageName', 'total_ns')))
if len(edges) != 2:
    raise ValueError(f'expected pause and resume markers, got {len(edges)}')
pause, resume = (edge['trace_ns'] for edge in edges)
beats = zones(directory / 'cpu.csv', 'ns_since_start', 'exec_time_ns').get(BEAT, [])
if not beats:
    raise ValueError('capture contains no heartbeat')
observation = dict(trace_start_ns=0, trace_end_ns=max(t + d for t, d in beats),
                   active_before=[pause - 2_000_000_000, pause - 500_000_000],
                   hold=[pause + 1_000_000_000, pause + 6_000_000_000],
                   active_after=[resume + 500_000_000, resume + 2_500_000_000],
                   expected_pixels='equal', before_rgb='before.rgb', after_rgb='after.rgb',
                   topology=['headless-1'], stimuli_intervals=[])
(directory / 'observation.json').write_text(json.dumps(observation, indent=2) + '\n')
PY
# Endpoint screenshots are outside the trace's zero-redraw window. Input
# remains idle after the capture, so the second held interval checks pixels.
sleep 2
shot "$NIRI" held-before
sleep 1
shot "$NIRI" held-after
magick "$OUT/held-before.png" -depth 8 rgb:- > "$case_dir/before.rgb"
magick "$OUT/held-after.png" -depth 8 rgb:- > "$case_dir/after.rgb"
stop_nested
python3 "$ROOT/tools/optic_settling.py" "$OUT"
