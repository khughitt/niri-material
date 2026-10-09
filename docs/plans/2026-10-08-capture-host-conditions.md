# Capture host conditions Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Every capture route runs on the least host its evidence needs and fails early on the wrong one. Static pixel fixtures get a `pixels` lane: it holds only timers and services and samples no GPU. Measurement launches must prove that their compositors rendered on the GPU that was sampled.

**Architecture:** `tools/capture-meta` gains:
- a `pixels` lane, with a partial hold, no sampling, and a `begin` verb in place of `settle`;
- pre-hold session, desktop and GPU-client checks, and an observed `host_condition`;
- a `renderer` verb that parses each launch's compositor logs, judges them against `environment.gpu.name`, and records the verdict on the open sub-run;
- a `release` check that refuses a measurement run with any settled sub-run whose renderer was not verified.

Every launcher (the smoke lib's `start_nested`, the three clip fixtures' own `start_nested`, and `optic-settling-smoke.sh`'s `start_drm`) starts niri with the GLES renderer log enabled and calls `renderer`. Fixtures build before they preflight. Two static pixel fixtures move to the new lane.

**Tech Stack:** Python 3 standard library (`unittest`, `re`), bash fixtures, the existing `tools/fake_capture_host.FakeHost` test double.

**Spec:** [docs/specs/2026-10-08-capture-host-conditions-design.md](../specs/2026-10-08-capture-host-conditions-design.md) (accepted, spec review round 3).

**Status:** revised after plan review round 1 (2026-10-08): hold discovery is gated by kind, `optic-settling-smoke.sh` builds before it preflights, and every `plan_hold` caller migrates.

## Global Constraints

- **Lanes:** `pixels`, `headless`, `dedicated`. `MEASURED_LANES = ("headless", "dedicated")`.
- **Thresholds:** `DEFAULT_THRESHOLDS` stays unchanged.
- **Pixel hold kinds:** `("timer", "service")`. The full hold uses `("timer", "service", "idle", "monitors")`. `plan_hold` discovers only the kinds it is given: a pixel hold never queries noctalia or the monitor connectors, so neither can refuse a pixel run.
- **`hold_settle`:** defaults to 0 on `pixels` and stays 10 on the other lanes.
- **Pixel preflight section:** `{"verdict": "unsampled", "at", "lock", "host_condition"}`.
- **Pixel sub-run entry:** `{"name", "started", "inputs", "verdict": "begun"}`.
- **`host_condition`:** `"tty"` when `XDG_SESSION_TYPE == "tty"` and no live desktop socket, otherwise `"desktop"`. It is written in `preflight` on every lane and verdict.
- **niri renderer log filter:** `RUST_LOG=niri=debug,smithay::backend::renderer::gles=info`.
- **niri renderer line:** `GL Renderer: "(.+)"`, matched after stripping `\x1b\[[0-9;]*m`.
- **Weston renderer line:** `\] GL renderer: (.+)$`.
- **Software renderer:** `llvmpipe|software rasterizer`, case-insensitive.
- **Renderer verdicts:**
  - per compositor: `verified`, `missing`, `mismatch` (measured lanes), or `recorded`, `missing` (`pixels`);
  - overall: `verified` only when every compositor is; on `pixels` it is always `recorded`.
- **`renderer` exit codes:**
  - 2 for no open sub-run of that name, a second call on one sub-run, an unreadable log, `--weston-log` missing on `headless`/`pixels` or given on `dedicated`, or no `environment.gpu.name` on a measured lane;
  - 1 for a measured-lane verdict other than `verified`;
  - 0 otherwise.
- **`release` coverage:** on a measured lane, after the hold is restored cleanly, exit 1 if any `sub_runs` entry with `verdict == "settled"` lacks `renderer.verdict == "verified"`. That holds whether the entry is finished or open. Refused settles are excluded.
- **Lane in shell:** `capture_preflight` sets `CAPTURE_LANE`, and the lib reads that variable (spec §6.2 named `show --field`; Task 7 amends the spec).
- **Schema:** `capture.json` stays `schema: 1`, and every new field is optional.
- **Python tooling gates:**
  - one module: `just --set one_cmd 'env NIRI_TOOLING_FAST=0 python3 -m unittest' test-one tools.<module>`;
  - all tooling: `just --set fast_cmd 'env NIRI_TOOLING_FAST=0 python3 -m tools.tooling_tests --full' test-fast`.

  Never call a test runner directly.
- **Commits:**
  - conventional commits, no AI attribution trailers;
  - `tasks start <step id>` before the step's first change, and `tasks done <step id> "<what landed>"` in the step's commit;
  - before every step commit, the all-tooling `test-fast` run above passes, and then `tasks check`.
- **Live runs:** none in this plan. Captures are the pilot tasks Task 7 files.

## Review Focus

1. **A stale renderer line from an earlier launch.** Logs are appended across
   launches. A slice must start at this launch's offset, or an earlier NVIDIA
   line could verify a later llvmpipe launch. This is pinned in Task 4
   (`test_verify_renderer_reads_only_this_launchs_bytes`).
2. **A renderer line not yet written when the launcher reads it.** The lib reads
   after its existing `sleep 1`, and the clip launchers read after
   `wait_kitty 1`, by which point a frame has been drawn. This is pinned by the
   order assertions in Task 4 and Task 5.
3. **A repeated sub-run name** (re-settled after finish, as `calibrate_probe_rect`
   and idle-budget observations do). `renderer` attaches to the latest *open*
   entry of that name, and both entries keep their own verdict. This is pinned
   in Task 3 (`test_renderer_attaches_to_the_latest_open_entry`).
4. **A dead desktop socket file in a TTY session.** It is not a live desktop and
   must not refuse a dedicated run. This is pinned in Task 1
   (`test_dedicated_ignores_a_dead_desktop_socket`).
5. **An old schema-1 record**, with no `host_condition`, no `renderer`, and no
   `environment.gpu` on a pixel run. `show` must still render it. This is
   pinned in Task 2 (`test_show_renders_old_and_pixel_records`).

---

### Task 1: Pre-hold checks and the observed host condition

**Files:**
- Modify: `tools/capture_hold.py` (`plan_hold`)
- Modify: `tools/capture-meta` (`client_reasons`, `judge_quiet`, `judge_settled`, `preflight`, new `pre_hold_reasons`, new `host_condition`)
- Test: `tools/test_capture_meta.py` (new tests; `LifecycleTests.hold_without_preflight` migrates), `tools/test_capture_hold.py`

**Interfaces:**
- Produces:
  - `ch.HOLD_KINDS = ("timer", "service", "idle", "monitors")`
  - `ch.plan_hold(host, socket, kinds)`, where `socket` is `str | None` and comes from `ch.desktop_socket(host)`, and `kinds` is a tuple of item kinds
  - `cm.client_reasons(inventories, lane)`, where `inventories` is a list of `{"compute": [...], "graphics": [...]}`
  - `cm.host_condition(socket) -> "tty" | "desktop"`
  - `cm.pre_hold_reasons(lane, socket, gpu) -> list[str]`
  - `cm.MEASURED_LANES`

- [ ] **Step 1: Write the failing tests**

In `tools/test_capture_meta.py`, add a module-level helper after `ch_hold_name()`:

```python
HOLD_ACTIONS = (("systemd-run",), ("systemctl", "--user", "stop"), ("noctalia", "msg", "caffeine-enable"),
                ("niri", "msg", "action", "power-off-monitors"))


def hold_actions(host):
    return [c for c in host.calls if any(tuple(c[:len(a)]) == a for a in HOLD_ACTIONS)]
```

Add these tests to `LifecycleTests`. Its host has a live desktop socket, a
timer, a declared service and a lit connector.

```python
    def lane_preflight(self, lane, gpu="quiet", env=None):   # gpu=None: no sampler (pixels lane)
        saved = {k: os.environ.get(k) for k in ("XDG_SESSION_TYPE", "DISPLAY", "WAYLAND_DISPLAY")}
        for key in ("DISPLAY", "WAYLAND_DISPLAY"):
            os.environ.pop(key, None)
        os.environ.update(env or {"XDG_SESSION_TYPE": "tty"})
        try:
            return cm.preflight(self.run, lane=lane, task="material-18c2a1", fixture="t.sh", seconds=3,
                                owner_pid=4242, thresholds=cm.DEFAULT_THRESHOLDS, tools=["tracy=0.13.1"],
                                proc=FakeProc([(i * 2, i * 100) for i in range(30)]),
                                gpu=FakeGpu([QUIET] * 30) if gpu == "quiet" else gpu,
                                host_load=FakeHostLoad(), host=self.host, sleep=lambda s: None,
                                lock=lambda: self.lock, hold_settle=0)
        finally:
            for key, value in saved.items():
                if value is None: os.environ.pop(key, None)
                else: os.environ[key] = value

    def test_dedicated_refuses_a_live_desktop_before_holding(self):
        with self.assertRaises(cm.Refused):
            self.lane_preflight("dedicated")
        preflight = cm.load_record(self.run)["preflight"]
        self.assertEqual(preflight["verdict"], "refused")
        self.assertIn("a desktop niri is running", " ".join(preflight["reasons"]))
        self.assertEqual(preflight["host_condition"], "desktop")
        self.assertEqual(hold_actions(self.host), [])
        self.assertIsNone(cm.read_lock(self.lock))
        self.assertNotIn("baseline", cm.load_record(self.run))

    def test_dedicated_refuses_a_graphics_client_before_holding(self):
        self.host.live.clear()
        gpu = FakeGpu([QUIET] * 30, clients={"compute": [], "graphics": ["niri"]})
        with self.assertRaises(cm.Refused):
            self.lane_preflight("dedicated", gpu=gpu)
        self.assertIn("gpu_clients present on the dedicated lane: niri",
                      cm.load_record(self.run)["preflight"]["reasons"])
        self.assertEqual(hold_actions(self.host), [])

    def test_dedicated_refuses_display_variables_before_holding(self):
        self.host.live.clear()
        with self.assertRaises(cm.Refused):
            self.lane_preflight("dedicated", env={"XDG_SESSION_TYPE": "tty", "WAYLAND_DISPLAY": "wayland-1"})
        self.assertEqual(hold_actions(self.host), [])

    def test_headless_refuses_a_compute_client_before_holding(self):
        gpu = FakeGpu([QUIET] * 30, clients={"compute": ["python3"], "graphics": []})
        with self.assertRaises(cm.Refused):
            self.lane_preflight("headless", gpu=gpu, env={"XDG_SESSION_TYPE": "wayland"})
        self.assertIn("compute clients present: python3", cm.load_record(self.run)["preflight"]["reasons"])
        self.assertEqual(hold_actions(self.host), [])

    def test_dedicated_ignores_a_dead_desktop_socket(self):
        self.host.live.clear()   # the socket file stays; niri msg version fails against it
        self.lane_preflight("dedicated", gpu=FakeGpu([QUIET] * 30, clients={"compute": [], "graphics": []}))
        preflight = cm.load_record(self.run)["preflight"]
        self.assertEqual((preflight["verdict"], preflight["host_condition"]), ("quiet", "tty"))
        self.assertEqual(cm.main(["release", str(self.run)]), 0)

    def test_host_condition_is_desktop_whenever_a_desktop_is_live(self):
        self.lane_preflight("headless", env={"XDG_SESSION_TYPE": "tty"})
        self.assertEqual(cm.load_record(self.run)["preflight"]["host_condition"], "desktop")
```

Before relying on it, check that `FakeHost` exposes `live` as a set. It is
assigned `self.live = set(live)` in `__init__`. Also check that `run()` of
`niri msg version` fails for a socket that is not in `live`.

Migrate every existing caller to the new signature. There are two sets:

- every `ch.plan_hold(host)` and `ch.plan_hold(self.host_)` call in
  `tools/test_capture_hold.py`, which becomes
  `ch.plan_hold(host, ch.desktop_socket(host), ch.HOLD_KINDS)`;
- `LifecycleTests.hold_without_preflight` in `tools/test_capture_meta.py`,
  which recovery cases use. Its `cm.ch.plan_hold(self.host)` becomes
  `cm.ch.plan_hold(self.host, cm.ch.desktop_socket(self.host), cm.ch.HOLD_KINDS)`.

When done, `grep -n 'plan_hold(' tools/` must show no one-argument call. Then
add to `PlanTests` in `tools/test_capture_hold.py`:

```python
    def test_a_timers_and_services_plan_never_probes_idle_or_monitors(self):
        # A desktop whose noctalia status fails and whose connector cannot be read:
        # the full hold refuses on either; a pixel hold must not query them.
        def host():
            h = self.host(timers=["wali-rotate.timer"], sockets=["niri.w.1.sock"], live=["niri.w.1.sock"],
                          connectors=LIT)
            h.fail[("noctalia", "msg", "status")] = "noctalia: no shell running"
            (h.sysfs / "card1-DP-1" / "enabled").unlink()
            return h
        full = host()
        with self.assertRaises(ch.CannotRun):
            ch.plan_hold(full, ch.desktop_socket(full), ch.HOLD_KINDS)
        pixel = host()
        plan = ch.plan_hold(pixel, ch.desktop_socket(pixel), ("timer", "service"))
        self.assertEqual([i["kind"] for i in plan["items"]], ["timer"])
        self.assertEqual((plan["desktop"], plan["not_held"]), ("present", []))
        self.assertNotIn(("noctalia", "msg", "status"), pixel.calls)
```

The control half (`full`) proves the probes would refuse: `checked` raises
`CannotRun` for the failed status, before the unreadable connector is reached.

- [ ] **Step 2: Run the tests and check that they fail**

Run: `just --set one_cmd 'env NIRI_TOOLING_FAST=0 python3 -m unittest' test-one tools.test_capture_meta tools.test_capture_hold`
Expected: the new tests fail. The dedicated desktop case reaches the hold and
calls `systemctl --user stop`; `host_condition` raises `KeyError`; and
`plan_hold` rejects its extra arguments with a `TypeError`, in the migrated
callers and in the new probe test.

- [ ] **Step 3: Implement**

In `tools/capture_hold.py`:

```python
HOLD_KINDS = ("timer", "service", "idle", "monitors")
```

Change `def plan_hold(host):` to `def plan_hold(host, socket, kinds):`.
Delete its line `socket = desktop_socket(host)`.

Gate discovery itself by kind. Filtering the items afterwards is not enough:
by then the noctalia status call and `lit_connectors` have already run, and
either can raise `CannotRun`. Timers and services are discovered as before:
both hold kinds include them, and the wake scan needs `timer_map` on every
lane. Replace the `if socket:` block with:

```python
    if socket and "idle" in kinds:
        if host.has("noctalia"):
            try:
                locked = json.loads(checked(host, "noctalia", "msg", "status")).get("locked")
            except (json.JSONDecodeError, AttributeError) as error:
                raise CannotRun(f"noctalia msg status: unexpected output: {error}") from error
            items.append({"kind": "idle", "action": "caffeine-enabled", "prior": "unknown", "locked": locked,
                          "socket": socket, "restore": "noctalia msg caffeine-disable"})
        else:
            not_held.append({"kind": "idle", "reason": "noctalia not on PATH"})
    if socket and "monitors" in kinds:
        items.append({"kind": "monitors", "action": "powered-off", "socket": socket,
                      "connectors": lit_connectors(host),
                      "restore": f"NIRI_SOCKET={socket} niri msg action power-on-monitors"})
```

Update the docstring to `"""Discover what to hold, probing only the kinds
asked for. Changes nothing on the host."""`.

In `tools/capture-meta`, beside `HOST_LOAD_KEYS`:

```python
MEASURED_LANES = ("headless", "dedicated")
```

Replace `client_reasons` and its two callers:

```python
def client_reasons(inventories, lane):
    reasons = []
    compute = sorted({c for inv in inventories for c in inv["compute"]})
    if compute: reasons.append(f"compute clients present: {', '.join(compute)}")
    if lane == "dedicated":
        graphics = sorted({c for inv in inventories for c in inv["graphics"]})
        if graphics: reasons.append(f"gpu_clients present on the dedicated lane: {', '.join(graphics)}")
    return reasons
```

In `judge_quiet` and `judge_settled`, change
`client_reasons(samples, lane)` to
`client_reasons([s.gpu_clients for s in samples], lane)`.

After `session_reasons`, add:

```python
def host_condition(socket):
    """Observed, never declared: whether someone is working at a live desktop is not visible here."""
    return "tty" if os.environ.get("XDG_SESSION_TYPE") == "tty" and not socket else "desktop"


def pre_hold_reasons(lane, socket, gpu):
    """Everything that can refuse before the hold changes the host (spec §7)."""
    reasons = session_reasons(lane)
    if lane == "dedicated" and socket:
        reasons.append(f"a desktop niri is running at {socket}; the dedicated lane needs the desktop stopped")
    if lane in MEASURED_LANES:
        reasons += client_reasons([gpu.clients()], lane)
    return reasons
```

In `preflight`, replace the lines from `recovered = recover_stale(host, lock_file)`
through `plan = ch.plan_hold(host)` with:

```python
        recovered = recover_stale(host, lock_file)
        socket = ch.desktop_socket(host)
        condition = host_condition(socket)
        early = pre_hold_reasons(lane, socket, gpu)
        if early:
            write_section(run_dir, "preflight", {"verdict": "refused", "at": now_rfc3339(), "reasons": early,
                                                 "thresholds": thresholds, "lock": lock_info,
                                                 "host_condition": condition})
            raise Refused("; ".join(early))
        plan = ch.plan_hold(host, socket, ch.HOLD_KINDS)
```

Delete the later `reasons = session_reasons(lane)` line, and change the next
line to `reasons = judge_quiet(summary, samples, thresholds, lane)`, which
replaces `reasons += …`. Add `"host_condition": condition` to the `section`
dict built after sampling.

The `except BaseException` block already releases the lock when `held` is
false, and that is the path a pre-hold refusal takes.

- [ ] **Step 4: Run the tests and check that they pass**

Run: `just --set one_cmd 'env NIRI_TOOLING_FAST=0 python3 -m unittest' test-one tools.test_capture_meta tools.test_capture_hold`
Expected: PASS, including the existing `test_dedicated_requires_tty_and_no_clients`.
Its graphics-client case now refuses before the hold, with the same reason
text.

- [ ] **Step 5: Commit**

```bash
just --set fast_cmd 'env NIRI_TOOLING_FAST=0 python3 -m tools.tooling_tests --full' test-fast
tasks check
git add tools/capture_hold.py tools/capture-meta tools/test_capture_meta.py tools/test_capture_hold.py tasks/
git commit -m "feat(capture-meta): refuse lane mismatches before the hold and record the host condition"
```

---

### Task 2: The `pixels` lane and `begin`

**Files:**
- Modify: `tools/capture-meta` (`environment`, `validate_record`, `preflight`, `cmd_preflight`, `settle`, new `begin`/`cmd_begin`, new `require_lock`, `finish_sub_run`, `render`, `build_parser`)
- Test: `tools/test_capture_meta.py`

**Interfaces:**
- Consumes: `ch.plan_hold(host, socket, kinds)`, `host_condition`, `MEASURED_LANES` (Task 1)
- Produces:
  - `cm.PIXEL_HOLD_KINDS = ("timer", "service")`
  - `cm.begin(run_dir, sub_run, inputs, lock=lock_path)`
  - CLI `capture-meta begin <run-dir> --sub-run NAME [--input PATH]...`
  - `--lane pixels`
  - `finish` accepts `begun` entries

- [ ] **Step 1: Write the failing tests**

Add to `LifecycleTests`:

```python
    def test_pixel_preflight_holds_timers_and_services_only_and_samples_nothing(self):
        self.lane_preflight("pixels", gpu=None, env={"XDG_SESSION_TYPE": "wayland"})
        record = cm.load_record(self.run)
        self.assertEqual([i.get("unit", i["kind"]) for i in record["hold"]["items"]],
                         ["wali-rotate.timer", "dropbox.service"])
        self.assertNotIn(("noctalia", "msg", "caffeine-enable"), [tuple(c) for c in self.host.calls])
        self.assertFalse([c for c in self.host.calls if "power-off-monitors" in c])
        self.assertNotIn("baseline", record)
        self.assertNotIn("gpu", record["environment"])
        self.assertEqual(record["preflight"]["verdict"], "unsampled")
        self.assertEqual(record["preflight"]["host_condition"], "desktop")
        self.assertNotIn("thresholds", record["preflight"])
        self.assertEqual(cm.main(["release", str(self.run)]), 0)
        self.assertEqual(self.host.snapshot(), self.before)

    def test_pixel_preflight_never_probes_idle_or_monitors(self):
        # Either probe refuses a measured hold; a pixel run must not reach them.
        self.host.fail[("noctalia", "msg", "status")] = "noctalia: no shell running"
        (self.host.sysfs / "card1-DP-1" / "enabled").unlink()
        self.lane_preflight("pixels", gpu=None, env={"XDG_SESSION_TYPE": "wayland"})
        self.assertEqual(cm.load_record(self.run)["preflight"]["verdict"], "unsampled")
        self.assertNotIn(("noctalia", "msg", "status"), [tuple(c) for c in self.host.calls])
        self.assertEqual(cm.main(["release", str(self.run)]), 0)

    def test_pixel_preflight_records_the_gpu_when_one_is_present(self):
        self.lane_preflight("pixels", gpu=FakeGpu([]), env={"XDG_SESSION_TYPE": "wayland"})
        self.assertEqual(cm.load_record(self.run)["environment"]["gpu"]["name"], "NVIDIA test")

    def test_begin_opens_a_pixel_sub_run_and_finish_closes_it(self):
        self.lane_preflight("pixels", gpu=None, env={"XDG_SESSION_TYPE": "wayland"})
        (self.run / "A.kdl").write_text("glass")
        cm.begin(self.run, "A", [self.run / "A.kdl"], lock=lambda: self.lock)
        entry = cm.load_record(self.run)["sub_runs"][-1]
        self.assertEqual((entry["name"], entry["verdict"]), ("A", "begun"))
        self.assertEqual(entry["inputs"][0]["name"], "A.kdl")
        cm.finish_sub_run(self.run, "A")
        self.assertIn("finished", cm.load_record(self.run)["sub_runs"][-1])

    def test_settle_refuses_a_pixel_run_and_begin_refuses_a_measured_run(self):
        self.lane_preflight("pixels", gpu=None, env={"XDG_SESSION_TYPE": "wayland"})
        with self.assertRaisesRegex(cm.CannotRun, "pixels run opens sub-runs with begin"):
            cm.settle(self.run, "A", [], 3, FakeProc([(i * 2, i * 100) for i in range(9)]),
                      FakeGpu([QUIET] * 3), sleep=lambda s: None, lock=lambda: self.lock)
        other = self.root / "runs" / "measured"; other.mkdir()
        PreflightTests("test_quiet_headless_writes_run_environment_baseline_preflight").run_preflight(
            other, lock=lambda: other / "lock")
        with self.assertRaisesRegex(cm.CannotRun, "begin is for pixels runs"):
            cm.begin(other, "A", [], lock=lambda: other / "lock")
```

Add to `ShowTests`:

```python
    def test_show_renders_old_and_pixel_records(self):
        old = {"schema": 1, "run": {"id": "old", "lane": "headless", "started": STARTED},
               "preflight": {"verdict": "quiet"},
               "sub_runs": [{"name": "A", "verdict": "settled", "started": STARTED}]}
        self.assertIn("A: settled", cm.render(old))
        pixel = {"schema": 1, "run": {"id": "px", "lane": "pixels", "started": STARTED},
                 "environment": {"kernel": "k", "cpu": "c", "cpu_threads": 1, "memory_total_kib": 1,
                                 "session": {"type": "wayland", "display": "w"}, "tools": {}},
                 "preflight": {"verdict": "unsampled", "at": STARTED, "host_condition": "desktop"},
                 "sub_runs": [{"name": "cell-1", "verdict": "begun", "started": STARTED}]}
        cm.validate_record(pixel)
        text = cm.render(pixel)
        for needle in ("GPU not sampled: pixels lane", "preflight unsampled", "host desktop",
                       "cell-1: begun", "never finished"):
            self.assertIn(needle, text)
```

`STARTED` is the existing module constant used by `SettleTests`. If it is
defined inside a class, use the literal `"2026-10-06T10:00:00-04:00"`.

- [ ] **Step 2: Run the tests and check that they fail**

Run: `just --set one_cmd 'env NIRI_TOOLING_FAST=0 python3 -m unittest' test-one tools.test_capture_meta`
Expected: FAIL. `pixels` reaches `environment(None, …)` and raises
`AttributeError`, `cm.begin` does not exist, and `validate_record` rejects an
environment with no `gpu`.

- [ ] **Step 3: Implement**

In `environment(gpu, tools, proc)`, build the dict without `gpu`, then:

```python
    if gpu is not None:
        env["gpu"] = {**gpu.static(), "device": os.environ.get("CAPTURE_META_RENDER_NODE", "/dev/dri/renderD128")}
    return env
```

In `validate_record`, replace the environment loop with:

```python
        for name in ("session", "tools"):
            if not isinstance(environment.get(name), dict):
                raise CannotRun(f"{path}: environment.{name} must be an object")
        if "gpu" in environment and not isinstance(environment["gpu"], dict):
            raise CannotRun(f"{path}: environment.gpu must be an object")
```

Beside `MEASURED_LANES`:

```python
PIXEL_HOLD_KINDS = ("timer", "service")
```

In `preflight`:

- pick the hold kinds by lane:
  `kinds = ch.HOLD_KINDS if lane in MEASURED_LANES else PIXEL_HOLD_KINDS`;
- pass `kinds` to `ch.plan_hold(host, socket, kinds)`;
- right after `sleep(hold_settle)`, add the pixel branch:

```python
        if lane == "pixels":
            write_section(run_dir, "environment", environment(gpu, tools, proc))
            write_section(run_dir, "preflight", {"verdict": "unsampled", "at": now_rfc3339(), "lock": lock_info,
                                                 "host_condition": condition})
            return
```

In `cmd_preflight`:

```python
def cmd_preflight(args):
    measured = args.lane in MEASURED_LANES
    gpu = GpuReader() if measured or shutil.which("nvidia-smi") else None
    hold_settle = args.hold_settle if args.hold_settle is not None else (10 if measured else 0)
    preflight(args.run_dir, args.lane, args.task, args.fixture, args.seconds,
              args.owner_pid if args.owner_pid is not None else os.getppid(),
              parse_thresholds(args.threshold), args.tool, ProcReader(), gpu,
              HostLoadReader() if measured else None, Host(), hold_settle=hold_settle)
```

Factor the lock check out of `settle` into:

```python
def require_lock(record, run_dir, lock):
    expected = record["preflight"].get("lock")
    holder = read_lock(lock())
    if (not isinstance(expected, dict) or holder is None or holder.get("run_id") != record["run"].get("id")
            or holder.get("owner_pid") != expected.get("owner_pid")
            or holder.get("run_dir") != str(pathlib.Path(run_dir).resolve())):
        raise Refused(f"lock no longer names this run ({record['run'].get('id')}); holder: {holder}")
```

`settle` calls `require_lock(record, run_dir, lock)` where its inline check
was. Keep `settle`'s existing `owner_pid` type validation. As the first check
after loading the record, add:

```python
    if record.get("run", {}).get("lane") == "pixels":
        raise CannotRun("settle samples the GPU; a pixels run opens sub-runs with begin")
```

Change its lane check to `if record["run"].get("lane") not in MEASURED_LANES:`.

Add:

```python
def begin(run_dir, sub_run, inputs, lock=lock_path):
    """Opens a sub-run on the pixels lane: its inputs, no samples (spec §6.2)."""
    run_dir = pathlib.Path(run_dir)
    record = load_record(run_dir)
    for section in ("run", "preflight"):
        if section not in record:
            raise CannotRun(f"begin needs a preflighted run; {RECORD} has no {section!r}")
    if record["run"].get("lane") != "pixels":
        raise CannotRun("begin is for pixels runs; measured lanes settle")
    if record["preflight"].get("verdict") != "unsampled":
        raise Refused("preflight did not pass; nothing to begin")
    require_lock(record, run_dir, lock)
    append_sub_run(run_dir, {"name": sub_run, "started": now_rfc3339(), "inputs": hashed(inputs),
                             "verdict": "begun"})


def cmd_begin(args):
    begin(args.run_dir, args.sub_run, args.input)
```

In `finish_sub_run`, change the verdict check to:

```python
        if entry["verdict"] not in ("settled", "begun"):
            raise CannotRun(f"sub-run {sub_run!r} was {entry['verdict']}; nothing ran to finish")
```

In `render`:

- after the environment block:
  `if run.get("lane") == "pixels": lines.append("  GPU not sampled: pixels lane")`;
- in the preflight block, append `f"  host {preflight['host_condition']}"` to the
  `preflight …` line when the key is present;
- in the sub-run loop, treat `begun` like `settled` for the
  "never finished" span: `elif "started" in entry and entry["verdict"] in ("settled", "begun"):`.

In `build_parser`:

- `pre.add_argument("--lane", choices=("pixels", "headless", "dedicated"), required=True)`;
- `pre.add_argument("--hold-settle", type=int, default=None)`;
- add a `begin` subparser:

```python
    bg = subs.add_parser("begin")
    bg.add_argument("run_dir")
    bg.add_argument("--sub-run", required=True)
    bg.add_argument("--input", action="append", default=[])
    bg.set_defaults(func=cmd_begin)
```

- [ ] **Step 4: Run the tests and check that they pass**

Run: `just --set one_cmd 'env NIRI_TOOLING_FAST=0 python3 -m unittest' test-one tools.test_capture_meta`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
just --set fast_cmd 'env NIRI_TOOLING_FAST=0 python3 -m tools.tooling_tests --full' test-fast
tasks check
git add tools/capture-meta tools/test_capture_meta.py tasks/
git commit -m "feat(capture-meta): add the pixels lane with a partial hold and begin"
```

---

### Task 3: The `renderer` verb and release coverage

**Files:**
- Modify: `tools/capture-meta` (new `renderer`/`cmd_renderer` and helpers, `validate_record`, `release`, `render`, `build_parser`)
- Create: `tools/testdata/renderer/niri-nvidia.log`, `niri-default.log`, `niri-llvmpipe.log`, `weston-nvidia.log`
- Test: `tools/test_capture_meta.py`

**Interfaces:**
- Consumes: `MEASURED_LANES`, `begin` (Task 2)
- Produces:
  - `cm.renderer(run_dir, sub_run, niri_log, weston_log)`, with `weston_log` either `pathlib.Path` or `None`
  - CLI `capture-meta renderer <run-dir> --sub-run NAME --niri-log PATH [--weston-log PATH]`
  - `cm.unverified_launches(record) -> list[str]`
  - the sub-run field `renderer: {"expected", "at", "niri": {"verdict", "lines"}, "weston"?: {...}, "verdict"}`

- [ ] **Step 1: Create the log fixtures from the actual formats**

`tools/testdata/renderer/niri-nvidia.log` holds the line exactly as niri
writes it, ANSI escapes included. The first line below was copied from a
retained `hidden-window-attribution.sh` run on 2026-09-28. Write it with
printf so the escapes are real bytes:

```bash
mkdir -p tools/testdata/renderer
printf '\033[2m2026-09-28T03:14:41.036811Z\033[0m \033[32m INFO\033[0m \033[1mrenderer_gles2\033[0m: \033[2msmithay::backend::renderer::gles\033[0m\033[2m:\033[0m GL Renderer: "NVIDIA GeForce RTX 3070/PCIe/SSE2"\n\033[2m2026-09-28T03:14:41.040000Z\033[0m \033[34mDEBUG\033[0m \033[2mniri\033[0m: started\n' > tools/testdata/renderer/niri-nvidia.log
printf '\033[2m2026-10-08T03:43:57.000000Z\033[0m \033[34mDEBUG\033[0m \033[2mniri\033[0m: started\n' > tools/testdata/renderer/niri-default.log
printf '\033[2m2026-10-08T03:43:57.000000Z\033[0m \033[32m INFO\033[0m \033[2msmithay::backend::renderer::gles\033[0m\033[2m:\033[0m GL Renderer: "llvmpipe (LLVM 19.1.7, 256 bits)"\n' > tools/testdata/renderer/niri-llvmpipe.log
printf '[21:51:39.201] Loading module '"'"'/usr/lib/libweston-15/gl-renderer.so'"'"'\n[21:51:39.332] GL renderer: NVIDIA GeForce RTX 3070/PCIe/SSE2\n[21:51:39.336] GL ES 3.2 - renderer features:\n' > tools/testdata/renderer/weston-nvidia.log
```

The Weston lines come from the retained `noise-layers-pilot-1791337568/weston.log`.

- [ ] **Step 2: Write the failing tests**

Add a class to `tools/test_capture_meta.py`:

```python
DATA = pathlib.Path(__file__).with_name("testdata") / "renderer"


class RendererTests(unittest.TestCase):
    def setUp(self):
        temp = tempfile.TemporaryDirectory(); self.addCleanup(temp.cleanup)
        self.root = pathlib.Path(temp.name)

    def measured(self, lane="headless", gpu_name="NVIDIA GeForce RTX 3070", name=None):
        run = self.root / (name or f"{lane}-run"); run.mkdir()
        cm.write_section(run, "run", {"id": run.name, "lane": lane, "started": STARTED})
        cm.write_section(run, "environment", {"gpu": {"name": gpu_name}, "session": {}, "tools": {}})
        cm.append_sub_run(run, {"name": "A", "verdict": "settled", "started": STARTED})
        return run

    def weston(self, text=None):
        path = self.root / "weston.slice"
        path.write_text(text if text is not None else (DATA / "weston-nvidia.log").read_text())
        return path

    def test_both_compositors_on_the_sampled_gpu_verify(self):
        run = self.measured()
        cm.renderer(run, "A", DATA / "niri-nvidia.log", self.weston())
        r = cm.load_record(run)["sub_runs"][-1]["renderer"]
        self.assertEqual((r["verdict"], r["niri"]["verdict"], r["weston"]["verdict"]),
                         ("verified", "verified", "verified"))
        self.assertEqual(r["niri"]["lines"], ["NVIDIA GeForce RTX 3070/PCIe/SSE2"])
        self.assertEqual(r["expected"], "NVIDIA GeForce RTX 3070")

    def test_default_filter_llvmpipe_and_other_gpu_are_recorded_then_refused(self):
        for i, (log, gpu, niri_verdict) in enumerate(((DATA / "niri-default.log", "NVIDIA GeForce RTX 3070", "missing"),
                                                      (DATA / "niri-llvmpipe.log", "NVIDIA GeForce RTX 3070", "mismatch"),
                                                      (DATA / "niri-nvidia.log", "NVIDIA GeForce RTX 4090", "mismatch"))):
            with self.subTest(log=log.name, gpu=gpu):
                run = self.measured(gpu_name=gpu, name=f"case-{i}")
                with self.assertRaises(cm.Refused):
                    cm.renderer(run, "A", log, self.weston())
                entry = cm.load_record(run)["sub_runs"][-1]
                self.assertNotIn("finished", entry)
                self.assertEqual(entry["renderer"]["niri"]["verdict"], niri_verdict)
                self.assertNotEqual(entry["renderer"]["verdict"], "verified")

    def test_weston_spelled_like_niri_is_missing(self):
        run = self.measured()
        with self.assertRaises(cm.Refused):
            cm.renderer(run, "A", DATA / "niri-nvidia.log",
                        self.weston("[21:51:39.332] GL Renderer: NVIDIA GeForce RTX 3070/PCIe/SSE2\n"))
        self.assertEqual(cm.load_record(run)["sub_runs"][-1]["renderer"]["weston"]["verdict"], "missing")

    def test_weston_log_is_required_nested_and_refused_on_drm(self):
        with self.assertRaisesRegex(cm.CannotRun, "--weston-log"):
            cm.renderer(self.measured(), "A", DATA / "niri-nvidia.log", None)
        with self.assertRaisesRegex(cm.CannotRun, "no Weston"):
            cm.renderer(self.measured("dedicated"), "A", DATA / "niri-nvidia.log", self.weston())
        drm = self.measured("dedicated", name="drm-ok")
        cm.renderer(drm, "A", DATA / "niri-nvidia.log", None)
        self.assertNotIn("weston", cm.load_record(drm)["sub_runs"][-1]["renderer"])

    def test_second_call_and_no_open_entry_cannot_run(self):
        run = self.measured()
        cm.renderer(run, "A", DATA / "niri-nvidia.log", self.weston())
        with self.assertRaisesRegex(cm.CannotRun, "already"):
            cm.renderer(run, "A", DATA / "niri-nvidia.log", self.weston())
        with self.assertRaisesRegex(cm.CannotRun, "no open sub-run"):
            cm.renderer(run, "B", DATA / "niri-nvidia.log", self.weston())

    def test_renderer_attaches_to_the_latest_open_entry(self):
        run = self.measured()
        cm.renderer(run, "A", DATA / "niri-nvidia.log", self.weston())
        cm.finish_sub_run(run, "A")
        cm.append_sub_run(run, {"name": "A", "verdict": "settled", "started": STARTED})
        with self.assertRaises(cm.Refused):
            cm.renderer(run, "A", DATA / "niri-llvmpipe.log", self.weston())
        first, second = cm.load_record(run)["sub_runs"]
        self.assertEqual((first["renderer"]["verdict"], second["renderer"]["niri"]["verdict"]),
                         ("verified", "mismatch"))

    def test_pixels_records_without_requiring(self):
        run = self.root / "px"; run.mkdir()
        cm.write_section(run, "run", {"id": "px", "lane": "pixels", "started": STARTED})
        cm.write_section(run, "environment", {"session": {}, "tools": {}})
        cm.append_sub_run(run, {"name": "A", "verdict": "begun", "started": STARTED})
        cm.renderer(run, "A", DATA / "niri-default.log", self.weston())
        r = cm.load_record(run)["sub_runs"][-1]["renderer"]
        self.assertEqual((r["verdict"], r["niri"]["verdict"], r["weston"]["verdict"]),
                         ("recorded", "missing", "recorded"))
        self.assertNotIn("expected", r)

    def test_unverified_launches_counts_settled_entries_finished_or_open(self):
        record = {"run": {"lane": "dedicated"}, "sub_runs": [
            {"name": "open", "verdict": "settled"},
            {"name": "done", "verdict": "settled", "finished": STARTED},
            {"name": "ok", "verdict": "settled", "renderer": {"verdict": "verified"}},
            {"name": "refused", "verdict": "refused"}]}
        self.assertEqual(cm.unverified_launches(record), ["open", "done"])
        record["run"]["lane"] = "pixels"
        self.assertEqual(cm.unverified_launches(record), [])
```

Add to `LifecycleTests` (the hold is real here):

```python
    def settle_one(self, name, clients=None):
        (self.run / "A.kdl").write_text("glass")
        cm.settle(self.run, name, [self.run / "A.kdl"], 3, FakeProc([(i * 2, i * 100) for i in range(9)]),
                  FakeGpu([QUIET] * 3, clients=clients), sleep=lambda s: None, lock=lambda: self.lock)

    def test_release_refuses_an_unverified_finished_launch_after_restoring(self):
        self.preflight()
        self.settle_one("A"); cm.finish_sub_run(self.run, "A")
        self.assertEqual(cm.main(["release", str(self.run)]), 1)
        self.assertEqual(self.host.snapshot(), self.before)
        self.assertIsNone(cm.read_lock(self.lock))
        self.assertEqual(cm.main(["release", str(self.run)]), 1)   # repeats

    def test_release_refuses_an_open_unverified_dedicated_launch(self):
        self.host.live.clear()
        self.lane_preflight("dedicated", gpu=FakeGpu([QUIET] * 30, clients={"compute": [], "graphics": []}))
        # FakeGpu reports a graphics client by default, which the dedicated lane refuses.
        self.settle_one("drm-aurora", clients={"compute": [], "graphics": []})   # never finished, as idle-budget power entries are today
        self.assertEqual(cm.main(["release", str(self.run)]), 1)
        self.assertIsNone(cm.read_lock(self.lock))

    def test_release_passes_a_verified_launch_and_ignores_a_refused_settle(self):
        self.preflight()
        self.settle_one("A")
        weston = self.root / "w.log"; weston.write_text((DATA / "weston-nvidia.log").read_text())
        # FakeGpu's static name is "NVIDIA test"; the fixture names a real GPU, so pin the expected name.
        record = cm.load_record(self.run); record["environment"]["gpu"]["name"] = "NVIDIA GeForce RTX 3070"
        cm.save_record(self.run, record)
        cm.renderer(self.run, "A", DATA / "niri-nvidia.log", weston)
        cm.finish_sub_run(self.run, "A")
        cm.append_sub_run(self.run, {"name": "B", "verdict": "refused", "reason": "gpu busy"})
        self.assertEqual(cm.main(["release", str(self.run)]), 0)
```

In `EndToEndTest.test_real_binary_against_fake_nvidia_smi`, insert a
`renderer` call between `settle` and `finish`, with log files the test writes.
The fake `nvidia-smi` reports the GPU name the test already uses, so write a
niri line that contains that name:

```python
            (root / "niri.slice").write_text(f'INFO smithay::backend::renderer::gles: GL Renderer: "{GPU_NAME}/PCIe/SSE2"\n')
            (root / "weston.slice").write_text(f"[00:00:00.000] GL renderer: {GPU_NAME}/PCIe/SSE2\n")
            code, _, stderr = cm_run("renderer", str(run), "--sub-run", "A-1", "--niri-log", str(root / "niri.slice"),
                                     "--weston-log", str(root / "weston.slice"))
            self.assertEqual(code, 0, stderr)
```

`GPU_NAME` stands for the name string the test's fake `nvidia-smi` prints in
its `--query-gpu=name,…` branch. Read it from the fake's text and bind it to a
local variable at the top of the test.

- [ ] **Step 3: Run the tests and check that they fail**

Run: `just --set one_cmd 'env NIRI_TOOLING_FAST=0 python3 -m unittest' test-one tools.test_capture_meta`
Expected: FAIL with `AttributeError: module 'capture_meta' has no attribute 'renderer'`.
The release cases return 0 where 1 is expected.

- [ ] **Step 4: Implement**

In `tools/capture-meta`, after `finish_sub_run`:

```python
ANSI = re.compile(r"\x1b\[[0-9;]*m")
RENDERER_LINE = {"niri": re.compile(r'GL Renderer: "(.+)"'), "weston": re.compile(r"\] GL renderer: (.+)$")}
SOFTWARE = re.compile(r"llvmpipe|software rasterizer", re.I)


def renderer_lines(path, compositor):
    try:
        text = pathlib.Path(path).read_text(errors="replace")
    except OSError as error:
        raise CannotRun(f"cannot read {compositor} log {path}: {error}") from error
    pattern = RENDERER_LINE[compositor]
    return [m.group(1).strip() for line in ANSI.sub("", text).splitlines() if (m := pattern.search(line))]


def judge_renderer(lines, expected):
    """expected None: the pixels lane records without requiring."""
    if not lines:
        return "missing"
    if expected is None:
        return "recorded"
    if any(expected not in line or SOFTWARE.search(line) for line in lines):
        return "mismatch"
    return "verified"


def renderer(run_dir, sub_run, niri_log, weston_log):
    """Judges and records one launch's compositor renderers before any stimulus (spec §6.3)."""
    run_dir = pathlib.Path(run_dir)
    record = load_record(run_dir)
    lane = record.get("run", {}).get("lane")
    if lane == "dedicated" and weston_log is not None:
        raise CannotRun("the dedicated lane runs no Weston; drop --weston-log")
    if lane in ("headless", "pixels") and weston_log is None:
        raise CannotRun(f"the {lane} lane runs niri under Weston; pass --weston-log")
    measured = lane in MEASURED_LANES
    expected = record.get("environment", {}).get("gpu", {}).get("name")
    if measured and not expected:
        raise CannotRun(f"{RECORD} has no environment.gpu.name to verify against")
    logs = {"niri": niri_log, **({"weston": weston_log} if weston_log is not None else {})}
    result = {"at": now_rfc3339()}
    if measured:
        result["expected"] = expected
    for compositor, path in logs.items():
        lines = renderer_lines(path, compositor)
        result[compositor] = {"verdict": judge_renderer(lines, expected if measured else None), "lines": lines}
    verdicts = [result[c]["verdict"] for c in logs]
    result["verdict"] = ("verified" if all(v == "verified" for v in verdicts) else "refused") if measured else "recorded"
    with record_lock(run_dir):
        record = load_record(run_dir)
        entry = next((e for e in reversed(record.get("sub_runs", [])) if e["name"] == sub_run
                      and e["verdict"] in ("settled", "begun") and "finished" not in e), None)
        if entry is None:
            raise CannotRun(f"{RECORD} has no open sub-run {sub_run!r} to check")
        if "renderer" in entry:
            raise CannotRun(f"sub-run {sub_run!r} already has its renderer checked; one launch per sub-run")
        entry["renderer"] = result
        save_record(run_dir, record)
    if measured and result["verdict"] != "verified":
        found = "; ".join(f"{c} {result[c]['verdict']} {result[c]['lines']}" for c in logs)
        raise Refused(f"{sub_run}: renderer is not {expected}: {found}")


def cmd_renderer(args):
    renderer(args.run_dir, args.sub_run, args.niri_log, args.weston_log)


def unverified_launches(record):
    """Settled measurement launches without a verified renderer, finished or open (spec §6.3)."""
    if record.get("run", {}).get("lane") not in MEASURED_LANES:
        return []
    return [e["name"] for e in record.get("sub_runs", [])
            if e["verdict"] == "settled" and e.get("renderer", {}).get("verdict") != "verified"]
```

In `validate_record`, inside the `sub_runs` block, add:

```python
        if any("renderer" in item and (not isinstance(item["renderer"], dict)
                                       or not isinstance(item["renderer"].get("verdict"), str)) for item in sub_runs):
            raise CannotRun(f"{path}: sub_runs renderer needs a verdict")
```

In `release`, change

```python
    if end is not None:
        exit_for(end)
        return
```

to

```python
    if end is not None:
        exit_for(end)
        missing = unverified_launches(load_record(run_dir))
        if missing:
            raise Refused(f"launches without a verified renderer: {', '.join(missing)}")
        return
```

`exit_for` returns only on a complete restore with a clean scan, so the hold is
back before this check, and a second release repeats it.

In `render`'s sub-run loop, append
`f"  renderer {entry['renderer']['verdict']}"` to an entry's line when it has
one.

In `build_parser`:

```python
    rnd = subs.add_parser("renderer")
    rnd.add_argument("run_dir")
    rnd.add_argument("--sub-run", required=True)
    rnd.add_argument("--niri-log", required=True, type=pathlib.Path)
    rnd.add_argument("--weston-log", type=pathlib.Path, default=None)
    rnd.set_defaults(func=cmd_renderer)
```

- [ ] **Step 5: Run the tests and check that they pass**

Run: `just --set one_cmd 'env NIRI_TOOLING_FAST=0 python3 -m unittest' test-one tools.test_capture_meta`
Expected: PASS. If an existing lifecycle test settles and then expects
`release` to return 0, give it a `renderer` call as in
`test_release_passes_a_verified_launch_and_ignores_a_refused_settle`. Do not
loosen the check.

- [ ] **Step 6: Commit**

```bash
just --set fast_cmd 'env NIRI_TOOLING_FAST=0 python3 -m tools.tooling_tests --full' test-fast
tasks check
git add tools/capture-meta tools/test_capture_meta.py tools/testdata/renderer tasks/
git commit -m "feat(capture-meta): verify and record each launch's compositor renderers"
```

---

### Task 4: Smoke lib — lane-aware launch, renderer check, timing guards, load wait

**Files:**
- Modify: `docs/materials/scripts/glass-optic-smoke-lib.sh` (`capture_preflight`, `settle_before_launch`, `start_nested`, `gpu_cooldown`, `capture_bg`, `trace_run`, `gpu_median_ns`, new `NIRI_RENDERER_LOG`, `log_size`, `verify_renderer`, `no_timing_on_pixels`, `await_load`)
- Modify: `docs/materials/scripts/hidden-window-attribution.sh` (drop `await_load`, the global `RUST_LOG` export and the renderer half of `check_renderer`)
- Test: `tools/test_glass_optic_smoke.py`

**Interfaces:**
- Consumes: the `capture-meta begin` and `renderer` CLIs (Tasks 2–3)
- Produces, in the lib:
  - shell variable `CAPTURE_LANE`, set by `capture_preflight <lane>`
  - `NIRI_RENDERER_LOG='niri=debug,smithay::backend::renderer::gles=info'`
  - `log_size <path>`, which prints the byte size or 0
  - `verify_renderer <sub-run> <niri-log> <niri-offset> [<weston-log> <weston-offset>]`
  - `await_load`

- [ ] **Step 1: Write the failing tests**

In `CaptureMetaAdoptionTest`:

Update `test_settle_before_launch_passes_config_and_name` to set the lane:
prefix the script's run line with `CAPTURE_LANE=headless;`. Expected lines are
unchanged. Do the same in `test_stopping_the_host_finishes_the_settled_sub_run_once`
and `test_settle_before_launch_skips_the_cooldown_under_a_capture_meta_stub`.

Add:

```python
    def test_pixels_lane_begins_instead_of_settling(self):
        with tempfile.TemporaryDirectory() as out:
            script = ('capture_meta() { printf "%s\\n" "$*" > "$OUT/call"; }\ngpu_cooldown() { echo cooled; }\n' +
                      self.function('settle_before_launch') +
                      '\nOUT=$1; CAPTURE_LANE=pixels; settle_before_launch "$OUT/A.kdl"; cat "$OUT/call"')
            result = self.run_bash(script, out)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(result.stdout.splitlines(), [f'begin {out} --sub-run A --input {out}/A.kdl'])

    def test_settle_before_launch_refuses_without_a_preflight_lane(self):
        with tempfile.TemporaryDirectory() as out:
            script = ('capture_meta() { :; }\n' + self.function('settle_before_launch') +
                      '\nOUT=$1; unset CAPTURE_LANE; settle_before_launch "$OUT/A.kdl"')
            self.assertNotEqual(self.run_bash(script, out).returncode, 0)

    def test_capture_preflight_sets_the_lane(self):
        with tempfile.TemporaryDirectory() as out:
            script = ('capture_meta() { :; }\nfail() { exit 1; }\n' + self.function('capture_preflight') +
                      '\nOUT=$1; CAPTURE_TASK=t; capture_preflight pixels; echo "$CAPTURE_LANE"')
            result = self.run_bash(script, out)
            self.assertEqual(result.stdout.strip(), 'pixels', result.stderr)

    def test_timing_helpers_fail_on_the_pixels_lane(self):
        for helper in ('gpu_cooldown', 'capture_bg', 'trace_run', 'gpu_median_ns'):
            with self.subTest(helper=helper), tempfile.TemporaryDirectory() as out:
                script = ('fail() { echo "FAIL: $*" >&2; exit 1; }\n' + self.function('no_timing_on_pixels') + '\n' +
                          self.function(helper) + f'\nOUT=$1; CAPTURE_LANE=pixels; {helper} x')
                result = self.run_bash(script, out)
                self.assertEqual(result.returncode, 1)
                self.assertIn(f'{helper} records timing', result.stderr)

    def test_verify_renderer_reads_only_this_launchs_bytes(self):
        with tempfile.TemporaryDirectory() as out:
            o = Path(out)
            (o / 'niri.log').write_text('GL Renderer: "llvmpipe"\n')
            (o / 'weston.log').write_text('[0] GL renderer: llvmpipe\n')
            script = ('capture_meta() { printf "%s\\n" "$*" > "$OUT/call"; }\nfail() { exit 1; }\n' +
                      self.function('log_size') + '\n' + self.function('verify_renderer') +
                      '\nOUT=$1; n=$(log_size "$OUT/niri.log"); w=$(log_size "$OUT/weston.log")\n'
                      'echo \'GL Renderer: "NVIDIA"\' >> "$OUT/niri.log"; echo \'[1] GL renderer: NVIDIA\' >> "$OUT/weston.log"\n'
                      'verify_renderer A-1 "$OUT/niri.log" "$n" "$OUT/weston.log" "$w"; cat "$OUT/call"')
            result = self.run_bash(script, out)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual((o / 'A-1.niri.renderer.log').read_text(), 'GL Renderer: "NVIDIA"\n')
            self.assertEqual((o / 'A-1.weston.renderer.log').read_text(), '[1] GL renderer: NVIDIA\n')
            self.assertEqual(result.stdout.strip(),
                             f'renderer {out} --sub-run A-1 --niri-log {out}/A-1.niri.renderer.log '
                             f'--weston-log {out}/A-1.weston.renderer.log')

    def test_verify_renderer_without_weston_for_drm(self):
        with tempfile.TemporaryDirectory() as out:
            (Path(out) / 'niri.log').write_text('')
            script = ('capture_meta() { printf "%s\\n" "$*" > "$OUT/call"; }\nfail() { exit 1; }\n' +
                      self.function('verify_renderer') + '\nOUT=$1; verify_renderer d-1 "$OUT/niri.log" 0; cat "$OUT/call"')
            result = self.run_bash(script, out)
            self.assertEqual(result.stdout.strip(), f'renderer {out} --sub-run d-1 --niri-log {out}/d-1.niri.renderer.log')

    def test_start_nested_logs_and_verifies_the_renderer_after_the_compositor_settles(self):
        body = self.function('start_nested')
        self.assertIn('RUST_LOG=$NIRI_RENDERER_LOG', body)
        self.assertIn("NIRI_RENDERER_LOG='niri=debug,smithay::backend::renderer::gles=info'", self.LIB)
        lines = [l.strip() for l in body.splitlines()]
        self.assertLess(lines.index('sleep 1'), next(i for i, l in enumerate(lines) if l.startswith('verify_renderer')))
        self.assertLess(next(i for i, l in enumerate(lines) if 'niri_from=' in l),
                        next(i for i, l in enumerate(lines) if l.startswith('weston ')))

    def test_await_load_lives_in_the_lib_and_hidden_window_uses_it(self):
        self.assertIn('await_load', self.LIB)
        hwa = (Path(__file__).resolve().parents[1] / 'docs/materials/scripts/hidden-window-attribution.sh').read_text()
        self.assertNotRegex(hwa, r'^await_load\(\)', 'hidden-window keeps no copy')
        self.assertNotIn("rg -q 'GL Renderer:.*NVIDIA'", hwa)
        self.assertNotIn('export RUST_LOG=', hwa)
```

`self.function(name)` matches `^name() {` … `^}`. Define the new lib functions
in that form, with the closing brace at column 0, so the helper can extract
them.

- [ ] **Step 2: Run the tests and check that they fail**

Run: `just --set one_cmd 'env NIRI_TOOLING_FAST=0 python3 -m unittest' test-one tools.test_glass_optic_smoke`
Expected: FAIL. `function('no_timing_on_pixels')` finds nothing (an
`AttributeError` on `None.group`), and the lane is not used.

- [ ] **Step 3: Implement in the lib**

Update the header comment's capture paragraph. Name the lane, for example:
"calls `capture_preflight headless` (or `pixels`, for static-pixel evidence:
docs/specs/2026-10-08-capture-host-conditions-design.md)". Then:

```bash
# niri logs its GL renderer only with the GLES target at info; every launch
# needs the line for capture-meta's renderer check.
NIRI_RENDERER_LOG='niri=debug,smithay::backend::renderer::gles=info'
capture_preflight() {
    CAPTURE_LANE=$1
    capture_meta preflight "$OUT" --lane "$1" --task "${CAPTURE_TASK:?task id authorizing this run}" \
        --fixture "$(basename "$0")" --owner-pid $$ --tool weston --tool kitty --tool "tracy=0.13.1" \
        || fail "preflight refused; see $OUT/capture.json"
}
# Timing helpers refuse on the pixels lane, which claims no time or cost.
no_timing_on_pixels() {
    [ "${CAPTURE_LANE:-}" != pixels ] || fail "$1 records timing; the pixels lane claims none (docs/specs/2026-10-08-capture-host-conditions-design.md)"
}
```

Make `no_timing_on_pixels <name>` the first statement of `gpu_cooldown`,
`capture_bg`, `trace_run` and `gpu_median_ns`, for example
`no_timing_on_pixels gpu_cooldown`.

Replace `settle_before_launch`:

```bash
settle_before_launch() {
    local cfg=$1 name=${2:-}
    [ -n "$name" ] || name=$(basename "$cfg" .kdl)
    case ${CAPTURE_LANE:?capture_preflight has not run} in
        pixels)
            capture_meta begin "$OUT" --sub-run "$name" --input "$cfg" || fail "begin refused before $name; see $OUT/capture.json" ;;
        *)
            [ -n "${CAPTURE_META:-}" ] || gpu_cooldown "$name"
            capture_meta settle "$OUT" --sub-run "$name" --input "$cfg" || fail "settle refused before $name; see $OUT/capture.json" ;;
    esac
    SUB_RUN=$name
}
```

Add after `finish_sub_run`:

```bash
log_size() { if [ -e "$1" ]; then wc -c < "$1"; else echo 0; fi; }
# One launch's renderer lines: the logs are appended across launches, so read
# from the offsets taken before this launch started (spec §6.3).
verify_renderer() {   # $1 sub-run, $2 niri log, $3 its offset, [$4 weston log, $5 its offset]
    tail -c "+$(($3 + 1))" "$2" > "$OUT/$1.niri.renderer.log"
    local weston=()
    if [ $# -ge 5 ]; then
        tail -c "+$(($5 + 1))" "$4" > "$OUT/$1.weston.renderer.log"
        weston=(--weston-log "$OUT/$1.weston.renderer.log")
    fi
    capture_meta renderer "$OUT" --sub-run "$1" --niri-log "$OUT/$1.niri.renderer.log" "${weston[@]}" \
        || fail "$1: renderer check refused; see $OUT/capture.json"
}
# A build's tail refuses the first settle; measurement fixtures wait for it
# after building and before capture_preflight. Skipped under a CAPTURE_META stub.
await_load() {
    [ -z "${CAPTURE_META:-}" ] || return 0
    for _ in $(seq 60); do
        awk '{ exit !($1 < 1.0) }' /proc/loadavg && return 0
        sleep 5
    done
    fail 'load1 did not fall below 1.0 within 5 min of the build'
}
```

In `start_nested`, after `settle_before_launch`, add:

```bash
    local niri_from weston_from
    niri_from=$(log_size "$OUT/niri.log"); weston_from=$(log_size "$OUT/weston.log")
```

On the niri launch line, add `RUST_LOG=$NIRI_RENDERER_LOG` beside
`WAYLAND_DISPLAY=$HOST`. After the final `sleep 1`, add:

```bash
    verify_renderer "$SUB_RUN" "$OUT/niri.log" "$niri_from" "$OUT/weston.log" "$weston_from"
```

- [ ] **Step 4: Implement in `hidden-window-attribution.sh`**

- Delete its `await_load()` definition. Its `build_tracy` keeps calling
  `await_load`, which now comes from the lib. `HWA_REHEARSAL=1` already sets
  `CAPTURE_META=true`, so the lib's skip covers the rehearsal.
- Delete `export RUST_LOG=…`: the lib's launcher sets it per launch.
- Rename `check_renderer` to `check_material_log`. Keep only its second half,
  the `llvmpipe|software rasterizer|error compiling material shader|material.*fallback|panic`
  grep. Drop `llvmpipe|software rasterizer` from that pattern, since the
  renderer belongs to `capture-meta` now. Update its call site.

- [ ] **Step 5: Run the tests and check that they pass**

Run: `just --set one_cmd 'env NIRI_TOOLING_FAST=0 python3 -m unittest' test-one tools.test_glass_optic_smoke`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
just --set fast_cmd 'env NIRI_TOOLING_FAST=0 python3 -m tools.tooling_tests --full' test-fast
tasks check
git add docs/materials/scripts/glass-optic-smoke-lib.sh docs/materials/scripts/hidden-window-attribution.sh tools/test_glass_optic_smoke.py tasks/
git commit -m "feat(capture): lane-aware launches verify their renderer in the smoke lib"
```

---

### Task 5: Launchers outside the lib, and the coverage scan

**Files:**
- Modify: `docs/materials/scripts/focus-swap-clips.sh`, `drag-lag-clips.sh`, `ring-motion-clips.sh` (`start_nested`)
- Modify: `docs/materials/scripts/optic-settling-smoke.sh` (`start_drm`)
- Test: `tools/test_glass_optic_smoke.py`, `tools/test_optic_settling.py`

**Interfaces:**
- Consumes: the `capture-meta renderer` CLI (Task 3), and the lib's `verify_renderer`, `log_size` and `NIRI_RENDERER_LOG` (Task 4, for `optic-settling-smoke.sh`, which sources the lib)

- [ ] **Step 1: Write the failing tests**

Add to `CaptureMetaAdoptionTest`:

```python
    SCRIPTS = Path(__file__).resolve().parents[1] / 'docs/materials/scripts'

    def script_function(self, script, name):
        return re.search(r'^' + name + r'\(\) \{.*?^}', (self.SCRIPTS / script).read_text(), re.S | re.M).group()

    def test_clip_launchers_log_and_verify_both_renderers(self):
        for name in ('ring-motion-clips.sh', 'drag-lag-clips.sh', 'focus-swap-clips.sh'):
            with self.subTest(script=name):
                body = self.script_function(name, 'start_nested')
                self.assertIn('RUST_LOG=niri=debug,smithay::backend::renderer::gles=info', body)
                self.assertIn('StandardOutput=append:', body)
                lines = [l.strip() for l in body.splitlines()]
                kitty = next(i for i, l in enumerate(lines) if l in ('wait_kitty 1', 'wait_app kitty'))
                check = next(i for i, l in enumerate(lines) if 'capture_meta renderer "$OUT" --sub-run "$2"' in l)
                self.assertLess(kitty, check)
                self.assertIn('--weston-log', lines[check] + (lines[check + 1] if check + 1 < len(lines) else ''))

    def test_every_launching_script_verifies_its_renderer(self):
        launch = re.compile(r'capture_meta (settle|begin)\b|settle_before_launch\b')
        for path in sorted(self.SCRIPTS.glob('*.sh')):
            text = path.read_text()
            if not launch.search(text):
                continue
            with self.subTest(script=path.name):
                self.assertRegex(text, r'capture_meta renderer\b|verify_renderer\b')
                self.assertRegex(text, r'gles=info|NIRI_RENDERER_LOG')

    def test_coverage_scan_catches_a_launcher_that_skips_the_check(self):
        launch = re.compile(r'capture_meta (settle|begin)\b|settle_before_launch\b')
        synthetic = 'start_nested() {\n    capture_meta settle "$OUT" --sub-run "$2"\n}\n'
        self.assertTrue(launch.search(synthetic))
        self.assertNotRegex(synthetic, r'capture_meta renderer\b|verify_renderer\b')
```

In `tools/test_optic_settling.py`, add a test beside the existing source
assertions. Use the module's existing way of reading `optic-settling-smoke.sh`
text: search it for `SMOKE`, `read_text` or `optic-settling-smoke.sh`, and
reuse that variable.

```python
    def test_start_drm_logs_and_verifies_the_drm_renderer(self):
        body = re.search(r'^start_drm\(\) \{.*?^}', SMOKE_TEXT, re.S | re.M).group()
        self.assertIn('RUST_LOG="$NIRI_RENDERER_LOG"', body)
        lines = [l.strip() for l in body.splitlines()]
        self.assertLess(lines.index('sleep 1'), next(i for i, l in enumerate(lines) if l.startswith('verify_renderer')))
        self.assertIn('verify_renderer "$SUB_RUN" "$OUT/niri.log" "$niri_from"', body)
        self.assertNotIn('--weston-log', body)
```

`SMOKE_TEXT` stands for that existing variable. If the module has none, read
the script once at module level from
`Path(__file__).resolve().parents[1] / 'docs/materials/scripts/optic-settling-smoke.sh'`.

- [ ] **Step 2: Run the tests and check that they fail**

Run: `just --set one_cmd 'env NIRI_TOOLING_FAST=0 python3 -m unittest' test-one tools.test_glass_optic_smoke tools.test_optic_settling`
Expected: FAIL. Neither the clip launchers nor `start_drm` call `renderer`.

- [ ] **Step 3: Implement the clip launchers**

Make the same change in each of the three clip fixtures' `start_nested`. Here
is `focus-swap-clips.sh`. In `drag-lag-clips.sh`, the renderer lines go after
its final `wait_app kitty`, which follows its `NESTED_WAYLAND` lookup.
`ring-motion-clips.sh` matches `focus-swap-clips.sh`.

```bash
start_nested() {   # $1 = config, $2 = sub-run name; sets NIRI_SOCKET
    capture_meta settle "$OUT" --sub-run "$2" --input "$1" || fail "settle refused before $2; see $OUT/capture.json"
    local niri_from weston_from
    niri_from=$( [ -e "$OUT/niri.log" ] && wc -c < "$OUT/niri.log" || echo 0 )
    weston_from=$( [ -e "$OUT/weston.log" ] && wc -c < "$OUT/weston.log" || echo 0 )
    HOST_SEQ=$((HOST_SEQ + 1))
    local host=$RUN-h$HOST_SEQ
    UNIT=$host-weston; HOST_SOCKET=$XDG_RUNTIME_DIR/$host
    systemd-run --user --unit="$UNIT" --collect \
        --property=StandardOutput=append:"$OUT/weston.log" --property=StandardError=append:"$OUT/weston.log" \
        weston --backend=headless --renderer=gl \
        --shell=kiosk-shell.so --width=1280 --height=720 --socket="$host" >/dev/null 2>&1
    for _ in $(seq 100); do [ -S "$HOST_SOCKET" ] && break; sleep 0.1; done
    [ -S "$HOST_SOCKET" ] || fail "Weston socket never appeared at $HOST_SOCKET"
    ln -s "$HOST_SOCKET" "$RT/$host"
    XDG_RUNTIME_DIR=$RT WAYLAND_DISPLAY=$host RUST_LOG=niri=debug,smithay::backend::renderer::gles=info \
        "$NIRI" -c "$1" >> "$OUT/niri.log" 2>&1 &
    NIRI_PID=$!
    for _ in $(seq 100); do ls "$RT"/niri.*.sock >/dev/null 2>&1 && break; sleep 0.1; done
    NIRI_SOCKET=$(ls -t "$RT"/niri.*.sock | head -1); export NIRI_SOCKET
    wait_kitty 1
    tail -c "+$((niri_from + 1))" "$OUT/niri.log" > "$OUT/$2.niri.renderer.log"
    tail -c "+$((weston_from + 1))" "$OUT/weston.log" > "$OUT/$2.weston.renderer.log"
    capture_meta renderer "$OUT" --sub-run "$2" --niri-log "$OUT/$2.niri.renderer.log" \
        --weston-log "$OUT/$2.weston.renderer.log" || fail "$2: renderer check refused; see $OUT/capture.json"
}
```

Keep each fixture's own lines (the `ring-motion-clips.sh` binary argument,
`drag-lag-clips.sh`'s `NESTED_WAYLAND`) where they are now. Only the offsets,
the `systemd-run` output properties, the `RUST_LOG` prefix and the three
renderer lines are new. The `capture_meta renderer` line must keep
`--sub-run "$2"` and `--weston-log` on the line pair the test reads.

- [ ] **Step 4: Implement `start_drm`**

In `optic-settling-smoke.sh` `start_drm`, after `settle_before_launch "$2" "${3-}"`:

```bash
    local niri_from; niri_from=$(log_size "$OUT/niri.log")
```

In the `env -u …` launch, add `RUST_LOG="$NIRI_RENDERER_LOG"` beside
`XDG_RUNTIME_DIR="$RT"`. After the final `sleep 1`:

```bash
    verify_renderer "$SUB_RUN" "$OUT/niri.log" "$niri_from"
```

The optic-settling rehearsal runs under a `CAPTURE_META` stub with
`OPTIC_SETTLING_STUB_TOOLS`, so `verify_renderer` calls the stub, which
accepts. Confirm this by running that rehearsal test in Step 5.

- [ ] **Step 5: Run the tests and check that they pass**

Run: `just --set one_cmd 'env NIRI_TOOLING_FAST=0 python3 -m unittest' test-one tools.test_glass_optic_smoke tools.test_optic_settling`
Expected: PASS, including the existing offline rehearsals in
`test_optic_settling.py`.

- [ ] **Step 6: Commit**

```bash
just --set fast_cmd 'env NIRI_TOOLING_FAST=0 python3 -m tools.tooling_tests --full' test-fast
tasks check
git add docs/materials/scripts/focus-swap-clips.sh docs/materials/scripts/drag-lag-clips.sh docs/materials/scripts/ring-motion-clips.sh docs/materials/scripts/optic-settling-smoke.sh tools/test_glass_optic_smoke.py tools/test_optic_settling.py tasks/
git commit -m "feat(capture): clip and DRM launchers verify their renderer"
```

---

### Task 6: Fixture lanes and build-before-preflight order

**Files:**
- Modify: `docs/materials/scripts/glass-edge-sheet.sh`, `glass-noise-layers-smoke.sh` (to `pixels`)
- Modify: `docs/materials/scripts/glass-iridescence-smoke.sh`, `glass-aurora-smoke.sh`, `glass-noise-site-smoke.sh`, `noise-layers-cost.sh`, `noise-placement-cost.sh`, `glass-render-order-smoke.sh`, `hidden-window-attribution.sh` (build, `await_load`, preflight)
- Modify: `docs/materials/scripts/focus-swap-clips.sh`, `drag-lag-clips.sh`, `ring-motion-clips.sh` (build, inline load wait, preflight)
- Modify: `docs/materials/scripts/optic-settling-smoke.sh` (client builds, `await_load`, preflight, then the dedicated-lane checks)
- Test: `tools/test_glass_optic_smoke.py`, `tools/test_optic_settling.py`

**Interfaces:**
- Consumes: `capture_preflight pixels` and `await_load` (Task 4)

- [ ] **Step 1: Write the failing tests**

Replace `test_smokes_preflight_after_sourcing_and_identify_after_build` with:

```python
    def test_lib_fixtures_build_wait_then_preflight_then_identify(self):
        measured = {'glass-aurora-smoke.sh': 'build_binaries', 'glass-iridescence-smoke.sh': 'build_binaries',
                    'glass-noise-site-smoke.sh': 'build_binaries', 'noise-layers-cost.sh': 'build_binaries',
                    'noise-placement-cost.sh': 'build_binaries', 'glass-render-order-smoke.sh': '\nselect_binaries\n',
                    'hidden-window-attribution.sh': '\nbuild_tracy\n'}
        for smoke, build in measured.items():
            with self.subTest(smoke=smoke):
                text = (self.SCRIPTS / smoke).read_text()
                order = [text.index('glass-optic-smoke-lib.sh'), text.index(build),
                         text.index('\nawait_load') if smoke != 'hidden-window-attribution.sh' else text.index(build),
                         text.index('capture_preflight headless'), text.index('\ncapture_identity')]
                self.assertEqual(order, sorted(order), smoke)
        for smoke in ('glass-edge-sheet.sh', 'glass-noise-layers-smoke.sh'):
            with self.subTest(smoke=smoke):
                text = (self.SCRIPTS / smoke).read_text()
                self.assertNotIn('capture_preflight headless', text)
                order = [text.index('glass-optic-smoke-lib.sh'), text.index('build_binaries'),
                         text.index('capture_preflight pixels'), text.index('capture_identity')]
                self.assertEqual(order, sorted(order), smoke)

    def test_clip_fixtures_build_wait_then_preflight(self):
        for name in ('ring-motion-clips.sh', 'drag-lag-clips.sh', 'focus-swap-clips.sh'):
            with self.subTest(script=name):
                text = (self.SCRIPTS / name).read_text()
                order = [text.index('cargo build --release'), text.index('load1 did not fall below 1.0'),
                         text.index('capture_meta preflight'), text.index('capture_meta identity')]
                self.assertEqual(order, sorted(order), name)
```

`hidden-window-attribution.sh`'s `build_tracy` calls `await_load` itself, so
its order list repeats the build index for the wait. Every other fixture
calls `await_load` on its own line.

`optic-settling-smoke.sh` takes a prebuilt `NIRI_BIN`. Its own builds are the
per-run Wayland clients (`build_client`, about 30 lines above
`capture_identity`), which compile only for selected cases, and today they run
after the preflight. Its check is behavioural, on the existing stub driver. Add
to `DriverCleanupTests` in `tools/test_optic_settling.py`:

```python
    def test_client_builds_precede_the_preflight_and_prepare_stays_offline(self):
        # Stub compiler and scanner: cc logs into meta.log beside the capture-meta verbs.
        for name, body in (('bin/cc', 'echo "cc $*" >> "$STUB_DIR/meta.log"\n'
                                      'while [ "$1" != -o ]; do shift; done\n: > "$2"\n'),
                           ('bin/wayland-scanner', ': > "$3"\n')):
            path = self.stubs / name
            path.write_text('#!/bin/sh\n' + body)
            path.chmod(0o755)
        script = self.root / 'docs/materials/scripts/optic-settling-smoke.sh'
        prepare = subprocess.run(['bash', str(script), 'prepare'], cwd=self.root,
                                 env=dict(self.env, CASES='idle-inhibitor'),
                                 capture_output=True, text=True, timeout=60)
        self.assertEqual(prepare.returncode, 0, prepare.stderr)
        verbs = [line.split()[0] for line in (self.stubs / 'meta.log').read_text().splitlines()]
        self.assertEqual(verbs, ['release'])                 # on_exit releases on every exit
        (self.stubs / 'meta.log').unlink()
        shutil.rmtree(self.out)
        driver = self.start('idle-inhibitor', STUB_REFUSE='1')
        _, stderr = driver.communicate(timeout=60)
        self.assertEqual(driver.returncode, 1, stderr)
        verbs = [line.split()[0] for line in (self.stubs / 'meta.log').read_text().splitlines()]
        self.assertEqual(verbs, ['cc', 'preflight', 'release'])
        self.assertEqual(self.pids(), [])                     # nothing was launched
```

`idle-inhibitor` is a headless case that builds one client, so a refused
preflight leaves exactly that order. `prepare` runs on the same stubs (the
stub `niri` answers `validate`) and logs only `on_exit`'s `release`.

- [ ] **Step 2: Run the tests and check that they fail**

Run: `just --set one_cmd 'env NIRI_TOOLING_FAST=0 python3 -m unittest' test-one tools.test_glass_optic_smoke tools.test_optic_settling`
Expected: FAIL. Preflight precedes the build in every fixture, and the
optic-settling stub run logs `['preflight', 'release']` with no `cc`.

- [ ] **Step 3: Reorder the lib fixtures**

For `glass-iridescence-smoke.sh`, `glass-aurora-smoke.sh`,
`glass-noise-site-smoke.sh`, `noise-layers-cost.sh` and
`noise-placement-cost.sh`, move the `capture_preflight headless` line from
after sourcing to after `build_binaries` and the lines that only copy or
select binaries. Then insert `await_load` immediately before it.
`noise-layers-cost.sh` becomes:

```bash
source "$(dirname "$0")/glass-optic-smoke-lib.sh"
...
build_binaries
await_load
capture_preflight headless
if [ -n "$AB" ]; then
    cp "$AB" "$OUT/niri-tracy-b"
    capture_identity --binary "$OUT/niri-tracy-b" --config "pilot=$PILOT" --config ab=1
else
    capture_identity --config "pilot=$PILOT"
fi
```

This reproduces the existing `if` block as it stands. Move only the two
lines, and keep any `trap` lines that precede them in place.

For `glass-render-order-smoke.sh`, place `await_load` and
`capture_preflight headless` after `select_binaries` and before the
`IDENTITY_EXTRA` block. Keep `validate_scope || exit` before `select_binaries`:
`test_cost_scope_rejects_capture_override_before_preflight` asserts that it
precedes the preflight. Its `SCOPE=pixels` waiver path (`CAPTURE_META` set) is
left as it is; the render-order follow-up filed in Task 7 retires it.

For `hidden-window-attribution.sh`, move `capture_preflight headless` from
line 37 to between `build_tracy` and `capture_identity` at the bottom of the
script.

For `glass-edge-sheet.sh` and `glass-noise-layers-smoke.sh`, move the
preflight after `build_binaries`, plus `glass-noise-layers-smoke.sh`'s
`cp "$BASE_NIRI" …`/`BASE_NIRI=` lines, and change it to
`capture_preflight pixels`. Pixel fixtures do not call `await_load`.

- [ ] **Step 4: Reorder the clip fixtures**

In each clip fixture, move the `capture_meta preflight … || fail …` statement
(two lines) from its current place to immediately before
`capture_meta identity`. Directly above it, insert the inline wait (these
fixtures do not source the lib):

```bash
# A build's tail refuses the first settle: wait for it before the preflight
# (docs/specs/2026-10-08-capture-host-conditions-design.md §6.5).
for _ in $(seq 60); do awk '{ exit !($1 < 1.0) }' /proc/loadavg && break; sleep 5; done
awk '{ exit !($1 < 1.0) }' /proc/loadavg || fail 'load1 did not fall below 1.0 within 5 min of the build'
```

Check that nothing between the old and new preflight position launches a
compositor. `niri validate` is not a launch. `ring-motion-clips.sh`'s
`scratch_build` runs `cargo build`, which belongs before the wait. If a
fixture launches anything in that span, stop and record it in a task note
instead of moving the preflight past it.

- [ ] **Step 5: Reorder `optic-settling-smoke.sh`**

Its `prepare` mode exits (`[ "$MODE" = prepare ] && exit 0`) before any
build, and it must stay offline: no build, no load wait, no preflight.
Today the order is:

1. `[ "$MODE" = prepare ] || capture_preflight "$LANE"`, then the
   `if [ "$MODE" != prepare ] && [ "$LANE" = dedicated ]` block
   (`dedicated_prerequisites`, `vt_record_home`, `vt_spare`, `vt.json`);
2. configs, the manifest and the matrix check;
3. the `prepare` exit;
4. `tools_ready`, `reserve_tracy_port`, then the `build_client` calls;
5. `capture_identity`.

Move the preflight and its dedicated block together to immediately before
`capture_identity`, after the last client build, with `await_load` in front.
Drop their `prepare` guards: the `prepare` exit now precedes them.

```bash
if selected screencast; then IDENTITY_EXTRA+=(--input "$ROOT/tools/screencast_consumer.py"); fi
# The client builds' tail would refuse the first settle (spec §6.5).
await_load
capture_preflight "$LANE"
if [ "$LANE" = dedicated ]; then
    dedicated_prerequisites
    vt_record_home
    VT_SPARE=$(vt_spare) || fail "no spare VT without a logind session"
    printf '{"home": %s, "spare": %s}\n' "$VT_HOME" "$VT_SPARE" > "$OUT/vt.json"
fi
capture_identity --config threshold-ms=5000 --config cases="${RUN_CASES[*]}" --config lane="$LANE" \
    ...
```

The dedicated checks stay after the preflight, as they are now: the lock and
stale-run recovery come first, and the existing dedicated stub tests expect
that order.

Check that nothing between the old and the new position launches a
compositor or reads `VT_SPARE`, `VT_HOME` or `vt.json`. These are not
launches: `cp "$NIRI_BIN"`, `niri validate` inside the configs, `tools_ready`
and `reserve_tracy_port`. If something does launch, stop and note it on the
task instead of moving past it.

A failure that now lands before the preflight (a client that does not
compile) still runs `on_exit`. Its `capture_meta release` returns without a
record, as it already does for the identity-sidecar check above the old
preflight.

- [ ] **Step 6: Run the tests and check that they pass**

Run: `just --set one_cmd 'env NIRI_TOOLING_FAST=0 python3 -m unittest' test-one tools.test_glass_optic_smoke tools.test_optic_settling`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
just --set fast_cmd 'env NIRI_TOOLING_FAST=0 python3 -m tools.tooling_tests --full' test-fast
tasks check
git add docs/materials/scripts/*.sh tools/test_glass_optic_smoke.py tools/test_optic_settling.py tasks/
git commit -m "feat(capture): static pixel fixtures take the pixels lane; fixtures build before preflight"
```

---

### Task 7: Documents, tracker follow-ups and the pilots

**Files:**
- Modify: `docs/specs/2026-10-08-capture-host-conditions-design.md` (amendments; status)
- Modify: `docs/specs/2026-09-11-material-capture-protocol-design.md` (§5 note)
- Modify: `docs/notes/2026-10-06-capture-lifecycle-brief.md`
- Modify: `docs/materials/capture-host-setup.md` (one line on the pixels lane)
- Tracker: new tasks and notes (no code)

- [ ] **Step 1: Amend the spec**

In `docs/specs/2026-10-08-capture-host-conditions-design.md`:

- **§6.2:** replace the sentence about `capture-meta show --field run.lane`
  with: "It reads the lane from `CAPTURE_LANE`, which `capture_preflight` sets;
  `settle_before_launch` refuses when it is unset."
- **§6.3:** replace "The lib already tracks `LOG_OFFSET` into its appended
  `niri.log`." with "The launcher takes both logs' byte sizes before the
  launch and slices from them (`log_size`, `verify_renderer`)."
- **§8.1:** drop the `show --field` clause.
- **Status:** "implemented on `capture-host-conditions` (plan
  `docs/plans/2026-10-08-capture-host-conditions.md`); pilots pending
  (§8.2)."

- [ ] **Step 2: Note the protocol design and update the brief**

In `docs/specs/2026-09-11-material-capture-protocol-design.md` §5, after the
paragraph that ends "no case config exists yet.", add:

> Since `material-18c2a1`, fixtures build before `capture_preflight` and wait
> for the build's load to pass, and static-pixel fixtures preflight on the
> `pixels` lane: see [per-route host conditions](2026-10-08-capture-host-conditions-design.md).

In `docs/notes/2026-10-06-capture-lifecycle-brief.md`, under
"Renderer-aware capture", change `material-6bd4a3`'s line to say what it waits
on now: the desktop-idle pilot task filed in Step 3. Add a line that the
per-route design is implemented. Answer the first "Unanswered questions"
bullet by linking the design.

In `docs/materials/capture-host-setup.md`, under "Other host state the lanes
assume", add one bullet: "The pixels lane
(`docs/specs/2026-10-08-capture-host-conditions-design.md`) needs no idle host
and no `nvidia-smi`; it holds user timers and the declared services only."

- [ ] **Step 3: File the follow-ups and pilots**

From the main checkout's task view: run `tasks start` on this step's id in
the worktree first, so the records land here.

```bash
tasks add "Pilot the pixels lane on a desktop in use with glass-noise-layers-smoke.sh" -p 2 --size xs --complexity low --process direct \
  --tag capture --need nested --need owner --parent material-2834d7 --spec 2026-10-08-capture-host-conditions \
  -b "Spec §8.2 pilot 1. Run glass-noise-layers-smoke.sh on --lane pixels on the desktop in use, with the owner's go-ahead for host use. Pass: every AE-0 assertion holds; the record has no baseline or GPU sampling, renderers recorded, timers held and restored; metrics.txt equals a retained run with identical provenance.binaries if one exists (else say so in the run note)."
tasks add "Pilot nested measurements with the desktop idle against the pinned TTY reference" -p 1 --size s --complexity mid --process direct \
  --tag capture --need quiet --parent material-2834d7 --spec 2026-10-08-capture-host-conditions \
  -b "Spec §8.2 pilot 2: reference runs, binary cb21ad03… and config hashes pinned there; I_c = [min-w-q, max+w+q], q = 0.001024 ms. Rehearse offline first to compare config hashes; on a mismatch take three fresh TTY reference runs first. Two desktop-idle runs (park --needs idle), the first is the pilot. Pass admits desktop-idle in §5.3 and maps nested measurements to idle in the queue."
for fx in glass-iridescence-smoke.sh glass-aurora-smoke.sh glass-noise-site-smoke.sh; do
  tasks add "Pixels-only mode for $fx on the pixels lane" --status idea --tag capture \
    -b "Spec §3.1/§10: the fixture mixes pixel assertions with a recorded timing; a mode that drops the timing can preflight --lane pixels."
done
tasks add "Retire glass-render-order-smoke.sh's pixel-only capture-meta waiver for the pixels lane" --status idea --tag capture \
  -b "SCOPE=pixels runs under a stubbed CAPTURE_META recorded as gpu-quietness=pixel-only-waiver. Spec §3.1/§10: with --lane pixels the scope can keep a real capture record; check that pixel_matrix and within_matrix under SCOPE=pixels record no wall-clock values first."
tasks add "niri-experiments: idle-budget and jelly-motion adopt renderer verification and build-before-preflight" -p 2 --size s --complexity low --process direct \
  --tag capture --parent material-2834d7 --spec 2026-10-08-capture-host-conditions \
  -b "Spec §6.4/§10. In niri-experiments: idle-budget.sh power start_drm sets RUST_LOG=\$NIRI_RENDERER_LOG and calls verify_renderer \"\$name\" \"\$OUT/niri.log\" <offset> after launch; fixtures/test_idle_budget.py asserts it; idle-budget.sh and jelly-motion.sh build, await_load, then capture_preflight; jelly-motion.sh drops its own weston.log GL renderer grep. Until this lands, release refuses their measurement runs."
```

Make the desktop-idle pilot depend on the niri-experiments task only if the
pilot uses an idle-budget fixture. It does not: it runs `noise-layers-cost.sh`.
So add no dependency.

Run `tasks check`.

- [ ] **Step 4: Wake the waiting idea**

```bash
tasks note material-6bd4a3 "Reviewed decisions (material-18c2a1, docs/specs/2026-10-08-capture-host-conditions-design.md): nested measurements stay on a TTY until the desktop-idle pilot (filed under material-2834d7) passes against the pinned TTY reference; static pixel fixtures run on a desktop in use via --lane pixels; every measured launch must verify its renderer against the sampled GPU; dedicated refuses a live desktop and GPU clients before the hold. Thresholds unchanged."
```

`material-925518` gets no note: the design names no software-rendered
consumer.

- [ ] **Step 5: Commit**

```bash
just --set fast_cmd 'env NIRI_TOOLING_FAST=0 python3 -m tools.tooling_tests --full' test-fast
tasks check
git add docs/ tasks/
git commit -m "docs(capture): record host-conditions follow-ups, pilots and amendments"
```
