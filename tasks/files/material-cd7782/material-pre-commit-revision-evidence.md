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
