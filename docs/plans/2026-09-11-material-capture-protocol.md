# Material performance-capture protocol: implementation plan

**Status:** code implemented on `material-bae9c9` and companion branch
`niri-experiments/results/capture-protocol`; manual busy-host refusal acceptance
passed after reboot with NVIDIA 615.71.09 (`material-7f7aa3`). Final review corrections
are implemented and the scoped re-review found both findings addressed with no new
blocking issues.

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship `tools/capture-meta`, the helper every material performance capture calls to refuse a busy machine, record provenance, and re-check quietness between sub-runs, and adopt it in the optic smoke lib and the idle-budget fixture.

**Architecture:** One standard-library Python script with injectable `/proc` and `nvidia-smi` readers writes a single `capture.json` per run directory in sections (`run`, `environment`, `baseline`, `preflight`, `provenance`, `sub_runs`). Fixtures call it as a subprocess at fixed points; the smoke lib wraps it in shell functions so every nested compositor launch passes through `settle`. The idle-budget fixture and analyzer move from their own `hardware.json`/`.config.sha256` files to the same record.

**Tech Stack:** Python 3.12 standard library (`json`, `hashlib`, `fcntl`, `subprocess`, `xml.etree`), `unittest`; bash for the fixtures; `nvidia-smi` at runtime only.

**Spec:** `docs/specs/2026-09-11-material-capture-protocol-design.md` (read it first; section numbers below refer to it).

**Task:** `material-bae9c9`. Tasks 1–7 and 10 land in this repository on branch `material-bae9c9` (worktree `.worktrees/material-bae9c9`). Tasks 8–9 land in the sibling `niri-experiments` repository on a new branch `results/capture-protocol` based on `results/idle-budget` at `8db5dc8`, in a new worktree `niri-experiments/.worktrees/material-bae9c9`; the parked `material-265eb0` session fast-forwards `results/idle-budget` onto it when it resumes. Do not touch `niri-experiments/.worktrees/material-265eb0`.

## Global Constraints

- Standard library only in `tools/capture-meta` and its test; no third-party imports (spec §2, §6).
- Exit codes: 0 passed; 1 refused (machine not quiet, a named `--binary`/`--input` path missing, lock held); 2 cannot run (no `nvidia-smi`, unwritable directory, malformed or already-written section, unknown schema) (spec §2).
- `capture.json` has `"schema": 1`; a sub-command refuses to overwrite a section that exists (exit 2) (spec §3).
- Measured keys carry units in the name (`_pct`, `_w`, `_mhz`, `_kib`); hashes are full SHA-256 hex; timestamps RFC 3339 with offset; `host` is the hostname, never a path (spec §3).
- Default thresholds, verbatim (spec §2.4): quiet = CPU busy median ≤ 10 %, load1 ≤ 2.0, available memory ≥ 20 % of `MemTotal` in every sample, GPU utilization median ≤ 5 %, P-state `P8` in every sample, no compute client in any sample (no client of any kind on the `dedicated` lane), GPU power IQR ≤ 1.0 W. Settle tolerance = CPU ± 5 points, available memory within 5 % of `MemTotal` of the baseline and above the floor, GPU utilization ± 3 points, GPU power ± 1.5 W, same P-state and client constraints.
- Preflight default window 20 s; settle default 10 s; 1 Hz sampling (spec §2.1, §2.3).
- The lock is `$XDG_RUNTIME_DIR/capture-meta.lock`; reclamation happens under `flock` on `$XDG_RUNTIME_DIR/capture-meta.lock.d`; `release` removes only a lock naming this run id and owner PID (spec §2.1).
- The smoke lib's top level never calls preflight; entry scripts do (spec §5).
- No hardware capture run is part of this plan (spec §7). Tests run offline with fake readers or a fake `nvidia-smi` on `PATH`.
- Test files in `tools/` scrub `GIT_*` from the environment at import (see `tools/test_package_pin.py`) because the pre-commit hook runs the suite.
- Run tests through the tracked front doors, never bare and never piped into `tail` (a pipe without `pipefail` hides the exit status): a single tooling module runs as `python3 tools/tt material-bae9c9-tools -- python3 -m unittest tools.test_capture_meta -v` (or `tools.test_glass_optic_smoke`), the repository gate is `just check`, and the experiments suite is `just --justfile fixtures/idle-budget.just test` with `MATERIAL_ROOT` exported. `tt` records the run and preserves the exit status. Read the last lines from the terminal; do not truncate them away.
- Commit messages: conventional commits, no attribution trailers. Run `tasks note material-bae9c9 "<one line>"` when scope shifts; `tasks check` before each commit is part of the hook.
- The pre-commit hook regenerates nothing: when it says `upstream-divergence.md is stale`, run `just upstream-report` and stage the result.

---

## File structure

| File | Responsibility |
| --- | --- |
| `tools/capture-meta` (new) | The whole helper: record I/O, samplers, quietness judgement, lock, the five sub-commands, `main()`. One file, like its siblings `package-pin` and `upstream-report`. |
| `tools/test_capture_meta.py` (new) | Unit tests with injected readers; one integration test with a fake `nvidia-smi`. |
| `docs/materials/scripts/glass-optic-smoke-lib.sh` (modify) | `capture_meta` wrapper, `capture_preflight`, `settle_before_launch`, `start_nested` third argument, `cleanup` release, `build_binaries` without loose files, `finish` hashing everything. |
| `docs/materials/scripts/glass-aurora-smoke.sh`, `glass-iridescence-smoke.sh` (modify) | `capture_preflight headless` after sourcing; `identity` after `build_binaries`. |
| `tools/test_glass_optic_smoke.py` (modify) | Cases for `settle_before_launch`, `start_nested`'s first statement, no preflight at lib top level, `finish` coverage. |
| `niri-experiments/fixtures/idle-budget.sh` (modify) | Preflight per mode after lib sourcing, per-mode `identity`, `start_scene` naming, drop `hardware.*`, tool versions, `IDLE_BUDGET_DEDICATED_SESSION`, `.config.sha256`. |
| `niri-experiments/fixtures/idle-budget.py` (modify) | `analyze` and `scene_evidence` read `capture.json`. |
| `niri-experiments/fixtures/test_idle_budget.py` (modify) | Synthetic matrices write `capture.json`; new rejection cases; shell tests for call order. |
| `niri-experiments/docs/results/2026-09-11-idle-budget.md`, `.sha256` (modify) | Status line and regenerated manifest. |
| `docs/materials/README.md`, `docs/specs/2026-09-11-material-capture-protocol-design.md` (modify) | Pointer to the protocol; spec status. |

Internal names used across tasks (defined in Task 1–3, consumed later):

```python
SCHEMA = 1
class Refused(Exception): ...        # exit 1
class CannotRun(Exception): ...      # exit 2
def load_record(run_dir: Path) -> dict            # {} when absent; CannotRun on malformed/unknown schema
def write_section(run_dir: Path, name: str, value) -> None   # CannotRun if section exists
def append_sub_run(run_dir: Path, entry: dict) -> None
@dataclass class Sample: cpu_busy_pct, load1, mem_available_kib, mem_total_kib, gpu_util_pct, gpu_power_w, gpu_clock_mhz, gpu_pstate, gpu_clients  # gpu_clients = {"compute": [...], "graphics": [...]}
class ProcReader:  cpu_ticks() -> (busy: int, total: int); load1() -> float; meminfo() -> (available_kib: int, total_kib: int)
class GpuReader:   query() -> dict(util_pct, power_w, clock_mhz, pstate); clients() -> {"compute": [...], "graphics": [...]}; static() -> dict(name, uuid, driver)
def sample_stream(proc, gpu, seconds, sleep=time.sleep) -> list[Sample]
def summarize(samples) -> dict
DEFAULT_THRESHOLDS: dict
def judge_quiet(summary, samples, thresholds, lane) -> list[str]
def judge_settled(summary, samples, baseline, thresholds, lane) -> list[str]
def acquire_lock(path, owner_pid, run_id, alive=pid_alive) -> dict     # Refused when held by a live PID
def release_lock(path, owner_pid, run_id) -> bool
```

---

### Task 1: Record I/O, CLI skeleton, and `show`

**Files:**
- Create: `tools/capture-meta`
- Create: `tools/test_capture_meta.py`

**Interfaces:**
- Produces: `SCHEMA`, `Refused`, `CannotRun`, `load_record`, `write_section`, `append_sub_run`, `render(record) -> str`, `main(argv) -> int` with sub-command dispatch table `COMMANDS`.

- [x] **Step 1: Write the failing tests**

```python
"""Unit tests for tools/capture-meta: `python3 -m unittest discover -s tools`."""
import importlib.machinery
import importlib.util
import json
import os
import pathlib
import tempfile
import unittest

for _name in [_key for _key in os.environ if _key.startswith("GIT_")]:
    del os.environ[_name]

spec = importlib.util.spec_from_loader(
    "capture_meta",
    importlib.machinery.SourceFileLoader(
        "capture_meta", str(pathlib.Path(__file__).with_name("capture-meta"))))
cm = importlib.util.module_from_spec(spec)
spec.loader.exec_module(cm)


class RecordTests(unittest.TestCase):
    def test_write_section_creates_record_and_refuses_rewrite(self):
        with tempfile.TemporaryDirectory() as directory:
            run = pathlib.Path(directory)
            self.assertEqual(cm.load_record(run), {})
            cm.write_section(run, "run", {"id": "trace-1"})
            record = json.loads((run / "capture.json").read_text())
            self.assertEqual(record["schema"], cm.SCHEMA)
            self.assertEqual(record["run"], {"id": "trace-1"})
            with self.assertRaises(cm.CannotRun):
                cm.write_section(run, "run", {"id": "other"})
            self.assertEqual(cm.load_record(run)["run"], {"id": "trace-1"})

    def test_append_sub_run_accumulates_in_order(self):
        with tempfile.TemporaryDirectory() as directory:
            run = pathlib.Path(directory)
            cm.append_sub_run(run, {"name": "a", "verdict": "settled"})
            cm.append_sub_run(run, {"name": "b", "verdict": "refused"})
            self.assertEqual([e["name"] for e in cm.load_record(run)["sub_runs"]], ["a", "b"])

    def test_load_record_rejects_malformed_and_unknown_schema(self):
        with tempfile.TemporaryDirectory() as directory:
            run = pathlib.Path(directory)
            (run / "capture.json").write_text("{not json")
            with self.assertRaises(cm.CannotRun):
                cm.load_record(run)
            (run / "capture.json").write_text(json.dumps({"schema": 99}))
            with self.assertRaises(cm.CannotRun):
                cm.load_record(run)


class ShowTests(unittest.TestCase):
    def test_render_lists_environment_provenance_and_verdicts(self):
        record = {"schema": 1,
                  "run": {"id": "trace-1", "task": "material-x", "fixture": "f.sh", "lane": "headless",
                          "started": "2026-09-11T04:34:33-04:00", "host": "box"},
                  "environment": {"kernel": "7.2.2", "cpu": "Threadripper", "cpu_threads": 32,
                                  "memory_total_kib": 1000,
                                  "gpu": {"name": "RTX", "uuid": "GPU-1", "driver": "610", "device": "/dev/dri/renderD128"},
                                  "session": {"type": "wayland", "display": "wayland-1"}, "tools": {"weston": "15"}},
                  "baseline": {"cpu_busy_pct": 1.0, "gpu_power_w": 18.0},
                  "preflight": {"verdict": "quiet", "thresholds": {}},
                  "provenance": {"source": {"commit": "abc", "branch": "main", "dirty": False},
                                 "binaries": [{"name": "niri", "sha256": "00"}], "inputs": [], "config": {"preset": "aurora"}},
                  "sub_runs": [{"name": "gpu-plain-1", "verdict": "settled"},
                               {"name": "gpu-aurora-1", "verdict": "refused", "reason": "gpu_power_w 24.1 exceeds baseline"}]}
        text = cm.render(record)
        for needle in ("trace-1", "RTX", "610", "abc", "niri 00", "gpu-plain-1: settled",
                       "gpu-aurora-1: refused (gpu_power_w 24.1 exceeds baseline)", "preset=aurora"):
            self.assertIn(needle, text)

    def test_main_show_exit_codes(self):
        with tempfile.TemporaryDirectory() as directory:
            run = pathlib.Path(directory)
            self.assertEqual(cm.main(["show", directory]), 2)          # no record
            cm.write_section(run, "run", {"id": "x", "task": "t", "fixture": "f", "lane": "headless",
                                          "started": "now", "host": "h"})
            self.assertEqual(cm.main(["show", directory]), 0)
```

- [x] **Step 2: Run tests to verify they fail**

Run: `cd .worktrees/material-bae9c9 && python3 tools/tt material-bae9c9-tools -- python3 -m unittest tools.test_capture_meta -v`
Expected: `FileNotFoundError` loading `tools/capture-meta` (module missing).

- [x] **Step 3: Write the skeleton**

```python
#!/usr/bin/env python3
"""Provenance, environment, and quietness record for material performance captures.

    tools/capture-meta preflight <run-dir> --lane headless|dedicated --task ID --fixture NAME
                                 [--seconds N] [--owner-pid PID] [--threshold KEY=VALUE]... [--tool NAME[=VERSION]]...
    tools/capture-meta identity  <run-dir> --source <checkout> [--binary PATH]... [--input PATH]... [--config KEY=VALUE]...
    tools/capture-meta settle    <run-dir> --sub-run NAME [--input PATH]... [--seconds N]
    tools/capture-meta release   <run-dir>
    tools/capture-meta show      <run-dir>

Every capture writes one capture.json per run directory (spec:
docs/specs/2026-09-11-material-capture-protocol-design.md). Sections are written by
the sub-command that owns them and never rewritten. Exits 0 passed, 1 refused (the
machine is not quiet, a named path is missing, the lock is held), 2 cannot run.

Standard library only.
"""
import argparse
import json
import pathlib
import sys

SCHEMA = 1
RECORD = "capture.json"


class Refused(Exception):
    """Exit 1: the run must not proceed, and the record says why."""


class CannotRun(Exception):
    """Exit 2: the helper itself could not do its job."""


def load_record(run_dir):
    path = pathlib.Path(run_dir) / RECORD
    if not path.exists():
        return {}
    try:
        record = json.loads(path.read_text())
    except json.JSONDecodeError as error:
        raise CannotRun(f"{path}: malformed: {error}") from error
    if record.get("schema") != SCHEMA:
        raise CannotRun(f"{path}: schema {record.get('schema')!r}, expected {SCHEMA}")
    return record


def save_record(run_dir, record):
    path = pathlib.Path(run_dir) / RECORD
    record["schema"] = SCHEMA
    try:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(record, indent=2, sort_keys=False) + "\n")
    except OSError as error:
        raise CannotRun(f"{path}: {error}") from error


def write_section(run_dir, name, value):
    record = load_record(run_dir)
    if name in record:
        raise CannotRun(f"{RECORD} already has a {name!r} section; use a fresh run directory")
    record[name] = value
    save_record(run_dir, record)


def append_sub_run(run_dir, entry):
    record = load_record(run_dir)
    record.setdefault("sub_runs", []).append(entry)
    save_record(run_dir, record)


def render(record):
    lines = []
    run = record.get("run", {})
    lines.append(f"run {run.get('id')}  task {run.get('task')}  fixture {run.get('fixture')}  lane {run.get('lane')}")
    lines.append(f"started {run.get('started')}  host {run.get('host')}")
    env = record.get("environment")
    if env:
        gpu = env.get("gpu", {})
        lines += ["", "environment",
                  f"  kernel {env.get('kernel')}",
                  f"  cpu {env.get('cpu')} ({env.get('cpu_threads')} threads)",
                  f"  memory {env.get('memory_total_kib')} KiB",
                  f"  gpu {gpu.get('name')} {gpu.get('driver')} {gpu.get('device')}",
                  f"  session {env.get('session', {}).get('type')} {env.get('session', {}).get('display')}"]
        for tool, version in sorted(env.get("tools", {}).items()):
            lines.append(f"  {tool} {version}")
    baseline = record.get("baseline")
    if baseline:
        lines += ["", "baseline"] + [f"  {key} {value}" for key, value in baseline.items()]
    preflight = record.get("preflight")
    if preflight:
        lines += ["", f"preflight {preflight.get('verdict')}"]
        for reason in preflight.get("reasons", []):
            lines.append(f"  {reason}")
    prov = record.get("provenance")
    if prov:
        source = prov.get("source", {})
        lines += ["", "provenance",
                  f"  source {source.get('commit')} {source.get('branch')}{' dirty' if source.get('dirty') else ''}"]
        for binary in prov.get("binaries", []):
            lines.append(f"  binary {binary['name']} {binary['sha256']}")
        for item in prov.get("inputs", []):
            lines.append(f"  input {item['name']} {item['sha256']}")
        for key, value in prov.get("config", {}).items():
            lines.append(f"  {key}={value}")
    subs = record.get("sub_runs")
    if subs:
        lines += ["", "sub-runs"]
        for entry in subs:
            reason = f" ({entry['reason']})" if entry.get("reason") else ""
            lines.append(f"  {entry['name']}: {entry['verdict']}{reason}")
    return "\n".join(lines) + "\n"


def cmd_show(args):
    record = load_record(args.run_dir)
    if not record:
        raise CannotRun(f"no {RECORD} in {args.run_dir}")
    sys.stdout.write(render(record))


def build_parser():
    parser = argparse.ArgumentParser(prog="capture-meta")
    subs = parser.add_subparsers(dest="command", required=True)
    show = subs.add_parser("show")
    show.add_argument("run_dir")
    show.set_defaults(func=cmd_show)
    return parser


def main(argv=None):
    parser = build_parser()
    args = parser.parse_args(argv)
    try:
        args.func(args)
    except Refused as error:
        print(f"capture-meta: refused: {error}", file=sys.stderr)
        return 1
    except CannotRun as error:
        print(f"capture-meta: {error}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())
```

`chmod +x tools/capture-meta`. Later tasks add sub-parsers to `build_parser` and their `cmd_*` functions; keep that pattern.

- [x] **Step 4: Run tests to verify they pass**

Run: `python3 tools/tt material-bae9c9-tools -- python3 -m unittest tools.test_capture_meta -v`
Expected: 5 tests OK.

- [x] **Step 5: Commit**

```bash
git add tools/capture-meta tools/test_capture_meta.py
git commit -m "feat(tools): capture-meta record I/O and show"
```

---

### Task 2: Samplers, summary, and quietness judgement

**Files:**
- Modify: `tools/capture-meta`
- Modify: `tools/test_capture_meta.py`

**Interfaces:**
- Produces: `Sample`, `ProcReader`, `GpuReader`, `sample_stream`, `summarize`, `DEFAULT_THRESHOLDS`, `parse_thresholds(pairs) -> dict`, `judge_quiet`, `judge_settled`, `median`, `iqr`.

- [x] **Step 1: Write the failing tests**

Append to `tools/test_capture_meta.py`:

```python
class FakeProc:
    """Feeds one CPU tick pair, load, and meminfo per read; wraps around."""
    def __init__(self, ticks, load1=0.5, available_kib=80_000, total_kib=100_000):
        self.ticks = list(ticks); self.i = 0
        self._load1 = load1; self.available_kib = available_kib; self.total_kib = total_kib
    def cpu_ticks(self):
        busy, total = self.ticks[min(self.i, len(self.ticks) - 1)]; self.i += 1
        return busy, total
    def load1(self):
        return self._load1
    def meminfo(self):
        return self.available_kib, self.total_kib


class FakeGpu:
    def __init__(self, queries, clients=None, static=None):
        self.queries = list(queries); self.i = 0
        self._clients = clients or {"compute": [], "graphics": ["niri"]}
        self._static = static or {"name": "NVIDIA test", "uuid": "GPU-1", "driver": "610"}
    def query(self):
        value = self.queries[min(self.i, len(self.queries) - 1)]; self.i += 1
        return dict(value)
    def clients(self):
        return {k: list(v) for k, v in self._clients.items()}
    def static(self):
        return dict(self._static)


QUIET = {"util_pct": 0.0, "power_w": 18.4, "clock_mhz": 360, "pstate": "P8"}


def quiet_samples(n=5, **overrides):
    proc = FakeProc([(i * 2, i * 100) for i in range(n + 1)])          # 2 % busy
    gpu = FakeGpu([dict(QUIET, **overrides)] * n)
    return cm.sample_stream(proc, gpu, n, sleep=lambda s: None)


class SamplingTests(unittest.TestCase):
    def test_sample_stream_derives_busy_pct_from_tick_deltas(self):
        samples = quiet_samples(3)
        self.assertEqual(len(samples), 3)
        self.assertEqual([round(s.cpu_busy_pct, 1) for s in samples], [2.0, 2.0, 2.0])
        self.assertEqual(samples[0].gpu_pstate, "P8")
        self.assertEqual(samples[0].mem_total_kib, 100_000)

    def test_summarize_reports_medians_iqr_pstates_and_client_union(self):
        proc = FakeProc([(0, 0), (2, 100), (4, 200), (6, 300)])
        gpu = FakeGpu([dict(QUIET, power_w=18.0), dict(QUIET, power_w=19.0), dict(QUIET, power_w=25.0, pstate="P0")])
        summary = cm.summarize(cm.sample_stream(proc, gpu, 3, sleep=lambda s: None))
        self.assertEqual(summary["gpu_power_w"], 19.0)
        self.assertEqual(summary["gpu_power_iqr_w"], 3.5)
        self.assertEqual(summary["gpu_pstates"], ["P0", "P8"])
        self.assertEqual(summary["gpu_clients"], {"compute": [], "graphics": ["niri"]})
        self.assertEqual(summary["mem_available_pct"], 80.0)
        self.assertEqual(summary["samples"], 3)


class GpuEvidenceTests(unittest.TestCase):
    XML = '<nvidia_smi_log><gpu><processes>{}</processes></gpu></nvidia_smi_log>'
    ENTRY = '<process_info><type>{}</type><process_name>/usr/bin/{}</process_name></process_info>'

    def test_mixed_type_counts_as_compute_and_graphics_stays_graphics(self):
        xml = self.XML.format(self.ENTRY.format("G", "niri") + self.ENTRY.format("C+G", "blender") + self.ENTRY.format("C", "python3"))
        self.assertEqual(cm.parse_clients(xml), {"compute": ["blender", "python3"], "graphics": ["niri"]})

    def test_missing_or_unsupported_inventory_cannot_run(self):
        for body in ('<nvidia_smi_log><gpu></gpu></nvidia_smi_log>',
                     self.XML.format("N/A"),
                     '<nvidia_smi_log><gpu><processes/></gpu><gpu><processes/></gpu></nvidia_smi_log>',
                     self.XML.format('<process_info><type>X</type><process_name>a</process_name></process_info>')):
            with self.subTest(body=body), self.assertRaises(cm.CannotRun):
                cm.parse_clients(body)
        self.assertEqual(cm.parse_clients(self.XML.format("")), {"compute": [], "graphics": []})

    def test_nonfinite_and_out_of_range_telemetry_cannot_run(self):
        for text in ("nan", "inf", "N/A", "[Not Supported]", ""):
            with self.subTest(text=text), self.assertRaises(cm.CannotRun):
                cm.parse_number(text, "utilization.gpu")
        good = {"util_pct": 0.0, "power_w": 18.4, "clock_mhz": 360.0, "pstate": "P8"}
        self.assertEqual(cm.validate_telemetry(dict(good))["clock_mhz"], 360)
        for bad in ({"util_pct": 101.0}, {"power_w": 0.0}, {"clock_mhz": -1.0}, {"pstate": "X8"}):
            with self.subTest(bad=bad), self.assertRaises(cm.CannotRun):
                cm.validate_telemetry({**good, **bad})


class JudgementTests(unittest.TestCase):
    def judge(self, lane="headless", **kw):
        samples = quiet_samples(5, **{k: v for k, v in kw.items() if k in QUIET})
        for key in ("load1", "available_kib"):
            if key in kw:
                for s in samples:
                    setattr(s, "load1" if key == "load1" else "mem_available_kib", kw[key])
        if "compute" in kw:
            for s in samples:
                s.gpu_clients = {"compute": kw["compute"], "graphics": ["niri"]}
        return cm.judge_quiet(cm.summarize(samples), samples, cm.DEFAULT_THRESHOLDS, lane)

    def test_quiet_baseline_passes(self):
        self.assertEqual(self.judge(), [])

    def test_each_threshold_refuses_and_names_the_field(self):
        self.assertIn("gpu_util_pct", " ".join(self.judge(util_pct=35.0)))
        self.assertIn("gpu_pstate", " ".join(self.judge(pstate="P0")))
        self.assertIn("load1", " ".join(self.judge(load1=4.5)))
        self.assertIn("mem_available_pct", " ".join(self.judge(available_kib=10_000)))

    def test_compute_client_refuses_graphics_only_on_dedicated(self):
        self.assertIn("compute", " ".join(self.judge(compute=["python3"])))
        self.assertEqual(self.judge(), [])                              # graphics client "niri" fine on headless
        self.assertIn("gpu_clients", " ".join(self.judge(lane="dedicated")))

    def test_settle_tolerances(self):
        base = cm.summarize(quiet_samples(5))
        ok = cm.summarize(quiet_samples(3, power_w=19.5))
        self.assertEqual(cm.judge_settled(ok, quiet_samples(3, power_w=19.5), base, cm.DEFAULT_THRESHOLDS, "headless"), [])
        hot = quiet_samples(3, power_w=24.1)
        reasons = cm.judge_settled(cm.summarize(hot), hot, base, cm.DEFAULT_THRESHOLDS, "headless")
        self.assertIn("gpu_power_w 24.1 exceeds baseline 18.4 by more than 1.5", " ".join(reasons))
        dropped = quiet_samples(3)
        for s in dropped:
            s.mem_available_kib = 70_000                                  # 10 % of total below the 80 % baseline
        self.assertIn("mem_available_pct", " ".join(cm.judge_settled(cm.summarize(dropped), dropped, base, cm.DEFAULT_THRESHOLDS, "headless")))

    def test_parse_thresholds_overrides_defaults(self):
        thresholds = cm.parse_thresholds(["cpu_busy_pct=20", "gpu_pstate=P5"])
        self.assertEqual(thresholds["cpu_busy_pct"], 20.0)
        self.assertEqual(thresholds["gpu_pstate"], "P5")
        self.assertEqual(thresholds["load1"], cm.DEFAULT_THRESHOLDS["load1"])
        with self.assertRaises(cm.CannotRun):
            cm.parse_thresholds(["nonsense=1"])
```

- [x] **Step 2: Run tests to verify they fail**

Run: `python3 tools/tt material-bae9c9-tools -- python3 -m unittest tools.test_capture_meta -v`
Expected: `AttributeError: module 'capture_meta' has no attribute 'sample_stream'`.

- [x] **Step 3: Implement**

Add to `tools/capture-meta` after the record functions (imports: `dataclasses`, `math`, `re`, `statistics`, `subprocess`, `time`, `xml.etree.ElementTree as ET`, `os`, `shutil`):

```python
@dataclasses.dataclass
class Sample:
    cpu_busy_pct: float
    load1: float
    mem_available_kib: int
    mem_total_kib: int
    gpu_util_pct: float
    gpu_power_w: float
    gpu_clock_mhz: int
    gpu_pstate: str
    gpu_clients: dict


class ProcReader:
    """The three /proc facts a sample needs. CAPTURE_META_PROC points the reader at a
    directory of fake stat/loadavg/meminfo files so the end-to-end test does not depend
    on the test machine being quiet; unset, it is /proc."""

    def __init__(self, root=None):
        self.root = pathlib.Path(root or os.environ.get("CAPTURE_META_PROC", "/proc"))

    def cpu_ticks(self):
        fields = (self.root / "stat").read_text().splitlines()[0].split()[1:]
        values = [int(v) for v in fields]
        idle = values[3] + values[4]            # idle + iowait
        total = sum(values[:8])                 # through steal; guest is already in user/nice
        return total - idle, total

    def load1(self):
        return float((self.root / "loadavg").read_text().split()[0])

    def meminfo(self):
        info = {}
        for line in (self.root / "meminfo").read_text().splitlines():
            key, _, rest = line.partition(":")
            info[key] = int(rest.split()[0])
        return info["MemAvailable"], info["MemTotal"]


class GpuReader:
    """nvidia-smi: the only sampler today. Another vendor means another class."""

    QUERY = ("nvidia-smi", "--query-gpu=utilization.gpu,power.draw,clocks.gr,pstate",
             "--format=csv,noheader,nounits")
    STATIC = ("nvidia-smi", "--query-gpu=name,uuid,driver_version", "--format=csv,noheader")

    def __init__(self):
        if shutil.which("nvidia-smi") is None:
            raise CannotRun("nvidia-smi not found; no GPU sampler for this host")

    def _run(self, *command):
        result = subprocess.run(command, capture_output=True, text=True, check=False)
        if result.returncode != 0:
            raise CannotRun(f"{' '.join(command)}: {result.stderr.strip()}")
        return result.stdout

    def query(self):
        rows = [r.strip() for r in self._run(*self.QUERY).strip().splitlines()]
        if len(rows) != 1:
            raise CannotRun(f"exactly one GPU expected, nvidia-smi listed {len(rows)}")
        util, power, clock, pstate = [x.strip() for x in rows[0].split(",")]
        return validate_telemetry({"util_pct": parse_number(util, "utilization.gpu"),
                                   "power_w": parse_number(power, "power.draw"),
                                   "clock_mhz": parse_number(clock, "clocks.gr"), "pstate": pstate})

    def clients(self):
        # Local tool output, not untrusted input: the stdlib parser is fine here.
        return parse_clients(self._run("nvidia-smi", "-q", "-x"))

    def static(self):
        name, uuid, driver = [x.strip() for x in self._run(*self.STATIC).strip().split(",")]
        return {"name": name, "uuid": uuid, "driver": driver}


def parse_number(text, field):
    """nvidia-smi prints `N/A`, `[Not Supported]`, or nothing when a field is unavailable;
    none of those is a measurement, and a NaN would slip past every `>` below."""
    try:
        value = float(text)
    except ValueError as error:
        raise CannotRun(f"nvidia-smi {field}: {text!r} is not a number") from error
    if not math.isfinite(value):
        raise CannotRun(f"nvidia-smi {field}: {text!r} is not finite")
    return value


def validate_telemetry(q):
    if not 0.0 <= q["util_pct"] <= 100.0:
        raise CannotRun(f"utilization {q['util_pct']} outside 0-100")
    if not 0.0 < q["power_w"] < 2000.0:
        raise CannotRun(f"power {q['power_w']} W outside the plausible range")
    if not 0 < q["clock_mhz"] < 10000:
        raise CannotRun(f"graphics clock {q['clock_mhz']} MHz outside the plausible range")
    if not re.fullmatch(r"P\d{1,2}", q["pstate"]):
        raise CannotRun(f"pstate {q['pstate']!r} is not a P-state")
    q["clock_mhz"] = int(q["clock_mhz"])
    return q


def parse_clients(xml_text):
    """One GPU with a complete process inventory. `<processes>` absent, or carrying text
    like `N/A`, means the driver did not report — that is CannotRun, never an empty list.
    Mixed types (`C+G`, `G+C`) count as compute: they have compute work by definition."""
    root = ET.fromstring(xml_text)
    gpus = root.findall("gpu")
    if len(gpus) != 1:
        raise CannotRun(f"exactly one GPU expected in nvidia-smi -q -x, found {len(gpus)}")
    processes = gpus[0].find("processes")
    if processes is None or (processes.text or "").strip():
        raise CannotRun("nvidia-smi -q -x has no complete process inventory")
    clients = {"compute": [], "graphics": []}
    for info in processes.findall("process_info"):
        kind = (info.findtext("type") or "").strip()
        name = pathlib.PurePath((info.findtext("process_name") or "").strip()).name
        if kind not in ("G", "C", "C+G", "G+C") or not name:
            raise CannotRun(f"nvidia-smi process entry malformed: type={kind!r} name={name!r}")
        clients["graphics" if kind == "G" else "compute"].append(name)
    return clients


def sample_stream(proc, gpu, seconds, sleep=time.sleep):
    samples = []
    busy0, total0 = proc.cpu_ticks()
    for _ in range(seconds):
        sleep(1)
        busy1, total1 = proc.cpu_ticks()
        delta = total1 - total0
        cpu = 100.0 * (busy1 - busy0) / delta if delta else 0.0
        busy0, total0 = busy1, total1
        available, total = proc.meminfo()
        q = gpu.query()
        samples.append(Sample(cpu, proc.load1(), available, total, q["util_pct"], q["power_w"],
                              q["clock_mhz"], q["pstate"], gpu.clients()))
    return samples


def median(values):
    return float(statistics.median(values))


def iqr(values):
    quartiles = statistics.quantiles(sorted(values), n=4, method="inclusive")
    return round(quartiles[2] - quartiles[0], 3)


def summarize(samples):
    total = samples[0].mem_total_kib
    clients = {"compute": sorted({c for s in samples for c in s.gpu_clients["compute"]}),
               "graphics": sorted({c for s in samples for c in s.gpu_clients["graphics"]})}
    return {
        "samples": len(samples),
        "cpu_busy_pct": round(median([s.cpu_busy_pct for s in samples]), 1),
        "load1": round(median([s.load1 for s in samples]), 2),
        "mem_available_kib": int(median([s.mem_available_kib for s in samples])),
        "mem_available_pct": round(100.0 * median([s.mem_available_kib for s in samples]) / total, 1),
        "gpu_util_pct": round(median([s.gpu_util_pct for s in samples]), 1),
        "gpu_power_w": round(median([s.gpu_power_w for s in samples]), 3),
        "gpu_power_iqr_w": iqr([s.gpu_power_w for s in samples]) if len(samples) > 1 else 0.0,
        "gpu_clock_mhz": int(median([s.gpu_clock_mhz for s in samples])),
        "gpu_pstates": sorted({s.gpu_pstate for s in samples}),
        "gpu_clients": clients,
    }


DEFAULT_THRESHOLDS = {
    "cpu_busy_pct": 10.0, "load1": 2.0, "mem_available_pct_min": 20.0,
    "gpu_util_pct": 5.0, "gpu_pstate": "P8", "gpu_power_iqr_w": 1.0,
    "settle_cpu_busy_pct": 5.0, "settle_mem_pct": 5.0, "settle_gpu_util_pct": 3.0, "settle_gpu_power_w": 1.5,
}


def parse_thresholds(pairs):
    thresholds = dict(DEFAULT_THRESHOLDS)
    for pair in pairs or []:
        key, sep, value = pair.partition("=")
        if not sep or key not in thresholds:
            raise CannotRun(f"--threshold {pair!r}: unknown key; choose from {', '.join(sorted(thresholds))}")
        thresholds[key] = value if key == "gpu_pstate" else float(value)
    return thresholds


def client_reasons(samples, lane):
    reasons = []
    compute = sorted({c for s in samples for c in s.gpu_clients["compute"]})
    if compute:
        reasons.append(f"compute clients present: {', '.join(compute)}")
    if lane == "dedicated":
        graphics = sorted({c for s in samples for c in s.gpu_clients["graphics"]})
        if graphics:
            reasons.append(f"gpu_clients present on the dedicated lane: {', '.join(graphics)}")
    return reasons


def judge_quiet(summary, samples, thresholds, lane):
    reasons = []
    if summary["cpu_busy_pct"] > thresholds["cpu_busy_pct"]:
        reasons.append(f"cpu_busy_pct {summary['cpu_busy_pct']} exceeds {thresholds['cpu_busy_pct']}")
    if summary["load1"] > thresholds["load1"]:
        reasons.append(f"load1 {summary['load1']} exceeds {thresholds['load1']}")
    floor = thresholds["mem_available_pct_min"]
    low = [s for s in samples if 100.0 * s.mem_available_kib / s.mem_total_kib < floor]
    if low:
        reasons.append(f"mem_available_pct below {floor} in {len(low)} sample(s)")
    if summary["gpu_util_pct"] > thresholds["gpu_util_pct"]:
        reasons.append(f"gpu_util_pct {summary['gpu_util_pct']} exceeds {thresholds['gpu_util_pct']}")
    if summary["gpu_pstates"] != [thresholds["gpu_pstate"]]:
        reasons.append(f"gpu_pstate {summary['gpu_pstates']} not always {thresholds['gpu_pstate']}")
    if summary["gpu_power_iqr_w"] > thresholds["gpu_power_iqr_w"]:
        reasons.append(f"gpu_power_iqr_w {summary['gpu_power_iqr_w']} exceeds {thresholds['gpu_power_iqr_w']}")
    return reasons + client_reasons(samples, lane)


def judge_settled(summary, samples, baseline, thresholds, lane):
    reasons = []
    def drift(key, limit, unit=""):
        delta = summary[key] - baseline[key]
        if abs(delta) > limit:
            verb = "exceeds" if delta > 0 else "falls below"
            reasons.append(f"{key} {summary[key]} {verb} baseline {baseline[key]} by more than {limit}{unit}")
    drift("cpu_busy_pct", thresholds["settle_cpu_busy_pct"])
    drift("mem_available_pct", thresholds["settle_mem_pct"])
    drift("gpu_util_pct", thresholds["settle_gpu_util_pct"])
    drift("gpu_power_w", thresholds["settle_gpu_power_w"])
    floor = thresholds["mem_available_pct_min"]
    if any(100.0 * s.mem_available_kib / s.mem_total_kib < floor for s in samples):
        reasons.append(f"mem_available_pct below floor {floor}")
    if summary["gpu_pstates"] != [thresholds["gpu_pstate"]]:
        reasons.append(f"gpu_pstate {summary['gpu_pstates']} not always {thresholds['gpu_pstate']}")
    return reasons + client_reasons(samples, lane)
```

- [x] **Step 4: Run tests to verify they pass**

Run: `python3 tools/tt material-bae9c9-tools -- python3 -m unittest tools.test_capture_meta -v`
Expected: all OK (15 tests). If `test_settle_tolerances` complains about the memory message, check that `summarize` rounds `mem_available_pct` to one decimal and the baseline is `80.0`.

- [x] **Step 5: Commit**

```bash
git add tools/capture-meta tools/test_capture_meta.py
git commit -m "feat(tools): capture-meta samplers and quietness judgement"
```

---

### Task 3: The lock

**Files:**
- Modify: `tools/capture-meta`
- Modify: `tools/test_capture_meta.py`

**Interfaces:**
- Produces: `lock_path() -> Path` (`$XDG_RUNTIME_DIR/capture-meta.lock`; `CannotRun` when the variable is unset), `pid_alive(pid) -> bool`, `guarded(path)` (context manager holding `flock(LOCK_EX)` on the sibling `<path>.d` directory), `acquire_lock(path, owner_pid, run_id, alive=pid_alive) -> dict`, `read_lock(path) -> dict | None` (`CannotRun` on unreadable contents), `release_lock(path, owner_pid, run_id) -> bool`.
- Every read, create, reclaim, and release of the lock file happens under `guarded(path)`. Publication is atomic: the JSON is written to `<path>.tmp.<pid>` and `os.link`ed to `path` (link fails with `EEXIST` if someone else published first), so no reader can ever see a half-written lock. Corrupt or empty contents are `CannotRun`, never evidence that the owner is dead.

- [x] **Step 1: Write the failing tests**

```python
class LockTests(unittest.TestCase):
    def test_acquire_release_and_ownership_check(self):
        with tempfile.TemporaryDirectory() as directory:
            path = pathlib.Path(directory) / "capture-meta.lock"
            info = cm.acquire_lock(path, owner_pid=111, run_id="run-a", alive=lambda pid: True)
            self.assertEqual(info, {"path": str(path), "owner_pid": 111, "run_id": "run-a", "reclaimed": False})
            self.assertEqual(sorted(p.name for p in path.parent.iterdir()), ["capture-meta.lock", "capture-meta.lock.d"])  # no tmp left behind
            self.assertEqual(cm.read_lock(path)["run_id"], "run-a")
            self.assertFalse(cm.release_lock(path, owner_pid=222, run_id="run-b"))   # not ours: left in place
            self.assertTrue(path.exists())
            self.assertTrue(cm.release_lock(path, owner_pid=111, run_id="run-a"))
            self.assertFalse(path.exists())
            self.assertTrue(cm.release_lock(path, owner_pid=111, run_id="run-a"))    # nothing to release is fine

    def test_half_written_or_corrupt_lock_is_never_reclaimed(self):
        with tempfile.TemporaryDirectory() as directory:
            path = pathlib.Path(directory) / "capture-meta.lock"
            path.write_text("")                       # a writer that died between create and publish, or a torn write
            with self.assertRaises(cm.CannotRun):
                cm.acquire_lock(path, 222, "run-b", alive=lambda pid: False)
            self.assertEqual(path.read_text(), "")
            path.write_text("{not json")
            with self.assertRaises(cm.CannotRun):
                cm.acquire_lock(path, 222, "run-b", alive=lambda pid: False)
            self.assertFalse(cm.release_lock(path, 222, "run-b"))       # not provably ours: left alone

    def test_guard_serializes_acquire_against_a_concurrent_holder(self):
        import fcntl, os, threading, time
        with tempfile.TemporaryDirectory() as directory:
            path = pathlib.Path(directory) / "capture-meta.lock"
            guard = path.with_name(path.name + ".d"); guard.mkdir()
            fd = os.open(guard, os.O_RDONLY); fcntl.flock(fd, fcntl.LOCK_EX)
            done = threading.Event(); result = {}
            def worker():
                result["info"] = cm.acquire_lock(path, 333, "run-c", alive=lambda pid: True); done.set()
            threading.Thread(target=worker, daemon=True).start()
            self.assertFalse(done.wait(0.3))          # blocked behind the guard
            fcntl.flock(fd, fcntl.LOCK_UN); os.close(fd)
            self.assertTrue(done.wait(3.0))
            self.assertEqual(result["info"]["run_id"], "run-c")

    def test_held_by_live_pid_refuses_stale_is_reclaimed(self):
        with tempfile.TemporaryDirectory() as directory:
            path = pathlib.Path(directory) / "capture-meta.lock"
            cm.acquire_lock(path, 111, "run-a", alive=lambda pid: True)
            with self.assertRaises(cm.Refused) as ctx:
                cm.acquire_lock(path, 222, "run-b", alive=lambda pid: True)
            self.assertIn("run-a", str(ctx.exception))
            info = cm.acquire_lock(path, 222, "run-b", alive=lambda pid: pid != 111)
            self.assertEqual(info["reclaimed"], True)
            self.assertEqual(info["previous_owner_pid"], 111)
            self.assertEqual(cm.read_lock(path)["owner_pid"], 222)

    def test_two_processes_racing_admit_exactly_one(self):
        import subprocess, sys
        with tempfile.TemporaryDirectory() as directory:
            path = pathlib.Path(directory) / "capture-meta.lock"
            code = ("import sys, importlib.util, importlib.machinery, pathlib\n"
                    "spec = importlib.util.spec_from_loader('cm', importlib.machinery.SourceFileLoader('cm', sys.argv[1]))\n"
                    "cm = importlib.util.module_from_spec(spec); spec.loader.exec_module(cm)\n"
                    "try:\n    cm.acquire_lock(pathlib.Path(sys.argv[2]), int(sys.argv[3]), sys.argv[3], alive=lambda p: True)\n"
                    "except cm.Refused:\n    sys.exit(1)\n")
            tool = str(pathlib.Path(__file__).with_name("capture-meta"))
            procs = [subprocess.Popen([sys.executable, "-c", code, tool, str(path), str(1000 + i)]) for i in range(8)]
            codes = sorted(p.wait() for p in procs)
            self.assertEqual(codes, [0] + [1] * 7)
```

- [x] **Step 2: Run tests to verify they fail**

Run: `python3 tools/tt material-bae9c9-tools -- python3 -m unittest tools.test_capture_meta.LockTests -v`
Expected: `AttributeError ... acquire_lock`.

- [x] **Step 3: Implement**

```python
import fcntl  # add to imports

LOCK_NAME = "capture-meta.lock"


def lock_path():
    runtime = os.environ.get("XDG_RUNTIME_DIR")
    if not runtime:
        raise CannotRun("XDG_RUNTIME_DIR is unset; the lock has nowhere to live")
    return pathlib.Path(runtime) / LOCK_NAME


def pid_alive(pid):
    try:
        os.kill(pid, 0)
    except ProcessLookupError:
        return False
    except PermissionError:
        return True
    return True


import contextlib  # add to imports


@contextlib.contextmanager
def guarded(path):
    """Every lock-file operation runs under flock on the sibling directory, so create,
    publish, reclaim, and release never interleave."""
    guard = pathlib.Path(path).with_name(pathlib.Path(path).name + ".d")
    guard.mkdir(exist_ok=True)
    fd = os.open(guard, os.O_RDONLY)
    try:
        fcntl.flock(fd, fcntl.LOCK_EX)
        yield
    finally:
        os.close(fd)                        # closing releases the flock


def read_lock(path):
    """None when absent; the holder dict when readable; CannotRun otherwise. Unreadable
    contents are a torn or foreign write, not proof of anything about the owner."""
    path = pathlib.Path(path)
    try:
        text = path.read_text()
    except FileNotFoundError:
        return None
    except OSError as error:
        raise CannotRun(f"lock {path}: {error}") from error
    try:
        holder = json.loads(text)
    except json.JSONDecodeError as error:
        raise CannotRun(f"lock {path} is unreadable ({error}); remove it by hand if no capture is running") from error
    if not isinstance(holder, dict) or not isinstance(holder.get("owner_pid"), int) or not holder.get("run_id"):
        raise CannotRun(f"lock {path} has no owner_pid/run_id; remove it by hand if no capture is running")
    return holder


def _publish_lock(path, owner_pid, run_id):
    """Write beside, then hard-link into place: the link is atomic and fails if a lock exists."""
    tmp = path.with_name(f"{path.name}.tmp.{os.getpid()}")
    tmp.write_text(json.dumps({"owner_pid": owner_pid, "run_id": run_id}))
    try:
        os.link(tmp, path)
    finally:
        tmp.unlink()


def acquire_lock(path, owner_pid, run_id, alive=pid_alive):
    path = pathlib.Path(path)
    with guarded(path):
        holder = read_lock(path)
        if holder is None:
            _publish_lock(path, owner_pid, run_id)
            return {"path": str(path), "owner_pid": owner_pid, "run_id": run_id, "reclaimed": False}
        if alive(holder["owner_pid"]):
            raise Refused(f"lock {path} held by pid {holder['owner_pid']} for run {holder['run_id']}")
        path.unlink()
        _publish_lock(path, owner_pid, run_id)
        return {"path": str(path), "owner_pid": owner_pid, "run_id": run_id, "reclaimed": True,
                "previous_owner_pid": holder["owner_pid"]}


def release_lock(path, owner_pid, run_id):
    path = pathlib.Path(path)
    with guarded(path):
        try:
            holder = read_lock(path)
        except CannotRun:
            return False                    # not provably ours; leave it
        if holder is None:
            return True
        if holder["owner_pid"] != owner_pid or holder["run_id"] != run_id:
            return False
        path.unlink()
        return True
```

- [x] **Step 4: Run tests to verify they pass**

Run: `python3 tools/tt material-bae9c9-tools -- python3 -m unittest tools.test_capture_meta.LockTests -v`
Expected: 5 OK. The race test must show `[0, 1, 1, 1, 1, 1, 1, 1]`; if two zeros appear, an operation is running outside `guarded()`.

- [x] **Step 5: Commit**

```bash
git add tools/capture-meta tools/test_capture_meta.py
git commit -m "feat(tools): capture-meta atomic ownership-checked lock"
```

---

### Task 4: `preflight`

**Files:**
- Modify: `tools/capture-meta`
- Modify: `tools/test_capture_meta.py`

**Interfaces:**
- Consumes: everything from Tasks 1–3.
- Produces: `environment(gpu, tools) -> dict`, `run_section(run_dir, lane, task, fixture) -> dict`, `preflight(run_dir, lane, task, fixture, seconds, owner_pid, thresholds, tools, proc, gpu, sleep, lock=lock_path) -> None` (raises `Refused` after writing the refusal), `cmd_preflight(args)`.
- `--tool NAME` runs `NAME --version` and keeps the first line; `--tool NAME=VERSION` records the given string.

- [x] **Step 1: Write the failing tests**

```python
class PreflightTests(unittest.TestCase):
    def run_preflight(self, run, lane="headless", gpu=None, proc=None, env=None, lock=None, **kw):
        proc = proc or FakeProc([(i * 2, i * 100) for i in range(30)])
        gpu = gpu or FakeGpu([QUIET] * 30)
        saved = dict(os.environ)
        os.environ.update(env or {})
        try:
            return cm.preflight(run, lane=lane, task="material-x", fixture="f.sh", seconds=3, owner_pid=os.getpid(),
                                thresholds=cm.DEFAULT_THRESHOLDS, tools=kw.get("tools", ["tracy=0.13.1"]),
                                proc=proc, gpu=gpu, sleep=lambda s: None, lock=lock or (lambda: run / "lock"))
        finally:
            os.environ.clear(); os.environ.update(saved)

    def test_quiet_headless_writes_run_environment_baseline_preflight(self):
        with tempfile.TemporaryDirectory() as directory:
            run = pathlib.Path(directory) / "trace-20260911T000000"; run.mkdir()
            self.run_preflight(run, env={"XDG_SESSION_TYPE": "wayland", "WAYLAND_DISPLAY": "wayland-1"})
            record = cm.load_record(run)
            self.assertEqual(record["run"]["id"], "trace-20260911T000000")
            self.assertEqual(record["run"]["lane"], "headless")
            self.assertEqual(record["run"]["task"], "material-x")
            self.assertRegex(record["run"]["started"], r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}[+-]\d{2}:\d{2}$")
            self.assertNotIn("/", record["run"]["host"])
            self.assertEqual(record["environment"]["gpu"]["name"], "NVIDIA test")
            self.assertEqual(record["environment"]["tools"]["tracy"], "0.13.1")
            self.assertEqual(record["environment"]["session"], {"type": "wayland", "display": "wayland-1"})
            self.assertEqual(record["baseline"]["samples"], 3)
            self.assertEqual(record["preflight"]["verdict"], "quiet")
            self.assertEqual(record["preflight"]["thresholds"], cm.DEFAULT_THRESHOLDS)
            self.assertEqual(record["preflight"]["lock"]["owner_pid"], os.getpid())
            self.assertTrue((run / "lock").exists())

    def test_busy_machine_refuses_and_records_reasons_and_clients(self):
        with tempfile.TemporaryDirectory() as directory:
            run = pathlib.Path(directory) / "r"; run.mkdir()
            gpu = FakeGpu([dict(QUIET, util_pct=35.0)] * 5, clients={"compute": [], "graphics": ["niri", "BitwigStudio"]})
            with self.assertRaises(cm.Refused):
                self.run_preflight(run, gpu=gpu)
            record = cm.load_record(run)
            self.assertEqual(record["preflight"]["verdict"], "refused")
            self.assertIn("gpu_util_pct 35.0 exceeds 5.0", record["preflight"]["reasons"])
            self.assertIn("BitwigStudio", record["baseline"]["gpu_clients"]["graphics"])
            self.assertFalse((run / "lock").exists())                 # refused runs hold no lock

    def test_dedicated_requires_tty_and_no_clients(self):
        with tempfile.TemporaryDirectory() as directory:
            run = pathlib.Path(directory) / "r"; run.mkdir()
            with self.assertRaises(cm.Refused):
                self.run_preflight(run, lane="dedicated", env={"XDG_SESSION_TYPE": "wayland", "WAYLAND_DISPLAY": "w"})
            self.assertIn("XDG_SESSION_TYPE", " ".join(cm.load_record(run)["preflight"]["reasons"]))
            run2 = pathlib.Path(directory) / "r2"; run2.mkdir()
            with self.assertRaises(cm.Refused):
                self.run_preflight(run2, lane="dedicated", env={"XDG_SESSION_TYPE": "tty"},
                                   gpu=FakeGpu([QUIET] * 5, clients={"compute": [], "graphics": ["niri"]}))
            run3 = pathlib.Path(directory) / "r3"; run3.mkdir()
            saved = {k: os.environ.pop(k, None) for k in ("DISPLAY", "WAYLAND_DISPLAY")}
            try:
                self.run_preflight(run3, lane="dedicated", env={"XDG_SESSION_TYPE": "tty"},
                                   gpu=FakeGpu([QUIET] * 5, clients={"compute": [], "graphics": []}))
            finally:
                os.environ.update({k: v for k, v in saved.items() if v is not None})
            self.assertEqual(cm.load_record(run3)["preflight"]["verdict"], "quiet")

    def test_held_lock_refuses_without_sampling(self):
        with tempfile.TemporaryDirectory() as directory:
            run = pathlib.Path(directory) / "r"; run.mkdir()
            lock = run / "lock"
            cm.acquire_lock(lock, os.getpid(), "other-run")
            with self.assertRaises(cm.Refused):
                self.run_preflight(run, lock=lambda: lock)
            record = cm.load_record(run)
            self.assertEqual(record["preflight"]["verdict"], "refused")
            self.assertIn("other-run", " ".join(record["preflight"]["reasons"]))
            self.assertNotIn("baseline", record)
            self.assertEqual(cm.read_lock(lock)["run_id"], "other-run")
```

- [x] **Step 2: Run tests to verify they fail**

Run: `python3 tools/tt material-bae9c9-tools -- python3 -m unittest tools.test_capture_meta.PreflightTests -v`
Expected: `AttributeError ... preflight`.

- [x] **Step 3: Implement**

```python
import datetime, platform, socket  # add to imports


def now_rfc3339():
    return datetime.datetime.now().astimezone().replace(microsecond=0).isoformat()


def tool_versions(tools):
    versions = {}
    for spec_ in tools or []:
        name, sep, version = spec_.partition("=")
        if sep:
            versions[name] = version
            continue
        result = subprocess.run([name, "--version"], capture_output=True, text=True, check=False)
        first = (result.stdout or result.stderr).strip().splitlines()
        versions[name] = first[0] if first else f"exit {result.returncode}"
    return versions


def cpu_model():
    for line in pathlib.Path("/proc/cpuinfo").read_text().splitlines():
        if line.startswith("model name"):
            return line.partition(":")[2].strip()
    return platform.processor() or "unknown"


def environment(gpu, tools, proc):
    _, total_kib = proc.meminfo()
    static = gpu.static()
    return {
        "kernel": platform.release(),
        "cpu": cpu_model(),
        "cpu_threads": os.cpu_count(),
        "memory_total_kib": total_kib,
        "gpu": {**static, "device": os.environ.get("CAPTURE_META_RENDER_NODE", "/dev/dri/renderD128")},
        "session": {"type": os.environ.get("XDG_SESSION_TYPE", ""),
                    "display": os.environ.get("WAYLAND_DISPLAY") or os.environ.get("DISPLAY") or ""},
        "tools": tool_versions(tools),
    }


def run_section(run_dir, lane, task, fixture):
    return {"id": pathlib.Path(run_dir).resolve().name, "task": task, "fixture": fixture, "lane": lane,
            "started": now_rfc3339(), "host": socket.gethostname()}


def session_reasons(lane):
    if lane != "dedicated":
        return []
    reasons = []
    if os.environ.get("XDG_SESSION_TYPE") != "tty":
        reasons.append(f"XDG_SESSION_TYPE is {os.environ.get('XDG_SESSION_TYPE')!r}, dedicated lane needs 'tty'")
    for key in ("DISPLAY", "WAYLAND_DISPLAY"):
        if os.environ.get(key):
            reasons.append(f"{key} is set; dedicated lane must have no display")
    return reasons


def preflight(run_dir, lane, task, fixture, seconds, owner_pid, thresholds, tools, proc, gpu,
              sleep=time.sleep, lock=lock_path):
    run_dir = pathlib.Path(run_dir)
    run = run_section(run_dir, lane, task, fixture)
    write_section(run_dir, "run", run)
    lock_file = lock()
    try:
        lock_info = acquire_lock(lock_file, owner_pid, run["id"])
    except Refused as error:
        write_section(run_dir, "preflight", {"verdict": "refused", "reasons": [str(error)], "thresholds": thresholds})
        raise
    try:
        write_section(run_dir, "environment", environment(gpu, tools, proc))
        reasons = session_reasons(lane)
        samples = sample_stream(proc, gpu, seconds, sleep)
        summary = summarize(samples)
        write_section(run_dir, "baseline", {"seconds": seconds, **summary})
        reasons += judge_quiet(summary, samples, thresholds, lane)
        verdict = {"verdict": "quiet" if not reasons else "refused", "reasons": reasons,
                   "thresholds": thresholds, "lock": lock_info}
        write_section(run_dir, "preflight", verdict)
        if reasons:
            raise Refused("; ".join(reasons))
    except BaseException:
        release_lock(lock_file, owner_pid, run["id"])
        raise


def cmd_preflight(args):
    preflight(args.run_dir, args.lane, args.task, args.fixture, args.seconds,
              args.owner_pid if args.owner_pid is not None else os.getppid(),
              parse_thresholds(args.threshold), args.tool, ProcReader(), GpuReader())
```

Add to `build_parser`:

```python
    pre = subs.add_parser("preflight")
    pre.add_argument("run_dir")
    pre.add_argument("--lane", choices=("headless", "dedicated"), required=True)
    pre.add_argument("--task", required=True)
    pre.add_argument("--fixture", required=True)
    pre.add_argument("--seconds", type=int, default=20)
    pre.add_argument("--owner-pid", type=int, default=None)
    pre.add_argument("--threshold", action="append", default=[])
    pre.add_argument("--tool", action="append", default=[])
    pre.set_defaults(func=cmd_preflight)
```

- [x] **Step 4: Run tests to verify they pass**

Run: `python3 tools/tt material-bae9c9-tools -- python3 -m unittest tools.test_capture_meta -v`
Expected: all OK. If `test_held_lock_refuses_without_sampling` fails on `baseline`, make sure the lock is acquired *before* sampling.

- [x] **Step 5: Commit**

```bash
git add tools/capture-meta tools/test_capture_meta.py
git commit -m "feat(tools): capture-meta preflight with lanes and lock"
```

---

### Task 5: `identity`

**Files:**
- Modify: `tools/capture-meta`
- Modify: `tools/test_capture_meta.py`

**Interfaces:**
- Produces: `sha256_file(path) -> str`, `source_facts(checkout) -> dict(commit, branch, dirty, diff_sha256?)`, `identity(run_dir, source, binaries, inputs, config) -> None`, `cmd_identity`. `Refused` when any binary/input path is missing.

- [x] **Step 1: Write the failing tests**

```python
class IdentityTests(unittest.TestCase):
    def repo(self, root):
        import subprocess
        subprocess.run(["git", "-C", str(root), "init", "-q", "-b", "main"], check=True)
        subprocess.run(["git", "-C", str(root), "-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q",
                        "--allow-empty", "-m", "init"], check=True)

    def test_hashes_match_sha256sum_and_records_source(self):
        import hashlib
        with tempfile.TemporaryDirectory() as directory:
            # The run directory sits beside the checkout, not inside it: files written into the
            # source tree would be untracked and correctly make the tree dirty.
            root = pathlib.Path(directory) / "src"; root.mkdir(); run = pathlib.Path(directory) / "run"; run.mkdir(); self.repo(root)
            (run / "niri").write_bytes(b"binary"); (run / "A.kdl").write_text("glass")
            cm.identity(run, source=root, binaries=[run / "niri"], inputs=[run / "A.kdl"],
                        config=["preset=aurora", "glass.ior=1.24"])
            prov = cm.load_record(run)["provenance"]
            self.assertEqual(prov["binaries"], [{"name": "niri", "sha256": hashlib.sha256(b"binary").hexdigest()}])
            self.assertEqual(prov["inputs"][0]["name"], "A.kdl")
            self.assertEqual(len(prov["source"]["commit"]), 40)
            self.assertEqual(prov["source"]["branch"], "main")
            self.assertFalse(prov["source"]["dirty"])
            self.assertEqual(prov["config"], {"preset": "aurora", "glass.ior": "1.24"})

    def test_dirty_tree_records_diff_hash(self):
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory) / "src"; root.mkdir(); run = pathlib.Path(directory) / "run"; run.mkdir(); self.repo(root)
            (root / "tracked.txt").write_text("v1\n")
            import subprocess
            subprocess.run(["git", "-C", str(root), "add", "tracked.txt"], check=True)
            subprocess.run(["git", "-C", str(root), "-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "-m", "t"], check=True)
            (root / "tracked.txt").write_text("v2\n")
            cm.identity(run, source=root, binaries=[], inputs=[], config=[])
            source = cm.load_record(run)["provenance"]["source"]
            self.assertTrue(source["dirty"])
            self.assertEqual(len(source["diff_sha256"]), 64)

    def test_untracked_source_file_makes_the_tree_dirty_and_enters_the_hash(self):
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory); self.repo(root)
            (root / "new_optic.rs").write_text("fn a() {}\n")
            first = cm.source_facts(root)
            self.assertTrue(first["dirty"])
            self.assertEqual(first["untracked"], ["new_optic.rs"])
            (root / "new_optic.rs").write_text("fn b() {}\n")
            self.assertNotEqual(cm.source_facts(root)["diff_sha256"], first["diff_sha256"])
            (root / ".gitignore").write_text("new_optic.rs\n")
            still = cm.source_facts(root)           # .gitignore itself is now the untracked file
            self.assertEqual(still["untracked"], [".gitignore"])

    def test_missing_path_refuses_and_writes_nothing(self):
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory) / "src"; root.mkdir(); run = pathlib.Path(directory) / "run"; run.mkdir(); self.repo(root)
            with self.assertRaises(cm.Refused):
                cm.identity(run, source=root, binaries=[run / "absent"], inputs=[], config=[])
            self.assertEqual(cm.load_record(run), {})
```

- [x] **Step 2: Run tests to verify they fail**

Run: `python3 tools/tt material-bae9c9-tools -- python3 -m unittest tools.test_capture_meta.IdentityTests -v`
Expected: `AttributeError ... identity`.

- [x] **Step 3: Implement**

```python
import hashlib  # add to imports


def sha256_file(path):
    with open(path, "rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def git(checkout, *args):
    result = subprocess.run(("git", "-C", str(checkout), *args), capture_output=True, text=True, check=False)
    if result.returncode != 0:
        raise CannotRun(f"git {' '.join(args)} in {checkout}: {result.stderr.strip()}")
    return result.stdout


def source_facts(checkout):
    """HEAD plus everything the working tree adds to it: the tracked diff and every
    untracked, non-ignored file. `git diff HEAD` alone would call a checkout with a new
    source file clean, and that file would never enter the hash."""
    commit = git(checkout, "rev-parse", "HEAD").strip()
    branch = git(checkout, "rev-parse", "--abbrev-ref", "HEAD").strip()
    diff = git(checkout, "diff", "HEAD")
    untracked = sorted(line for line in
                       git(checkout, "ls-files", "--others", "--exclude-standard").splitlines() if line)
    facts = {"commit": commit, "branch": branch, "dirty": bool(diff.strip()) or bool(untracked)}
    if facts["dirty"]:
        digest = hashlib.sha256(diff.encode())
        for rel in untracked:
            digest.update(rel.encode() + b"\0")
            digest.update(sha256_file(pathlib.Path(checkout) / rel).encode() + b"\0")
        facts["diff_sha256"] = digest.hexdigest()
        facts["untracked"] = untracked
    return facts


def hashed(paths):
    entries = []
    for path in paths:
        path = pathlib.Path(path)
        if not path.is_file():
            raise Refused(f"{path} does not exist; the fixture named it as a binary or input")
        entries.append({"name": path.name, "sha256": sha256_file(path)})
    return entries


def parse_pairs(pairs, flag):
    config = {}
    for pair in pairs or []:
        key, sep, value = pair.partition("=")
        if not sep or not key:
            raise CannotRun(f"{flag} {pair!r}: expected KEY=VALUE")
        config[key] = value
    return config


def identity(run_dir, source, binaries, inputs, config):
    section = {"source": source_facts(source), "binaries": hashed(binaries), "inputs": hashed(inputs),
               "config": parse_pairs(config, "--config")}
    write_section(run_dir, "provenance", section)


def cmd_identity(args):
    identity(args.run_dir, args.source, args.binary, args.input, args.config)
```

Parser:

```python
    ident = subs.add_parser("identity")
    ident.add_argument("run_dir")
    ident.add_argument("--source", required=True)
    ident.add_argument("--binary", action="append", default=[])
    ident.add_argument("--input", action="append", default=[])
    ident.add_argument("--config", action="append", default=[])
    ident.set_defaults(func=cmd_identity)
```

- [x] **Step 4: Run tests to verify they pass**

Run: `python3 tools/tt material-bae9c9-tools -- python3 -m unittest tools.test_capture_meta.IdentityTests -v`
Expected: 4 OK.

- [x] **Step 5: Commit**

```bash
git add tools/capture-meta tools/test_capture_meta.py
git commit -m "feat(tools): capture-meta identity"
```

---

### Task 6: `settle`, `release`, and the end-to-end run

**Files:**
- Modify: `tools/capture-meta`
- Modify: `tools/test_capture_meta.py`

**Interfaces:**
- Produces: `settle(run_dir, sub_run, inputs, seconds, proc, gpu, sleep, lock) -> None`, `cmd_settle`, `cmd_release`. `settle` appends to `sub_runs[]` and raises `Refused` on an off-baseline verdict, a missing input, or a lock that no longer names this run.

- [x] **Step 1: Write the failing tests**

```python
class SettleTests(unittest.TestCase):
    def prepared(self, directory):
        run = pathlib.Path(directory) / "r"; run.mkdir()
        lock = run / "lock"
        PreflightTests("test_quiet_headless_writes_run_environment_baseline_preflight").run_preflight(run, lock=lambda: lock)
        (run / "A.kdl").write_text("glass")
        return run, lock

    def test_settled_entry_carries_inputs(self):
        with tempfile.TemporaryDirectory() as directory:
            run, lock = self.prepared(directory)
            cm.settle(run, "A-move-1", [run / "A.kdl"], 3, FakeProc([(i * 2, i * 100) for i in range(9)]),
                      FakeGpu([dict(QUIET, power_w=19.0)] * 3), sleep=lambda s: None, lock=lambda: lock)
            entry = cm.load_record(run)["sub_runs"][0]
            self.assertEqual(entry["name"], "A-move-1")
            self.assertEqual(entry["verdict"], "settled")
            self.assertEqual(entry["inputs"][0]["name"], "A.kdl")
            self.assertEqual(entry["gpu_power_w"], 19.0)
            self.assertRegex(entry["settled_at"], r"^\d{4}-")

    def test_off_baseline_appends_refused_and_raises(self):
        with tempfile.TemporaryDirectory() as directory:
            run, lock = self.prepared(directory)
            with self.assertRaises(cm.Refused):
                cm.settle(run, "B-move-1", [run / "A.kdl"], 3, FakeProc([(i * 2, i * 100) for i in range(9)]),
                          FakeGpu([dict(QUIET, power_w=24.1)] * 3), sleep=lambda s: None, lock=lambda: lock)
            entry = cm.load_record(run)["sub_runs"][-1]
            self.assertEqual(entry["verdict"], "refused")
            self.assertIn("gpu_power_w 24.1 exceeds baseline 18.4 by more than 1.5", entry["reason"])

    def test_missing_input_and_foreign_lock_refuse(self):
        with tempfile.TemporaryDirectory() as directory:
            run, lock = self.prepared(directory)
            args = (3, FakeProc([(i * 2, i * 100) for i in range(9)]), FakeGpu([QUIET] * 3))
            with self.assertRaises(cm.Refused):
                cm.settle(run, "x", [run / "missing.kdl"], *args, sleep=lambda s: None, lock=lambda: lock)
            lock.write_text(json.dumps({"owner_pid": 1, "run_id": "someone-else"}))
            with self.assertRaises(cm.Refused):
                cm.settle(run, "y", [run / "A.kdl"], *args, sleep=lambda s: None, lock=lambda: lock)

    def test_release_command_is_ownership_checked(self):
        with tempfile.TemporaryDirectory() as directory:
            run, lock = self.prepared(directory)
            saved = os.environ.get("XDG_RUNTIME_DIR"); os.environ["XDG_RUNTIME_DIR"] = str(run)
            try:
                lock.rename(run / cm.LOCK_NAME)
                self.assertEqual(cm.main(["release", str(run)]), 0)
                self.assertFalse((run / cm.LOCK_NAME).exists())
                cm.acquire_lock(run / cm.LOCK_NAME, 1, "someone-else")
                self.assertEqual(cm.main(["release", str(run)]), 1)
                self.assertTrue((run / cm.LOCK_NAME).exists())
            finally:
                if saved is None: os.environ.pop("XDG_RUNTIME_DIR", None)
                else: os.environ["XDG_RUNTIME_DIR"] = saved


class EndToEndTest(unittest.TestCase):
    def test_real_binary_against_fake_nvidia_smi(self):
        import subprocess, sys, textwrap
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory); bins = root / "bin"; bins.mkdir()
            fake = bins / "nvidia-smi"
            fake.write_text(textwrap.dedent("""\
                #!/bin/sh
                case "$*" in
                  *utilization.gpu*) echo "0, 18.40, 360, P8" ;;
                  *name,uuid*) echo "NVIDIA test, GPU-1, 610.57.04" ;;
                  *-q*-x*) echo '<nvidia_smi_log><gpu><processes><process_info><type>G</type><process_name>/usr/bin/niri</process_name></process_info></processes></gpu></nvidia_smi_log>' ;;
                  # A complete inventory is required: an empty <processes/> would still pass; a missing element exits 2.
                  *) exit 9 ;;
                esac
                """)); fake.chmod(0o755)
            run = root / "material-x" / "headless-20260911T000000"; run.mkdir(parents=True)
            runtime = root / "rt"; runtime.mkdir()
            proc = root / "proc"; proc.mkdir()
            (proc / "stat").write_text("cpu  100 0 50 9000 10 0 0 0 0 0\n")     # constant ticks: 0 % busy
            (proc / "loadavg").write_text("0.42 0.40 0.39 1/900 1\n")
            (proc / "meminfo").write_text("MemTotal:       100000 kB\nMemFree:         50000 kB\nMemAvailable:    80000 kB\n")
            src = root / "src"; src.mkdir()
            (root / "A.kdl").write_text("glass")
            subprocess.run(["git", "-C", str(src), "init", "-q", "-b", "main"], check=True)
            subprocess.run(["git", "-C", str(src), "-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "--allow-empty", "-m", "i"], check=True)
            env = {**os.environ, "PATH": f"{bins}:{os.environ['PATH']}", "XDG_RUNTIME_DIR": str(runtime),
                   "CAPTURE_META_PROC": str(proc), "XDG_SESSION_TYPE": "wayland", "WAYLAND_DISPLAY": "wayland-1"}
            tool = str(pathlib.Path(__file__).with_name("capture-meta"))
            def cm_run(*args):
                return subprocess.run([sys.executable, tool, *args], env=env, capture_output=True, text=True)
            self.assertEqual(cm_run("preflight", str(run), "--lane", "headless", "--task", "material-x",
                                    "--fixture", "t.sh", "--seconds", "1", "--tool", "tracy=0.13.1").returncode, 0)
            self.assertEqual(cm_run("identity", str(run), "--source", str(src), "--input", str(root / "A.kdl")).returncode, 0)
            self.assertEqual(cm_run("settle", str(run), "--sub-run", "A-1", "--input", str(root / "A.kdl"), "--seconds", "1").returncode, 0)
            self.assertEqual(cm_run("release", str(run)).returncode, 0)
            show = cm_run("show", str(run))
            self.assertEqual(show.returncode, 0, show.stderr)
            self.assertIn("A-1: settled", show.stdout)
            record = json.loads((run / "capture.json").read_text())
            self.assertFalse(record["provenance"]["source"]["dirty"])
            self.assertEqual(record["environment"]["gpu"]["driver"], "610.57.04")
            self.assertEqual(record["baseline"]["gpu_clients"]["graphics"], ["niri"])
            self.assertEqual(record["baseline"]["mem_available_pct"], 80.0)
            self.assertEqual(record["baseline"]["cpu_busy_pct"], 0.0)
            self.assertFalse((runtime / "capture-meta.lock").exists())
```

- [x] **Step 2: Run tests to verify they fail**

Run: `python3 tools/tt material-bae9c9-tools -- python3 -m unittest tools.test_capture_meta.SettleTests tools.test_capture_meta.EndToEndTest -v`
Expected: `AttributeError ... settle`.

- [x] **Step 3: Implement**

```python
def settle(run_dir, sub_run, inputs, seconds, proc, gpu, sleep=time.sleep, lock=lock_path):
    run_dir = pathlib.Path(run_dir)
    record = load_record(run_dir)
    for section in ("run", "baseline", "preflight"):
        if section not in record:
            raise CannotRun(f"settle needs a preflighted run; {RECORD} has no {section!r}")
    if record["preflight"]["verdict"] != "quiet":
        raise Refused("preflight was refused; nothing to settle into")
    lane = record["run"]["lane"]
    holder = read_lock(lock())
    if holder is None or holder.get("run_id") != record["run"]["id"]:
        raise Refused(f"lock no longer names this run ({record['run']['id']}); holder: {holder}")
    entry_inputs = hashed(inputs)
    samples = sample_stream(proc, gpu, seconds, sleep)
    summary = summarize(samples)
    reasons = judge_settled(summary, samples, record["baseline"], record["preflight"]["thresholds"], lane)
    entry = {"name": sub_run, "settled_at": now_rfc3339(), "seconds": seconds, **summary,
             "inputs": entry_inputs, "verdict": "settled" if not reasons else "refused"}
    if reasons:
        entry["reason"] = "; ".join(reasons)
    append_sub_run(run_dir, entry)
    if reasons:
        raise Refused(entry["reason"])


def cmd_settle(args):
    settle(args.run_dir, args.sub_run, args.input, args.seconds, ProcReader(), GpuReader())


def cmd_release(args):
    record = load_record(args.run_dir)
    run = record.get("run")
    if not run:
        return                                      # never preflighted: nothing to release
    owner = record.get("preflight", {}).get("lock", {}).get("owner_pid")
    if owner is None:
        return                                      # refused before acquiring
    if not release_lock(lock_path(), owner, run["id"]):
        raise Refused(f"lock is not this run's ({run['id']}); left in place")
```

Parser:

```python
    st = subs.add_parser("settle")
    st.add_argument("run_dir")
    st.add_argument("--sub-run", required=True)
    st.add_argument("--input", action="append", default=[])
    st.add_argument("--seconds", type=int, default=10)
    st.set_defaults(func=cmd_settle)
    rel = subs.add_parser("release")
    rel.add_argument("run_dir")
    rel.set_defaults(func=cmd_release)
```

- [x] **Step 4: Run the whole suite**

Run: `just check`
Expected: the tooling suite reports `OK` with the previous 70 tests plus the new ones (about 97); fmt, clippy, `tasks check`, and the two `--check` tools pass.

- [x] **Step 5: Commit**

```bash
git add tools/capture-meta tools/test_capture_meta.py
git commit -m "feat(tools): capture-meta settle and release, end-to-end test"
```

---

### Task 7: Adopt in the optic smoke lib and smokes

**Files:**
- Modify: `docs/materials/scripts/glass-optic-smoke-lib.sh` (header comment; `cleanup`; `build_binaries`; `start_nested`; `finish`; new functions)
- Modify: `docs/materials/scripts/glass-aurora-smoke.sh:19`, `docs/materials/scripts/glass-iridescence-smoke.sh:16`
- Modify: `tools/test_glass_optic_smoke.py`

**Interfaces:**
- Produces (bash): `capture_meta "$@"` (runs `${CAPTURE_META:-python3 "$ROOT/tools/capture-meta"} "$@"`; tests set `CAPTURE_META` to a stub), `capture_preflight <lane>`, `capture_identity [extra --input/--config args]...`, `settle_before_launch <config> [<name>]`, `start_nested <niri> <config> [<name>]`.

- [x] **Step 1: Write the failing tests**

Append to `tools/test_glass_optic_smoke.py`:

```python
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
        for smoke in ('glass-aurora-smoke.sh', 'glass-iridescence-smoke.sh'):
            text = (Path(__file__).resolve().parents[1] / 'docs/materials/scripts' / smoke).read_text()
            source = text.index('glass-optic-smoke-lib.sh')
            pre = text.index('capture_preflight headless')
            build = text.index('build_binaries')
            ident = text.index('capture_identity')
            self.assertTrue(source < pre < build < ident, smoke)
```

- [x] **Step 2: Run tests to verify they fail**

Run: `python3 tools/tt material-bae9c9-tools -- python3 -m unittest tools.test_glass_optic_smoke -v`
Expected: four failures (`settle_before_launch` not found, etc.).

- [x] **Step 3: Modify the lib**

Header comment: add after the `# Env:` paragraph:

```bash
# Every run records provenance and machine quietness through tools/capture-meta
# (docs/specs/2026-09-11-material-capture-protocol-design.md). The entry script
# calls `capture_preflight headless` right after sourcing this lib and
# `capture_identity` after build_binaries; start_nested settles before every
# launch. This lib never preflights on its own: idle-budget sources it in a
# mode that must stay offline.
```

`cleanup`: after `remove_runtime_dir || rc=1` add `capture_meta release "$OUT" || true`.

New functions, placed after `trap cleanup EXIT`:

```bash
# --- capture record ---------------------------------------------------------
# CAPTURE_META lets a test substitute a recording stub; unset, it is the tool.
capture_meta() { ${CAPTURE_META:-python3 "$ROOT/tools/capture-meta"} "$@"; }
# Entry scripts call this first thing after sourcing. The fixture name is the
# calling script; the task id comes from CAPTURE_TASK, which every smoke sets.
capture_preflight() {
    capture_meta preflight "$OUT" --lane "$1" --task "${CAPTURE_TASK:?task id authorizing this run}" \
        --fixture "$(basename "$0")" --owner-pid $$ --tool weston --tool kitty --tool "tracy=0.13.1" \
        || fail "preflight refused; see $OUT/capture.json"
}
# After build_binaries: both binaries, this lib, and the calling script are the
# static inputs. Extra --input/--config arguments pass through.
capture_identity() {   # NIRI_TRACY is unset for fixtures that measure the installed binary only
    capture_meta identity "$OUT" --source "$ROOT" --binary "$NIRI" ${NIRI_TRACY:+--binary "$NIRI_TRACY"} \
        --input "$ROOT/docs/materials/scripts/glass-optic-smoke-lib.sh" --input "$0" "$@" \
        || fail "identity refused"
}
# Before every nested launch: the sub-run is named after its config unless the
# caller's observation is not its config (idle-budget reuses six configs).
settle_before_launch() {
    local cfg=$1 name=${2:-}
    [ -n "$name" ] || name=$(basename "$cfg" .kdl)
    capture_meta settle "$OUT" --sub-run "$name" --input "$cfg" || fail "settle refused before $name; see $OUT/capture.json"
}
```

`build_binaries`: delete its last three lines (`sha256sum … binaries.sha256`, `impl.version`, `source.commit`) — `capture.json` carries them. The other loose files the spec names (`kernel.txt`, `lscpu.txt`, `nvidia-smi.txt`, `installed.version`) were written by the retained `aurora-iridescence-hardware.sh` fixture in niri-experiments, not by this lib; nothing else to remove here. `metrics.txt` and `tracy.version` stay.

`start_nested`: change signature and first line:

```bash
start_nested() {   # $1 niri, $2 config, $3 sub-run name (defaults to the config's basename)
    settle_before_launch "$2" "${3-}"
    weston --backend=headless ...
```

`finish`:

```bash
finish() {
    (cd "$OUT" && find . -type f ! -name SHA256SUMS -print0 | LC_ALL=C sort -z | xargs -0 sha256sum > SHA256SUMS)
    if rg -n 'material.*(error|fallback)|error compiling material shader|panic' "$OUT/niri.log"; then
```

- [x] **Step 4: Modify the smokes**

In both smokes, immediately after the `. …/glass-optic-smoke-lib.sh` line add `capture_preflight headless`, and immediately after `build_binaries` add `capture_identity` with the pinned scene facts, e.g. for aurora:

```bash
capture_identity --config preset=aurora --config output=1280x720 --config scale=1 --config vrr=off
```

Each smoke sets `CAPTURE_TASK` near its top (`: "${CAPTURE_TASK:=material-bae9c9}"` is wrong — the task is whoever runs the smoke; use `: "${CAPTURE_TASK:?task id authorizing this run}"` so the caller must export it). Update the smoke's usage comment to mention `CAPTURE_TASK`.

- [x] **Step 5: Run tests**

Run: `python3 tools/tt material-bae9c9-tools -- python3 -m unittest tools.test_glass_optic_smoke -v && bash -n docs/materials/scripts/glass-optic-smoke-lib.sh docs/materials/scripts/glass-aurora-smoke.sh docs/materials/scripts/glass-iridescence-smoke.sh`
Expected: 6 OK; `bash -n` silent.

- [x] **Step 6: Commit**

```bash
git add docs/materials/scripts/glass-optic-smoke-lib.sh docs/materials/scripts/glass-aurora-smoke.sh docs/materials/scripts/glass-iridescence-smoke.sh tools/test_glass_optic_smoke.py
git commit -m "feat(material): optic smokes record captures through capture-meta"
```

---

### Task 8: Adopt in the idle-budget fixture (niri-experiments)

**Files:**
- Modify: `niri-experiments/fixtures/idle-budget.sh` (`runtime()` lines 40–133; `start_scene` lines 175–203)
- Modify: `niri-experiments/fixtures/jelly-motion.sh:7-15` — the other live consumer of the lib
- Modify: `niri-experiments/fixtures/test_idle_budget.py` (`ShellTests`)
- Leave unchanged: `niri-experiments/docs/results/2026-09-11-jelly-motion-sweep.sha256` is the immutable manifest of the accepted archived run.

Work in a new worktree:

```bash
cd "$NIRI_EXPERIMENTS"  # the niri-experiments checkout
git worktree add .worktrees/material-bae9c9 -b results/capture-protocol results/idle-budget
export MATERIAL_ROOT="$NIRI_MATERIAL/.worktrees/material-bae9c9"  # under the main niri-material checkout
cd .worktrees/material-bae9c9
```

**Interfaces:**
- Consumes: `capture_preflight`, `capture_identity`, `start_nested <niri> <cfg> <name>`, `capture_meta` from Task 7 via the sourced lib.

- [x] **Step 1: Write the failing shell test**

Add to `ShellTests` in `fixtures/test_idle_budget.py`:

```python
    def test_runtime_calls_capture_meta_in_order_per_mode(self):
        import os, subprocess
        expected = {
            "prepare": ["identity"],
            "trace": ["preflight headless", "identity"],
            "power": ["preflight dedicated", "identity"],
        }
        for mode in ("prepare", "trace", "power"):
            with self.subTest(mode=mode), tempfile.TemporaryDirectory() as directory:
                root = Path(directory); bins = root / "bin"; bins.mkdir()
                scripts = {
                    "niri": 'case "$1" in --version) echo "niri '+"a"*40+'" ;; validate) : ;; *) exit 99 ;; esac',
                    "magick": 'if [ "$1" = --version ]; then echo synthetic; else for last; do :; done; : > "$last"; fi',
                    "kitty": 'test "$1" = --version', "swaybg": 'test "$1" = --version',
                }
                for tool, body in scripts.items():
                    path = bins / tool; path.write_text("#!/bin/sh\n"+body+"\n"); path.chmod(0o755)
                binary = bins / "niri"
                (bins / "niri.source.tar").write_bytes(b"src"); (bins / "niri.marker.patch").write_bytes(b"patch")
                identity = dict(binary_sha256=idle_budget.digest(binary), source_sha256=idle_budget.digest(bins / "niri.source.tar"),
                    source_commit="a"*40, features=["profile-with-tracy"] if mode == "trace" else [],
                    patch_sha256=idle_budget.digest(bins / "niri.marker.patch") if mode == "trace" else None)
                (bins / "niri.identity.json").write_text(json.dumps(identity))
                runtime = root / "runtime"; runtime.mkdir()
                stub = bins / "capture-meta-stub"
                stub.write_text('#!/bin/sh\nprintf "%s\\n" "$*" >> "$OUT/capture-calls"\n'); stub.chmod(0o755)
                script = r"""
source "$FIXTURE"
trace_all() { exit 0; }
power_all() { exit 0; }
runtime "$TEST_MODE"
"""
                env = {**os.environ, "PATH": str(bins)+":"+os.environ["PATH"], "CAPTURE_META": str(stub),
                    "FIXTURE": str(Path(__file__).with_name("idle-budget.sh")), "TEST_MODE": mode,
                    "OUT": str(root / "material-265eb0/run"), "NIRI_BIN": str(binary),
                    "NIRI_MATERIAL_WORK_ROOT": directory, "XDG_RUNTIME_DIR": str(runtime),
                    "POWER_OUTPUT": "DP-1", "POWER_MODE": "1920x1080@60", "POWER_SCALE": "1"}
                for key in ("DISPLAY", "WAYLAND_DISPLAY", "IDLE_BUDGET_DEDICATED_SESSION"): env.pop(key, None)
                result = subprocess.run(["bash", "-c", script], env=env, capture_output=True, text=True, timeout=10)
                self.assertEqual(result.returncode, 0, result.stderr)
                calls = (Path(env["OUT"]) / "capture-calls").read_text().splitlines()
                heads = [" ".join(c.split()[:1] + ([c.split()[c.split().index("--lane") + 1]] if "--lane" in c else [])) for c in calls if not c.startswith("release")]
                self.assertEqual(heads, expected[mode])
                ident = next(c for c in calls if c.startswith("identity"))
                self.assertIn("--input", ident)
                for name in ("A.kdl", "O.kdl", "idle-budget.sh", "glass-optic-smoke-lib.sh"):
                    self.assertIn(name, ident, mode)
                self.assertEqual("identity.json" in ident, mode != "prepare")
                self.assertEqual("marker.patch" in ident, mode == "trace")
                for stale in ("hardware.json", "hardware.csv", "nvidia-smi.version"):
                    self.assertFalse((Path(env["OUT"]) / stale).exists(), stale)
```

Also update `test_runtime_validates_identity_before_reaching_launch_boundary` (line 319): set `"CAPTURE_META": "true"` in its env (the `true` builtin accepts any arguments and exits 0; the lib's wrapper must therefore be defined so `CAPTURE_META` is word-split, as in Task 7), drop `"IDLE_BUDGET_DEDICATED_SESSION": "YES"` and the `nvidia-smi` stub from its env/scripts (they are gone from the fixture), keep `"XDG_SESSION_TYPE": "tty"`. Note the stub must not be a shell function: `runtime()` sources the lib after the test script runs, and the lib's definition would replace a function stub.

- [x] **Step 2: Run to verify failure**

Run: `just --justfile fixtures/idle-budget.just test`
Expected: the new test fails (`capture-calls` missing) and the identity-boundary test may fail on the `nvidia-smi` stub removal until Step 3.

- [x] **Step 3: Modify `runtime()`**

Replace, in order:

(a) After `trap 'exit 143' TERM` (line 67) insert:

```bash
    if [ "$mode" != prepare ]; then
        CAPTURE_TASK=material-265eb0 capture_preflight "$([ "$mode" = power ] && echo dedicated || echo headless)"
    fi
```

(b) Delete the `IDLE_BUDGET_DEDICATED_SESSION` block (lines 95–98): the dedicated lane's preflight enforces `tty` and no display.

(c) Replace `sha256sum "$OUT"/*.kdl "$OUT"/idle-budget* > "$OUT/inputs.sha256"` (line 114) and the `cp … glass-optic-smoke-lib.sh` line with:

```bash
    cp docs/materials/scripts/glass-optic-smoke-lib.sh "$OUT/"
    identity_inputs=(--input "$OUT"/A.kdl --input "$OUT"/B.kdl --input "$OUT"/P.kdl --input "$OUT"/C.kdl --input "$OUT"/D.kdl --input "$OUT"/O.kdl
                     --input "$OUT/idle-budget.py" --input "$OUT/idle-budget.sh" --input "$OUT/idle-budget.just"
                     --input "$OUT/glass-optic-smoke-lib.sh" --input "$WALL")
    if [ "$mode" != prepare ]; then identity_inputs+=(--input "$OUT/identity.json" --input "$OUT/source.tar"); fi
    if [ "$mode" = trace ]; then identity_inputs+=(--input "$OUT/marker.patch"); fi
    local capture_output=1280x720@60 capture_scale=1
    if [ "$mode" = power ]; then capture_output=$POWER_MODE; capture_scale=$POWER_SCALE; fi
    capture_meta identity "$OUT" --source "$root" --binary "$NIRI" "${identity_inputs[@]}" \
        --config lane="$([ "$mode" = power ] && echo dedicated || echo headless)" \
        --config output="$capture_output" --config scale="$capture_scale" --config vrr=off \
        || fail 'identity refused'
```

(d) Delete the `nvidia-smi … hardware.csv` line, the `HARDWARE` heredoc, and the `for tool in nvidia-smi kitty swaybg magick` line (lines 117–125). The `[ "$mode" = prepare ] && return` stays where it is, now directly before `if [ "$mode" = trace ]; then trace_all; else power_all; fi`.

(e) In `start_scene` (line 179–180) change the launch to:

```bash
    if [ "$mode" = trace ]; then start_nested "$NIRI" "$OUT/$case.kdl" "$name"
    else capture_meta settle "$OUT" --sub-run "$name" --input "$OUT/$case.kdl" || fail "settle refused before $name"; start_drm "$OUT/$case.kdl"; fi
```

and delete `sha256sum "$OUT/$case.kdl" > "$OUT/$name.config.sha256"` (line 199).

(f) Delete line 270, `printf '%s\n' "$XDG_SESSION_TYPE" > "$OUT/session-type.txt"`; the analyzer reads `capture.json`'s `environment.session.type` instead (Task 9). Until Task 9 lands, the analyzer still reads `session-type.txt`, so the power synthetic test in Task 9 Step 1 removes that file together.

(g) **`fixtures/jelly-motion.sh`** sources the same live lib and calls `start_nested` at lines 53 and 61; after Task 7 its first launch would be refused for lack of a preflight. Migrate it: after line 7 (`. docs/materials/scripts/glass-optic-smoke-lib.sh`) insert `CAPTURE_TASK=material-36e968 capture_preflight headless` (the task its `tt` targets already name); after line 10 (the installed-binary hash check) insert

```bash
capture_identity --input "$fixture/diagnostic-grid.png" \
    --config binary=installed --config output=1280x720@60 --config scale=1 --config vrr=off
```

and delete lines 13–15 (`source.commit`, `nvidia-smi.txt`, `lscpu.txt`) — `capture.json` carries them; keep `version.txt` and `source.diff` (the diff against the pinned `7526af1d` is a result, not environment). Keep `docs/results/2026-09-11-jelly-motion-sweep.sha256` byte-for-byte unchanged: it authenticates the accepted archived run, including that run's captured fixture. A future capture generates a new run manifest containing the migrated fixture hash.

(h) Add the caller-level check to `ShellTests`, so a future fixture that sources the lib cannot skip the protocol:

```python
    def test_every_live_lib_consumer_preflights_after_sourcing_and_before_launching(self):
        import re
        fixtures = Path(__file__).parent
        consumers = [p for p in fixtures.glob("*.sh") if "glass-optic-smoke-lib.sh" in p.read_text()]
        # A fixture that pins the lib's hash before sourcing it is a frozen snapshot of a
        # completed experiment: it refuses today's lib on purpose and is not migrated.
        pinned = [p for p in consumers if re.search(r"[0-9a-f]{64}\s+\"?\$?\{?lib\}?\"?.*sha256sum -c", p.read_text())]
        live = [p for p in consumers if p not in pinned]
        self.assertEqual(sorted(p.name for p in pinned), ["aurora-iridescence-hardware.sh"])
        self.assertEqual(sorted(p.name for p in live), ["idle-budget.sh", "jelly-motion.sh"])
        for path in live:
            text = path.read_text()
            source = text.index("glass-optic-smoke-lib.sh")
            preflight = text.index("capture_preflight")
            launch = min(i for i in (text.find("start_nested"), text.find("start_drm")) if i >= 0)
            self.assertTrue(source < preflight < launch, path.name)
            self.assertLess(text.index("capture_identity") if "capture_identity" in text else text.index("capture_meta identity"), launch, path.name)
```

- [x] **Step 4: Run the shell tests**

Run: `just --justfile fixtures/idle-budget.just test` and `just --justfile fixtures/jelly-motion.just test`
Expected: both pass; `IntegrationTests`/`PowerIntegrityTests` still pass because the analyzer is unchanged until Task 9. (`bash -n fixtures/jelly-motion.sh` is silent.)

- [x] **Step 5: Commit (in the experiments worktree)**

```bash
git add fixtures/idle-budget.sh fixtures/jelly-motion.sh fixtures/test_idle_budget.py
git commit -m "feat(fixtures): idle-budget and jelly-motion record captures through capture-meta"
```

---

### Task 9: Migrate the idle-budget analyzer to `capture.json`

**Files:**
- Modify: `niri-experiments/fixtures/idle-budget.py:136-158` (`scene_evidence`), `:321-340` (`analyze`)
- Modify: `niri-experiments/fixtures/test_idle_budget.py` (`IntegrationTests.trace_run`, `PowerIntegrityTests.test_power_cli_uses_raw_48_window_matrix`, new cases)
- Modify: `niri-experiments/docs/results/2026-09-11-idle-budget.md`, `docs/results/2026-09-11-idle-budget.sha256`

**Interfaces:**
- Produces: `capture_record(run) -> dict` (reads `capture.json`, requires `schema == 1`), `sub_run_entry(record, name) -> dict` (exactly one match, verdict `settled`, else `ValueError`), `scene_evidence(run, name, case)` with the same return as today.

- [x] **Step 1: Write the failing tests**

In `IntegrationTests.trace_run`, replace the `hardware.json` line and the per-observation `.config.sha256` line with a `capture.json` writer. Add a helper on the class:

```python
    def capture_record(self, root, lane, observations_cases):
        import hashlib
        digest = lambda name: hashlib.sha256((root / name).read_bytes()).hexdigest()
        return {"schema": 1,
                "run": {"id": root.name, "task": "material-265eb0", "fixture": "idle-budget.sh", "lane": lane,
                        "started": "2026-09-11T00:00:00-04:00", "host": "test"},
                "environment": {"kernel": "t", "cpu": "t", "cpu_threads": 1, "memory_total_kib": 1,
                                "gpu": {"name": "NVIDIA test", "uuid": "GPU-1", "driver": "test", "device": "/dev/dri/renderD128"},
                                "session": {"type": "tty" if lane == "dedicated" else "wayland", "display": ""}, "tools": {}},
                "baseline": {"samples": 20}, "preflight": {"verdict": "quiet", "reasons": [], "thresholds": {}},
                "provenance": {"source": {"commit": "a" * 40, "branch": "results/x", "dirty": False},
                               "binaries": [{"name": "niri", "sha256": digest("niri")}], "inputs": [], "config": {}},
                "sub_runs": [{"name": name, "verdict": "settled", "seconds": 10,
                              "inputs": [{"name": f"{case}.kdl", "sha256": digest(f"{case}.kdl")}]}
                             for name, case in observations_cases]}
```

In `trace_run`, after the matrix loop, write `(root / "capture.json").write_text(json.dumps(self.capture_record(root, "headless", [(o["name"], o["case"]) for o in observations])))`. Configs must be written before that (they are, inside the loop). Delete `(root / "hardware.json")…` and `(root / f"{name}.config.sha256")…`.

In `test_power_cli_uses_raw_48_window_matrix`, likewise write a `capture.json` with lane `dedicated` and the 48 `(name, case)` pairs, and delete its `.config.sha256` and `session-type.txt` lines.

Add new cases to `IntegrationTests`:

```python
    def test_analyzer_rejects_missing_duplicate_refused_and_mismatched_sub_runs(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); self.trace_run(root)
            self.assertEqual(self.cli("analyze", directory).returncode, 0)
            record = json.loads((root / "capture.json").read_text())
            def check(mutate, needle):
                mutated = json.loads(json.dumps(record)); mutate(mutated)
                (root / "capture.json").write_text(json.dumps(mutated))
                result = self.cli("analyze", directory)
                self.assertNotEqual(result.returncode, 0, needle)
                self.assertIn(needle, result.stdout + result.stderr)
            check(lambda r: r["sub_runs"].pop(0), "no sub-run entry for A-move-1")
            check(lambda r: r["sub_runs"].append(dict(r["sub_runs"][0])), "duplicate sub-run entry for A-move-1")
            check(lambda r: r["sub_runs"][0].update(verdict="refused", reason="hot"), "A-move-1 sub-run was refused")
            check(lambda r: r["sub_runs"][0]["inputs"][0].update(sha256="0" * 64), "observation config identity mismatch")
            check(lambda r: r["environment"]["gpu"].update(name="llvmpipe"), "software renderer")
            check(lambda r: r.update(schema=2), "capture.json schema")
```

- [x] **Step 2: Run to verify failure**

Run: `just --justfile fixtures/idle-budget.just test`
Expected: `FileNotFoundError: … hardware.json` from `analyze`, and the new test fails.

- [x] **Step 3: Migrate the analyzer**

Replace `scene_evidence`'s first two checks and add helpers:

```python
def capture_record(run):
    record = read_json(run / "capture.json")
    if record.get("schema") != 1:
        raise ValueError(f"capture.json schema {record.get('schema')!r} unsupported")
    return record


def sub_run_entry(record, name):
    matches = [e for e in record.get("sub_runs", []) if e.get("name") == name]
    if not matches:
        raise ValueError(f"no sub-run entry for {name}")
    if len(matches) > 1:
        raise ValueError(f"duplicate sub-run entry for {name}")
    if matches[0].get("verdict") != "settled":
        raise ValueError(f"{name} sub-run was refused: {matches[0].get('reason')}")
    return matches[0]


def scene_evidence(run, name, case):
    record = capture_record(run)
    entry = sub_run_entry(record, name)
    recorded = {i["name"]: i["sha256"] for i in entry.get("inputs", [])}
    if recorded.get(f"{case}.kdl") != digest(run / f"{case}.kdl"):
        raise ValueError("observation config identity mismatch")
    if read_json(run / f"{name}.identity.json") != read_json(run / "identity.json"):
        raise ValueError("observation binary identity mismatch")
    renderer = (run / f"{name}.renderer.log").read_text()   # lines 141-157 of the current file continue unchanged
```

In `analyze`, replace the `session-type.txt` and `hardware.json` reads:

```python
    record = capture_record(run)
    if record["run"]["lane"] != ("dedicated" if mode == "power" else "headless"):
        raise ValueError("capture lane does not match the manifest mode")
    if record.get("preflight", {}).get("verdict") != "quiet":
        raise ValueError("preflight was not quiet")
    if mode == "power":
        if record["environment"]["session"]["type"] != "tty":
            raise ValueError("power requires dedicated tty session evidence")
        preflight = read_json(run / "preflight.json")
        check_inventory(preflight["xml"], preflight["device_users"], set(), set())
        check_devices(preflight["devices"])
    hardware = record["environment"]["gpu"]
    if not hardware["name"] or not hardware["driver"] or any(x in hardware["name"].lower() for x in ("llvmpipe", "software")):
        raise ValueError("hardware identity missing/software renderer")
    recorded_binary = {b["name"]: b["sha256"] for b in record["provenance"]["binaries"]}.get("niri")
    if recorded_binary != provenance["binary_sha256"]:
        raise ValueError("capture-time binary hash differs from build identity")
```

and keep `"hardware": hardware` in the result (its keys are now `name`/`uuid`/`driver`/`device`).

- [x] **Step 4: Run the suite**

Run: `just --justfile fixtures/idle-budget.just test`
Expected: all OK (the previous 19 plus 2).

- [x] **Step 5: Update the results doc and manifest**

In `docs/results/2026-09-11-idle-budget.md`, under the status paragraph add one sentence: "The fixture records provenance, environment, and between-observation quietness through the native `tools/capture-meta` (spec `docs/specs/2026-09-11-material-capture-protocol-design.md` in `niri-material`); `hardware.json`, `<name>.config.sha256`, and the `IDLE_BUDGET_DEDICATED_SESSION` gate are replaced by `capture.json`." Adjust the "Power execution" paragraph's sentence about `IDLE_BUDGET_DEDICATED_SESSION` to name the `dedicated` lane's preflight instead. Then regenerate the manifest:

```bash
m=docs/results/2026-09-11-idle-budget.sha256
sha256sum fixtures/idle-budget.py fixtures/idle-budget.sh fixtures/idle-budget.just fixtures/idle-budget-trace-marker.patch fixtures/test_idle_budget.py docs/results/2026-09-11-idle-budget.md > "$m.tmp" && [ -s "$m.tmp" ] && mv "$m.tmp" "$m"
```

- [x] **Step 6: Commit**

```bash
git add fixtures/idle-budget.py fixtures/test_idle_budget.py docs/results/2026-09-11-idle-budget.md docs/results/2026-09-11-idle-budget.sha256
git commit -m "feat(idle-budget): analyzer reads capture.json for hardware, identity, and scene evidence"
```

---

### Task 10: Docs, one manual check, and closeout

**Files:**
- Modify: `docs/materials/README.md` (or the evidence-doc conventions section it holds)
- Modify: `docs/materials/2026-09-11-material-hardware-evidence.md:9` (one sentence)
- Modify: `docs/specs/2026-09-11-material-capture-protocol-design.md:3-4` (status)
- Modify: `tasks/material-bae9c9.md` via the CLI only

- [x] **Step 1: Pointers**

In `docs/materials/README.md`, where evidence documents are described, add: "Every measurement run writes `capture.json` through `tools/capture-meta`; paste `tools/capture-meta show <run-dir>` into the evidence doc's environment section rather than retyping it (protocol: `../specs/2026-09-11-material-capture-protocol-design.md`)."

In the hardware evidence doc's "Environment and identity" section, add after the table: "This table was written by hand; runs after `material-bae9c9` paste `tools/capture-meta show` instead."

- [x] **Step 2: The one manual check (operator, this host)**

With the desktop running normally and any GPU-busy application open, from the worktree:

```bash
export XDG_RUNTIME_DIR=${XDG_RUNTIME_DIR:-/run/user/$(id -u)}
d=$(mktemp -d "$NIRI_MATERIAL_WORK_ROOT/material-bae9c9/headless-XXXXXX")
python3 tools/capture-meta preflight "$d" --lane headless --task material-bae9c9 --fixture manual --seconds 5; echo "exit $?"
python3 tools/capture-meta show "$d"
```

Expected: exit 1 when the GPU is busy, with the failing threshold in `reasons` and the client list in `baseline.gpu_clients.graphics`; exit 0 when quiet. Record the observed exit code, reasons, and the two or three client names in a `tasks note material-bae9c9 "…"`. Then `python3 tools/capture-meta release "$d"` and remove `$d`. This is the spec §7 hand check; it is not automated and not evidence of anything but the refusal path.

- [x] **Step 3: Spec status and task**

Change the spec's status line to: "**Status:** implemented on `material-bae9c9` (this plan); `tools/capture-meta` and the optic smoke adoption are in this repository, the idle-budget adoption on `niri-experiments` `results/capture-protocol`. Thresholds are host-derived defaults pending the first dedicated-lane run (`material-265eb0`)." Then:

```bash
tasks note material-bae9c9 "Thresholds in DEFAULT_THRESHOLDS are from this host's idle RTX 3070; material-265eb0's first dedicated run should revisit them and update the spec."
just upstream-report
git add docs/materials/README.md docs/materials/2026-09-11-material-hardware-evidence.md docs/specs/2026-09-11-material-capture-protocol-design.md docs/materials/upstream-divergence.md
tasks done material-bae9c9 "capture-meta (preflight/identity/settle/release/show) with tests; optic smoke lib and idle-budget fixture adopt it; analyzer reads capture.json"
git add tasks/
git commit -m "docs(material): capture protocol landed; point evidence docs at capture-meta show"
```

`tasks done` will refuse while the plan's step children are open; close each step child as its task commits land (Tasks 1–10), then run the `done` above.

- [x] **Step 4: Verify**

Run, as three separate commands so each exit status is seen: `just check`; then in the experiments worktree with `MATERIAL_ROOT` exported, `just --justfile fixtures/idle-budget.just test`; then `just --justfile fixtures/jelly-motion.just test`.
Expected: `OK` from each suite; `just check` also passes fmt, clippy, and `tasks check`. Then hand the branch to `superpowers:finishing-a-development-branch`.
