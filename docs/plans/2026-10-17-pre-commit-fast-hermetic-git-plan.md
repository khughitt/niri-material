# Hermetic git for tooling fixtures: implementation plan

Status: Approved 2026-10-17 (plan review round 3, codex/gpt-6-astra; owner). Execution: native (inline), then one fresh final reviewer.

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Tooling tests run git without the host's `GIT_*` environment or its global and system git config, which takes the global `identity-check` hook's ~19 s back out of the fast pre-commit route.

**Architecture:** One helper, `tools/hermetic_git.isolate(environ)`, scrubs `GIT_*` and pins `GIT_CONFIG_GLOBAL=/dev/null` and `GIT_CONFIG_NOSYSTEM=1`. The tooling coordinator calls it once at the top of `main()`, which covers the parent before discovery and every worker, since a worker is the same entry point in a new process. Every test module that runs git calls it at import, so a focused run stays hermetic. Verification runs on the branch with `--dry-run`, then for the record after the merge, from the registered checkout.

**Tech Stack:** Python 3 `unittest`, git ≥ 2.32 (`GIT_CONFIG_GLOBAL`), the `tools/tooling_tests.py` coordinator, `just`, `tt-latency`, `tasks`.

**Spec:** `docs/specs/2026-10-17-pre-commit-fast-hermetic-git-design.md`

## Global Constraints

- `isolate` deletes every environment key starting with `GIT_`, then sets `GIT_CONFIG_GLOBAL` to `os.devnull` and `GIT_CONFIG_NOSYSTEM` to `1`; nothing else changes.
- A test that needs a global git config of its own sets `GIT_CONFIG_GLOBAL` in its own subprocess environment (as `test_identity_check` does today); `isolate` never reads or preserves one.
- Config-declared hooks exist only on Git ≥ 2.54. CI installs Ubuntu's packaged Git, and CI's tooling job refuses unexpected skips. No new test may skip.
- Imports follow the tools convention: `from tools.hermetic_git import isolate`. Runs start from the repository root, which puts the root on `sys.path`.
- Tests run through the front door only: `just --set one_cmd 'env NIRI_TOOLING_FAST=0 python3 -m unittest' test-one <ids>` while editing, `just --set fast_cmd 'env NIRI_TOOLING_FAST=0 python3 -m tools.tooling_tests --full' test-fast` before each commit. Never call `python3 -m unittest` or the coordinator bare, except inside a test's own subprocess.
- Branch acceptance uses `tt-latency verify … --dry-run` only. The single recording verify runs after the merge, from the registered checkout, with no live claim or newer record in any worktree.
- Thresholds (spec §4): `test_upstream_report` ≤ 9 s and fast tooling ≤ 26 s with the global hook installed; the median of three staged fast-route hook runs ≤ 40 s on the idle host.
- Each step child is started before its work and closed in the commit that lands it: `tasks start <child>` … `tasks done <child> "<what landed>"`. Under `[git] commit = true`, `tasks done` defers its own commit while changes are staged, so each closing step stages the code first, then runs `tasks done`, then commits code and record together.

## Review Focus

1. **Config hooks on older Git.** On CI's Ubuntu Git a config-declared hook never fires. The suppression test must pin "inert control" there, not skip and not fail. Task 1 pins both branches through the version gate.
2. **A module that imports `isolate` but calls it late.** If a module runs git at import before the call, or a later import resets the environment, a focused run leaks. Task 4's inventory probe imports each module in a fresh process with a decoy environment and checks the environment after import.
3. **Tests that legitimately carry their own global config.** `test_identity_check` builds `GIT_CONFIG_GLOBAL` into its subprocess environment from `os.environ`; after `isolate` that base already holds `/dev/null`, and its own override must still win. Task 4 runs that module in full after the change.
4. **The coordinator's own environment.** Workers inherit the parent's environment through `dict(os.environ, …)`. Isolation must hold even for a synthetic module that never calls `isolate` (Task 2's fixture), in both `--fast` and `--full`.
5. **Agreement on a decoy.** The fixture and `optic-settling-smoke.sh` both inherit `GIT_DIR` and would agree on a decoy HEAD. Task 3 compares the recorded `source_commit` with a HEAD read under a separately isolated environment, and asserts first that the decoy's HEAD differs.

---

### Task 1: The `isolate` helper

**Files:**
- Create: `tools/hermetic_git.py`
- Create: `tools/test_hermetic_git.py`

**Interfaces:**
- Produces: `tools.hermetic_git.isolate(environ: MutableMapping[str, str]) -> None`, idempotent, mutating `environ` in place.
- Produces (in `tools/test_hermetic_git.py`, for Tasks 3 and 4): `ROOT: pathlib.Path` (repository root) and `git(cwd, env, *args) -> str` (stripped stdout, `check=True`).

Start: `tasks start material-5e6dd7`.

- [ ] **Step 1: Write the failing tests**

`tools/test_hermetic_git.py`:

```python
"""isolate() keeps the host's git environment and configuration out of tooling tests."""
import os
import pathlib
import re
import subprocess
import tempfile
import unittest

from tools.hermetic_git import isolate

isolate(os.environ)

ROOT = pathlib.Path(__file__).resolve().parents[1]
IDENTITY = ('-c', 'user.name=Fixture', '-c', 'user.email=fixture@example.invalid')


def git(cwd, env, *args):
    return subprocess.run(['git', '-C', str(cwd), *args], env=env, check=True,
                          capture_output=True, text=True).stdout.strip()


def git_version():
    text = git(ROOT, os.environ, '--version')
    return tuple(int(part) for part in re.match(r'git version (\d+)\.(\d+)', text).groups())


class IsolateTests(unittest.TestCase):
    def test_scrubs_git_keys_and_pins_config_sources(self):
        environ = {'PATH': '/bin', 'GIT_DIR': 'd', 'GIT_WORK_TREE': 'w', 'GIT_INDEX_FILE': 'i',
                   'GIT_CONFIG_GLOBAL': 'g', 'GIT_CONFIG_COUNT': '1'}
        isolate(environ)
        expected = {'PATH': '/bin', 'GIT_CONFIG_GLOBAL': os.devnull, 'GIT_CONFIG_NOSYSTEM': '1'}
        self.assertEqual(environ, expected)
        isolate(environ)
        self.assertEqual(environ, expected)


class SuppressionTests(unittest.TestCase):
    """Each host hook form fires in a control commit, then not after isolate."""

    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.base = pathlib.Path(temporary.name)
        self.marker = self.base / 'marker'
        self.repo = self.base / 'repo'
        self.clean = dict(os.environ)
        isolate(self.clean)
        git(self.base, self.clean, 'init', '-q', '-b', 'main', str(self.repo))

    def host(self, config):
        path = self.base / 'host-gitconfig'
        path.write_text(config)
        return dict(self.clean, GIT_CONFIG_GLOBAL=str(path))

    def commit(self, env, message):
        (self.repo / 'f').write_text(message)
        git(self.repo, env, 'add', 'f')
        git(self.repo, env, *IDENTITY, 'commit', '-q', '-m', message)

    def control_then_isolated(self, env):
        self.commit(env, 'control')
        fired = self.marker.exists()
        self.marker.unlink(missing_ok=True)
        isolate(env)
        self.commit(env, 'isolated')
        return fired, self.marker.exists()

    def test_core_hooks_path_in_host_config(self):
        hooks = self.base / 'hooks'
        hooks.mkdir()
        script = hooks / 'pre-commit'
        script.write_text(f"#!/bin/sh\ntouch '{self.marker}'\n")
        script.chmod(0o755)
        fired, after = self.control_then_isolated(self.host(f'[core]\n\thooksPath = {hooks}\n'))
        self.assertTrue(fired, 'control commit did not run the core.hooksPath hook')
        self.assertFalse(after, 'the host core.hooksPath hook ran after isolate')

    def test_config_declared_hook_in_host_config(self):
        env = self.host(f"[hook \"probe\"]\n\tevent = pre-commit\n\tcommand = touch '{self.marker}'\n")
        fired, after = self.control_then_isolated(env)
        if git_version() < (2, 54):
            # Config hooks arrived in Git 2.54; on older Git (CI's packaged one) the form is inert.
            self.assertFalse(fired, 'a config hook fired on Git older than 2.54')
        else:
            self.assertTrue(fired, 'control commit did not run the config-declared hook')
        self.assertFalse(after, 'the host config hook ran after isolate')


if __name__ == '__main__':
    unittest.main()
```

- [ ] **Step 2: Run it to make sure it fails**

Run: `just --set one_cmd 'env NIRI_TOOLING_FAST=0 python3 -m unittest' test-one tools.test_hermetic_git`
Expected: ERROR, `ModuleNotFoundError: No module named 'tools.hermetic_git'`.

- [ ] **Step 3: Write the helper**

`tools/hermetic_git.py`:

```python
"""Run git in tooling tests without the host's git environment or configuration.

Git hooks export GIT_DIR, GIT_WORK_TREE and GIT_INDEX_FILE, which override `git -C`, so
a fixture run from a hook would write to the repository being committed to. A host's
global or system config can declare hooks (config hooks, core.hooksPath) and settings
that every fixture commit would then run; CI has neither. Design:
docs/specs/2026-10-17-pre-commit-fast-hermetic-git-design.md.
"""
import os
from collections.abc import MutableMapping


def isolate(environ: MutableMapping[str, str]) -> None:
    """Drop every GIT_* variable, then read no global or system git config."""
    for name in [key for key in environ if key.startswith('GIT_')]:
        del environ[name]
    environ['GIT_CONFIG_GLOBAL'] = os.devnull
    environ['GIT_CONFIG_NOSYSTEM'] = '1'
```

- [ ] **Step 4: Run it to make sure it passes**

Run: `just --set one_cmd 'env NIRI_TOOLING_FAST=0 python3 -m unittest' test-one tools.test_hermetic_git`
Expected: `OK`, 3 tests. On this host (Git 2.56) the config-hook control fires; that is the branch under test here.

- [ ] **Step 5: Fast check, close, commit**

```bash
just --set fast_cmd 'env NIRI_TOOLING_FAST=0 python3 -m tools.tooling_tests --full' test-fast
git add tools/hermetic_git.py tools/test_hermetic_git.py
tasks done material-5e6dd7 "tools/hermetic_git.isolate with suppression tests for core.hooksPath and config hooks"
tasks check
git add tasks/material-5e6dd7.md
git commit -m "feat(tools): hermetic git helper for tooling tests (material-561927)"
```

Expected: the full tooling run is `OK`. If `--check-paths` static coverage refuses the new module, add it where the error names, in this commit.

### Task 2: Coordinator isolation

**Files:**
- Modify: `tools/tooling_tests.py` (`main()`, top; imports)
- Test: `tools/test_tooling_tests.py` (`CoordinatorTests`)

**Interfaces:**
- Consumes: `tools.hermetic_git.isolate` (Task 1).
- Produces: every coordinator process (parent and `--worker`) runs with `GIT_CONFIG_GLOBAL=/dev/null`, `GIT_CONFIG_NOSYSTEM=1` and no other `GIT_*` key.

Start: `tasks start material-fa47f7`.

- [ ] **Step 1: Write the failing test**

Append to `CoordinatorTests` in `tools/test_tooling_tests.py`:

```python
    def test_workers_run_with_hermetic_git_in_both_modes(self):
        # The synthetic module never calls isolate: the coordinator must.
        self.fixture("import json\nclass C(unittest.TestCase):\n"
                     "    def test_env(self):\n"
                     "        with open(os.environ['PROBE_OUT'], 'w') as out:\n"
                     "            json.dump({k: v for k, v in os.environ.items() if k.startswith('GIT_')}, out)\n")
        poisoned = dict(GIT_DIR=str(self.root / 'decoy.git'), GIT_INDEX_FILE=str(self.root / 'index'),
                        GIT_CONFIG_GLOBAL=str(self.root / 'host-gitconfig'))
        for command in (['--full', '--ci'], ['--fast']):
            with self.subTest(command=command):
                out = self.root / f'probe-{command[0][2:]}.json'
                self.command = [sys.executable, '-m', 'tools.tooling_tests', *command]
                run = self.run_suite(PROBE_OUT=str(out), **poisoned)
                self.assertEqual(run.returncode, 0, run.stdout + run.stderr)
                self.assertEqual(json.loads(out.read_text()),
                                 {'GIT_CONFIG_GLOBAL': os.devnull, 'GIT_CONFIG_NOSYSTEM': '1'})
```

- [ ] **Step 2: Run it to make sure it fails**

Run: `just --set one_cmd 'env NIRI_TOOLING_FAST=0 python3 -m unittest' test-one tools.test_tooling_tests.CoordinatorTests.test_workers_run_with_hermetic_git_in_both_modes`
Expected: FAIL, the recorded dict carries the decoy `GIT_DIR`, `GIT_INDEX_FILE` and `GIT_CONFIG_GLOBAL`.

- [ ] **Step 3: Isolate at the entry point**

In `tools/tooling_tests.py`, add `from tools.hermetic_git import isolate` beside the other imports, and make it the first statement of `main()`:

```python
def main():
    # Parent and every --worker enter here, so no tooling case sees the host's git
    # environment or config, whichever bucket it lands in.
    isolate(os.environ)
    parser = argparse.ArgumentParser(description=__doc__)
```

- [ ] **Step 4: Run it to make sure it passes, then the module**

Run: `just --set one_cmd 'env NIRI_TOOLING_FAST=0 python3 -m unittest' test-one tools.test_tooling_tests`
Expected: `OK`.

- [ ] **Step 5: Fast check, close, commit**

`tools/tooling_tests.py` is a full-route path, so the commit's hook runs full tooling.

```bash
just --set fast_cmd 'env NIRI_TOOLING_FAST=0 python3 -m tools.tooling_tests --full' test-fast
git add tools/tooling_tests.py tools/test_tooling_tests.py
tasks done material-fa47f7 "tooling coordinator isolates git for the parent and every worker"
tasks check
git add tasks/material-fa47f7.md
git commit -m "feat(tools): tooling coordinator runs git hermetically (material-561927)"
```

### Task 3: `test_optic_settling` reads the checkout's own HEAD

**Files:**
- Modify: `tools/test_optic_settling.py` (imports; `PrepareTests.test_prepare_validates_inventory_and_cleans_runtime`, after `manifest = json.loads(...)`)
- Test: `tools/test_hermetic_git.py` (new `FocusedRunTests`)

**Interfaces:**
- Consumes: `isolate` (Task 1); `ROOT`, `git` from `tools/test_hermetic_git.py` (Task 1).
- Produces: `test_optic_settling` calls `isolate(os.environ)` at import (Task 4's inventory relies on it).

Start: `tasks start material-9b865d`.

- [ ] **Step 1: Write the comparison and the focused-run test**

In `PrepareTests.test_prepare_validates_inventory_and_cleans_runtime`, directly after `manifest = json.loads((base / 'out/manifest.json').read_text())`:

```python
            # Read HEAD under an environment built for this call alone: the fixture and the
            # smoke script both inherit the process environment, and would agree on a decoy.
            clean = dict(os.environ)
            isolate(clean)
            checkout_head = subprocess.check_output(['git', '-C', str(root), 'rev-parse', 'HEAD'],
                                                    env=clean, text=True).strip()
            self.assertEqual(manifest['source_commit'], checkout_head)
```

Add `from tools.hermetic_git import isolate` to the module's `from tools…` imports (no module-level call yet).

Append to `tools/test_hermetic_git.py` (add `import sys` to its imports):

```python
PREPARE = 'tools.test_optic_settling.PrepareTests.test_prepare_validates_inventory_and_cleans_runtime'


class FocusedRunTests(unittest.TestCase):
    def test_optic_settling_prepare_records_the_checkout_head_under_a_decoy_git_dir(self):
        clean = dict(os.environ)
        isolate(clean)
        head = git(ROOT, clean, 'rev-parse', 'HEAD')
        with tempfile.TemporaryDirectory() as temporary:
            decoy = pathlib.Path(temporary) / 'decoy'
            git(temporary, clean, 'init', '-q', '-b', 'main', str(decoy))
            git(decoy, clean, *IDENTITY, 'commit', '-q', '--allow-empty', '-m', 'decoy')
            self.assertNotEqual(git(decoy, clean, 'rev-parse', 'HEAD'), head)
            run = subprocess.run([sys.executable, '-m', 'unittest', PREPARE], cwd=ROOT,
                                 env=dict(os.environ, GIT_DIR=str(decoy / '.git')),
                                 capture_output=True, text=True, timeout=120)
            self.assertEqual(run.returncode, 0, run.stdout + run.stderr)
```

- [ ] **Step 2: Run the focused-run test to make sure it fails**

Run: `just --set one_cmd 'env NIRI_TOOLING_FAST=0 python3 -m unittest' test-one tools.test_hermetic_git.FocusedRunTests`
Expected: FAIL; the child's output shows `AssertionError` on `manifest['source_commit']` (the decoy's commit) against the checkout's HEAD.

- [ ] **Step 3: Isolate at import**

In `tools/test_optic_settling.py`, directly after the `from tools…` imports:

```python
# A hook exports GIT_DIR, which overrides `git -C`; the source-commit lookups would
# read another repository. Host git config stays out too.
isolate(os.environ)
```

- [ ] **Step 4: Run both to make sure they pass**

Run: `just --set one_cmd 'env NIRI_TOOLING_FAST=0 python3 -m unittest' test-one tools.test_hermetic_git tools.test_optic_settling`
Expected: `OK` (the `test_optic_settling` skips are the pre-existing ones).

- [ ] **Step 5: Fast check, close, commit**

```bash
just --set fast_cmd 'env NIRI_TOOLING_FAST=0 python3 -m tools.tooling_tests --full' test-fast
git add tools/test_optic_settling.py tools/test_hermetic_git.py
tasks done material-9b865d "optic-settling prepare compares its recorded source_commit with a clean-env HEAD; module isolates at import"
tasks check
git add tasks/material-9b865d.md
git commit -m "test(tools): optic-settling reads the checkout HEAD under a decoy GIT_DIR (material-561927)"
```

### Task 4: Every git-running test module isolates at import

**Files:**
- Modify: `tools/test_upstream_report.py:15-21`, `tools/test_identity_check.py:9-14`, `tools/test_target_dir_check.py`, `tools/test_package_pin.py`, `tools/test_capture_meta.py`, `tools/test_glass_optic_smoke.py:11-13` (the module-level `GIT_*` scrub loops)
- Modify: `tools/test_gates.py` (imports; module-level call)
- Modify: `AGENTS.md` (Gates, the "For Python tooling" bullet: one sentence)
- Test: `tools/test_hermetic_git.py` (new `InventoryTests`)

**Interfaces:**
- Consumes: `isolate`, `ROOT` (Task 1).
- Produces: nothing new; every `tools/test_*.py` whose source quotes `git` leaves its process environment hermetic after import.

Start: `tasks start material-2d059c`.

- [ ] **Step 1: Write the inventory test**

Append to `tools/test_hermetic_git.py` (add `import json` to its imports):

```python
# A module "runs git" when its source quotes the command name, which every git call,
# argv list and shutil.which in these modules does.
GIT_USE = re.compile(r'''["']git["']''')
PROBE = '''
import importlib, json, os, sys
sys.path.insert(0, 'tools')
importlib.import_module(sys.argv[1])
print(json.dumps({key: value for key, value in os.environ.items() if key.startswith('GIT_')}))
'''


class InventoryTests(unittest.TestCase):
    def test_every_git_running_module_isolates_at_import(self):
        modules = sorted(path.stem for path in (ROOT / 'tools').glob('test_*.py')
                         if GIT_USE.search(path.read_text()))
        self.assertTrue({'test_gates', 'test_optic_settling', 'test_upstream_report'} <= set(modules), modules)
        with tempfile.TemporaryDirectory() as temporary:
            decoy = dict(os.environ, GIT_DIR=str(pathlib.Path(temporary) / 'decoy.git'),
                         GIT_INDEX_FILE=str(pathlib.Path(temporary) / 'index'),
                         GIT_CONFIG_GLOBAL=str(pathlib.Path(temporary) / 'host-gitconfig'))
            for module in modules:
                with self.subTest(module=module):
                    run = subprocess.run([sys.executable, '-c', PROBE, module], cwd=ROOT, env=decoy,
                                         capture_output=True, text=True, timeout=60)
                    self.assertEqual(run.returncode, 0, run.stderr)
                    self.assertEqual(json.loads(run.stdout.splitlines()[-1]),
                                     {'GIT_CONFIG_GLOBAL': os.devnull, 'GIT_CONFIG_NOSYSTEM': '1'})
```

- [ ] **Step 2: Run it to make sure it fails**

Run: `just --set one_cmd 'env NIRI_TOOLING_FAST=0 python3 -m unittest' test-one tools.test_hermetic_git.InventoryTests`
Expected: the scan lists `test_capture_meta`, `test_gates`, `test_hermetic_git`, `test_identity_check`, `test_optic_settling`, `test_package_pin`, `test_target_dir_check`, `test_upstream_report`. Subtests FAIL for `test_gates` (decoy keys survive) and for the five scrub-only modules among them (empty dict: the scrub removed the decoys and set nothing). `test_optic_settling` and `test_hermetic_git` pass. `test_glass_optic_smoke` is not in the scan, since it reaches git only through the scripts it runs; it changes in step 3 all the same. Any module the scan lists beyond these gets the same call in step 3.

- [ ] **Step 3: Replace the scrubs and add the missing call**

In each of the six modules, replace the scrub loop

```python
for _name in [_key for _key in os.environ if _key.startswith("GIT_")]:
    del os.environ[_name]
```

with

```python
isolate(os.environ)
```

and add `from tools.hermetic_git import isolate` to that module's imports. Keep each module's comment above the loop; where it says the scrub removes hook variables, extend it with "and reads no host git config". In `tools/test_identity_check.py`, keep its `GIT_CONFIG_GLOBAL`/`GIT_CONFIG_NOSYSTEM` entries in `self.env`: its own override must win over the base.

In `tools/test_gates.py`, after its imports:

```python
from tools.hermetic_git import isolate

# Fixture repositories here must not see a hook's GIT_INDEX_FILE or GIT_DIR, nor host git config.
isolate(os.environ)
```

In `AGENTS.md`, Gates, append to the "For Python tooling" bullet: "A test module that runs git calls `isolate(os.environ)` from `tools/hermetic_git.py` at import; the coordinator also applies it to every worker."

- [ ] **Step 4: Run the inventory and each changed module**

Run: `just --set one_cmd 'env NIRI_TOOLING_FAST=0 python3 -m unittest' test-one tools.test_hermetic_git tools.test_upstream_report tools.test_identity_check tools.test_target_dir_check tools.test_package_pin tools.test_capture_meta tools.test_glass_optic_smoke tools.test_gates`
Expected: `OK`. A module that fails here relied on host git config (an identity, a default branch, a filter). Fix the fixture so it supplies what it needs; never widen `isolate`.

- [ ] **Step 5: Fast check, close, commit**

```bash
just --set fast_cmd 'env NIRI_TOOLING_FAST=0 python3 -m tools.tooling_tests --full' test-fast
git add tools/test_*.py AGENTS.md
tasks done material-2d059c "every git-running tooling test module isolates at import; inventory probe covers new modules"
tasks check
git add tasks/material-2d059c.md
git commit -m "test(tools): git-running tooling modules isolate at import (material-561927)"
```

Record this commit's UTC time; it is the branch remedy timestamp (the last `tools/` change):
`TZ=UTC git log -1 --date=format-local:%Y-%m-%dT%H:%M:%SZ --format=%cd`

### Task 5: Branch acceptance and final review

**Files:**
- Modify: `docs/specs/2026-10-17-pre-commit-fast-hermetic-git-design.md` (status line; a "Branch results" paragraph under §4)
- Modify: task records through `tasks note` / `tasks done` only.

**Interfaces:**
- Consumes: Tasks 1–4 committed; the branch remedy timestamp from Task 4.
- Produces: the dry-run verdict and measurements Task 6 compares against.

Start: `tasks start material-c946ff`.

- [ ] **Step 1: Check the host is idle.** `uptime` and `host-load`. If the load average exceeds 4, or another session runs a build or capture, park: `tasks park material-c946ff "rerun Task 5 measurements (warm check, 2 probes, 3 staged hooks), then dry-run verify" --reason quiet --waiting-on user --minutes 15`.

- [ ] **Step 2: Both routes pass, hook still installed.** Confirm `git config --global --get-all hook.identity-check.event` prints `pre-commit`. Then `just check` and `just check-full`; both exit 0. In each output, the `Ran N tests` count equals 529 plus the tests this branch added, and the `SKIP` lines match those on `materials-26.04`.

- [ ] **Step 3: The cause is gone.**

```bash
s=$(date +%s.%N); just --set one_cmd 'env NIRI_TOOLING_FAST=1 python3 -m unittest' test-one tools.test_upstream_report >/dev/null 2>&1; echo "upstream_report rc=$? $(python3 -c "print(round($(date +%s.%N)-$s,1))") s"
s=$(date +%s.%N); just --set fast_cmd 'env NIRI_TOOLING_FAST=1 python3 -m tools.tooling_tests --fast' test-fast >/dev/null 2>&1; echo "fast tooling rc=$? $(python3 -c "print(round($(date +%s.%N)-$s,1))") s"
```

Expected: both `rc=0`; `test_upstream_report` ≤ 9 s; fast tooling ≤ 26 s. A miss stops here: note the numbers on the parent before changing anything.

- [ ] **Step 4: Three staged fast-route hook runs.** Stage through a private index and run `.githooks/pre-commit` itself, which routes on the staged paths and records `hook-pre-commit-fast` through `tools/tt`. The hook's `upstream-report --stage` rewrites `docs/materials/upstream-divergence.md` in the working tree, so each run saves and restores it on every exit and removes its temporary index, in a subshell so the trap cannot leak:

```bash
REPORT=docs/materials/upstream-divergence.md
stage_run() (  # $1: path whose staged blob gains a trailing newline
  set -u
  saved=$(mktemp); idx=$(mktemp)
  cp -p "$REPORT" "$saved"
  cleanup() { cp -p "$saved" "$REPORT"; rm -f "$saved" "$idx"; }
  trap cleanup EXIT
  trap 'exit 130' INT TERM
  cp "$(git rev-parse --git-path index)" "$idx"
  mode=$(git ls-files -s -- "$1" | cut -d' ' -f1)
  blob=$( { cat "$1"; echo; } | git hash-object -w --stdin)
  GIT_INDEX_FILE=$idx git update-index --cacheinfo "$mode,$blob,$1"
  s=$(date +%s.%N); GIT_INDEX_FILE=$idx .githooks/pre-commit >/dev/null 2>&1; rc=$?
  echo "$1 rc=$rc $(python3 -c "print(round($(date +%s.%N)-$s,1))") s"
)
before=$(git status --porcelain)
for i in 1 2 3; do stage_run tools/upstream-report; done
[ "$(git status --porcelain)" = "$before" ] && echo "working tree unchanged" || git status --short
jq -r --arg since "<branch remedy timestamp>" 'select(.project=="material" and .target=="hook-pre-commit-fast" and .at>=$since) | "\(.at) \(.seconds) exit=\(.exit)"' ~/.local/share/ops/runs.jsonl
```

Expected: `working tree unchanged`; three `rc=0` runs; median ≤ 40 s. These runs also prove the hook's inherited `GIT_INDEX_FILE` reaches no fixture.

- [ ] **Step 5: Dry-run verify.** `tt-latency verify material-561927 --after <branch remedy timestamp> --dry-run`. Expected: exit 0. It records nothing; never run it without `--dry-run` on the branch.

- [ ] **Step 6: Record branch results.** Set the spec's status line to `Status: Implemented on the branch; post-merge verification pending (material-561927).` and add a "Branch results" paragraph under §4 with the step 2 counts, the step 3 seconds, the three hook seconds and the dry-run verdict. Then:

```bash
tasks note material-561927 "accept (branch): upstream_report <u> s, fast tooling <f> s, fast hooks <a>/<b>/<c> s (median <m>), verify --dry-run exit 0 after <branch remedy timestamp>"
git add docs/specs/2026-10-17-pre-commit-fast-hermetic-git-design.md
tasks done material-c946ff "branch acceptance: probes and staged fast hooks under target; dry-run verify exit 0"
tasks check
git add tasks/material-c946ff.md
git commit -m "docs(specs): record branch acceptance for hermetic git (material-561927)"
```

- [ ] **Step 7: Final branch review and corrections.** Dispatch one fresh reviewer on the most capable model over `git diff materials-26.04...HEAD`, with the spec and this plan; it stays read-only and returns its full review. Save it as `.superpowers/sdd/material-561927/final-review-report.md` with the reviewer identity. Note the round: `tasks note material-561927 "review: impl round <n> — verdict: <revise|accept>; findings: <label> <count>, … | none; reviewer: <harness/model>"`. Fix Critical and Important findings in corrective rounds, up to five: one fix commit plus one scoped re-review each, each fix proven RED→GREEN with a green fast check. Minor findings go to the parent as `minor (deferred): …` notes. A fix touching `tools/` invalidates steps 3–5: rerun them with that fix's commit time as the new branch remedy timestamp (still `--dry-run`).

### Task 6: Merge, post-merge verification, close

Runs from the registered checkout (the main checkout that `tasks projects` lists for `material`), not the worktree.

**Files:**
- Modify: `docs/specs/2026-10-17-pre-commit-fast-hermetic-git-design.md` (status line; "Post-merge results")
- Modify: task records through `tasks` only.

**Interfaces:**
- Consumes: the reviewed branch `fix/material-561927`.
- Produces: the recording `tt-latency verify` and the closed incident.

- [ ] **Step 1: Hand the claims to the main checkout.** In the worktree, `tasks park material-561927 "merge fix/material-561927, then Task 6 post-merge verification from the main checkout" --waiting-on agent`. Confirm `git -C .worktrees/material-561927 status --porcelain` is empty.

- [ ] **Step 2: Merge and stamp the remedy time.** In the main checkout, `git status --porcelain` must not list `docs/materials/upstream-divergence.md` or any path the branch touches; otherwise stop and resolve with whoever owns those edits. Then:

The block stops, without stamping anything, unless the merge completed: a conflicting `--no-ff` merge leaves `MERGE_HEAD` and unmerged paths behind, and those are checked before the timestamp is written. Shell state does not persist between commands, so the remedy timestamp lives in a file in the git directory, and later steps read it from there.

```bash
bash <<'EOF'
set -eu
[ "$(git branch --show-current)" = materials-26.04 ] || { echo "stop: not on materials-26.04" >&2; exit 1; }
if ! git merge --ff-only fix/material-561927; then
  if ! git merge --no-ff fix/material-561927 -m "merge: hermetic git for tooling fixtures (material-561927)"; then
    echo "stop: merge failed; resolve it (or git merge --abort) and rerun this step" >&2; exit 1
  fi
fi
if git rev-parse -q --verify MERGE_HEAD >/dev/null; then echo "stop: a merge is still in progress" >&2; exit 1; fi
[ -z "$(git diff --name-only --diff-filter=U)" ] || { echo "stop: unmerged paths remain" >&2; exit 1; }
git merge-base --is-ancestor fix/material-561927 HEAD || { echo "stop: fix/material-561927 is not in HEAD" >&2; exit 1; }
date -u +%Y-%m-%dT%H:%M:%SZ > "$(git rev-parse --git-path material-561927-remedy)"
cat "$(git rev-parse --git-path material-561927-remedy)"
EOF
```

Only when the block exits 0:

```bash
REMEDY=$(cat "$(git rev-parse --git-path material-561927-remedy)")
tasks start material-561927
tasks start material-c0c135
tasks note material-561927 "remedy: fix/material-561927 merged into materials-26.04 at $REMEDY"
```

- [ ] **Step 3: Retire the worktree.** So no sibling holds a newer record or a live claim when verification records:

First the inspections, whose findings you act on by hand:

```bash
tt-report
git -C .worktrees/material-561927 status --ignored --porcelain   # copy out anything still needed; target/ is build output
ls .worktrees/material-561927/.superpowers/sdd/ 2>/dev/null       # copy material-561927's ledger and review report out first if its Ruling/minor lines are not yet in a final message
```

Then one guarded block. It refuses, and removes nothing, while any host pointer resolves into the worktree. It checks both the worktree's logical path and its real path, since `.worktrees` is a link into local storage, across every pointer location the repository instructions name: launchers in `~/bin` and `~/.local/bin`, user systemd units (links, and unit files or drop-ins that name the path), and the known config includes (`~/.config/niri`, links and files that name the path). Content is searched through symlinks (`grep -R`), so a unit or include managed as a dotfiles link to a file outside the worktree still counts when that file names the worktree. A pointer found here was repointed by a live test and is restored first (the task's notes record what, from, to and how), then the block reruns.

```bash
bash <<'EOF'
set -eu
shopt -s nullglob globstar
wt=.worktrees/material-561927
logical=$PWD/$wt
real=$(cd "$wt" && pwd -P)
places=(~/bin ~/.local/bin ~/.config/systemd/user ~/.config/niri)
hits=()
for dir in "${places[@]}"; do
  [ -e "$dir" ] || continue
  for path in "$dir"/**; do
    if [ -L "$path" ]; then
      # -m resolves even when the target's directories do not exist; the raw text catches the rest.
      for target in "$(readlink -m "$path")" "$(readlink "$path")"; do
        case "$target" in "$real"|"$real"/*|"$logical"|"$logical"/*) hits+=("$path -> $target"); break ;; esac
      done
    fi
  done
  while IFS= read -r file; do hits+=("$file names the worktree"); done \
    < <(grep -RIlsF -e "$real" -e "$logical" "$dir" || true)  # -R: a dotfiles-linked unit names it too
done
if [ "${#hits[@]}" -gt 0 ]; then
  printf 'stop: host pointer into the worktree; restore it, then rerun:\n' >&2
  printf '  %s\n' "${hits[@]}" >&2
  exit 1
fi
git worktree unlock "$wt"
git worktree remove "$wt"
git branch -d fix/material-561927
EOF
```

Expected: exit 0 with the worktree and branch gone. Any `stop:` line means nothing was removed.

Piloted on scratch fixtures (fake `HOME`, `.worktrees` linked into storage). Each of these stops the block with the worktree and branch kept: a launcher linking to a not-yet-existing path in the worktree (real path), a launcher linking through `.worktrees`, a unit linking into the worktree, a drop-in naming the worktree, a niri include naming it, a unit linked to a file outside the worktree whose `ExecStart` names it, and a config directory linked outside `HOME` holding an include that names it. A clean `HOME`, including one with a directory link loop, removes both.

- [ ] **Step 4: Three qualifying post-merge runs.** On the idle host (park `--reason quiet --waiting-on user --minutes 10` otherwise), in the main checkout on `materials-26.04`, run Task 5 step 4's `stage_run` block unchanged (three `tools/upstream-report` runs). Then list the post-merge window, every checkout included:

```bash
REMEDY=$(cat "$(git rev-parse --git-path material-561927-remedy)")
jq -r --arg since "$REMEDY" 'select(.project=="material" and .target=="hook-pre-commit-fast" and .at>=$since) | "\(.at) \(.seconds) exit=\(.exit) \(.by)"' ~/.local/share/ops/runs.jsonl
```

Expected: three `exit=0` runs from this checkout, each under 40 s. A run from another checkout branched before the remedy is named in the closing note.

- [ ] **Step 5: Record verification and close.**

```bash
REMEDY=$(cat "$(git rev-parse --git-path material-561927-remedy)")
tt-latency verify material-561927 --after "$REMEDY" --dry-run   # must exit 0 first
tt-latency verify material-561927 --after "$REMEDY"             # recording, once
```

Set the spec's status line to `Status: Implemented (material-561927).` and add a "Post-merge results" paragraph with the three run seconds and the verify line. Then close Task 6's child and the parent, the parent's message carrying the verify output:

```bash
REMEDY=$(cat "$(git rev-parse --git-path material-561927-remedy)")
git add docs/specs/2026-10-17-pre-commit-fast-hermetic-git-design.md
tasks done material-c0c135 "post-merge: three fast hooks <a>/<b>/<c> s; tt-latency verify exit 0"
tasks done material-561927 "tt-latency verify material-561927 --after $REMEDY exit 0: <verify output>"
tasks check
git add tasks/material-c0c135.md tasks/material-561927.md
git commit -m "docs(specs): hermetic git verified after merge (material-561927)"
rm "$(git rev-parse --git-path material-561927-remedy)"
```
