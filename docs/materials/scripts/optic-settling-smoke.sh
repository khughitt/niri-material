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
# The dedicated lane (--lane dedicated; docs/specs/2026-10-02-real-tty-settling-lane-design.md)
# runs niri on DRM from a TTY with the desktop stopped: drm-aurora, tty-resume,
# unlock and screencast. It needs DRM_OUTPUT and DRM_MODE, switches VTs through
# vt-lib.sh, and restores the starting VT on every exit. A second output stays
# unverified, as does every case outside the lane being run. The idle-inhibitor case runs a
# real zwp_idle_inhibit_manager_v1 client (idle-inhibit-client.c, built per
# run with wayland-scanner and cc) and proves from niri's IdleInhibit trace
# messages that the inhibition took hold and released. The
# screencopy case is a capture without input (grim, wlr-screencopy), not a
# screencast: niri's PipeWire cast path renders and schedules separately.
#
# Exit status is the analyzer's: 0 when every case of the lane passed, with
# analysis.json verdict "passed" (complete) or "lane-passed" (complete=false,
# the other-lane cases listed under "unverified"); nonzero for a failed or
# invalid case, a compositor panic, or a refused/interrupted run. Every exit
# keeps the partial evidence and writes SHA256SUMS over it (capture.json, which
# the lock release still updates, excepted) before releasing the capture lock.
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
. "$HERE/vt-lib.sh"
if [ "$LANE" = dedicated ]; then
    : "${DRM_OUTPUT:?physical output name, e.g. DP-1}" "${DRM_MODE:?fixed WIDTHxHEIGHT@REFRESH}"
    [ -n "$DRM_OUTPUT" ] && [ -n "$DRM_MODE" ] || fail 'DRM_OUTPUT and DRM_MODE must be non-empty'
fi
# Host items are owner actions (docs/materials/capture-host-setup.md); refuse naming the missing one.
dedicated_prerequisites() {
    local root link exe pid status connector found=
    [ -z "${WAYLAND_DISPLAY:-}" ] || fail 'the dedicated lane runs from a TTY login, not inside a Wayland session'
    ! pgrep -x niri > /dev/null || fail 'a niri is already running: stop the desktop session first'
    # A killed earlier run can leave its niri holding DRM; it runs the run's
    # snapshot copy, named binary, which pgrep -x niri does not see.
    root=$(readlink -f "$NIRI_MATERIAL_WORK_ROOT")
    for link in /proc/[0-9]*/exe; do
        exe=$(readlink "$link" 2>/dev/null) || continue
        exe=${exe% (deleted)}
        case $exe in
            "$root"/*/binary)
                pid=${link#/proc/}
                fail "a niri from an earlier run is still running: pid ${pid%/exe} runs $exe" ;;
        esac
    done
    # One output, DRM_OUTPUT: a second connected one, or none, would fail
    # only at analysis.
    for status in "$DRM_SYSFS"/card*-*/status; do
        [ -e "$status" ] || continue
        [ "$(cat "$status")" = connected ] || continue
        connector=${status%/status}
        connector=${connector##*/}
        connector=${connector#card*-}
        [ "$connector" = "$DRM_OUTPUT" ] \
            || fail "output $connector is connected besides DRM_OUTPUT $DRM_OUTPUT: disconnect it for the dedicated lane"
        found=1
    done
    [ -n "$found" ] || fail "DRM_OUTPUT $DRM_OUTPUT is not connected (no connected connector under $DRM_SYSFS)"
    sudo -n -l /usr/bin/chvt > /dev/null 2>&1 \
        || fail 'no NOPASSWD rule for /usr/bin/chvt (docs/materials/capture-host-setup.md)'
    gst-inspect-1.0 pipewiresrc > /dev/null 2>&1 || fail 'no pipewiresrc element: install gst-plugin-pipewire'
}

# The library owns Weston and niri. This driver also owns the capture timeout,
# its clients and the export; a signal must stop all of them before releasing
# preflight.
PROBE_PID=; OTHER_PID=; EXPORT_PID=; BG_PID=; SLEEP_PID=; INHIBIT_PID=; BUS_PID=; CAST_PID=; LOCK_PID=
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
    # A second signal must not cut cleanup short: VT restoration and the lock
    # release still run. A no-op handler, not SIG_IGN, so the children cleanup
    # starts (timeout, chvt, capture-meta) keep default signal handling.
    trap - EXIT
    trap : INT TERM HUP
    for pid in "$SLEEP_PID" "$CAP_PID" "$EXPORT_PID" "$BG_PID" "$PROBE_PID" "$OTHER_PID" "$INHIBIT_PID" "$CAST_PID" "$LOCK_PID" "$NIRI_PID" "$BUS_PID" "$WESTON_PID"; do
        [ -z "$pid" ] || reap "$pid"
    done
    remove_runtime_dir || rc=1
    vt_restore "$OUT/vt-restore.json" || rc=1
    write_sums || rc=1
    capture_meta release "$OUT" || rc=1
    exit "$rc"
}
write_sums() {
    (cd "$OUT" && find . -type f ! -name SHA256SUMS ! -name capture.json -print0 | LC_ALL=C sort -z \
        | xargs -0 -r sha256sum > SHA256SUMS)
}
trap 'exit 130' INT
trap 'exit 143' TERM
trap on_exit EXIT

# Bash runs a trap only once its foreground child exits, so every wait that
# can last (sleeps, the stimulus schedule, exports) runs in the background
# under `wait`, which a signal interrupts at once, with its pid recorded for
# on_exit to reap.
nap() {
    sleep "$1" & SLEEP_PID=$!
    wait "$SLEEP_PID"
    SLEEP_PID=
}
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
# Offline tests substitute stub Tracy tools; only a stubbed capture record
# (CAPTURE_META) may accompany them, so no recorded run uses unidentified tools.
if [ -n "${OPTIC_SETTLING_STUB_TOOLS:-}" ]; then
    [ -n "${CAPTURE_META:-}" ] || fail 'OPTIC_SETTLING_STUB_TOOLS needs a stub CAPTURE_META'
fi
# Offline stub runs only: a time scale for the drive schedule and stand-ins
# for the consumer and lock client. A recorded run never uses them.
TIMESCALE=${OPTIC_SETTLING_STUB_TIMESCALE:-1}
CAST_CONSUMER=${OPTIC_SETTLING_STUB_CONSUMER:-$ROOT/tools/screencast_consumer.py}
DRM_SYSFS=${OPTIC_SETTLING_STUB_DRM_SYSFS:-/sys/class/drm}
if [ -z "${OPTIC_SETTLING_STUB_TOOLS:-}" ]; then
    [ "$TIMESCALE" = 1 ] \
        && [ -z "${OPTIC_SETTLING_STUB_CONSUMER:-}${OPTIC_SETTLING_STUB_LOCK:-}${OPTIC_SETTLING_STUB_DRM_SYSFS:-}" ] \
        || fail 'stub timescale, consumer, lock client and DRM sysfs need OPTIC_SETTLING_STUB_TOOLS'
    # vt-lib.sh's test seams: a recorded run switches and reads the real VTs.
    [ -z "${VT_CHVT:-}" ] || fail 'VT_CHVT needs OPTIC_SETTLING_STUB_TOOLS'
    [ -z "${VT_LOGINCTL:-}" ] || fail 'VT_LOGINCTL needs OPTIC_SETTLING_STUB_TOOLS'
    [ "$VT_ACTIVE_FILE" = /sys/class/tty/tty0/active ] || fail 'VT_ACTIVE_FILE needs OPTIC_SETTLING_STUB_TOOLS'
fi
[ "$MODE" = prepare ] || capture_preflight "$LANE"
if [ "$MODE" != prepare ] && [ "$LANE" = dedicated ]; then
    dedicated_prerequisites
    vt_record_home
    VT_SPARE=$(vt_spare) || fail "no spare VT without a logind session"
    printf '{"home": %s, "spare": %s}\n' "$VT_HOME" "$VT_SPARE" > "$OUT/vt.json"
fi
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
DRM_TOP=
[ "$LANE" != dedicated ] || DRM_TOP="output \"$DRM_OUTPUT\" { mode \"$DRM_MODE\"; scale 1; }
debug { dbus-interfaces-in-non-session-instances; }"
WALL2=$OUT/cool-mid.png
magick -size 1280x720 xc:'rgb(90,115,140)' "$WALL2"
case_config() {   # $1 case dir, $2 GLASS_EXTRA, $3 TOP_EXTRA, $4 file name (default case.kdl)
    GLASS_EXTRA=$2 TOP_EXTRA=$3 write_config "$1/${4:-case.kdl}"
}
write_case_configs() {   # $1 case name, $2 dir
    local base=${1%-r[0-9]*}
    mkdir -p "$2"
    case $base in
        aurora-full|combined|client-damage|impulse-idle|vis-workspace|vis-tab|vis-offscreen|vis-overview|dpms-ipc|dpms-input|screencopy|idle-inhibitor)
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
        drm-aurora|tty-resume|unlock|screencast) case_config "$2" "$AURORA" "$SIGNAL5
$DRM_TOP" ;;
        output-removal) : ;;   # outside every lane this driver runs
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
python3 - "$OUT" "$MODE" "$LANE" "${CASES:-}" "${DRM_OUTPUT:-}" "${DRM_MODE:-}" <<'PY'
import hashlib, json, pathlib, sys
out, mode, lane, only = pathlib.Path(sys.argv[1]), sys.argv[2], sys.argv[3], sys.argv[4].split()
S = 1_000_000_000
A4 = dict(state='active', min_hz=4, max_hz=4)
A2 = dict(state='active', min_hz=2, max_hz=2)
STATIC = dict(state='static')
HELD = dict(state='settled')
SETUP = dict(state='setup')
damage = dict(label='client', min_redraws=1, min_draws=1)
def case(name, family, edges, segments, stimuli=(), pixels=(), capture_s=30, lane='headless', why=None, **extra):
    return dict(name=name, family=family, lane=lane, edges=edges, segments=segments,
                stimuli=list(stimuli), pixels=list(pixels), capture_s=capture_s, why=why, **extra)
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
    # grim is wlr-screencopy: a capture without input, not a screencast.
    case('screencopy', 'session-activation', [0], [A4, HELD], [dict(label='screencopy', min_draws=1)],
         [dict(before='copy-1.rgb', after='copy-2.rgb', expect='equal')], capture_s=40),
    # Dedicated lane: niri on DRM from a TTY. drm-aurora is the lane's cadence control.
    case('drm-aurora', 'active-idle-resume', [0, 1], [A4, HELD, A4], pixels=held_pair, lane='dedicated'),
    # Paused on the spare VT, niri cannot redraw: only the return must, and
    # the resume edge falls inside the return, not the switch out.
    case('tty-resume', 'session-activation', [0, 1, 0], [A4, HELD, A4, HELD],
         [dict(label='vt-out'), dict(label='vt-return', min_redraws=1)], capture_s=45, lane='dedicated',
         edge_in=[[1, 'vt-return']]),
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
    # A real inhibitor while held: no input activity, so no resume edge, and
    # ordinary client updates still draw.
    case('idle-inhibitor', 'session-activation', [0], [A4, HELD],
         [dict(label='inhibit', min_redraws=1, messages=['IdleInhibit inhibited=1']), damage,
          dict(label='release', min_redraws=1, messages=['IdleInhibit inhibited=0'])], capture_s=36),
    case('reload-held', 'reload-clock', [0, 1], [A4, HELD, dict(state='active', min_hz=1, max_hz=1)],
         [dict(label=label) for label in ('same-side', 'unrelated', 'drift', 'policy', 'amount', 'raise')],
         capture_s=54),
    case('reload-lower', 'reload-clock', [0], [A4, HELD], [dict(label='lower')], capture_s=36),
    case('reload-disable', 'reload-clock', [0, 1], [A4, HELD, A4], [dict(label='disable')], capture_s=36),
]
for item in cases:
    item['hold_ns'] = 5 * S
    item['repetitions'] = 1
    if item['lane'] in ('headless', 'dedicated') and item['edges']:
        # Tracy collects a frame's GPU zones only when a later frame renders,
        # so a case ends with one more client frame that flushes the draws of
        # the frames before it. Its own GPU zones stay uncollected, but its
        # redraw must reach the trace: that proves the trace ran to the end.
        # A journal aligns through optic edges, so an edge-free case (the
        # gate-off control, whose cadence never stops) has none.
        item['stimuli'].append(dict(label='collect', min_redraws=1))
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
if lane == 'dedicated':
    manifest['drm'] = dict(output=sys.argv[5], mode=sys.argv[6])
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
assert analysis['verdict'] in ('passed', 'lane-passed') and planned['mode'] == 'pilot' \
    and not planned.get('development'), 'a passed full pilot is required'
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
if [ -n "${OPTIC_SETTLING_STUB_TOOLS:-}" ]; then TOOLS=$OPTIC_SETTLING_STUB_TOOLS; else tools_ready; fi
reserve_tracy_port
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
# Set only once built (or stubbed): a use without a build fails under set -u.
if selected idle-inhibitor; then
    build_client idle-inhibit-client "$PROTOCOLS/stable/xdg-shell/xdg-shell.xml" \
        "$PROTOCOLS/unstable/idle-inhibit/idle-inhibit-unstable-v1.xml"
    INHIBIT_BIN=$OUT/clients/idle-inhibit-client
fi
if [ -n "${OPTIC_SETTLING_STUB_LOCK:-}" ]; then
    LOCK_BIN=$OPTIC_SETTLING_STUB_LOCK
elif selected unlock; then
    build_client session-lock-client "$PROTOCOLS/staging/ext-session-lock/ext-session-lock-v1.xml"
    LOCK_BIN=$OUT/clients/session-lock-client
fi
if selected screencast; then IDENTITY_EXTRA+=(--input "$ROOT/tools/screencast_consumer.py"); fi
capture_identity --config threshold-ms=5000 --config cases="${RUN_CASES[*]}" --config lane="$LANE" \
    ${DRM_OUTPUT:+--config drm-output="$DRM_OUTPUT"} ${DRM_MODE:+--config drm-mode="$DRM_MODE"} \
    "${IDENTITY_EXTRA[@]}"

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
    # A niri that ignores TERM holds the device and the VT: KILL after reap's bound.
    reap "$NIRI_PID"; NIRI_PID=
    [ -z "$BUS_PID" ] || reap "$BUS_PID"
    BUS_PID=
    rm -f "$RT"/niri.*.sock "$RT/bus"; sleep 0.5
}
start_host() { if [ "$LANE" = dedicated ]; then start_drm "$@"; else start_nested "$@"; fi; }
stop_host() { if [ "$LANE" = dedicated ]; then stop_drm; else stop_nested; fi; }
topology() {
    # Name, current mode and scale per output: the analyzer checks them against DRM_OUTPUT and DRM_MODE.
    if [ "$LANE" = dedicated ]; then
        msg "$NIRI" -j outputs | jq -c '[.[] | {name, scale: .logical.scale, mode: (if .current_mode == null
            then null else .modes[.current_mode] | {width, height, refresh_hz: (.refresh_rate / 1000)} end)}]'
    else
        echo '["headless-1"]'
    fi
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
    # info: prints no trailing newline, and `read` fails at EOF without one.
    local size
    size=$(magick "$OUT/geometry-drm.png" -format '%w %h' info:) || fail 'could not read the DRM screen size'
    printf '%s\n' "$size" > "$OUT/screen-drm.txt"
    grep -qxE '[0-9]+ [0-9]+' "$OUT/screen-drm.txt" || fail "could not read the DRM screen size from geometry-drm.png"
    stop_host
    await_gpu_rest
}

# --- stimuli ------------------------------------------------------------------
mono() { python3 -c 'import time; print(time.monotonic_ns())'; }
# at S: sleep until S seconds after the capture connected. Every drive is a
# schedule of absolute times, so IPC latency (about 0.5 s per pointer motion)
# cannot push a stimulus past its window or the capture's end.
# The wait runs in the background (see nap): the matrix waits up to 609 s here.
at() {
    python3 - "$T0" "$1" "$TIMESCALE" <<'PY2' & SLEEP_PID=$!
import sys, time
t0, s = int(sys.argv[1]), float(sys.argv[2]) * float(sys.argv[3])
delay = t0 + s * 1e9 - time.monotonic_ns()
if delay < -0.5e9:
    sys.exit(f'schedule slipped: {s} s came {-delay / 1e9:.2f} s late')
time.sleep(max(0, delay) / 1e9)
PY2
    local rc=0
    wait "$SLEEP_PID" || rc=$?
    SLEEP_PID=
    [ "$rc" -eq 0 ] || fail "stimulus schedule failed at $1 s"
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
grab_screencopy() {   # wlr-screencopy through grim; no screencast consumer
    local k
    for k in 1 2 3 4 5; do
        XDG_RUNTIME_DIR=$RT WAYLAND_DISPLAY=$DISPLAY_NAME grim "$OUT/$CASE-copy-$k.png"
        sleep 0.5
    done
    magick "$OUT/$CASE-copy-1.png" -depth 8 rgb:- > "$CASE_DIR/copy-1.rgb"
    magick "$OUT/$CASE-copy-5.png" -depth 8 rgb:- > "$CASE_DIR/copy-2.rgb"
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
post_held() { nap 4; shot_rgb held-1; nap 1; shot_rgb held-2; }
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
    nap 2; shot_rgb before; print_line 2; nap 1; shot_rgb after
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
drive_screencopy() { keepalive; at 25; stim screencopy 0.5 grab_screencopy; }
start_inhibitor() {
    : "${INHIBIT_BIN:?the idle inhibit client was not built for this run}"
    XDG_RUNTIME_DIR=$RT WAYLAND_DISPLAY=$DISPLAY_NAME "$INHIBIT_BIN" >> "$OUT/inhibit.log" 2>&1 &
    INHIBIT_PID=$!
    for _ in $(seq 50); do [ "$(windows_with "$NIRI" gos-inhibit)" -ge 1 ] && break; sleep 0.1; done
    [ "$(windows_with "$NIRI" gos-inhibit)" -eq 1 ] || fail "$CASE: the idle inhibitor never mapped"
}
stop_inhibitor() {
    local rc=0
    kill -TERM "$INHIBIT_PID"
    wait "$INHIBIT_PID" || rc=$?
    INHIBIT_PID=
    [ "$rc" -eq 0 ] || fail "$CASE: the idle inhibitor exited $rc"
    for _ in $(seq 50); do [ "$(windows_with "$NIRI" gos-inhibit)" -eq 0 ] && break; sleep 0.1; done
    [ "$(windows_with "$NIRI" gos-inhibit)" -eq 0 ] || fail "$CASE: the idle inhibitor window stayed mapped"
}
# Inhibit about 3.5 s into the held interval, draw a client line while
# inhibited, then release; held throughout, with no resume edge.
drive_idle_inhibitor() {
    keepalive
    at 21; stim inhibit 3 start_inhibitor
    at 26; stim client 1.5 print_line 1
    at 29; stim release 3 stop_inhibitor
}
drive_drm_aurora() { drive_cycle; }
post_drm_aurora() { post_held; }
# Out to the spare VT and back: niri's session pauses, then activation calls
# notify_activity. The switch out with its hold and the return are separate
# stimuli, so the resume edge must fall in the return. Both switches are
# verified; each window stays under 6 s.
vt_out() {
    vt_switch "$VT_SPARE" || fail "$CASE: did not reach spare VT $VT_SPARE"
    nap 3   # backgrounded, so TERM while away is handled at once
}
vt_return() { vt_switch "$VT_HOME" || fail "$CASE: the return to VT $VT_HOME did not land"; }
drive_tty_resume() { keepalive; at 26; stim vt-out 0 vt_out; stim vt-return 1.5 vt_return; }
lock_start() {
    : "${LOCK_BIN:?the session lock client was not built for this run}"
    XDG_RUNTIME_DIR=$RT WAYLAND_DISPLAY=$DISPLAY_NAME "$LOCK_BIN" > "$CASE_DIR/lock.out" 2>> "$OUT/lock.log" &
    LOCK_PID=$!
    for _ in $(seq 50); do grep -qx locked "$CASE_DIR/lock.out" && return; alive "$LOCK_PID" || break; sleep 0.1; done
    fail "$CASE: the session never locked (see lock.log)"
}
# A client that ignores its signal must not hold an unattended run (and the
# lock): 10 s, then the case fails and on_exit kills it. Its status is read
# only once it has exited.
await_exit() {   # pid, what: sets EXIT_RC
    local _
    for _ in $(seq 100); do alive "$1" || break; sleep 0.1; done
    ! alive "$1" || fail "$CASE: $2 did not exit within 10 s"
    EXIT_RC=0
    wait "$1" || EXIT_RC=$?
}
unlock_now() {
    kill -USR1 "$LOCK_PID" 2>/dev/null || fail "$CASE: the lock client exited before the unlock (see lock.log)"
    await_exit "$LOCK_PID" 'the lock client'
    LOCK_PID=
    [ "$EXIT_RC" -eq 0 ] || fail "$CASE: the lock client exited $EXIT_RC"
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
    kill -TERM "$CAST_PID" 2>/dev/null || fail "$CASE: the screencast consumer exited (see cast.log)"
    await_exit "$CAST_PID" 'the screencast consumer'
    CAST_PID=
    [ "$EXIT_RC" -eq 0 ] || fail "$CASE: the screencast consumer exited $EXIT_RC (see cast.log)"
}
thief_line() { echo "thief $1" > "$OTHER_FIFO"; }
# Arm a sample, cause its damage, wait for the frame that damage produced.
# A request no frame answers fails the case; it is never retried.
cast_sample() {   # label, damage command...
    local label=$1 target=$CASE_DIR/$1.raw
    shift
    alive "$CAST_PID" || fail "$CASE: the screencast consumer exited (see cast.log)"
    printf '%s\n' "$target" > "$CASE_DIR/sample-request"
    kill -USR1 "$CAST_PID" 2>/dev/null || fail "$CASE: the screencast consumer exited (see cast.log)"
    for _ in $(seq 20); do [ -e "$target.armed" ] && break; alive "$CAST_PID" || break; sleep 0.1; done
    if [ ! -e "$target.armed" ]; then
        alive "$CAST_PID" || fail "$CASE: the screencast consumer exited (see cast.log)"
        fail "$CASE: $label was never armed"
    fi
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
# The crops are only as good as the calibration, so the headless probe_rect's
# checks apply before casting: the case's probe has the calibrated size and
# is large enough for both crops.
setup_screencast() {
    local px py pw ph j iw ih
    read -r px py pw ph < "$OUT/probe-rect-drm.txt"
    j=$(msg "$NIRI" -j windows | jq -c '.[] | select(.app_id=="gos-probe") | .layout.window_size')
    iw=$(jq -r '.[0] | floor' <<< "$j"); ih=$(jq -r '.[1] | floor' <<< "$j")
    [ "$pw" -eq "$iw" ] && [ "$ph" -eq "$ih" ] \
        || fail "$CASE: calibrated probe ${pw}x${ph}, the case's probe is ${iw}x${ih} over IPC"
    [ "$pw" -gt 440 ] && [ "$ph" -gt 240 ] || fail "$CASE: probe ${pw}x${ph} is too small for the screencast crops"
}
# Crops in output pixels (scale 1): the probe's top text rows, and glass
# below its few lines with Aurora alone. Every sample is a frame of the
# calibrated screen, or the crops would land elsewhere.
post_screencast() {
    local k w h px py pw ph sw sh
    read -r px py pw ph < "$OUT/probe-rect-drm.txt"
    read -r sw sh < "$OUT/screen-drm.txt"
    for k in 1 2 3; do
        w=$(jq -r .width "$CASE_DIR/sample-$k.raw.json"); h=$(jq -r .height "$CASE_DIR/sample-$k.raw.json")
        [ "$w" = "$sw" ] && [ "$h" = "$sh" ] \
            || fail "$CASE: sample-$k is ${w}x${h}, the calibrated screen is ${sw}x${sh}"
        magick -size "${w}x${h}" -depth 8 "rgb:$CASE_DIR/sample-$k.raw" \
            -crop "$((pw - 20))x80+$((px + 10))+$((py + 10))" +repage -depth 8 rgb:- > "$CASE_DIR/client-$k.rgb"
        magick -size "${w}x${h}" -depth 8 "rgb:$CASE_DIR/sample-$k.raw" \
            -crop "400x200+$((px + 40))+$((py + ph - 240))" +repage -depth 8 rgb:- > "$CASE_DIR/aurora-$k.rgb"
    done
}
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
    start_host "$NIRI" "$CASE_DIR/live.kdl" "$CASE"
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
    local other_cmd=$IDLE
    OTHER_FIFO=
    if [ "$base" = screencast ]; then
        OTHER_FIFO=$RT/other-$CASE.fifo
        mkfifo "$OTHER_FIFO"
        other_cmd="printf '\033[?25l'; exec 3<>'$OTHER_FIFO'; while read -r line <&3; do printf '%s\n' \"\$line\"; done"
    fi
    XDG_RUNTIME_DIR=$RT WAYLAND_DISPLAY=$DISPLAY_NAME \
        kitty --config NONE --class gos-other -o cursor_blink_interval=0 -o mouse_hide_wait=0 sh -c "$other_cmd" >> "$OUT/other.log" 2>&1 &
    OTHER_PID=$!
    for _ in $(seq 100); do [ "$(windows_with "$NIRI" gos-other)" -ge 1 ] && break; sleep 0.1; done
    [ "$(windows_with "$NIRI" gos-other)" -eq 1 ] || fail "$CASE: focus thief never opened"
    [ -n "$no_pointer" ] || pointer
    if declare -F "setup_$fn" > /dev/null; then "setup_$fn"; fi
    [ "$(msg "$NIRI" -j windows | jq -r '.[] | select(.app_id=="gos-probe") | .is_focused')" = false ] \
        || fail "$CASE: the probe is focused"
    [ -n "$no_pointer" ] || pointer
    nap 1
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
    python3 - "$CASE_DIR" "$T0" "$(topology)" <<'PY'
import json, pathlib, sys
directory = pathlib.Path(sys.argv[1])
journal = []
for line in (directory / 'journal.tsv').read_text().splitlines():
    label, start, end = line.split('\t')
    journal.append(dict(label=label, start_mono_ns=int(start), end_mono_ns=int(end)))
# The capture connected at capture_start_mono_ns and was declared to run
# capture_s from there: the analyzer requires the trace to reach that end.
observation = dict(capture_start_mono_ns=int(sys.argv[2]), journal=journal, topology=json.loads(sys.argv[3]))
(directory / 'observation.json').write_text(json.dumps(observation, indent=2) + '\n')
PY
    stop_host
    for pid in "$BG_PID" "$PROBE_PID" "$OTHER_PID" "$INHIBIT_PID" "$CAST_PID" "$LOCK_PID"; do [ -z "$pid" ] || reap "$pid"; done
    BG_PID=; PROBE_PID=; OTHER_PID=; INHIBIT_PID=; CAST_PID=; LOCK_PID=
    rm -f "$FIFO"
    [ -z "$OTHER_FIFO" ] || rm -f "$OTHER_FIFO"
    await_gpu_rest
}

if [ "$LANE" = dedicated ] && selected screencast; then calibrate_drm_probe; fi
for name in "${RUN_CASES[@]}"; do
    echo "case $name" >&2
    run_case "$name"
done
python3 "$ROOT/tools/optic_settling.py" "$OUT"
