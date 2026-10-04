# Pre-commit latency revision evidence

These are diagnostic prototypes, not a shipped remedy or qualifying post-remedy runs.
The Ubuntu full suite still fails the intermittent TERM/trap case.

## Whole warm fast-hook records

```json
[
  {
    "at": "2026-10-04T17:36:25Z",
    "target": "hook-pre-commit",
    "seconds": 30.986,
    "exit": 0,
    "tests": 287,
    "rev": "ca634f24",
    "selection": null
  },
  {
    "at": "2026-10-04T17:42:15Z",
    "target": "hook-pre-commit",
    "seconds": 32.972,
    "exit": 0,
    "tests": 287,
    "rev": "ca634f24",
    "selection": null
  },
  {
    "at": "2026-10-04T17:42:49Z",
    "target": "hook-pre-commit",
    "seconds": 34.424,
    "exit": 0,
    "tests": 287,
    "rev": "ca634f24",
    "selection": null
  }
]
```

## Per-stage records

```jsonl
{"stage": "stage-report", "seconds": 0.226, "exit": 0}
{"stage": "hygiene", "seconds": 1.753, "exit": 0}
{"stage": "target-isolation", "seconds": 0.275, "exit": 0}
{"stage": "format", "seconds": 1.174, "exit": 0}
{"stage": "clippy", "seconds": 0.374, "exit": 0}
{"stage": "tooling-fast", "seconds": 26.094, "exit": 0}
{"stage": "tasks", "seconds": 0.032, "exit": 0}
{"stage": "report-check", "seconds": 0.196, "exit": 0}
{"stage": "package-pin", "seconds": 0.053, "exit": 0}
{"stage": "stage-report", "seconds": 0.186, "exit": 0}
{"stage": "hygiene", "seconds": 1.681, "exit": 0}
{"stage": "target-isolation", "seconds": 0.308, "exit": 0}
{"stage": "format", "seconds": 1.158, "exit": 0}
{"stage": "clippy", "seconds": 0.7, "exit": 0}
{"stage": "tooling-fast", "seconds": 27.628, "exit": 0}
{"stage": "tasks", "seconds": 0.064, "exit": 0}
{"stage": "report-check", "seconds": 0.378, "exit": 0}
{"stage": "package-pin", "seconds": 0.127, "exit": 0}
{"stage": "stage-report", "seconds": 0.409, "exit": 0}
{"stage": "hygiene", "seconds": 2.296, "exit": 0}
{"stage": "target-isolation", "seconds": 0.296, "exit": 0}
{"stage": "format", "seconds": 2.503, "exit": 0}
{"stage": "clippy", "seconds": 0.987, "exit": 0}
{"stage": "tooling-fast", "seconds": 26.573, "exit": 0}
{"stage": "tasks", "seconds": 0.032, "exit": 0}
{"stage": "report-check", "seconds": 0.197, "exit": 0}
{"stage": "package-pin", "seconds": 0.048, "exit": 0}
```

## Clean Ubuntu installed versions

```text
ca-certificates 20260601~24.04.1
dbus 1.14.10-4ubuntu4.1
gir1.2-gst-plugins-base-1.0 1.24.2-1ubuntu0.5
gir1.2-gstreamer-1.0 1.24.2-1ubuntu0.1
git 1:2.43.0-1ubuntu7.3
gstreamer1.0-pipewire 1.0.5-1ubuntu3.3
gstreamer1.0-plugins-base 1.24.2-1ubuntu0.5
gstreamer1.0-tools 1.24.2-1ubuntu0.1
jq 1.7.1-3ubuntu0.24.04.2
procps 2:4.0.4-4ubuntu3.3
python3 3.12.3-0ubuntu2.1
python3-gi 3.48.2-1
xz-utils 5.6.1+really5.4.5-1ubuntu0.3
just 1.58.0
cargo 1.99.0 (5f94df478 2026-08-27)
GNU bash, version 5.2.21(1)-release (x86_64-pc-linux-gnu)
```

## git-pilot result

```json
{
  "ran": 66,
  "skipped": [],
  "failures": 0,
  "errors": 0
}
```

## pilot result

```json
{
  "ran": 6,
  "skipped": [],
  "failures": 0,
  "errors": 0
}
```

## lock-pilot result

```json
{
  "ran": 3,
  "skipped": [],
  "failures": 0,
  "errors": 0
}
```

## full result

```json
{
  "ran": 287,
  "skipped": [
    [
      "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_ring_replaces_inherited_default_response_with_retained_candidate",
      "set MATERIAL_RETAINED_NIRI to validate generated KDL"
    ],
    [
      "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_positive_face_gate_requires_more_than_one_code",
      "ImageMagick is required for positive face probes"
    ],
    [
      "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_remaining_within_configs_validate_with_retained_candidate",
      "set MATERIAL_RETAINED_NIRI to validate generated KDL"
    ],
    [
      "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_signed_transfer_helpers_reject_black_clipped_and_wrong_solid_output",
      "ImageMagick is required for pixel probes"
    ],
    [
      "test_glass_render_order_metrics.RenderOrderMetricsTest.test_cli_distinguishes_failed_gate_from_invalid_capture",
      "ImageMagick is required for capture decoding"
    ]
  ],
  "failures": 1,
  "errors": 0
}
```

## failure-control result

```json
{
  "ran": 1,
  "skipped": [],
  "failures": 1,
  "errors": 0
}
```

## dynamic-skip-control result

```json
{
  "ran": 1,
  "skipped": [
    [
      "test_vt_lib.VtLibTests.test_a_hanging_chvt_is_bounded",
      "unexpected dependency skip control"
    ]
  ],
  "failures": 0,
  "errors": 0
}
```

## Reproduction inputs

Download the official just 1.58.0 Linux musl archive and the Cargo 1.99.0 Linux GNU archive as `just.tar.gz` and `cargo.tar.xz`. Build the following Dockerfile with target `complete`. On a disposable container, mount the task worktree read-only at `/source`, these scripts read-only at `/probe`, and an empty writable result directory at `/results`; run `bash /probe/final-run.sh`. Its full-suite failure is the preserved result, not an expected-success command. The scripts modify only the copied repository and use the existing CI recipe override.

### Dockerfile

```dockerfile
FROM ubuntu:24.04 AS tooling-deps
ENV DEBIAN_FRONTEND=noninteractive
RUN --mount=type=cache,target=/var/cache/apt rm -f /etc/apt/apt.conf.d/docker-clean && apt-get -o Acquire::Retries=3 -o Acquire::http::Timeout=60 update && apt-get -o Acquire::Retries=3 -o Acquire::http::Timeout=60 install -y --no-install-recommends python3 git jq procps xz-utils ca-certificates
RUN --mount=type=cache,target=/var/cache/apt sed -i 's|http://|https://|g' /etc/apt/sources.list.d/ubuntu.sources && apt-get -o Acquire::Retries=3 -o Acquire::https::Timeout=60 update && apt-get -o Acquire::Retries=3 -o Acquire::https::Timeout=60 install -y --no-install-recommends python3-gi gir1.2-gstreamer-1.0 gir1.2-gst-plugins-base-1.0 gstreamer1.0-tools gstreamer1.0-plugins-base gstreamer1.0-pipewire dbus
FROM tooling-deps AS complete
COPY just.tar.gz cargo.tar.xz /bootstrap/
RUN tar -xzf /bootstrap/just.tar.gz -C /usr/local/bin just && for component in cargo; do mkdir /installer; tar -xJf /bootstrap/$component.tar.xz -C /installer --strip-components=1; /installer/install.sh --prefix=/usr/local --disable-ldconfig; rm -r /installer; done && rm -r /bootstrap
```

### run_tests.py

```python
import json, pathlib, sys, unittest
sys.path.insert(0, str(pathlib.Path.cwd()))
suite = unittest.defaultTestLoader.discover("tools")
import test_gates
original = test_gates.Gates.setUp
def setup(self):
    original(self)
    self.stub("host-budget", 'test "$1" = run && test "$2" = -- || exit 2\nshift 2\nexec "$@"')
test_gates.Gates.setUp = setup
import subprocess
original_run = subprocess.run
def observed_run(*args, **kwargs):
    result = original_run(*args, **kwargs)
    if args and isinstance(args[0], (list, tuple)) and "merge-tree" in args[0] and "no-such-ref" in args[0]:
        print(json.dumps({"invalid_ref_git_exit": result.returncode, "stdout": result.stdout, "stderr": result.stderr}), flush=True)
    return result
subprocess.run = observed_run
allowed = {
 "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_ring_replaces_inherited_default_response_with_retained_candidate",
 "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_remaining_within_configs_validate_with_retained_candidate",
 "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_positive_face_gate_requires_more_than_one_code",
 "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_signed_transfer_helpers_reject_black_clipped_and_wrong_solid_output",
 "test_glass_render_order_metrics.RenderOrderMetricsTest.test_cli_distinguishes_failed_gate_from_invalid_capture",
}
def cases(s):
    for item in s:
        if isinstance(item, unittest.TestSuite): yield from cases(item)
        else: yield item
inventory = list(cases(suite))
blocked = [t.id() for t in inventory if (getattr(type(t), "__unittest_skip__", False) or getattr(getattr(t, t._testMethodName), "__unittest_skip__", False)) and t.id() not in allowed]
print(json.dumps({"inventory":len(inventory), "unexpected_dependency_skips":blocked}), flush=True)
if blocked: sys.exit(2)
if "--unexpected-skip" in sys.argv:
    def skipped(self): self.skipTest("unexpected dependency skip control")
    import test_vt_lib
    test_vt_lib.VtLibTests.test_a_hanging_chvt_is_bounded = skipped
    suite = unittest.defaultTestLoader.loadTestsFromName("test_vt_lib.VtLibTests.test_a_hanging_chvt_is_bounded")
if "--fail-lifecycle" in sys.argv:
    def broken(self): self.fail("deliberate lifecycle failure control")
    import test_vt_lib
    test_vt_lib.VtLibTests.test_a_hanging_chvt_is_bounded = broken
    suite = unittest.defaultTestLoader.loadTestsFromName("test_vt_lib.VtLibTests.test_a_hanging_chvt_is_bounded")
if "--lock-pilot" in sys.argv:
    suite = unittest.TestSuite(unittest.defaultTestLoader.loadTestsFromName("test_optic_settling.DriverCleanupTests.test_term_with_the_session_locked_reaps_the_lock_client") for _ in range(3))
if "--git-pilot" in sys.argv:
    suite = unittest.defaultTestLoader.loadTestsFromName("test_upstream_report")
if "--pilot" in sys.argv:
    names = ["test_optic_settling.DriverCleanupTests.test_a_screencast_run_reaches_its_crops", "test_vt_lib.VtLibTests.test_a_hanging_chvt_is_bounded", "test_screencast_consumer.ConsumerEndToEndTests.test_startup_sampling_and_summary", "test_gates.Gates.test_focused_recipe_preserves_arguments_and_counts_stderr", "test_target_dir_check.TargetDirCheckTest.test_worktrees_with_their_own_target_pass", "test_target_dir_check.TargetDirCheckTest.test_the_hint_overrides_a_shared_parent_config"]
    suite = unittest.defaultTestLoader.loadTestsFromNames(names)
result = unittest.TextTestRunner(verbosity=2).run(suite)
info = {"ran":result.testsRun,"skipped":[[t.id(),why] for t,why in result.skipped],"failures":len(result.failures),"errors":len(result.errors)}
print(json.dumps(info),flush=True)
pathlib.Path("/results/" + ("dynamic-skip-control.json" if "--unexpected-skip" in sys.argv else "lock-pilot.json" if "--lock-pilot" in sys.argv else "git-pilot.json" if "--git-pilot" in sys.argv else "failure-control.json" if "--fail-lifecycle" in sys.argv else "pilot.json" if "--pilot" in sys.argv else "full.json")).write_text(json.dumps(info,indent=2))
sys.exit(not result.wasSuccessful() or any(t.id() not in allowed for t,_ in result.skipped))
```

### final-run.sh

```bash
#!/bin/bash
set -eu
cp -a /source /repo
chown -R root:root /repo
rm /repo/.git
cd /repo
git init -q -b main
git -c user.name=probe -c user.email=probe@example.invalid add -A
git -c user.name=probe -c user.email=probe@example.invalid commit -qm snapshot
python3 - <<'FIXTURE'
from pathlib import Path
p=Path('tools/fake_screencast.py')
p.write_text(p.read_text().replace('register_object_with_closures2(', 'register_object('))
FIXTURE
python3 - <<'REPORT'
from pathlib import Path
p=Path('tools/upstream-report')
s=p.read_text().replace('    fields = result.stdout.split("\\0")\n    paths = []', '    fields = result.stdout.split("\\0")\n    if not fields[0]:\n        raise ReportError(f"git merge-tree returned no result tree: {result.stderr.strip()}")\n    paths = []')
assert s != p.read_text(), 'report probe patch did not match'
p.write_text(s)
REPORT
just --version > /results/just-version.txt
cargo --version > /results/cargo-version.txt
if command -v rustc >/dev/null; then echo unexpected-rustc; exit 1; fi
dpkg-query -W -f='${Package} ${Version}\n' python3 git jq procps xz-utils ca-certificates python3-gi gir1.2-gstreamer-1.0 gir1.2-gst-plugins-base-1.0 gstreamer1.0-tools gstreamer1.0-plugins-base gstreamer1.0-pipewire dbus > /results/package-versions.txt
gst-inspect-1.0 pipewiresrc > /results/pipewiresrc.txt
just --set test_cmd 'python3 /probe/run_tests.py --git-pilot' ci-test > /results/git-pilot.log 2>&1 || { tail -100 /results/git-pilot.log; exit 1; }
tail -8 /results/git-pilot.log
just --set test_cmd 'python3 /probe/run_tests.py --pilot' ci-test > /results/pilot.log 2>&1 || { tail -100 /results/pilot.log; exit 1; }
cat /results/pilot.log
just --set test_cmd 'python3 /probe/run_tests.py --lock-pilot' ci-test > /results/lock-pilot.log 2>&1 || { tail -100 /results/lock-pilot.log; exit 1; }
cat /results/lock-pilot.log
just --set test_cmd 'python3 /probe/run_tests.py' ci-test > /results/full.log 2>&1 || { tail -100 /results/full.log; exit 1; }
tail -12 /results/full.log
if just --set test_cmd 'python3 /probe/run_tests.py --fail-lifecycle' ci-test > /results/failure-control.log 2>&1; then
 echo 'unexpectedly passed deliberate lifecycle failure'; exit 1
fi
python3 - <<'CHECK'
import json
from pathlib import Path
info=json.loads(Path('/results/failure-control.json').read_text())
assert info['failures']==1 and info['errors']==0 and info['ran']==1, info
CHECK
echo 'deliberate lifecycle failure correctly rejected'
if just --set test_cmd 'python3 /probe/run_tests.py --unexpected-skip' ci-test > /results/dynamic-skip-control.log 2>&1; then
 echo 'unexpectedly passed dynamic skip control'; exit 1
fi
python3 - <<'CHECK'
import json
from pathlib import Path
info=json.loads(Path('/results/dynamic-skip-control.json').read_text())
assert info['ran']==1 and len(info['skipped'])==1 and info['failures']==0 and info['errors']==0, info
CHECK
echo 'unexpected dynamic skip correctly rejected'
```


## CI probe narrative moved from spec revision 1

The following preserves the earlier investigation and measurements; it is historical evidence, not a claim that implementation has landed.

The clean Ubuntu 24.04 pilot exposed two version requirements before the full
run: the distribution's just 1.21 cannot parse `set quiet`, and Cargo 1.75 misses
the parent `build-dir` isolation assertion. Use the existing setup-just action
with the validated 1.58.0 version and the stable Rust toolchain action to supply
Cargo 1.99.0. The clean probe uses official standalone Cargo and no rustc: metadata
needs no compilation, and the pilot proves the isolation assertions still execute.

The consumer fixture also calls `register_object_with_closures2`, introduced in
GLib 2.84; Ubuntu's GLib 2.80 does not provide it. Change the single fixture call
to the established binding `register_object` API, with no runtime fallback or
version branch. [GIO documents its binding and deprecation](https://docs.gtk.org/gio/method.DBusConnection.register_object_with_closures.html):
it remains available on the newer incident host but is deprecated since 2.84.
This is confined to the fake service, and requires the consumer startup/failure
suite to pass on both environments before shipping. Do not skip those cases to
accommodate the older runner.

The consumer checks for the `pipewiresrc` factory before applying its test pipeline
override. Therefore the tooling job also needs `gstreamer1.0-pipewire`, even though
it does not run a PipeWire server or consume real compositor frames. Assert
`gst-inspect-1.0 pipewiresrc` succeeds before discovery, alongside the skip guard.

The clean experiment uses Ubuntu 24.04 image digest
`sha256:534baea6a22c03a63003dbc8dbe78fe34bc0d7e595d9a9dc9834884ff530eb55`.
APT's confirmed runtime list is:

```text
python3 git jq procps dbus python3-gi
gir1.2-gstreamer-1.0 gir1.2-gst-plugins-base-1.0
gstreamer1.0-tools gstreamer1.0-plugins-base gstreamer1.0-pipewire
```

The standalone archive/bootstrap experiment also installs `ca-certificates` and
`xz-utils`. CI reuses its existing setup actions to supply just and Cargo; those
setup actions' transport requirements must be satisfied independently of Python
suite dependencies. The validated runtime versions are Python 3.12.3, PyGObject
3.48.2, GStreamer 1.24.2, PipeWire plugin 1.0.5, Git 2.43.0, jq 1.7.1, and procps
4.0.4. just is 1.58.0 and Cargo is 1.99.0; the Cargo release archive's SHA-256 was
checked against the official stable manifest.

Prototypes modify only disposable copies: the fake-service registration call,
gate's host-budget stub, and report validation described below. The CI guard is a scratch adapter, invoked through
an existing CI recipe's command override, not a shipped recipe. A six-case pilot
passed in **24.006 seconds with no skips**, covering driver crops, bounded VT
failure, consumer startup/summary, gate arguments/timing, and both Cargo-isolation
positive checks. All six consumer startup/failure cases also passed on the incident
host in **2.539 seconds** with the portable registration call.

Before media installation, the guard rejected eight required consumer/PyGObject
skips with exit 2. This negative control checked the reported skip identities;
failures before unittest startup do not supply dependency-guard evidence. Two
additional single-case controls confirmed nonzero verdicts for a deliberately
failing lifecycle test and an unexpected dynamic skip (the latter otherwise
reports unittest success).

The first complete clean run discovered **287 cases in 193.244 seconds**, with
exactly the five optional skips, zero errors, and one failure: the upstream-report
invalid-ref test. All lifecycle and consumer cases passed. Git 2.43 permits an
invalid-ref `merge-tree` invocation to exit 1 with no result tree; the current
report treats all exit-1 outputs as conflicts. Newer Git on the incident host
returns exit 128 for that input (Git 2.56), so the existing test had not exposed
this there. The Ubuntu diagnostic observed exit 1, empty stdout, and
`merge-tree: no-such-ref - not something we can merge` on stderr.

CI enablement must add an explicit protocol check in `tools/upstream-report`:
a return status of 0 or 1 still requires a nonempty result-tree field. Reject
missing trees with `ReportError` and retain stderr in the diagnostic. Preserve
status-1 directory conflicts with a valid tree and no conflicted-file entries.
This corrects error classification without a Git version branch or upgrade-only
workaround. Add deterministic empty-output regression coverage independent of
Git version, and retain the existing directory/conflict tests.

The report correction passed all **66 report tests in 6.010 seconds**, including
real directory conflicts and the older Git's invalid-ref result. A subsequent
full run again discovered 287 cases and exactly five optional skips, but failed
one locked-session TERM cleanup case in 193.597 seconds: the shell reported
`trap: unexpected EOF while looking for matching ')'` and exited 2 instead of 143.
That same case passed in the previous run. This is an unresolved intermittent
cleanup failure, not a missing dependency; do not skip it, widen its bound, or
retry until green and call it resolved. The guarded front door correctly failed
on this real lifecycle failure.

Implementation must diagnose this intermittent TERM/trap failure on Ubuntu before
enabling the new CI job and fast omission. Preserve the lifecycle assertion, exit-status and
cleanup guarantees, and production waits. A passing rerun alone does not discharge
this prerequisite; record the reproduction and disposition in the implementation
plan and evidence. Three focused repetitions passed in 12.078 seconds, followed
by another complete run of 287 cases in 193.802 seconds: exactly five optional
skips, zero errors, and the same one TERM/trap failure. The suite has **not** passed
in this environment. Its runtime dependencies and full coverage are established;
the remaining failure is an explicit shipping blocker, owned by the agent after
spec review, rather than evidence that CI is ready.


## Route recount for spec revision 2

Non-merge commits reachable from `materials-26.04` at `ca634f24`, with committer timestamps ≥ 2026-09-28T00:00:00Z. Route full first, then the current justfile docs-only allowlist, then fast. Changed paths use `git diff-tree --root --no-commit-id --name-only --no-renames -r -z <commit>`, strict UTF-8. Both rename endpoints and deleted paths count. Merge commits are excluded because their empty diff-tree output cannot reconstruct a pre-commit index; this is a commit-history proxy, not hook-run evidence. Daily counts below use America/New_York committer dates.

The recount reproduces the human review totals: 56 full / 69 fast under broad patterns, 34 / 91 under the proposed lifecycle subjects. Adding the directly sourced `glass-optic-smoke-lib.sh` gives 36 / 89. All three classify 128 other commits as docs-only. The new coordinator/mode helper `tools/tooling_tests.py` has no historical commits and does not affect these totals.

| Patterns | Full | Fast | Full share |
| --- | ---: | ---: | ---: |
| Broad | 56 | 69 | 44.8% |
| Review subjects | 34 | 91 | 27.2% |
| Subjects plus sourced helper | 36 | 89 | 28.8% |

| Local date | Broad full / fast | Review full / fast | With helper full / fast | Docs only |
| --- | ---: | ---: | ---: | ---: |
| 2026-09-27 | 0 / 4 | 0 / 4 | 0 / 4 | 1 |
| 2026-09-28 | 1 / 7 | 0 / 8 | 0 / 8 | 7 |
| 2026-09-29 | 0 / 9 | 0 / 9 | 0 / 9 | 3 |
| 2026-09-30 | 8 / 6 | 2 / 12 | 2 / 12 | 19 |
| 2026-10-01 | 15 / 16 | 10 / 21 | 10 / 21 | 38 |
| 2026-10-02 | 25 / 14 | 20 / 19 | 20 / 19 | 35 |
| 2026-10-03 | 6 / 13 | 2 / 17 | 4 / 15 | 20 |
| 2026-10-04 | 1 / 0 | 0 / 1 | 0 / 1 | 5 |

The driver reads the helper at `docs/materials/scripts/optic-settling-smoke.sh:51`, immediately before sourcing `vt-lib.sh`. The helper-only changes are `146d56b3` and `264c4155` (both October 3 locally). Including this actual dependency is a bounded correction to the suggested list, not a recursive import graph. The read-coverage regression must catch comparable future drift.

Reproduction (run in a checkout at the stated main-branch revision):

```python
import collections, datetime, fnmatch, subprocess
from zoneinfo import ZoneInfo

def git(*args):
    return subprocess.check_output(["git", *args])

broad = ["tools/*", "docs/materials/scripts/*", ".githooks/*",
         "justfile", ".github/workflows/*"]
narrow = ["tools/optic_settling.py", "tools/screencast_consumer.py",
          "tools/fake_screencast.py", "tools/test_optic_settling.py",
          "tools/test_screencast_consumer.py", "tools/test_vt_lib.py",
          "docs/materials/scripts/vt-lib.sh",
          "docs/materials/scripts/optic-settling-smoke.sh",
          "docs/materials/scripts/*-client.c", ".githooks/*",
          "justfile", ".github/workflows/*"]
patterns = {"broad": broad, "narrow": narrow,
            "with_helper": narrow + ["docs/materials/scripts/glass-optic-smoke-lib.sh"]}
docs = subprocess.check_output(["just", "--evaluate", "docs_paths"], text=True).split()
rows = []
cutoff = datetime.datetime(2026, 9, 28, tzinfo=datetime.timezone.utc).timestamp()
for line in git("log", "ca634f24", "--no-merges", "--since=2026-09-27T00:00:00Z",
                "--format=%H %ct").decode().splitlines():
    sha, timestamp = line.split()
    if int(timestamp) < cutoff:
        continue
    paths = [p.decode("utf-8") for p in git("diff-tree", "--root", "--no-commit-id",
             "--name-only", "--no-renames", "-r", "-z", sha).split(b"\0") if p]
    row = {"sha": sha, "day": datetime.datetime.fromtimestamp(
           int(timestamp), ZoneInfo("America/New_York")).date().isoformat()}
    for name, globs in patterns.items():
        full = not paths or any(fnmatch.fnmatchcase(p, g) for p in paths for g in globs)
        only_docs = paths and all(any(fnmatch.fnmatchcase(p, g) for g in docs) for p in paths)
        row[name] = "full" if full else "docs" if only_docs else "fast"
    rows.append(row)
for name in patterns:
    print(name, dict(collections.Counter(r[name] for r in rows)))
    for day in sorted({r["day"] for r in rows}):
        print(day, dict(collections.Counter(r[name] for r in rows if r["day"] == day)))
print("helper-only", [r["sha"][:8] for r in rows if r["narrow"] != r["with_helper"]])
```

Code-commit classification receipts (short hashes; docs-only commits omitted):

| Commit | Local date | Broad | Review subjects | With helper |
| --- | --- | --- | --- | --- |
| `97874e9b` | 2026-10-04 | full | fast | fast |
| `146d56b3` | 2026-10-03 | full | fast | full |
| `264c4155` | 2026-10-03 | full | fast | full |
| `cc5480f3` | 2026-10-03 | full | fast | fast |
| `81165c50` | 2026-10-03 | fast | fast | fast |
| `560a2444` | 2026-10-03 | fast | fast | fast |
| `afb48478` | 2026-10-03 | fast | fast | fast |
| `292dd00f` | 2026-10-03 | fast | fast | fast |
| `88a08f5f` | 2026-10-03 | fast | fast | fast |
| `0266f81c` | 2026-10-03 | full | fast | fast |
| `a34495d2` | 2026-10-03 | fast | fast | fast |
| `9ba519c7` | 2026-10-03 | fast | fast | fast |
| `df61f388` | 2026-10-03 | fast | fast | fast |
| `a38489e5` | 2026-10-03 | fast | fast | fast |
| `b2418acf` | 2026-10-03 | fast | fast | fast |
| `b9ebfaea` | 2026-10-03 | fast | fast | fast |
| `776bd8a4` | 2026-10-03 | fast | fast | fast |
| `3125163e` | 2026-10-03 | full | full | full |
| `fdccb92c` | 2026-10-03 | fast | fast | fast |
| `e6d3b3f8` | 2026-10-03 | full | full | full |
| `1b997476` | 2026-10-02 | full | full | full |
| `6438c1b1` | 2026-10-02 | full | full | full |
| `09eb1f65` | 2026-10-02 | full | full | full |
| `1135acb5` | 2026-10-02 | fast | fast | fast |
| `61f4869d` | 2026-10-02 | full | full | full |
| `e6bedb03` | 2026-10-02 | full | full | full |
| `c1c87845` | 2026-10-02 | full | full | full |
| `507fda30` | 2026-10-02 | full | full | full |
| `f2564c99` | 2026-10-02 | fast | fast | fast |
| `9ffc3701` | 2026-10-02 | full | full | full |
| `33ddada6` | 2026-10-02 | full | full | full |
| `b5095cd5` | 2026-10-02 | full | full | full |
| `65802e90` | 2026-10-02 | full | fast | fast |
| `462dc8dd` | 2026-10-02 | full | fast | fast |
| `f9864acf` | 2026-10-02 | full | full | full |
| `c8f249df` | 2026-10-02 | full | full | full |
| `f05b8c92` | 2026-10-02 | fast | fast | fast |
| `4a586bc8` | 2026-10-02 | fast | fast | fast |
| `7c2e18ec` | 2026-10-02 | fast | fast | fast |
| `3755cb19` | 2026-10-02 | full | full | full |
| `3127fa07` | 2026-10-02 | full | full | full |
| `76aaba02` | 2026-10-02 | full | full | full |
| `4b048e15` | 2026-10-02 | full | full | full |
| `757adadd` | 2026-10-02 | full | full | full |
| `53c14959` | 2026-10-02 | full | full | full |
| `28075cd9` | 2026-10-02 | full | full | full |
| `b162a2b8` | 2026-10-02 | fast | fast | fast |
| `6366b268` | 2026-10-02 | fast | fast | fast |
| `fef578cc` | 2026-10-02 | fast | fast | fast |
| `c2f5b9a3` | 2026-10-02 | fast | fast | fast |
| `7c142439` | 2026-10-02 | full | fast | fast |
| `93e3a86e` | 2026-10-02 | full | full | full |
| `c8a72507` | 2026-10-02 | fast | fast | fast |
| `4148a022` | 2026-10-02 | fast | fast | fast |
| `310b4e30` | 2026-10-02 | fast | fast | fast |
| `70ef9972` | 2026-10-02 | full | fast | fast |
| `caf8c8c6` | 2026-10-02 | full | fast | fast |
| `5749b622` | 2026-10-02 | fast | fast | fast |
| `56d198a0` | 2026-10-02 | fast | fast | fast |
| `2f3b5b6a` | 2026-10-01 | full | full | full |
| `7e80e25f` | 2026-10-01 | full | fast | fast |
| `0896772a` | 2026-10-01 | full | full | full |
| `641d07a3` | 2026-10-01 | fast | fast | fast |
| `c93180be` | 2026-10-01 | full | full | full |
| `7d240563` | 2026-10-01 | fast | fast | fast |
| `2b25edb6` | 2026-10-01 | fast | fast | fast |
| `39a6a2f8` | 2026-10-01 | full | fast | fast |
| `8e736f2f` | 2026-10-01 | full | fast | fast |
| `3dc0cd7f` | 2026-10-01 | full | fast | fast |
| `81ec2a1f` | 2026-10-01 | fast | fast | fast |
| `d9631ff8` | 2026-10-01 | fast | fast | fast |
| `6efa5b5d` | 2026-10-01 | fast | fast | fast |
| `32ef13d4` | 2026-10-01 | fast | fast | fast |
| `83ca476a` | 2026-10-01 | fast | fast | fast |
| `1a8c5a94` | 2026-10-01 | fast | fast | fast |
| `1070d815` | 2026-10-01 | fast | fast | fast |
| `689770db` | 2026-10-01 | fast | fast | fast |
| `cc563ead` | 2026-10-01 | fast | fast | fast |
| `a3e27218` | 2026-10-01 | fast | fast | fast |
| `f66704b4` | 2026-10-01 | fast | fast | fast |
| `a9f912a5` | 2026-10-01 | fast | fast | fast |
| `645b9ef3` | 2026-10-01 | full | full | full |
| `067fc5e3` | 2026-10-01 | full | full | full |
| `54d93893` | 2026-10-01 | full | full | full |
| `84ea236a` | 2026-10-01 | full | full | full |
| `21f73a73` | 2026-10-01 | full | fast | fast |
| `7f5746b6` | 2026-10-01 | fast | fast | fast |
| `540a0240` | 2026-10-01 | full | full | full |
| `8695c824` | 2026-10-01 | full | full | full |
| `4721e8de` | 2026-10-01 | full | full | full |
| `d1cd7685` | 2026-09-30 | full | fast | fast |
| `e4f3337a` | 2026-09-30 | full | full | full |
| `02fc43c0` | 2026-09-30 | fast | fast | fast |
| `63702813` | 2026-09-30 | fast | fast | fast |
| `70117d11` | 2026-09-30 | fast | fast | fast |
| `797f0b17` | 2026-09-30 | fast | fast | fast |
| `66139756` | 2026-09-30 | fast | fast | fast |
| `870c4694` | 2026-09-30 | full | fast | fast |
| `c94aa4ec` | 2026-09-30 | full | fast | fast |
| `4117a435` | 2026-09-30 | full | fast | fast |
| `f7124e8c` | 2026-09-30 | fast | fast | fast |
| `a7073ac0` | 2026-09-30 | full | fast | fast |
| `6829d4c9` | 2026-09-30 | full | fast | fast |
| `491f4fff` | 2026-09-30 | full | full | full |
| `e42f35bf` | 2026-09-29 | fast | fast | fast |
| `b908fc48` | 2026-09-29 | fast | fast | fast |
| `1767db05` | 2026-09-29 | fast | fast | fast |
| `a19bf874` | 2026-09-29 | fast | fast | fast |
| `8e3fdcb6` | 2026-09-29 | fast | fast | fast |
| `eb9b246c` | 2026-09-29 | fast | fast | fast |
| `f84f125f` | 2026-09-29 | fast | fast | fast |
| `a1eedcbf` | 2026-09-29 | fast | fast | fast |
| `a6cbb8ab` | 2026-09-29 | fast | fast | fast |
| `50cddb5e` | 2026-09-28 | fast | fast | fast |
| `5ed1a44a` | 2026-09-28 | fast | fast | fast |
| `68e2c5eb` | 2026-09-28 | full | fast | fast |
| `88635932` | 2026-09-28 | fast | fast | fast |
| `0f16d945` | 2026-09-28 | fast | fast | fast |
| `2584ecf2` | 2026-09-28 | fast | fast | fast |
| `3bcb80b0` | 2026-09-28 | fast | fast | fast |
| `982ae897` | 2026-09-28 | fast | fast | fast |
| `6ee28697` | 2026-09-27 | fast | fast | fast |
| `2101a871` | 2026-09-27 | fast | fast | fast |
| `721b8df7` | 2026-09-27 | fast | fast | fast |
| `5c4961d7` | 2026-09-27 | fast | fast | fast |

No additional hook, container, or parallel timing experiment was run for this revision. The existing 46.755-second method comprises four independent `subTest` scenarios (`tools/test_optic_settling.py:1041–1054`); these must become separately schedulable native test methods without shortening waits. Full-route speed and read-observation mechanics remain implementation verification requirements.


## Round 3 plan inputs and focused TERM diagnosis

Human review timings (reported by the owner; independently timed warm, isolated cases): 32 slow methods total approximately 204 s; the four-scenario screencast method 45.9 s; next longest DRM/TERM method 19.2 s; four more driver cases 12–17 s each; VT class approximately 23.1 s. At 8 children the owner projects a 36–40 s whole hook; use 10 children by default to seek 35 s headroom, with NEXTEST_TEST_THREADS as a strict positive integer cap. These are scheduling inputs, not measured parallel outcomes.

The runtime-read proposal was replaced by source/AST checks. No tracing dependency, privilege, or runtime observation enters the plan.

The installed TERM/EXIT actions are literal (`exit 143`, `on_exit`); no runtime assembled trap string was found. The lock marker is emitted before `lock_start()` returns, immediately before `stim()` computes `end=$(( $(mono) + $(awk …) ))`. This creates a plausible signal window inside the older Bash parser. The full-case attribution is an inference from that call order; the minimal probe proves the mechanism.

A [GNU Bash maintainer response](https://lists.gnu.org/archive/html/bug-bash/2024-02/msg00029.html) confirms a related older-Bash trap/command-substitution parser defect for SIGCHLD. This is supporting context, not proof that the driver follows exactly that report. Local Ubuntu/host experiments below demonstrate the TERM mechanism directly.

The first diagnostic pilot signaled from the first `mono()` call (the separate start assignment), which exited 143 cleanly and did not exercise the end expression; its expected-failure assertion failed. The corrected pilot injects only at the second `mono()` call, using a private marker shared across command substitutions. It exercises the actual extracted function body unchanged, then a disposable string-only split variant. No repository implementation was changed.

Commands through the test front door: `just --set one_cmd "python3 /tmp/material-stim-diagnose.py" test-one 1`, then `test-one 10`. Both corrected calls exited 0 because all expected original/split verdicts matched. Original Ubuntu: exit 2 and trap EOF in all ten repetitions; original host: exit 143. Split: exit 143, empty stderr and cleanup marker on both versions in all ten. No media dependency install or full suite was run here; Docker used the existing ubuntu:24.04 base with Bash 5.2.21, while the host uses 5.3.20.

Reproduction script (temporary diagnostic; extract from this fenced block to a scratch file, then invoke via `just test-one` as above):

```python
from pathlib import Path
import json, subprocess, sys
root = Path.cwd()
source = (root / 'docs/materials/scripts/optic-settling-smoke.sh').read_text()
body = source.split('stim() {', 1)[1].split('\n}\n', 1)[0]
old = '    end=$(( $(mono) + $(awk -v s="$effect" \'BEGIN { printf "%d", s * 1e9 }\') ))'
assert old in body
fixed = body.replace(old, '    end=$(mono)\n    local effect_ns\n    effect_ns=$(awk -v s="$effect" \'BEGIN { printf "%d", s * 1e9 }\')\n    end=$((end + effect_ns))')
prefix = '''set -eu
on_exit() { local rc=$?; trap - EXIT; trap : INT TERM HUP; rm -rf "$CASE_DIR"; printf 'cleanup\\n'; exit "$rc"; }
trap 'exit 143' TERM
trap on_exit EXIT
CASE_DIR=$(mktemp -d)
mono() { if [ -e "$CASE_DIR/first" ]; then kill -TERM $$; else : > "$CASE_DIR/first"; fi; printf 100; }
'''
repeats = int(sys.argv[1]) if len(sys.argv)>1 else 1
receipts=[]
for iteration in range(repeats):
    for variant, contents in [('original', body), ('split', fixed)]:
        script = prefix + 'stim() {' + contents + '\n}\nstim lock 2 true\nprintf survived\n'
        for environment in ('host','ubuntu'):
            argv=['bash','-c',script] if environment=='host' else ['docker','run','--rm','ubuntu:24.04','bash','-c',script]
            result=subprocess.run(argv,capture_output=True,text=True,timeout=15)
            receipt=dict(iteration=iteration, variant=variant, environment=environment,
                         exit=result.returncode,stdout=result.stdout,stderr=result.stderr)
            receipts.append(receipt)
            expected=2 if environment=='ubuntu' and variant=='original' else 143
            assert result.returncode==expected,receipt
            if variant=='split': assert not result.stderr,receipt
            if environment=='ubuntu' and variant=='original':
                assert 'unexpected EOF' in result.stderr,receipt
    print(f'Iteration {iteration+1}: original Ubuntu failed as expected; split exited 143 cleanly on both versions.', flush=True)
Path('/tmp/material-stim-diagnose-results.json').write_text(json.dumps(dict(repeats=repeats,
      source='docs/materials/scripts/optic-settling-smoke.sh:545-550',
      original_body=body,split_body=fixed,receipts=receipts),indent=2))
```

Complete ten-iteration receipts:

```json
{
  "repeats": 10,
  "source": "docs/materials/scripts/optic-settling-smoke.sh:545-550",
  "original_body": "\n    local label=$1 effect=$2 start end; shift 2\n    start=$(mono)\n    \"$@\"\n    end=$(( $(mono) + $(awk -v s=\"$effect\" 'BEGIN { printf \"%d\", s * 1e9 }') ))\n    printf '%s\\t%s\\t%s\\n' \"$label\" \"$start\" \"$end\" >> \"$CASE_DIR/journal.tsv\"",
  "split_body": "\n    local label=$1 effect=$2 start end; shift 2\n    start=$(mono)\n    \"$@\"\n    end=$(mono)\n    local effect_ns\n    effect_ns=$(awk -v s=\"$effect\" 'BEGIN { printf \"%d\", s * 1e9 }')\n    end=$((end + effect_ns))\n    printf '%s\\t%s\\t%s\\n' \"$label\" \"$start\" \"$end\" >> \"$CASE_DIR/journal.tsv\"",
  "receipts": [
    {
      "iteration": 0,
      "variant": "original",
      "environment": "host",
      "exit": 143,
      "stdout": "cleanup\n",
      "stderr": ""
    },
    {
      "iteration": 0,
      "variant": "original",
      "environment": "ubuntu",
      "exit": 2,
      "stdout": "cleanup\n",
      "stderr": "environment: trap: line 2: unexpected EOF while looking for matching `)'\n"
    },
    {
      "iteration": 0,
      "variant": "split",
      "environment": "host",
      "exit": 143,
      "stdout": "cleanup\n",
      "stderr": ""
    },
    {
      "iteration": 0,
      "variant": "split",
      "environment": "ubuntu",
      "exit": 143,
      "stdout": "cleanup\n",
      "stderr": ""
    },
    {
      "iteration": 1,
      "variant": "original",
      "environment": "host",
      "exit": 143,
      "stdout": "cleanup\n",
      "stderr": ""
    },
    {
      "iteration": 1,
      "variant": "original",
      "environment": "ubuntu",
      "exit": 2,
      "stdout": "cleanup\n",
      "stderr": "environment: trap: line 2: unexpected EOF while looking for matching `)'\n"
    },
    {
      "iteration": 1,
      "variant": "split",
      "environment": "host",
      "exit": 143,
      "stdout": "cleanup\n",
      "stderr": ""
    },
    {
      "iteration": 1,
      "variant": "split",
      "environment": "ubuntu",
      "exit": 143,
      "stdout": "cleanup\n",
      "stderr": ""
    },
    {
      "iteration": 2,
      "variant": "original",
      "environment": "host",
      "exit": 143,
      "stdout": "cleanup\n",
      "stderr": ""
    },
    {
      "iteration": 2,
      "variant": "original",
      "environment": "ubuntu",
      "exit": 2,
      "stdout": "cleanup\n",
      "stderr": "environment: trap: line 2: unexpected EOF while looking for matching `)'\n"
    },
    {
      "iteration": 2,
      "variant": "split",
      "environment": "host",
      "exit": 143,
      "stdout": "cleanup\n",
      "stderr": ""
    },
    {
      "iteration": 2,
      "variant": "split",
      "environment": "ubuntu",
      "exit": 143,
      "stdout": "cleanup\n",
      "stderr": ""
    },
    {
      "iteration": 3,
      "variant": "original",
      "environment": "host",
      "exit": 143,
      "stdout": "cleanup\n",
      "stderr": ""
    },
    {
      "iteration": 3,
      "variant": "original",
      "environment": "ubuntu",
      "exit": 2,
      "stdout": "cleanup\n",
      "stderr": "environment: trap: line 2: unexpected EOF while looking for matching `)'\n"
    },
    {
      "iteration": 3,
      "variant": "split",
      "environment": "host",
      "exit": 143,
      "stdout": "cleanup\n",
      "stderr": ""
    },
    {
      "iteration": 3,
      "variant": "split",
      "environment": "ubuntu",
      "exit": 143,
      "stdout": "cleanup\n",
      "stderr": ""
    },
    {
      "iteration": 4,
      "variant": "original",
      "environment": "host",
      "exit": 143,
      "stdout": "cleanup\n",
      "stderr": ""
    },
    {
      "iteration": 4,
      "variant": "original",
      "environment": "ubuntu",
      "exit": 2,
      "stdout": "cleanup\n",
      "stderr": "environment: trap: line 2: unexpected EOF while looking for matching `)'\n"
    },
    {
      "iteration": 4,
      "variant": "split",
      "environment": "host",
      "exit": 143,
      "stdout": "cleanup\n",
      "stderr": ""
    },
    {
      "iteration": 4,
      "variant": "split",
      "environment": "ubuntu",
      "exit": 143,
      "stdout": "cleanup\n",
      "stderr": ""
    },
    {
      "iteration": 5,
      "variant": "original",
      "environment": "host",
      "exit": 143,
      "stdout": "cleanup\n",
      "stderr": ""
    },
    {
      "iteration": 5,
      "variant": "original",
      "environment": "ubuntu",
      "exit": 2,
      "stdout": "cleanup\n",
      "stderr": "environment: trap: line 2: unexpected EOF while looking for matching `)'\n"
    },
    {
      "iteration": 5,
      "variant": "split",
      "environment": "host",
      "exit": 143,
      "stdout": "cleanup\n",
      "stderr": ""
    },
    {
      "iteration": 5,
      "variant": "split",
      "environment": "ubuntu",
      "exit": 143,
      "stdout": "cleanup\n",
      "stderr": ""
    },
    {
      "iteration": 6,
      "variant": "original",
      "environment": "host",
      "exit": 143,
      "stdout": "cleanup\n",
      "stderr": ""
    },
    {
      "iteration": 6,
      "variant": "original",
      "environment": "ubuntu",
      "exit": 2,
      "stdout": "cleanup\n",
      "stderr": "environment: trap: line 2: unexpected EOF while looking for matching `)'\n"
    },
    {
      "iteration": 6,
      "variant": "split",
      "environment": "host",
      "exit": 143,
      "stdout": "cleanup\n",
      "stderr": ""
    },
    {
      "iteration": 6,
      "variant": "split",
      "environment": "ubuntu",
      "exit": 143,
      "stdout": "cleanup\n",
      "stderr": ""
    },
    {
      "iteration": 7,
      "variant": "original",
      "environment": "host",
      "exit": 143,
      "stdout": "cleanup\n",
      "stderr": ""
    },
    {
      "iteration": 7,
      "variant": "original",
      "environment": "ubuntu",
      "exit": 2,
      "stdout": "cleanup\n",
      "stderr": "environment: trap: line 2: unexpected EOF while looking for matching `)'\n"
    },
    {
      "iteration": 7,
      "variant": "split",
      "environment": "host",
      "exit": 143,
      "stdout": "cleanup\n",
      "stderr": ""
    },
    {
      "iteration": 7,
      "variant": "split",
      "environment": "ubuntu",
      "exit": 143,
      "stdout": "cleanup\n",
      "stderr": ""
    },
    {
      "iteration": 8,
      "variant": "original",
      "environment": "host",
      "exit": 143,
      "stdout": "cleanup\n",
      "stderr": ""
    },
    {
      "iteration": 8,
      "variant": "original",
      "environment": "ubuntu",
      "exit": 2,
      "stdout": "cleanup\n",
      "stderr": "environment: trap: line 2: unexpected EOF while looking for matching `)'\n"
    },
    {
      "iteration": 8,
      "variant": "split",
      "environment": "host",
      "exit": 143,
      "stdout": "cleanup\n",
      "stderr": ""
    },
    {
      "iteration": 8,
      "variant": "split",
      "environment": "ubuntu",
      "exit": 143,
      "stdout": "cleanup\n",
      "stderr": ""
    },
    {
      "iteration": 9,
      "variant": "original",
      "environment": "host",
      "exit": 143,
      "stdout": "cleanup\n",
      "stderr": ""
    },
    {
      "iteration": 9,
      "variant": "original",
      "environment": "ubuntu",
      "exit": 2,
      "stdout": "cleanup\n",
      "stderr": "environment: trap: line 2: unexpected EOF while looking for matching `)'\n"
    },
    {
      "iteration": 9,
      "variant": "split",
      "environment": "host",
      "exit": 143,
      "stdout": "cleanup\n",
      "stderr": ""
    },
    {
      "iteration": 9,
      "variant": "split",
      "environment": "ubuntu",
      "exit": 143,
      "stdout": "cleanup\n",
      "stderr": ""
    }
  ]
}
```

These do not establish that the production lifecycle failure is fixed. The plan must test actual driver cleanup, preserve 143 and BOUND_S=5, and run isolated/concurrent acceptance on the incident host and clean Ubuntu. Docker invocations were foreground, --rm, with timeout 15 s per probe; no containers or task subprocesses remain.
