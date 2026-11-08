#!/usr/bin/env bash
# regs.sh <noise.frag> <label>: register and instruction count of the material
# fragment program with that noise optic, on this host's NVIDIA driver.
# Env: WT (a checkout carrying program_probe.rs), BASE (a full material
# source, e.g. the probe's PROBE_OUT.frag), RUNS (output dir). Needs zstd
# and CUDA's nvdisasm.
set -eu
P=$(dirname "$(readlink -f "$0")")
WT=${WT:?checkout with the probe test}
BASE=${BASE:?full material source to swap the noise optic into}
RUNS=${RUNS:?output directory}
noise=$(readlink -f "$1"); label=$2
work=$RUNS/$label; rm -rf "$work"; mkdir -p "$work/cache"
python3 -I "$P/swap.py" "$BASE" "$noise" "$work/src.frag"
(cd "$WT" && __GL_SHADER_DISK_CACHE=1 __GL_SHADER_DISK_CACHE_SKIP_CLEANUP=1 __GL_SHADER_DISK_CACHE_PATH=$work/cache \
  PROBE_SRC=$work/src.frag PROBE_OUT=$work/out just test-one -p niri --run-ignored only dump_material_program_binary >"$work/test.log" 2>&1) \
  || { tail -20 "$work/test.log"; exit 1; }
bin=$(find "$work/cache" -name '*.bin')
python3 -I - "$bin" "$work/frag.nvuc" <<'PY'
import sys, re, subprocess
b = open(sys.argv[1], "rb").read()
offs = [m.start() for m in re.finditer(rb"\x28\xb5\x2f\xfd", b)] + [len(b)]
for o, e in zip(offs, offs[1:]):
    r = subprocess.run(["zstd", "-d", "-c", "-q"], input=b[o:e], capture_output=True)
    if r.stdout.startswith(b"NVuc") and len(r.stdout) > 20000:
        open(sys.argv[2], "wb").write(r.stdout); break
else:
    sys.exit("no fragment ucode found")
PY
/opt/cuda/bin/nvdisasm -b SM86 "$work/frag.nvuc" > "$work/frag.sass" 2>&1 || true
regs=$(grep -m1 "Register count" "$work/frag.sass" | sed 's/.*=//')
ins=$(python3 -I -c '
import struct, sys
b = open(sys.argv[1], "rb").read()
for i in range(struct.unpack_from("<H", b, 10)[0]):
    typ, size, off = struct.unpack_from("<III", b, 0x20 + i * 0x20)
    if typ == 1 and size:
        print(size // 16); break
' "$work/frag.nvuc")
printf '%-24s regs %4s  instructions %6s  STL %s LDL %s\n' "$label" "$regs" "$ins" "$(grep -c ' STL' "$work/frag.sass" || true)" "$(grep -c ' LDL' "$work/frag.sass" || true)"
