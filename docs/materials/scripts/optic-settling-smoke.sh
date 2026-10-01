#!/usr/bin/env bash
# optic-settling-smoke.sh: bounded Tracy captures for sustained optic settling
# (design docs/specs/2026-09-29-sustained-optic-settling-design.md §8, plan
# Task 4). One case per check-family control, each a fresh nested niri under
# a headless Weston host with an unfocused probe kitty under glass.
#
#   OUT=<fresh dir> NIRI_BIN=<identified Tracy binary> CAPTURE_TASK=<task> \
#   NIRI_MATERIAL_WORK_ROOT=<evidence root> [PILOT_DIR=<passed pilot>] \
#   docs/materials/scripts/optic-settling-smoke.sh prepare|pilot|matrix [--lane headless|dedicated]
#
# The case table (the manifest heredoc below) declares, per case, the optic
# edges the trace must carry, the state of each interval between them (active
# at a cadence, static, settled), the journaled stimuli and the pixel pairs.
# tools/optic_settling.py derives its own observation windows from those
# declarations; this driver only records configs, a stimulus journal on
# CLOCK_MONOTONIC (the clock of the edges' real_ns) and decoded pixels.
# Cases outside the lane (TTY resume, unlock, an idle inhibitor, a second
# output) are recorded unverified, never run.
#
# CASES (space-separated) limits pilot/matrix to named cases for
# development; such a run is never a passed pilot.
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

# The library owns Weston and niri. This driver also owns the capture timeout,
# its clients and the export; a signal must stop all of them before releasing
# preflight.
PROBE_PID=; OTHER_PID=; EXPORT_PID=; BG_PID=
alive() { local state; state=$(ps -o stat= -p "$1" 2>/dev/null) || return 1; [[ $state != Z* ]]; }
# A client stuck before its event loop (kitty blocks TERM until then) must not
# hold cleanup open: TERM, then KILL after 5 s.
reap() {
    kill "$1" 2>/dev/null || true
    for _ in $(seq 50); do alive "$1" || break; sleep 0.1; done
    kill -KILL "$1" 2>/dev/null || true
    wait "$1" 2>/dev/null || true
}
on_exit() {
    local rc=$?
    trap - EXIT INT TERM
    for pid in "$CAP_PID" "$EXPORT_PID" "$BG_PID" "$PROBE_PID" "$OTHER_PID" "$NIRI_PID" "$WESTON_PID"; do
        [ -z "$pid" ] || reap "$pid"
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

# The shared helper's idle client exits after 600 s, and a closing window
# redraws: the matrix holds for 600 s, so its clients idle without end.
IDLE='printf "\033[?25l"; exec sleep infinity'

# --- configs ------------------------------------------------------------------
# The probe is an unfocused kitty under glass (focus "none", no beam). Aurora
# 0.5 at 4 Hz is the sustained optic unless a case says otherwise.
AURORA='aurora 0.5 { drift-hz 4; }'
SIGNAL5='signal { idle-after-ms 5000; }'
WALL2=$OUT/cool-mid.png
magick -size 1280x720 xc:'rgb(90,115,140)' "$WALL2"
case_config() {   # $1 case dir, $2 GLASS_EXTRA, $3 TOP_EXTRA, $4 file name (default case.kdl)
    GLASS_EXTRA=$2 TOP_EXTRA=$3 write_config "$1/${4:-case.kdl}"
}
write_case_configs() {   # $1 case name, $2 dir
    local base=${1%-r[0-9]*}
    mkdir -p "$2"
    case $base in
        aurora-full|combined|client-damage|impulse-idle|vis-workspace|vis-tab|vis-offscreen|vis-overview|dpms-ipc|dpms-input|screencast)
            case_config "$2" "$AURORA" "$SIGNAL5" ;;
        aurora-reduced) case_config "$2" "$AURORA" 'signal { motion "reduced"; idle-after-ms 5000; }' ;;
        attention-cycle) case_config "$2" '' "$SIGNAL5" ;;
        gate-off) case_config "$2" "$AURORA" 'signal { idle-after-ms 0; }' ;;
        amount-0) case_config "$2" 'aurora 0 { drift-hz 4; }' "$SIGNAL5" ;;
        drift-0) case_config "$2" 'aurora 0.5 { drift-hz 0; }' "$SIGNAL5" ;;
        motion-off) case_config "$2" "$AURORA" 'signal { motion "off"; idle-after-ms 5000; }' ;;
        animations-off) case_config "$2" "$AURORA" "$SIGNAL5
animations { off; }" ;;
        startup-no-input) case_config "$2" "$AURORA" "$SIGNAL5" ;;
        default-threshold) case_config "$2" "$AURORA" '' ;;
        reload-held)
            case_config "$2" "$AURORA" "$SIGNAL5"
            case_config "$2" "$AURORA" 'signal { idle-after-ms 6000; }' 1-same-side.kdl
            case_config "$2" "$AURORA" 'signal { idle-after-ms 6000; }
input { keyboard { repeat-rate 31; }; }' 2-unrelated.kdl
            case_config "$2" 'aurora 0.5 { drift-hz 2; }' 'signal { idle-after-ms 6000; }
input { keyboard { repeat-rate 31; }; }' 3-drift.kdl
            case_config "$2" 'aurora 0.5 { drift-hz 2; }' 'signal { motion "reduced"; idle-after-ms 6000; }
input { keyboard { repeat-rate 31; }; }' 4-policy.kdl
            case_config "$2" 'aurora 0.8 { drift-hz 2; }' 'signal { motion "reduced"; idle-after-ms 6000; }
input { keyboard { repeat-rate 31; }; }' 5-amount.kdl
            case_config "$2" 'aurora 0.8 { drift-hz 2; }' 'signal { motion "reduced"; idle-after-ms 60000; }
input { keyboard { repeat-rate 31; }; }' 6-raise.kdl ;;
        reload-lower)
            case_config "$2" "$AURORA" 'signal { idle-after-ms 30000; }'
            case_config "$2" "$AURORA" "$SIGNAL5" 1-lower.kdl ;;
        reload-disable)
            case_config "$2" "$AURORA" "$SIGNAL5"
            case_config "$2" "$AURORA" 'signal { idle-after-ms 0; }' 1-disable.kdl ;;
        tty-resume|unlock|idle-inhibitor|output-removal) : ;;   # outside every lane this driver runs
        *) fail "no config for case $1" ;;
    esac
    local kdl
    for kdl in "$2"/*.kdl; do
        [ -e "$kdl" ] || continue
        "$NIRI" validate -c "$kdl" >> "$OUT/validate.log" 2>&1 || fail "generated config $kdl did not validate"
    done
}

# --- case table ---------------------------------------------------------------
# Rates: Aurora 4 Hz full, 2 Hz reduced; breathe and pulse are measured by the
# pilot (material-signals-smoke recorded 8 Hz breathe and 26.6 Hz pulse on
# llvmpipe). Settled intervals hold for HOLD_S, 600 s once in the matrix.
python3 - "$OUT" "$MODE" "$LANE" "${CASES:-}" <<'PY'
import hashlib, json, pathlib, sys
out, mode, lane, only = pathlib.Path(sys.argv[1]), sys.argv[2], sys.argv[3], sys.argv[4].split()
S = 1_000_000_000
A4 = dict(state='active', min_hz=4, max_hz=4)
A2 = dict(state='active', min_hz=2, max_hz=2)
STATIC = dict(state='static')
HELD = dict(state='settled')
SETUP = dict(state='setup')
damage = dict(label='client', min_redraws=1, min_draws=1)
def case(name, family, edges, segments, stimuli=(), pixels=(), capture_s=30, lane='headless', why=None):
    return dict(name=name, family=family, lane=lane, edges=edges, segments=segments,
                stimuli=list(stimuli), pixels=list(pixels), capture_s=capture_s, why=why)
held_pair = [dict(before='held-1.rgb', after='held-2.rgb', expect='equal')]
cases = [
    case('aurora-full', 'active-idle-resume', [0, 1], [A4, HELD, A4], pixels=held_pair),
    case('aurora-reduced', 'active-idle-resume', [0, 1], [A2, HELD, A2], pixels=held_pair),
    case('combined', 'combined-motion', [0, 1, 0],
         [dict(state='active', min_hz=8, max_hz=12), HELD, dict(state='active', min_hz=8, max_hz=12), HELD],
         capture_s=42),
    case('gate-off', 'gate-policy', [], [A4], capture_s=23),
    *(case(name, 'gate-policy', [0], [STATIC, HELD], [damage], capture_s=34)
      for name in ('amount-0', 'drift-0', 'motion-off', 'animations-off')),
    case('startup-no-input', 'gate-policy', [0], [SETUP, HELD], [damage], capture_s=20),
    case('default-threshold', 'gate-policy', [0], [A4, HELD], capture_s=51),
    case('client-damage', 'client-damage', [0], [A4, HELD],
         [damage, dict(label='backdrop', min_redraws=1, min_draws=1)],
         [dict(before='damage-1.rgb', after='damage-2.rgb', expect='different'),
          dict(before='aurora-1.rgb', after='aurora-2.rgb', expect='equal')], capture_s=45),
    case('impulse-idle', 'finite-attention', [0], [A4, HELD],
         [dict(label='shot-1'), dict(label='impulse', min_redraws=1, min_draws=1), dict(label='shot-2')],
         [dict(before='impulse-1.rgb', after='impulse-2.rgb', expect='equal')], capture_s=45),
    case('attention-cycle', 'finite-attention', [0, 1],
         [dict(state='active', min_hz=20, max_hz=30), HELD, dict(state='active', min_hz=20, max_hz=30)]),
    *(case(name, 'visibility', [0], [STATIC, HELD], [dict(label='reveal', min_redraws=1, min_draws=1)],
           capture_s=37) for name in ('vis-workspace', 'vis-tab', 'vis-offscreen')),
    case('vis-overview', 'visibility', [0], [A4, HELD], [dict(label='close-overview', min_redraws=1)],
         capture_s=37),
    case('dpms-ipc', 'outputs-dpms', [0], [A4, HELD],
         [dict(label='power-off'), dict(label='power-on', min_redraws=1, min_draws=1)], capture_s=44),
    case('dpms-input', 'outputs-dpms', [0, 1], [A4, HELD, A4], [dict(label='power-off')], capture_s=31),
    case('output-removal', 'outputs-dpms', [], [], lane='two-output',
         why='the nested winit backend has one output'),
    case('screencast', 'session-activation', [0], [A4, HELD], [dict(label='screencopy', min_draws=1)],
         [dict(before='cast-1.rgb', after='cast-2.rgb', expect='equal')], capture_s=40),
    case('tty-resume', 'session-activation', [], [], lane='dedicated',
         why='TTY session resume needs a real TTY session'),
    case('unlock', 'session-activation', [], [], lane='dedicated',
         why='unlocking needs an authenticating lock client on a real session'),
    case('idle-inhibitor', 'session-activation', [], [], lane='inhibitor-client',
         why='no idle-inhibit client is installed'),
    case('reload-held', 'reload-clock', [0, 1], [A4, HELD, dict(state='active', min_hz=1, max_hz=1)],
         [dict(label=label) for label in ('same-side', 'unrelated', 'drift', 'policy', 'amount', 'raise')],
         capture_s=54),
    case('reload-lower', 'reload-clock', [0], [A4, HELD], [dict(label='lower')], capture_s=36),
    case('reload-disable', 'reload-clock', [0, 1], [A4, HELD, A4], [dict(label='disable')], capture_s=36),
]
for item in cases:
    item['hold_ns'] = 5 * S
    item['repetitions'] = 1
    if item['lane'] == 'headless' and item['edges']:
        # Tracy collects a frame's GPU zones only when a later frame renders,
        # so a case ends with one more client frame (min 0: its own zones
        # stay uncollected) that flushes the draws of the frames before it.
        # A journal aligns through optic edges, so an edge-free case (the
        # gate-off control, whose cadence never stops) has none.
        item['stimuli'].append(dict(label='collect'))
if mode == 'matrix':
    # Three active/idle/resume cycles of each rate, the first held for 600 s.
    extra = []
    for base in ('aurora-full', 'aurora-reduced'):
        template = next(c for c in cases if c['name'] == base)
        extra += [dict(template, name=f'{base}-r{k}') for k in (2, 3)]
    cases += extra
    first = next(c for c in cases if c['name'] == 'aurora-full')
    first['hold_ns'] = 600 * S
    first['capture_s'] = 30 + 600 - 5   # resume at 621 s: 603 s settled
if only:
    unknown = set(only) - {c['name'] for c in cases}
    if unknown:
        raise SystemExit(f'unknown CASES: {sorted(unknown)}')
for item in cases:
    item['required'] = item['lane'] == lane and (not only or item['name'] in only)
    item['selected'] = item['lane'] == lane and (not only or item['name'] in only)
identity = json.loads((out / 'binary.identity.json').read_text())
manifest = dict(schema=2, mode=mode, lane=lane, development=bool(only),
                source_commit=identity['source_commit'],
                binary_sha256=identity['binary_sha256'], cases=cases)
(out / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
PY
# Development runs (CASES) keep their unselected cases unverified.
mapfile -t RUN_CASES < <(jq -r '.cases[] | select(.selected) | .name' "$OUT/manifest.json")
mapfile -t ALL_CASES < <(jq -r '.cases[] | select(.lane == "'"$LANE"'") | .name' "$OUT/manifest.json")
for name in "${ALL_CASES[@]}"; do write_case_configs "$name" "$OUT/$name"; done
python3 - "$ROOT" "$OUT" <<'PY'
import json, pathlib, sys
sys.path.insert(0, sys.argv[1])
from tools.optic_settling import config_identity
out = pathlib.Path(sys.argv[2])
manifest = json.loads((out / 'manifest.json').read_text())
for case in manifest['cases']:
    here = out / case['name']
    case['config_sha256'] = config_identity(here) if here.is_dir() else None
(out / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
PY
if [ "$MODE" = matrix ]; then
    : "${PILOT_DIR:?passed pilot directory}"
    python3 - "$PILOT_DIR" "$OUT/manifest.json" "$ROOT" <<'PY'
import json, pathlib, re, sys
sys.path.insert(0, sys.argv[3])
from tools.optic_settling import run_config_identity
pilot = pathlib.Path(sys.argv[1]); current = json.loads(pathlib.Path(sys.argv[2]).read_text())
out = pathlib.Path(sys.argv[2]).parent
analysis = json.loads((pilot / 'analysis.json').read_text())
planned = json.loads((pilot / 'manifest.json').read_text())
assert analysis['verdict'] == 'passed' and planned['mode'] == 'pilot' and not planned.get('development'), \
    'a passed full pilot is required'
assert planned['lane'] == current['lane'], 'matching lane pilot required'
assert planned['binary_sha256'] == current['binary_sha256'] and planned['source_commit'] == current['source_commit'], 'pilot binary mismatch'
# Configs name files inside their own run, so compare them with the run
# directory normalized; the pilot's own analysis already checked its raw hashes.
old = {case['name']: run_config_identity(pilot / case['name']) if (pilot / case['name']).is_dir() else None
       for case in planned['cases']}
for case in current['cases']:
    base = re.sub(r'-r[0-9]+$', '', case['name'])
    here = run_config_identity(out / case['name']) if (out / case['name']).is_dir() else None
    assert base in old and old[base] == here, f"{case['name']}: config differs from the pilot's {base}"
assert set(old) <= {case['name'] for case in current['cases']}, 'the matrix drops pilot cases'
PY
fi
[ "$MODE" = prepare ] && exit 0
: "${CAPTURE_TASK:?task authorizing this capture}"
tools_ready
reserve_tracy_port
capture_identity --config threshold-ms=5000 --config cases="${RUN_CASES[*]}"

# --- stimuli ------------------------------------------------------------------
mono() { python3 -c 'import time; print(time.monotonic_ns())'; }
# at S: sleep until S seconds after the capture connected. Every drive is a
# schedule of absolute times, so IPC latency (about 0.5 s per pointer motion)
# cannot push a stimulus past its window or the capture's end.
at() {
    python3 - "$T0" "$1" <<'PY2'
import sys, time
t0, s = int(sys.argv[1]), float(sys.argv[2])
delay = t0 + s * 1e9 - time.monotonic_ns()
if delay < -0.5e9:
    sys.exit(f'schedule slipped: {s} s came {-delay / 1e9:.2f} s late')
time.sleep(max(0, delay) / 1e9)
PY2
}
pointer() { msg "$NIRI" action spawn -- wlrctl pointer move 1 0; }
keepalive() {   # motions at 0, 3, 6, 9 and 12 s: the pause comes about 17.5 s in
    local s
    for s in 0 3 6 9 12; do at "$s"; pointer; done
}
# stim LABEL EFFECT_S CMD...: journal the stimulus from just before the
# command to EFFECT_S after it returns, on the edges' clock.
stim() {
    local label=$1 effect=$2 start end; shift 2
    start=$(mono)
    "$@"
    end=$(( $(mono) + $(awk -v s="$effect" 'BEGIN { printf "%d", s * 1e9 }') ))
    printf '%s\t%s\t%s\n' "$label" "$start" "$end" >> "$CASE_DIR/journal.tsv"
}
print_line() { echo "line $1" > "$FIFO"; }
reload() { cp "$CASE_DIR/$1" "$CASE_DIR/live.kdl.tmp"; mv "$CASE_DIR/live.kdl.tmp" "$CASE_DIR/live.kdl"; }
shot_rgb() {   # label: screenshot into the case dir as raw RGB
    shot "$NIRI" "$CASE-$1"
    magick "$OUT/$CASE-$1.png" -depth 8 rgb:- > "$CASE_DIR/$1.rgb"
}
crop_rgb() {   # source label, target label, geometry
    magick "$OUT/$CASE-$1.png" -crop "$3" +repage -depth 8 rgb:- > "$CASE_DIR/$2.rgb"
}
other_id() { msg "$NIRI" -j windows | jq -r '.[] | select(.app_id=="gos-other") | .id'; }
probe_id() { msg "$NIRI" -j windows | jq -r '.[] | select(.app_id=="gos-probe") | .id'; }
set_demand() { msg "$NIRI" set-window-signal --id "$(probe_id)" --source demo --accent '#e5a33c' --level demand --motion "$1"; }
swap_backdrop() {
    XDG_RUNTIME_DIR=$RT WAYLAND_DISPLAY=$DISPLAY_NAME swaybg -m fill -i "$WALL2" >> "$OUT/swaybg.log" 2>&1 &
    BG_PID=$!
    sleep 0.5
    pkill -x -f "swaybg -m fill -i $WALL" || fail 'the startup swaybg was not running'
}
screencopy() {
    local k
    for k in 1 2 3 4 5; do
        XDG_RUNTIME_DIR=$RT WAYLAND_DISPLAY=$DISPLAY_NAME grim "$OUT/$CASE-cast-$k.png"
        sleep 0.5
    done
    magick "$OUT/$CASE-cast-1.png" -depth 8 rgb:- > "$CASE_DIR/cast-1.rgb"
    magick "$OUT/$CASE-cast-5.png" -depth 8 rgb:- > "$CASE_DIR/cast-2.rgb"
}
# The GPU leaves P8 for a few seconds after a compositor exits; the next
# case's settle must not see that tail.
await_gpu_rest() {
    local streak=0
    for _ in $(seq 60); do
        if [ "$(nvidia-smi --query-gpu=pstate --format=csv,noheader)" = P8 ]; then
            streak=$((streak + 1)); [ "$streak" -lt 6 ] || return 0
        else
            streak=0
        fi
        sleep .5
    done
    fail 'GPU did not return to P8 within 30 s of the last compositor exit'
}

# --- per-case drives: setup_* runs before the capture connects, drive_* after
# it on an absolute schedule (seconds since the capture connected), post_*
# after the capture ends. The keepalive's pause comes at about 17.5 s; a
# settled interval holds at least 5 s before its first stimulus.
drive_cycle() { keepalive; at "$((CAPTURE_S - 4))"; pointer; }   # resume 4 s before the end, before the next pause
post_held() { sleep 4; shot_rgb held-1; sleep 1; shot_rgb held-2; }
drive_aurora_full() { drive_cycle; }
post_aurora_full() { post_held; }
drive_aurora_reduced() { drive_cycle; }
post_aurora_reduced() { post_held; }
setup_combined() { set_demand breathe; }
# Resuming at 26.13 s offsets Aurora's held grid from attention's absolute one;
# the second pause follows 5 s later.
drive_combined() { keepalive; at 26.13; pointer; }
drive_gate_off() { keepalive; }
drive_static() { keepalive; at 25; stim client 1.5 print_line 1; }
drive_amount_0() { drive_static; }
drive_drift_0() { drive_static; }
drive_motion_off() { drive_static; }
drive_animations_off() { drive_static; }
NO_POINTER_startup_no_input=1
drive_startup_no_input() { at 10; stim client 1.5 print_line 1; }
drive_default_threshold() { keepalive; }
drive_client_damage() { keepalive; at 25; stim client 1.5 print_line 1; at 32.5; stim backdrop 3 swap_backdrop; }
post_client_damage() {
    sleep 2; shot_rgb before; print_line 2; sleep 1; shot_rgb after
    crop_rgb before damage-1 1280x720+0+0; crop_rgb after damage-2 1280x720+0+0
    # Below kitty's few printed lines, inside the probe: Aurora alone.
    crop_rgb before aurora-1 400x200+80+450; crop_rgb after aurora-2 400x200+80+450
}
setup_impulse_idle() { msg "$NIRI" set-window-signal --id "$(probe_id)" --source demo --accent '#e5a33c'; }
drive_impulse_idle() {
    keepalive
    at 21; stim shot-1 1 shot_rgb impulse-1
    at 25; stim impulse 2.5 msg "$NIRI" pulse-window-signal --id "$(probe_id)" --source demo --kind done
    at 30; stim shot-2 1 shot_rgb impulse-2
}
setup_attention_cycle() { set_demand pulse; }
drive_attention_cycle() { drive_cycle; }
setup_vis_workspace() {
    msg "$NIRI" action focus-window --id "$(probe_id)"
    msg "$NIRI" action move-window-to-workspace-down --focus false
    msg "$NIRI" action focus-window --id "$(other_id)"
}
setup_vis_tab() {
    msg "$NIRI" action focus-window --id "$(other_id)"
    msg "$NIRI" action consume-or-expel-window-left
    msg "$NIRI" action toggle-column-tabbed-display
    msg "$NIRI" action focus-window --id "$(other_id)"
}
setup_vis_offscreen() {
    local k
    for k in 2 3 4; do
        msg "$NIRI" action spawn -- kitty --config NONE --class "gos-col$k" -o cursor_blink_interval=0 -o mouse_hide_wait=0 sh -c "$IDLE"
        for _ in $(seq 100); do [ "$(windows_with "$NIRI" "gos-col$k")" -ge 1 ] && break; sleep 0.1; done
        pointer   # setup outlasts the threshold without input
    done
    msg "$NIRI" action focus-column-last
}
drive_reveal() { keepalive; at 21; stim reveal 2 msg "$NIRI" action focus-window --id "$(probe_id)"; }
drive_vis_workspace() { drive_reveal; }
drive_vis_tab() { drive_reveal; }
drive_vis_offscreen() { drive_reveal; }
setup_vis_overview() { msg "$NIRI" action open-overview; }
drive_vis_overview() { keepalive; at 21; stim close-overview 2 msg "$NIRI" action close-overview; }
drive_dpms_ipc() {
    keepalive
    at 25; stim power-off 1 msg "$NIRI" action power-off-monitors
    at 28; stim power-on 1.5 msg "$NIRI" action power-on-monitors
}
drive_dpms_input() { keepalive; at 25; stim power-off 1 msg "$NIRI" action power-off-monitors; at 27; pointer; }
drive_screencast() { keepalive; at 25; stim screencopy 0.5 screencopy; }
# niri's config watcher notices a replaced file within about half a second;
# each reload's journaled effect covers that and the frame it causes.
drive_reload_held() {
    keepalive
    at 25; stim same-side 2 reload 1-same-side.kdl
    at 29; stim unrelated 2 reload 2-unrelated.kdl
    at 33; stim drift 2 reload 3-drift.kdl
    at 37; stim policy 2 reload 4-policy.kdl
    at 41; stim amount 2 reload 5-amount.kdl
    at 45; stim raise 2 reload 6-raise.kdl
}
drive_reload_lower() { keepalive; at 20; stim lower 2 reload 1-lower.kdl; }
drive_reload_disable() { keepalive; at 25; stim disable 2 reload 1-disable.kdl; }

run_case() {
    CASE=$1
    local base=${CASE%-r[0-9]*} fn no_pointer
    fn=${base//-/_}
    CASE_DIR=$OUT/$CASE
    CAPTURE_S=$(jq -r --arg n "$CASE" '.cases[] | select(.name == $n) | .capture_s' "$OUT/manifest.json")
    no_pointer=NO_POINTER_$fn; no_pointer=${!no_pointer:-}
    : > "$CASE_DIR/journal.tsv"
    # niri reloads the file it was started with, so the live config is a copy.
    cp "$CASE_DIR/case.kdl" "$CASE_DIR/live.kdl"
    start_nested "$NIRI" "$CASE_DIR/live.kdl" "$CASE"
    # Clients connect to the nested Wayland socket, which the IPC socket's
    # name carries (niri.<display>.<pid>.sock); the IPC socket is not one.
    DISPLAY_NAME=$(basename "$NIRI_SOCKET" | sed -E 's/^niri\.(.+)\.[0-9]+\.sock$/\1/')
    [ -S "$RT/$DISPLAY_NAME" ] || fail "no nested Wayland socket $RT/$DISPLAY_NAME"
    # kitty hides the pointer after mouse_hide_wait (3 s by default), and the
    # cursor change redraws: every kitty here keeps it shown.
    # Input activity from launch: the 5 s threshold must not expire during
    # setup, or the trace carries an extra pause/resume pair.
    [ -n "$no_pointer" ] || pointer
    FIFO=$RT/probe-$CASE.fifo
    mkfifo "$FIFO"
    XDG_RUNTIME_DIR=$RT WAYLAND_DISPLAY=$DISPLAY_NAME \
        kitty --config NONE --class gos-probe -o background_opacity=0 -o cursor_blink_interval=0 -o mouse_hide_wait=0 \
        sh -c "printf '\033[?25l'; exec 3<>'$FIFO'; while read -r line <&3; do printf '%s\n' \"\$line\"; done" \
        >> "$OUT/probe.log" 2>&1 &
    PROBE_PID=$!
    for _ in $(seq 100); do [ "$(windows_with "$NIRI" gos-probe)" -ge 1 ] && break; sleep 0.1; done
    [ "$(windows_with "$NIRI" gos-probe)" -eq 1 ] || fail "$CASE: probe never opened"
    [ -n "$no_pointer" ] || pointer
    XDG_RUNTIME_DIR=$RT WAYLAND_DISPLAY=$DISPLAY_NAME \
        kitty --config NONE --class gos-other -o cursor_blink_interval=0 -o mouse_hide_wait=0 sh -c "$IDLE" >> "$OUT/other.log" 2>&1 &
    OTHER_PID=$!
    for _ in $(seq 100); do [ "$(windows_with "$NIRI" gos-other)" -ge 1 ] && break; sleep 0.1; done
    [ "$(windows_with "$NIRI" gos-other)" -eq 1 ] || fail "$CASE: focus thief never opened"
    [ -n "$no_pointer" ] || pointer
    if declare -F "setup_$fn" > /dev/null; then "setup_$fn"; fi
    [ "$(msg "$NIRI" -j windows | jq -r '.[] | select(.app_id=="gos-probe") | .is_focused')" = false ] \
        || fail "$CASE: the probe is focused"
    [ -n "$no_pointer" ] || pointer
    sleep 1
    # Tracy records GPU zones only several seconds into a trace (6.5-10.6 s
    # observed) while CPU zones start at once; trace time counts from the
    # compositor's start. The keepalive puts the pause after GPU zones begin.
    [ -n "$no_pointer" ] || pointer
    timeout "$((CAPTURE_S + 60))" "$TOOLS/tracy-capture" -o "$CASE_DIR/capture.tracy" \
        -a 127.0.0.1 -p "$TRACY_PORT" -s "$CAPTURE_S" > "$CASE_DIR/capture.log" 2>&1 &
    CAP_PID=$!
    capture_ready "$CASE"
    T0=$(mono)
    "drive_$fn"
    if jq -e --arg n "$CASE" '.cases[] | select(.name == $n) | .stimuli | any(.label == "collect")' \
        "$OUT/manifest.json" > /dev/null; then
        at "$(awk -v s="$CAPTURE_S" 'BEGIN { print s - 1 }')"; stim collect 0.6 print_line collect
    fi
    capture_wait
    if declare -F "post_$fn" > /dev/null; then "post_$fn"; fi
    csvexport --messages "$CASE_DIR/capture.tracy" "$CASE_DIR/messages.csv"
    csvexport --unwrap "$CASE_DIR/capture.tracy" "$CASE_DIR/cpu.csv"
    csvexport --gpu "$CASE_DIR/capture.tracy" "$CASE_DIR/gpu.csv"
    printf '{"messages":true,"cpu":true,"gpu":true}\n' > "$CASE_DIR/export.json"
    python3 - "$CASE_DIR" <<'PY'
import json, pathlib, sys
directory = pathlib.Path(sys.argv[1])
journal = []
for line in (directory / 'journal.tsv').read_text().splitlines():
    label, start, end = line.split('\t')
    journal.append(dict(label=label, start_mono_ns=int(start), end_mono_ns=int(end)))
(directory / 'observation.json').write_text(json.dumps(dict(journal=journal, topology=['headless-1']), indent=2) + '\n')
PY
    stop_nested
    for pid in "$BG_PID" "$PROBE_PID" "$OTHER_PID"; do [ -z "$pid" ] || reap "$pid"; done
    BG_PID=; PROBE_PID=; OTHER_PID=
    rm -f "$FIFO"
    await_gpu_rest
}

for name in "${RUN_CASES[@]}"; do
    echo "case $name" >&2
    run_case "$name"
done
python3 "$ROOT/tools/optic_settling.py" "$OUT"
