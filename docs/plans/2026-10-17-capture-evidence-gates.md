# Capture gates judge measured contention: implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** settle and the release scan annotate host events that carry no
evidence of contention, record each severity and rule, and a read-only
`capture-meta judge` re-judges any retained run.

**Architecture:** `tools/capture-meta` splits its settle judgement into
`judge_window`, which works on a window's summary and is shared by `settle`
and `judge`, and `judge_settled`, which adds the per-sample memory floor.
`tools/capture_hold.py` gains `classify`, which gives the disturbances that
`scan` flags a severity by lane. `restore_run` writes the classified list.
`judge` reads `capture.json` and nothing else.

**Tech Stack:** Python 3 standard library, `unittest`, the repository's
`FakeHost`/`FakeProc`/`FakeGpu` doubles.

**Spec:** `docs/specs/2026-10-17-capture-evidence-gates-design.md`
(approved at spec review round 2).

**Task records:** each `### Task N` has a child task of `material-2e4ea5`.
Run `tasks start <child>` before its first step, and `tasks done <child> "<what landed>"`
in its commit.

**Paths:** every path below is relative to the worktree root,
`.worktrees/capture-gates-evidence/`. Run every command from there.

## Global Constraints

- The schema stays 1. Every addition to `capture.json` is optional, and old
  records still validate and render.
- The rules identity is the string `"2026-10-17"`, recorded as
  `preflight.rules`.
- Thresholds are unchanged, and no threshold is added. Preflight's quietness
  judgement is unchanged.
- Severities are exactly `fail` and `annotate`. Scan verdicts are `clean`,
  `annotated`, `disturbed`, `unscanned` and `not-run`.
- Settle rule names: `unclaimed-power-step` and `unclaimed-power-spread`.
  The scan's VT rule is `vt-login`. Every other scan entry's rule is its
  `kind`.
- An annotated sub-run keeps `verdict: "settled"`, and an annotated scan
  exits 0.
- `judge` writes nothing, takes no lock and creates no file. On exit,
  **no evidence (2)** outranks **fail (1)**, which outranks **pass (0)**.
- Tests copy recorded values in. They never read `$NIRI_MATERIAL_WORK_ROOT`.
- No doc or comment holds a machine-specific absolute path.

## Review Focus

1. **A window with only a compute client on `headless`:** power must still
   fail. The client test covers compute as well as graphics. Test in Task 1.
2. **The first sampled entry, or the only earlier one, is refused:**
   `gpu_power_step_w` is taken against the baseline, never against a refused
   window. Test in Task 1.
3. **Re-classifying entries that already carry a severity** (a record
   `judge` replays): the old `severity`/`rule` is replaced, never kept. Test
   in Task 2.
4. **Old sub-run entries without window fields** (early records hold only
   `name`/`verdict`/`started`): `judge` keeps the recorded verdict and does
   not crash. Test in Task 3.
5. **`judge` on a directory with no `capture.json`, or a corrupt one:** exit
   2, and no file appears in the directory. Test in Task 3.

---

### Task 1: Settle annotates client-free GPU power on the nested lane

**Files:**
- Modify: `tools/capture-meta`. Constants near `SCHEMA`
  (line 30); `judge_settled` (lines 425-435); `preflight` (the four
  `write_section(run_dir, "preflight", …)` calls, lines 940-990); `settle`
  (lines 1047-1070); `render` (the preflight line and the sub-run loop).
- Modify: `docs/specs/2026-09-11-material-capture-protocol-design.md` §2.4.
- Test: `tools/test_capture_meta.py`. `JudgementTests.test_quiet_and_settle`
  (lines 555-563), a new `GateSettleTests` class after `JudgementTests`, and
  new tests in `SettleTests` and `ShowTests`.

**Interfaces:**
- Produces: `RULES = "2026-10-17"`;
  `judge_window(summary, baseline, thresholds, lane) -> (list[str], list[dict])`,
  where each annotation is `{"rule": str, "detail": str}`;
  `judge_settled(summary, samples, baseline, thresholds, lane) -> (list[str], list[dict])`;
  `unclaimed_power(summary, lane) -> bool`. The sub-run entry fields are
  `gpu_power_step_w: float` and `annotations: list[dict]` (present only when
  non-empty).

- [ ] **Step 1: Write the failing tests**

Add after `class JudgementTests` in `tools/test_capture_meta.py`:

```python
DESKTOP = ("Xwayland", "kitty", "niri", "noctalia", "qs")


def window(power, iqr=0.2, util=0.0, cpu=1.0, mem=80.0, pstates=("P8",), graphics=(), compute=()):
    """A settle window's summary, as capture.json records it."""
    return {"cpu_busy_pct": cpu, "mem_available_pct": mem, "gpu_util_pct": util, "gpu_power_w": power,
            "gpu_power_iqr_w": iqr, "gpu_pstates": list(pstates),
            "gpu_clients": {"compute": list(compute), "graphics": list(graphics)}}


class GateSettleTests(unittest.TestCase):
    """Recorded windows from retained runs (2026-10-17 gates design §6.1)."""

    def judged(self, summary, baseline, lane="headless"):
        return cm.judge_window(summary, baseline, cm.DEFAULT_THRESHOLDS, lane)

    def test_client_free_floor_steps_from_2026_10_16_are_annotated(self):
        cases = (  # (run/sub-run, window, baseline, detail)
            ("noise-idle-ab-full-1792148041/none-b3-1", window(17.115, iqr=0.027), window(10.39),
             "gpu_power_w 17.115 is 6.725 above baseline 10.39; no GPU client in the window"),
            ("noise-idle-ab-full-1792148490/none-b1-1", window(10.39, iqr=0.285, cpu=4.2), window(17.125, cpu=4.2),
             "gpu_power_w 10.39 is 6.735 below baseline 17.125; no GPU client in the window"),
            ("cost-pilot-1792145107/tilt10-a2", window(10.685, iqr=0.188, cpu=0.9), window(8.945, cpu=1.1),
             "gpu_power_w 10.685 is 1.74 above baseline 8.945; no GPU client in the window"))
        for name, summary, baseline, detail in cases:
            with self.subTest(name):
                self.assertEqual(self.judged(summary, baseline),
                                 ([], [{"rule": "unclaimed-power-step", "detail": detail}]))

    def test_the_2026_09_24_idle_lock_step_still_fails_with_the_desktop_as_client(self):
        reasons, annotations = self.judged(window(11.585, iqr=0.258, graphics=DESKTOP),
                                           window(9.9, cpu=1.1, graphics=DESKTOP))
        self.assertEqual(reasons, ["gpu_power_w 11.585 exceeds baseline 9.9 by more than 1.5"])
        self.assertEqual(annotations, [])

    def test_a_compute_client_alone_keeps_the_power_rule(self):
        reasons, annotations = self.judged(window(17.0, compute=("python3",)), window(10.4))
        self.assertIn("gpu_power_w 17.0 exceeds baseline 10.4 by more than 1.5", reasons)
        self.assertEqual(annotations, [])

    def test_contention_rules_are_unchanged_with_or_without_clients(self):
        for graphics in (DESKTOP, ()):
            with self.subTest(graphics=graphics):
                reasons, _ = self.judged(window(10.035, util=5.0, graphics=graphics),
                                         window(10.055, util=0.0, graphics=graphics))
                self.assertIn("gpu_util_pct 5.0 exceeds baseline 0.0 by more than 3.0", reasons)
        reasons, _ = self.judged(window(10.755, pstates=("P5", "P8")), window(10.555), lane="dedicated")
        self.assertIn("gpu_pstate ['P5', 'P8'] not always P8", reasons)

    def test_dedicated_power_still_fails_without_clients(self):
        reasons, annotations = self.judged(window(17.0), window(10.4), lane="dedicated")
        self.assertEqual(reasons, ["gpu_power_w 17.0 exceeds baseline 10.4 by more than 1.5"])
        self.assertEqual(annotations, [])

    def test_power_spread_is_annotated_only_when_unclaimed(self):
        self.assertEqual(self.judged(window(10.2, iqr=1.328), window(10.15)),
                         ([], [{"rule": "unclaimed-power-spread",
                                "detail": "gpu_power_iqr_w 1.328 exceeds 1.0; no GPU client in the window"}]))
        self.assertEqual(self.judged(window(10.2, iqr=1.328, graphics=("niri",)), window(10.15, graphics=("niri",))),
                         ([], []))
```

In `JudgementTests.test_quiet_and_settle`, change the two `judge_settled`
uses to unpack the pair:

```python
        reasons, _ = cm.judge_settled(cm.summarize(samples), samples, cm.summarize(quiet_samples()), cm.DEFAULT_THRESHOLDS, "headless")
        self.assertIn("gpu_power_w 24.1 exceeds baseline 18.4 by more than 1.5", " ".join(reasons))
        dropped = quiet_samples(3)
        for sample in dropped: sample.mem_available_kib = 70_000
        reasons, _ = cm.judge_settled(cm.summarize(dropped), dropped, cm.summarize(quiet_samples()), cm.DEFAULT_THRESHOLDS, "headless")
        self.assertIn("mem_available_pct", " ".join(reasons))
```

Add to `SettleTests`:

```python
    def test_client_free_power_step_settles_with_an_annotation(self):
        with tempfile.TemporaryDirectory() as directory:
            run, lock = self.prepared(directory)
            stderr = io.StringIO()
            with contextlib.redirect_stderr(stderr):
                cm.settle(run, "none-b3-1", [run / "A.kdl"], 3, FakeProc([(i * 2, i * 100) for i in range(9)]),
                          FakeGpu([dict(QUIET, power_w=25.0)] * 3, clients={"compute": [], "graphics": []}),
                          sleep=lambda s: None, lock=lambda: lock)
            entry = cm.load_record(run)["sub_runs"][0]
            self.assertEqual(entry["verdict"], "settled")
            self.assertNotIn("reason", entry)
            self.assertEqual([a["rule"] for a in entry["annotations"]], ["unclaimed-power-step"])
            self.assertEqual(entry["gpu_power_step_w"], 6.6)
            self.assertIn("annotated: unclaimed-power-step", stderr.getvalue())

    def test_power_step_is_taken_against_the_previous_settled_window(self):
        with tempfile.TemporaryDirectory() as directory:
            run, lock = self.prepared(directory)                  # baseline 18.4 W, niri as client
            with self.assertRaises(cm.Refused):
                self.settled(run, lock, "R", power_w=24.1)        # refused: never a step's reference
            self.settled(run, lock, "A", power_w=19.0)
            self.settled(run, lock, "C", power_w=19.5)
            steps = [(e["name"], e["verdict"], e["gpu_power_step_w"]) for e in cm.load_record(run)["sub_runs"]]
            self.assertEqual(steps, [("R", "refused", 5.7), ("A", "settled", 0.6), ("C", "settled", 0.5)])

    def test_preflight_records_the_rules_identity(self):
        with tempfile.TemporaryDirectory() as directory:
            run, _ = self.prepared(directory)
            self.assertEqual(cm.load_record(run)["preflight"]["rules"], "2026-10-17")
```

Add to `ShowTests`:

```python
    def test_show_prints_power_steps_annotations_and_rules(self):
        record = {"schema": 1, "run": {"id": "r", "lane": "headless", "started": STARTED},
                  "preflight": {"verdict": "quiet", "rules": "2026-10-17"},
                  "sub_runs": [{"name": "none-b3-1", "verdict": "settled", "gpu_power_step_w": 6.69,
                                "annotations": [{"rule": "unclaimed-power-step", "detail": "gpu_power_w 17.115 is 6.725 above baseline 10.39; no GPU client in the window"}]}]}
        text = cm.render(record)
        self.assertIn("preflight quiet  rules 2026-10-17", text)
        self.assertIn("none-b3-1: settled  step +6.69 W", text)
        self.assertIn("    annotated unclaimed-power-step: gpu_power_w 17.115 is 6.725 above baseline 10.39", text)
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `just --set one_cmd 'env NIRI_TOOLING_FAST=0 python3 -m unittest' test-one tools.test_capture_meta.GateSettleTests tools.test_capture_meta.JudgementTests tools.test_capture_meta.SettleTests tools.test_capture_meta.ShowTests`
Expected: FAIL, with `AttributeError: module 'capture_meta' has no attribute 'judge_window'` and a `KeyError` on `annotations`, `gpu_power_step_w` and `rules`. `test_quiet_and_settle` fails, because `judge_settled` still returns a list.

- [ ] **Step 3: Implement**

In `tools/capture-meta`, below `SCHEMA = 1`:

```python
RULES = "2026-10-17"   # the gate rules a run is judged by: docs/specs/2026-10-17-capture-evidence-gates-design.md
```

Replace `judge_settled` (lines 425-435) with:

```python
SETTLE_DELTAS = (("cpu_busy_pct", "settle_cpu_busy_pct"), ("mem_available_pct", "settle_mem_pct"),
                 ("gpu_util_pct", "settle_gpu_util_pct"), ("gpu_power_w", "settle_gpu_power_w"))


def unclaimed_power(summary, lane):
    """GPU power is no claim of the nested lane, and with no GPU client in the window
    nothing could have competed for it (2026-10-17 gates design §2)."""
    clients = summary["gpu_clients"]
    return lane == "headless" and not clients["graphics"] and not clients["compute"]


def judge_window(summary, baseline, thresholds, lane):
    """The settle rules a window's summary carries: (reasons, annotations). settle and judge share it."""
    reasons, annotations = [], []
    unclaimed = unclaimed_power(summary, lane)
    for key, limit in SETTLE_DELTAS:
        delta = summary[key] - baseline[key]
        if abs(delta) <= thresholds[limit]:
            continue
        if key == "gpu_power_w" and unclaimed:
            side = "above" if delta > 0 else "below"
            annotations.append({"rule": "unclaimed-power-step",
                                "detail": f"gpu_power_w {summary[key]} is {round(abs(delta), 3)} {side} "
                                          f"baseline {baseline[key]}; no GPU client in the window"})
            continue
        verb = "exceeds" if delta > 0 else "falls below"
        reasons.append(f"{key} {summary[key]} {verb} baseline {baseline[key]} by more than {thresholds[limit]}")
    if unclaimed and summary["gpu_power_iqr_w"] > thresholds["gpu_power_iqr_w"]:
        annotations.append({"rule": "unclaimed-power-spread",
                            "detail": f"gpu_power_iqr_w {summary['gpu_power_iqr_w']} exceeds "
                                      f"{thresholds['gpu_power_iqr_w']}; no GPU client in the window"})
    if summary["gpu_pstates"] != [thresholds["gpu_pstate"]]:
        reasons.append(f"gpu_pstate {summary['gpu_pstates']} not always {thresholds['gpu_pstate']}")
    return reasons + client_reasons([summary["gpu_clients"]], lane), annotations


def judge_settled(summary, samples, baseline, thresholds, lane):
    reasons, annotations = judge_window(summary, baseline, thresholds, lane)
    floor = thresholds["mem_available_pct_min"]
    if any(100.0 * s.mem_available_kib / s.mem_total_kib < floor for s in samples):
        reasons.append(f"mem_available_pct below floor {floor}")
    return reasons, annotations
```

In `preflight`, add `"rules": RULES` to each of the four dicts written as the
`preflight` section: the lock refusal, the pre-hold refusal, the pixels
`unsampled` section and the sampled `section`. For example:

```python
        write_section(run_dir, "preflight", {"verdict": "refused", "at": now_rfc3339(), "reasons": [str(error)],
                                             "thresholds": thresholds, "rules": RULES})
```

In `settle`, add `"gpu_power_iqr_w"` to `threshold_keys`. Then replace
everything from `reasons = judge_settled(...)` through the end of the
function with:

```python
    reasons, annotations = judge_settled(summary, samples, record["baseline"], thresholds, record["run"]["lane"])
    previous = next((e["gpu_power_w"] for e in reversed(record.get("sub_runs", []))
                     if e["verdict"] == "settled" and "gpu_power_w" in e), record["baseline"]["gpu_power_w"])
    entry = {"name": sub_run, "started": started, "settled_at": now_rfc3339(), "seconds": seconds, **summary,
             "gpu_power_step_w": round(summary["gpu_power_w"] - previous, 3),
             "inputs": entry_inputs, "verdict": "settled" if not reasons else "refused"}
    if annotations:
        entry["annotations"] = annotations
    if reasons:
        entry["reason"] = "; ".join(reasons)
    append_sub_run(run_dir, entry)
    for annotation in annotations:
        print(f"capture-meta: annotated: {annotation['rule']}: {annotation['detail']}", file=sys.stderr)
    if reasons:
        raise Refused(entry["reason"])
```

In `render`, give the preflight line its rules identity:

```python
        condition = f"  host {preflight['host_condition']}" if "host_condition" in preflight else ""
        rules = f"  rules {preflight['rules']}" if "rules" in preflight else ""
        lines += ["", f"preflight {preflight.get('verdict')}{at}{condition}{rules}"]
```

In the sub-run loop, add the step before the renderer column, then the
annotations under the line:

```python
            step = f"  step {entry['gpu_power_step_w']:+} W" if "gpu_power_step_w" in entry else ""
            lines.append(f"  {entry['name']}: {entry['verdict']}{reason}{span}{step}{checked}")
            lines += [f"    annotated {a['rule']}: {a['detail']}" for a in entry.get("annotations", [])]
```

In `docs/specs/2026-09-11-material-capture-protocol-design.md` §2.4, after
the sentence ending "…and the same P-state and client constraints.", insert:

```markdown
The GPU power tolerance applies on the dedicated lane, and on the headless
lane only when a GPU client appears in the window; a client-free headless
window records a power departure as an annotation
([gates design](2026-10-17-capture-evidence-gates-design.md) §4.1).
```

- [ ] **Step 4: Run the tests to verify they pass**

Run the Step 2 command again.
Expected: PASS. Then run the whole module:
`just --set one_cmd 'env NIRI_TOOLING_FAST=0 python3 -m unittest' test-one tools.test_capture_meta`
Expected: PASS. The existing refusal tests keep refusing, because `FakeGpu`
lists `niri` as a graphics client by default.

- [ ] **Step 5: Run fast tooling and commit**

Run: `just --set fast_cmd 'env NIRI_TOOLING_FAST=0 python3 -m tools.tooling_tests --full' test-fast`
Expected: PASS.

```bash
tasks done material-a0492a "settle annotates client-free power on headless; gpu_power_step_w; preflight.rules"
git add tools/capture-meta tools/test_capture_meta.py docs/specs/2026-09-11-material-capture-protocol-design.md tasks/
git commit -m "feat(capture-meta): annotate client-free GPU power steps in settle"
```

### Task 2: The release scan classifies disturbances by lane

**Files:**
- Modify: `tools/capture_hold.py`: imports, and `classify` after `scan`
  (line 527).
- Modify: `tools/capture-meta`: `SCAN_VERDICTS` (line 577),
  `describe_disturbance` and `exit_for` (lines 607-629), `restore_run`'s
  `record_end` (lines 863-882), and `render`'s hold-end line.
- Modify: `docs/specs/2026-10-04-capture-disturber-hold-design.md` §5,
  `docs/materials/capture-host-setup.md` ("Disturber hold").
- Test: `tools/test_capture_hold.py` (new `ClassifyTests`),
  `tools/test_capture_meta.py` (`HoldRecordTests`, `LifecycleTests`).

**Interfaces:**
- Consumes: the `ch.scan(host, hold, wakes) -> (verdict, list[dict])` that
  exists today. Its verdict is discarded, and its entries are
  `{"at", "at_us", "manager", "unit", "kind"}`.
- Produces: `ch.classify(disturbances: list[dict], lane) -> (str, list[dict])`,
  whose verdict is one of `clean`, `annotated` or `disturbed`. Each returned
  entry is the input entry plus `severity` (`fail`/`annotate`) and `rule`.
  It also produces `cm.SCAN_VERDICTS` with `annotated`, and
  `cm.describe_disturbance(d)`, which appends ` [<severity>]` when present.

- [ ] **Step 1: Write the failing tests**

Add to `tools/test_capture_hold.py`, after `ScanTests`:

```python
def flagged(kind, unit, manager="system"):
    return {"at": "t", "at_us": T0, "manager": manager, "unit": unit, "kind": kind}


class ClassifyTests(unittest.TestCase):
    def test_vt_login_is_annotated_on_nested_lanes_and_fails_elsewhere(self):
        for lane, verdict, severity in (("headless", "annotated", "annotate"), ("pixels", "annotated", "annotate"),
                                        ("dedicated", "disturbed", "fail"), (None, "disturbed", "fail")):
            with self.subTest(lane=lane):
                got, entries = ch.classify([flagged("restart", "getty@tty2.service")], lane)
                self.assertEqual(got, verdict)
                self.assertEqual((entries[0]["severity"], entries[0]["rule"]), (severity, "vt-login"))
        self.assertEqual(ch.classify([flagged("restart", "autovt@tty3.service")], "pixels")[0], "annotated")

    def test_other_units_and_kinds_fail_on_every_lane(self):
        for d in (flagged("restart", "getty-static.service"), flagged("restart", "dropbox.service", "user"),
                  flagged("timer-fired", "systemd-tmpfiles-clean.service"),
                  flagged("timer-added", "run-p2107991-i6325666.timer", "user"),
                  flagged("held-started", "getty@tty2.service"), flagged("monitor-woke", "card1-DP-1", "sysfs")):
            for lane in ("headless", "pixels", "dedicated"):
                with self.subTest(unit=d["unit"], kind=d["kind"], lane=lane):
                    verdict, entries = ch.classify([d], lane)
                    self.assertEqual(verdict, "disturbed")
                    self.assertEqual((entries[0]["severity"], entries[0]["rule"]), ("fail", d["kind"]))

    def test_one_failing_entry_disturbs_and_an_empty_list_is_clean(self):
        verdict, entries = ch.classify([flagged("restart", "getty@tty2.service"),
                                        flagged("timer-fired", "man-db.service")], "headless")
        self.assertEqual((verdict, [e["severity"] for e in entries]), ("disturbed", ["annotate", "fail"]))
        self.assertEqual(ch.classify([], "headless"), ("clean", []))

    def test_an_earlier_severity_is_replaced(self):
        old = dict(flagged("restart", "getty@tty2.service"), severity="fail", rule="restart")
        verdict, entries = ch.classify([old], "headless")
        self.assertEqual((verdict, entries[0]["severity"], entries[0]["rule"]), ("annotated", "annotate", "vt-login"))
```

Add to `tools/test_capture_meta.py` `HoldRecordTests`:

```python
    def test_exit_for_passes_an_annotated_scan_and_refuses_a_dedicated_vt_login(self):
        getty = {"kind": "restart", "unit": "getty@tty2.service", "manager": "system", "at": "t"}
        complete = {"state": "complete", "attempts": []}
        verdict, entries = cm.ch.classify([getty], "headless")
        stderr = io.StringIO()
        with contextlib.redirect_stderr(stderr):
            cm.exit_for({"restore": complete, "scan": {"verdict": verdict, "disturbances": entries}})
        self.assertIn("annotated: restart getty@tty2.service (system) at t [annotate]", stderr.getvalue())
        verdict, entries = cm.ch.classify([getty], "dedicated")
        with self.assertRaisesRegex(cm.Refused, r"getty@tty2.service \(system\) at t \[fail\]"):
            cm.exit_for({"restore": complete, "scan": {"verdict": verdict, "disturbances": entries}})
        cm.validate_record({"hold_end": {"scan": {"verdict": "annotated", "disturbances": entries}}})

    def test_show_counts_annotations_and_prints_severity(self):
        record = {"run": {"id": "r"}, "hold": {"desktop": "absent", "items": []},
                  "hold_end": {"restore": {"state": "complete", "attempts": [{"by": "release", "failures": []}]},
                               "scan": {"verdict": "annotated", "disturbances": [
                                   {"kind": "restart", "unit": "getty@tty2.service", "manager": "system", "at": "t",
                                    "severity": "annotate", "rule": "vt-login"}]}}}
        text = cm.render(record)
        self.assertIn("scan annotated (1)", text)
        self.assertIn("  restart getty@tty2.service (system) at t [annotate]", text)
```

Add to `LifecycleTests`:

```python
    def test_console_login_restart_is_annotated_and_release_passes(self):
        self.preflight()
        self.host.journal["system"].append({"__REALTIME_TIMESTAMP": str(self.host.now_us() + 5),
                                            "UNIT": "getty@tty2.service", "MESSAGE_ID": cm.ch.RESTART})
        self.host.clock += 60_000_000
        stderr = io.StringIO()
        with contextlib.redirect_stderr(stderr):
            self.assertEqual(cm.main(["release", str(self.run)]), 0)
        scan = cm.load_record(self.run)["hold_end"]["scan"]
        self.assertEqual(scan["verdict"], "annotated")
        self.assertEqual([(d["unit"], d["severity"], d["rule"]) for d in scan["disturbances"]],
                         [("getty@tty2.service", "annotate", "vt-login")])
        self.assertIn("annotated: restart getty@tty2.service", stderr.getvalue())
        self.assertEqual(self.host.snapshot(), self.before)
```

In `LifecycleTests.test_disturbed_run_exits_one_after_restoring`, add after
its last assertion:

```python
        entry = cm.load_record(self.run)["hold_end"]["scan"]["disturbances"][0]
        self.assertEqual((entry["severity"], entry["rule"]), ("fail", "timer-fired"))
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `just --set one_cmd 'env NIRI_TOOLING_FAST=0 python3 -m unittest' test-one tools.test_capture_hold.ClassifyTests tools.test_capture_meta.HoldRecordTests tools.test_capture_meta.LifecycleTests`
Expected: FAIL, with `AttributeError: module 'capture_hold' has no attribute 'classify'`.
`test_console_login_restart_is_annotated_and_release_passes` exits 1, not 0.

- [ ] **Step 3: Implement**

In `tools/capture_hold.py`, add `import re` to the imports, then, after
`scan`:

```python
VT_UNIT = re.compile(r"(getty|autovt)@tty\d+\.service")
VT_ANNOTATED_LANES = ("headless", "pixels")


def classify(disturbances, lane):
    """Severity and rule for each flagged entry (2026-10-17 gates design §4.2). A console
    login restarts its getty: a nested compositor never sees it, a DRM session on a VT
    does. Every other kind fails on every lane, and so does everything when the lane is
    unknown."""
    classified = []
    for d in disturbances:
        vt = d["kind"] == "restart" and VT_UNIT.fullmatch(d["unit"]) is not None
        severity = "annotate" if vt and lane in VT_ANNOTATED_LANES else "fail"
        classified.append({**d, "severity": severity, "rule": "vt-login" if vt else d["kind"]})
    if not classified:
        return "clean", classified
    return ("disturbed" if any(d["severity"] == "fail" for d in classified) else "annotated"), classified
```

In `tools/capture-meta`:

```python
SCAN_VERDICTS = ("clean", "annotated", "disturbed", "unscanned", "not-run")
```

```python
def describe_disturbance(d):
    severity = f" [{d['severity']}]" if "severity" in d else ""
    return f"{d['kind']} {d['unit']} ({d['manager']}) at {d['at']}{severity}"
```

In `exit_for`, before `if verdict == "clean":`:

```python
    if verdict == "annotated":
        for d in scan["disturbances"]:
            print(f"capture-meta: annotated: {describe_disturbance(d)}", file=sys.stderr)
        return
```

In `restore_run`'s `record_end`, read the record once and classify what the
scan flags:

```python
    def record_end(hold, wakes, attempt):
        current = load_record(run_dir)
        end = current.get("hold_end", {})
        ...
                try:
                    _, flagged = ch.scan(host, hold, wakes)
                    verdict, disturbances = ch.classify(flagged, current.get("run", {}).get("lane"))
                    scan = {**base, "verdict": verdict, "disturbances": disturbances}
```

In `render`, count the annotations on the hold-end line:

```python
        verdict = scan.get("verdict")
        if verdict == "annotated":
            verdict = f"annotated ({len(scan['disturbances'])})"
        lines.append(f"hold end  restore {restore.get('state')} ({last})  scan {verdict}")
```

In `docs/specs/2026-10-04-capture-disturber-hold-design.md`, append to §5,
after the "Idle locks have no entry…" paragraph:

```markdown
Each flagged entry carries a `severity` (`fail` or `annotate`) and a `rule`,
and `scan.verdict` gains `annotated` (every entry annotated; release exits 0).
A `restart` of `getty@tty*` or `autovt@tty*` is annotated on the headless and
pixels lanes and fails on dedicated; every other entry fails
([gates design](2026-10-17-capture-evidence-gates-design.md) §4.2).
```

In `docs/materials/capture-host-setup.md`, replace "scans the journal; a
disturbed run fails its fixture." with:

```markdown
scans the journal; a disturbed run fails its fixture. A console login on
another VT (a `getty@`/`autovt@` restart) is recorded as an annotation on the
headless and pixels lanes and still fails a dedicated run
([gates design](../specs/2026-10-17-capture-evidence-gates-design.md)).
```

- [ ] **Step 4: Run the tests to verify they pass**

Run the Step 2 command again, then
`just --set one_cmd 'env NIRI_TOOLING_FAST=0 python3 -m unittest' test-one tools.test_capture_hold tools.test_capture_meta`.
Expected: PASS.

- [ ] **Step 5: Run fast tooling and commit**

Run: `just --set fast_cmd 'env NIRI_TOOLING_FAST=0 python3 -m tools.tooling_tests --full' test-fast`
Expected: PASS.

```bash
tasks done material-1fcd36 "release scan classifies by lane; VT logins annotated on nested lanes; annotated verdict exits 0"
git add tools/capture_hold.py tools/capture-meta tools/test_capture_hold.py tools/test_capture_meta.py docs/specs/2026-10-04-capture-disturber-hold-design.md docs/materials/capture-host-setup.md tasks/
git commit -m "feat(capture-meta): classify scan disturbances by lane"
```

### Task 3: `capture-meta judge` re-judges a record, read-only

**Files:**
- Modify: `tools/capture-meta`. Add the judge functions after `release`
  (before `def guard`), and the `judge` subparser in `build_parser`.
- Test: `tools/test_capture_meta.py`, a new `JudgeTests` class after
  `ShowTests`.

**Interfaces:**
- Consumes: `judge_window`, `DEFAULT_THRESHOLDS` and `RULES` (Task 1);
  `ch.classify` and `describe_disturbance` (Task 2); `load_record`.
- Produces: `PASS = "pass"`, `FAIL = "fail"`, `NO_EVIDENCE = "no evidence"`.
  `judge_preflight(record)`, `judge_windows(record)` and `judge_scan(record)`
  each return `(outcome: str, recorded: str, lines: list[str])`.
  `judge_record(record) -> dict[str, tuple]`, keyed `preflight`, `settle`,
  `scan`. `cmd_judge(args)` raises `CannotRun` (exit 2) or `Refused` (exit 1).

- [ ] **Step 1: Write the failing tests**

Add after `class ShowTests`:

```python
def sampled(name, verdict, power, iqr=0.2, util=0.0, cpu=1.0, pstates=("P8",), graphics=(), reason=None):
    e = {"name": name, "verdict": verdict, **window(power, iqr, util, cpu, 80.0, pstates, graphics)}
    if reason:
        e["reason"] = reason
    return e


def capture_record(sub_runs=(), lane="headless", baseline_w=10.39, graphics=(), preflight=None, hold=True,
           scan=None, restore="complete"):
    r = {"run": {"id": "r", "lane": lane, "started": STARTED},
         "baseline": {"seconds": 20, **window(baseline_w, graphics=graphics)},
         "preflight": preflight if preflight is not None else {"verdict": "quiet", "thresholds": dict(cm.DEFAULT_THRESHOLDS)},
         "sub_runs": list(sub_runs)}
    if hold:
        r["hold"] = {"items": []}
    if scan is not None:
        r["hold_end"] = {"restore": {"state": restore, "attempts": []},
                         "scan": {"until_us": 1, "disturbances": [], **scan}}
    return r


CLEAN = {"verdict": "clean"}


class JudgeTests(unittest.TestCase):
    def judged(self, rec):
        """Runs `judge` on a saved record and checks it changed nothing in the run directory."""
        with tempfile.TemporaryDirectory() as directory:
            run = pathlib.Path(directory)
            if rec is not None:
                cm.save_record(run, rec)
            before = {p.name: (p.read_bytes(), p.stat().st_mtime_ns) for p in run.iterdir()}
            stdout = io.StringIO()
            with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(io.StringIO()):
                code = cm.main(["judge", str(run)])
            after = {p.name: (p.read_bytes(), p.stat().st_mtime_ns) for p in run.iterdir()}
            self.assertEqual(after, before)
            return code, stdout.getvalue()

    def test_a_refused_client_free_floor_step_now_settles(self):   # noise-idle-ab-full-1792148041
        code, out = self.judged(capture_record([sampled("none-b2-1", "settled", 10.425),
                                        sampled("none-b3-1", "refused", 17.115, iqr=0.027,
                                              reason="gpu_power_w 17.115 exceeds baseline 10.39 by more than 1.5")],
                                       scan=CLEAN))
        self.assertEqual(code, 0, out)
        self.assertIn("none-b3-1: recorded refused, now settled (unclaimed-power-step", out)
        self.assertIn("rules recorded before 2026-10-17, current 2026-10-17", out)

    def test_a_recorded_vt_login_disturbance_now_passes(self):   # cost-pilot-1792145940
        getty = {"kind": "restart", "unit": "getty@tty2.service", "manager": "system", "at": "t"}
        code, out = self.judged(capture_record([sampled("tilt0-a1", "settled", 10.3)], baseline_w=10.345,
                                       scan={"verdict": "disturbed", "disturbances": [getty]}))
        self.assertEqual(code, 0, out)
        self.assertIn("recorded disturbed, now annotated", out)

    def test_the_2026_09_24_idle_lock_record_still_fails(self):   # trace-20260924T163317, no hold
        code, out = self.judged(capture_record([sampled("B-resize-2", "settled", 9.995, graphics=DESKTOP),
                                        sampled("B-resize-3", "refused", 11.585, iqr=0.258, graphics=DESKTOP,
                                              reason="gpu_power_w 11.585 exceeds baseline 9.9 by more than 1.5")],
                                       baseline_w=9.9, graphics=DESKTOP, hold=False))
        self.assertEqual(code, 1, out)
        self.assertIn("settle: fail", out)
        self.assertIn("no hold: predates the disturber hold", out)

    def test_a_refused_preflight_with_no_sub_runs_fails(self):   # render-order-readiness.pCbsOC
        reasons = ["cpu_busy_pct 16.6 exceeds 10.0", "load1 5.53 exceeds 2.0", "gpu_util_pct 26.0 exceeds 5.0",
                   "gpu_pstate ['P3', 'P5', 'P8'] not always P8", "gpu_power_iqr_w 7.463 exceeds 1.0",
                   "compute clients present: BitwigStudio"]
        code, out = self.judged(capture_record(preflight={"verdict": "refused", "reasons": reasons}, hold=False))
        self.assertEqual(code, 1, out)
        self.assertIn("preflight: fail", out)
        self.assertIn("refused: load1 5.53 exceeds 2.0", out)

    def test_scan_states_without_evidence(self):
        refused = {"verdict": "refused", "reasons": ["gpu_util_pct 90.0 exceeds 5.0"]}
        quiet_windows = [sampled("A", "settled", 10.4)]
        power_refusal = sampled("B", "refused", 17.0, graphics=("niri",), reason="gpu_power_w 17.0 exceeds baseline 10.39 by more than 1.5")
        cases = (  # (name, record, exit, needle)
            ("not-run, preflight refused", capture_record(preflight=refused, scan={"verdict": "not-run", "preflight_exit": 1}), 1, "scan: fail"),
            ("not-run, could not run", capture_record(scan={"verdict": "not-run", "preflight_exit": 2}), 2, "scan: no evidence"),
            ("unscanned, empty list", capture_record(quiet_windows, scan={"verdict": "unscanned", "error": "journalctl: exit 1"}), 2, "unscanned (journalctl: exit 1)"),
            ("held, never released", capture_record(quiet_windows), 2, "never released"),
            ("restore failed", capture_record(quiet_windows, scan=CLEAN, restore="failed"), 2, "not fully restored"),
            ("unscanned outranks a refused window", capture_record([power_refusal], scan={"verdict": "unscanned"}), 2, "settle: fail"),
            ("no preflight", {**capture_record(quiet_windows, scan=CLEAN), "preflight": None}, 2, "no preflight section"))
        for name, rec, code_expected, needle in cases:
            with self.subTest(name):
                if rec.get("preflight") is None:
                    rec.pop("preflight")
                code, out = self.judged(rec)
                self.assertEqual(code, code_expected, out)
                self.assertIn(needle, out)

    def test_a_quiet_record_from_before_the_hold_passes_and_says_so(self):
        code, out = self.judged(capture_record([sampled("A", "settled", 9.9, graphics=DESKTOP)], baseline_w=9.9,
                                       graphics=DESKTOP, hold=False))
        self.assertEqual(code, 0, out)
        self.assertIn("no hold: predates the disturber hold", out)

    def test_old_entries_and_thresholds_are_kept_or_defaulted(self):
        old = capture_record([{"name": "A", "verdict": "settled", "started": STARTED},
                      {"name": "B", "verdict": "refused", "reason": "gpu busy"}],
                     preflight={"verdict": "quiet"}, scan=CLEAN)
        code, out = self.judged(old)
        self.assertEqual(code, 1, out)
        self.assertIn("A: window not recorded; kept as settled", out)
        self.assertIn("B: window not recorded; kept as refused", out)
        self.assertIn("judged at today's defaults", out)
        floor = capture_record([sampled("C", "refused", 10.4, reason="mem_available_pct below floor 20.0")], scan=CLEAN)
        self.assertEqual(self.judged(floor)[0], 1)

    def test_pixel_runs_have_no_windows_to_judge(self):
        pixel = capture_record([{"name": "cell-1", "verdict": "begun", "started": STARTED}], lane="pixels",
                       preflight={"verdict": "unsampled"}, scan=CLEAN)
        pixel.pop("baseline")
        self.assertEqual(self.judged(pixel)[0], 0)

    def test_a_missing_or_corrupt_record_cannot_be_judged(self):
        self.assertEqual(self.judged(None)[0], 2)
        with tempfile.TemporaryDirectory() as directory:
            (pathlib.Path(directory) / cm.RECORD).write_text("{not json")
            with contextlib.redirect_stderr(io.StringIO()):
                self.assertEqual(cm.main(["judge", directory]), 2)
            self.assertEqual(sorted(p.name for p in pathlib.Path(directory).iterdir()), [cm.RECORD])
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `just --set one_cmd 'env NIRI_TOOLING_FAST=0 python3 -m unittest' test-one tools.test_capture_meta.JudgeTests`
Expected: FAIL. argparse exits 2 with "invalid choice: 'judge'", so every
exit-code assertion fails, apart from the missing-record cases, which
expect 2 anyway.

- [ ] **Step 3: Implement**

In `tools/capture-meta`, before `def guard(`:

```python
PASS, FAIL, NO_EVIDENCE = "pass", "fail", "no evidence"
WINDOW_KEYS = ("cpu_busy_pct", "mem_available_pct", "gpu_util_pct", "gpu_power_w", "gpu_power_iqr_w",
               "gpu_pstates", "gpu_clients")
BASELINE_KEYS = ("cpu_busy_pct", "mem_available_pct", "gpu_util_pct", "gpu_power_w")


def judge_preflight(record):
    """Preflight's rules are unchanged: the recorded verdict stands (2026-10-17 gates design §5)."""
    preflight = record.get("preflight")
    if preflight is None:
        return NO_EVIDENCE, "none", ["no preflight section"]
    verdict = preflight.get("verdict")
    if verdict in ("quiet", "unsampled"):
        return PASS, verdict, []
    if verdict == "refused":
        return FAIL, verdict, [f"refused: {r}" for r in preflight.get("reasons") or ["no reason recorded"]]
    return NO_EVIDENCE, str(verdict), [f"verdict {verdict!r} is not one preflight writes"]


def judge_windows(record):
    """Re-applies the settle rules to every sampled sub-run. The per-sample memory floor is
    not recorded, so a refusal that names it stands."""
    entries = [e for e in record.get("sub_runs", []) if e["verdict"] in ("settled", "refused")]
    recorded = ", ".join(f"{sum(e['verdict'] == v for e in entries)} {v}" for v in ("settled", "refused"))
    if not entries:
        return PASS, "no sampled sub-runs", []
    baseline = record.get("baseline")
    if not isinstance(baseline, dict) or any(key not in baseline for key in BASELINE_KEYS):
        return NO_EVIDENCE, recorded, ["sub-runs were sampled but the baseline is missing"]
    kept = (record.get("preflight") or {}).get("thresholds") or {}
    thresholds = {**DEFAULT_THRESHOLDS, **kept}
    lines, failed = [], False
    missing = sorted(set(DEFAULT_THRESHOLDS) - set(kept))
    if missing:
        lines.append(f"thresholds not recorded, judged at today's defaults: {', '.join(missing)}")
    lane = record.get("run", {}).get("lane")
    for entry in entries:
        if any(key not in entry for key in WINDOW_KEYS):
            failed = failed or entry["verdict"] == "refused"
            lines.append(f"{entry['name']}: window not recorded; kept as {entry['verdict']}")
            continue
        reasons, annotations = judge_window(entry, baseline, thresholds, lane)
        if "mem_available_pct below floor" in entry.get("reason", ""):
            reasons.append("mem_available_pct below floor (recorded; samples not kept)")
        now = "refused" if reasons else "settled"
        failed = failed or bool(reasons)
        if now != entry["verdict"]:
            why = "; ".join(reasons) or "; ".join(f"{a['rule']}: {a['detail']}" for a in annotations)
            lines.append(f"{entry['name']}: recorded {entry['verdict']}, now {now} ({why})")
    return (FAIL if failed else PASS), recorded, lines


def judge_scan(record):
    """The scan's state decides before any entry is classified: an empty disturbance list
    means none found only in a scan that ran."""
    end = record.get("hold_end") or {}
    scan = end.get("scan")
    if scan is None:
        if "hold" not in record:
            return PASS, "none", ["no hold: predates the disturber hold, or preflight refused before holding"]
        return NO_EVIDENCE, "none", ["hold recorded but never released; no scan"]
    verdict = scan["verdict"]
    if end.get("restore", {}).get("state") == "failed":
        return NO_EVIDENCE, verdict, ["hold not fully restored"]
    if verdict == "unscanned":
        return NO_EVIDENCE, verdict, [f"unscanned ({scan.get('error')})"]
    if verdict == "not-run":
        code = scan.get("preflight_exit")
        return (FAIL if code == 1 else NO_EVIDENCE), verdict, [f"preflight rolled back its hold (exit {code})"]
    now, entries = ch.classify(scan["disturbances"], record.get("run", {}).get("lane"))
    lines = [f"recorded {verdict}, now {now}"] if now != verdict else []
    lines += [describe_disturbance(d) for d in entries]
    return (FAIL if now == "disturbed" else PASS), verdict, lines


def judge_record(record):
    return {"preflight": judge_preflight(record), "settle": judge_windows(record), "scan": judge_scan(record)}


def cmd_judge(args):
    """Re-judges a run under the current rules from its capture.json alone; writes nothing."""
    if not (pathlib.Path(args.run_dir) / RECORD).is_file():
        raise CannotRun(f"no {RECORD} in {args.run_dir}")
    record = load_record(args.run_dir)
    parts = judge_record(record)
    rules = (record.get("preflight") or {}).get("rules", "before 2026-10-17")
    print(f"judge {record.get('run', {}).get('id')}  rules recorded {rules}, current {RULES}")
    for name, (outcome, recorded, lines) in parts.items():
        print(f"  {name}: {outcome} (recorded {recorded})")
        for line in lines:
            print(f"    {line}")
    def named(outcome):
        return ", ".join(name for name, part in parts.items() if part[0] == outcome)
    if named(NO_EVIDENCE):
        raise CannotRun(f"no evidence: {named(NO_EVIDENCE)}")
    if named(FAIL):
        raise Refused(f"fails under the current rules: {named(FAIL)}")
```

In `build_parser`, before `return parser`:

```python
    jdg = subs.add_parser("judge")
    jdg.add_argument("run_dir")
    jdg.set_defaults(func=cmd_judge)
```

- [ ] **Step 4: Run the tests to verify they pass**

Run the Step 2 command again, then the whole module:
`just --set one_cmd 'env NIRI_TOOLING_FAST=0 python3 -m unittest' test-one tools.test_capture_meta`.
Expected: PASS.

- [ ] **Step 5: Run fast tooling and commit**

Run: `just --set fast_cmd 'env NIRI_TOOLING_FAST=0 python3 -m tools.tooling_tests --full' test-fast`
Expected: PASS.

```bash
tasks done material-34a75c "capture-meta judge re-judges a record read-only; no evidence outranks fail"
git add tools/capture-meta tools/test_capture_meta.py tasks/
git commit -m "feat(capture-meta): judge re-applies the gate rules to a record"
```

### Task 4: Replay the retained records and close the documents

**Files:**
- Modify: `docs/specs/2026-10-17-capture-evidence-gates-design.md` (status
  line, and §6.2 results).
- Modify: `docs/specs/2026-10-08-capture-host-conditions-design.md` §8.2
  (line 527).

**Interfaces:**
- Consumes: `tools/capture-meta judge` (Task 3).
- Produces: the replay result, as a `tasks note` and as spec §6.2 text.

- [ ] **Step 1: Replay every retained record**

The replay is offline. It reads records under `$NIRI_MATERIAL_WORK_ROOT`
and writes only to a scratch directory:

```bash
out=$(mktemp -d)
find "$NIRI_MATERIAL_WORK_ROOT" -name capture.json -printf '%h\n' | sort | while read -r d; do
  tools/capture-meta judge "$d" >"$out/$(printf %s "$d" | sha1sum | cut -c1-12).txt" 2>&1
  printf '%s %s\n' "$?" "${d#"$NIRI_MATERIAL_WORK_ROOT"/}"
done >"$out/exits.txt"
awk '{print $1}' "$out/exits.txt" | sort | uniq -c
grep -l "recorded settled, now refused" "$out"/*.txt || echo "no settled window fails"
```

- [ ] **Step 2: Check each expectation from spec §6.2**

```bash
for run in material-6901e0/cost-pilot-1792145107 material-6901e0/cost-pilot-1792145940 \
           noise-idle-ab-full-1792148041 noise-idle-ab-full-1792148490; do grep " $run\$" "$out/exits.txt"; done   # each 0
for run in material-265eb0/trace-20260924T163317 material-265eb0/trace-20260924T165644 \
           material-265eb0/power-full-20260927T030030 hold-pilot/pc-1 noise-layers-pilot-1791337568 \
           render-order-readiness.pCbsOC; do grep " $run\$" "$out/exits.txt"; done                              # each 1
grep -c '^2 ' "$out/exits.txt"; grep '^2 ' "$out/exits.txt"                                                   # no-evidence list
```

Expected:
- the first four exit 0;
- the next six exit 1;
- `grep -l` prints "no settled window fails".

For each exit-2 record, read its output file and name the reason (an
unreadable schema, never released, unscanned, or a `not-run` with exit 2).
Every refused preflight should exit 1 (29 on 2026-10-17), or 2 where its
rollback could not run. If any expectation fails, stop: the rules or `judge`
disagree with the spec. Fix that under its own task, with a test, before
going on.

- [ ] **Step 3: Record the result**

Append to spec §6.2, under a `**Replay, <date>:**` lead-in:

- the exit counts;
- the four 2026-10-16 runs and their exit 0;
- the six still-failing runs;
- each exit-2 record with its reason;
- the statement "no recorded `settled` window fails".

Also make these edits:

- **This spec's status line:** "implemented on `capture-gates-evidence`
  (plan `docs/plans/2026-10-17-capture-evidence-gates.md`); retained-record
  replay in §6.2".
- **Host conditions §8.2:** replace "no disturbance is recorded;" with "no
  disturbance of severity `fail` is recorded (an annotated floor step or VT
  login is reported in the run's `run:` note);".

Then:

```bash
tasks note material-2e4ea5 "replay: <exit counts>; 2026-10-16 refusals all exit 0; 2026-09-24, timer-fired, pstate and pCbsOC refusals exit 1; no settled window fails"
tasks note material-04818d "The settle gate change its done-criterion allows is reviewed and implemented: docs/specs/2026-10-17-capture-evidence-gates-design.md (client-free headless power steps are annotated). The floor's cause stays this task's question."
```

- [ ] **Step 4: Check and commit**

Run: `tasks check`, then `just check`.
Expected: both clean.

```bash
tasks done material-62191b "retained-record replay recorded; host-conditions pilot criteria take severity"
git add docs/specs/2026-10-17-capture-evidence-gates-design.md docs/specs/2026-10-08-capture-host-conditions-design.md tasks/
git commit -m "docs(specs): record the gate replay and severity-aware pilot criteria"
```
