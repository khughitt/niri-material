"""Explicit tooling modes and bounded native unittest workers (standard library only)."""
import argparse
import ast
from collections.abc import Sequence
import fnmatch
import shlex
from concurrent.futures import ThreadPoolExecutor
import json
import os
from pathlib import Path
import re
import signal
import subprocess
import sys
import tempfile
import threading
import time
import traceback
import unittest

LIFECYCLE_CLASSES = {'DriverCleanupTests', 'VtLibTests'}
OPTIONAL_SKIPS = {
    'test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_ring_replaces_inherited_default_response_with_retained_candidate',
    'test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_remaining_within_configs_validate_with_retained_candidate',
    'test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_positive_face_gate_requires_more_than_one_code',
    'test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_signed_transfer_helpers_reject_black_clipped_and_wrong_solid_output',
    'test_glass_render_order_metrics.RenderOrderMetricsTest.test_cli_distinguishes_failed_gate_from_invalid_capture',
}
_workers = set()
_worker_lock = threading.Lock()
_cancelled = threading.Event()
_result_dir = None


def fast_mode() -> bool:
    raw = os.environ.get('NIRI_TOOLING_FAST')
    if raw not in (None, '0', '1'):
        raise ValueError('NIRI_TOOLING_FAST must be unset, 0 or 1')
    return raw == '1'


def worker_limit() -> int:
    raw = os.environ.get('NEXTEST_TEST_THREADS')
    if raw is None:
        return 10
    if re.fullmatch(r'[0-9]+', raw) is None or int(raw) < 1:
        raise ValueError('NEXTEST_TEST_THREADS must be a positive decimal integer')
    return min(10, int(raw))


def flatten(suite: unittest.TestSuite) -> list[unittest.TestCase]:
    def cases(node):
        if isinstance(node, unittest.TestSuite):
            for child in node:
                yield from cases(child)
        else:
            yield node
    result = list(cases(suite))
    ids = [case.id() for case in result]
    if len(set(ids)) != len(ids):
        raise ValueError('duplicate native test IDs in discovery')
    return result


def interrupt(signum, frame):
    raise KeyboardInterrupt(f'signal {signum}')


def stop_worker(process):
    if process.poll() is None:
        try:
            process.terminate()
        except ProcessLookupError:
            pass
        try:
            process.wait(timeout=15)
        except subprocess.TimeoutExpired:
            process.kill()
    process.wait()


def cancel_pending_and_reap_workers() -> None:
    with _worker_lock:
        _cancelled.set()
        workers = list(_workers)
    for process in workers:
        stop_worker(process)


class RecordedResult(unittest.TestResult):
    def __init__(self):
        super().__init__()
        self.started = []
        self.elapsed = {}
        self.began = {}

    def startTest(self, case):
        self.started.append(case)
        self.began[case.id()] = time.monotonic()
        super().startTest(case)

    def stopTest(self, case):
        self.elapsed[case.id()] = time.monotonic() - self.began[case.id()]
        super().stopTest(case)


def worker(result_path, ids):
    os.environ['NIRI_TOOLING_FAST'] = '0'
    sys.path.insert(0, str(Path.cwd() / 'tools'))
    signal.signal(signal.SIGTERM, interrupt)
    signal.signal(signal.SIGINT, interrupt)
    result = RecordedResult()
    interrupted = None
    try:
        unittest.defaultTestLoader.loadTestsFromNames(ids).run(result)
    except BaseException:
        interrupted = traceback.format_exc()
    finally:
        # Native unittest can unwind past doCleanups on KeyboardInterrupt.
        signal.signal(signal.SIGTERM, signal.SIG_IGN)
        signal.signal(signal.SIGINT, signal.SIG_IGN)
        for case in result.started:
            case.doCleanups()
    receipt = dict(ids=[case.id() for case in result.started], ran=result.testsRun,
                   skipped=[(case.id(), reason) for case, reason in result.skipped],
                   failures=[(case.id(), detail) for case, detail in result.failures],
                   errors=[(case.id(), detail) for case, detail in result.errors], elapsed=result.elapsed)
    if interrupted:
        receipt['errors'].append(('interrupted worker', interrupted))
    Path(result_path).write_text(json.dumps(receipt))
    return 0 if not receipt['failures'] and not receipt['errors'] else 1


def run_job(ids: list[str]) -> dict[str, object]:
    with tempfile.NamedTemporaryFile(dir=_result_dir, suffix='.json', delete=False) as out:
        path = Path(out.name)
    path.unlink()
    with _worker_lock:
        if _cancelled.is_set():
            raise RuntimeError('tooling worker scheduling cancelled')
        process = subprocess.Popen([sys.executable, '-m', 'tools.tooling_tests', '--worker', str(path), *ids],
                                   env=dict(os.environ, NIRI_TOOLING_FAST='0'), start_new_session=True,
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        _workers.add(process)
    try:
        try:
            stdout, stderr = process.communicate(timeout=120)
        except subprocess.TimeoutExpired:
            stop_worker(process)
            stdout, stderr = process.communicate()
            raise RuntimeError(f'tooling worker timed out: {ids}\n{stdout}{stderr}')
        try:
            receipt = json.loads(path.read_text())
        except (OSError, ValueError) as error:
            raise RuntimeError(f'tooling worker exited {process.returncode} without a valid receipt: {ids}\n{stdout}{stderr}') from error
        if not isinstance(receipt, dict) or set(receipt) != {'ids', 'ran', 'skipped', 'failures', 'errors', 'elapsed'}:
            raise RuntimeError(f'malformed tooling receipt: {ids}')
        if receipt['ids'] != ids or type(receipt['ran']) is not int or receipt['ran'] != len(ids):
            raise RuntimeError(f'incomplete tooling worker inventory: {ids}\n{receipt}')
        if not isinstance(receipt['elapsed'], dict) or set(receipt['elapsed']) != set(ids):
            raise RuntimeError(f'malformed tooling timings: {ids}')
        for field in ('skipped', 'failures', 'errors'):
            if not isinstance(receipt[field], list) or any(
                    not isinstance(pair, list) or len(pair) != 2 or
                    not all(isinstance(value, str) for value in pair) for pair in receipt[field]):
                raise RuntimeError(f'malformed tooling {field}: {ids}')
        if process.returncode and not receipt['failures'] and not receipt['errors']:
            raise RuntimeError(f'tooling worker exited {process.returncode}: {ids}\n{stdout}{stderr}')
        return receipt
    finally:
        stop_worker(process)
        with _worker_lock:
            _workers.discard(process)
        process.stdout.close()
        process.stderr.close()
        path.unlink(missing_ok=True)


def run_full(ci: bool = False) -> int:
    global _result_dir
    limit = worker_limit()  # fail before discovery or spawning
    os.environ['NIRI_TOOLING_FAST'] = '0'
    loader = unittest.defaultTestLoader
    loader.errors.clear()
    inventory = flatten(loader.discover('tools'))
    if loader.errors:
        raise ValueError('\n'.join(loader.errors))
    if ci:
        for case in inventory:
            method = getattr(case, case._testMethodName)
            for subject in (type(case), method):
                if getattr(subject, '__unittest_skip__', False) and case.id() not in OPTIONAL_SKIPS:
                    raise ValueError(f'unexpected CI skip: {case.id()}: {subject.__unittest_skip_why__}')
    lifecycle = [case.id() for case in inventory if type(case).__name__ in LIFECYCLE_CLASSES]
    remainder = [case.id() for case in inventory if type(case).__name__ not in LIFECYCLE_CLASSES]
    jobs = ([remainder] if remainder else []) + [[case_id] for case_id in lifecycle]
    expected = {case.id() for case in inventory}
    assigned = [case_id for job in jobs for case_id in job]
    if len(assigned) != len(expected) or set(assigned) != expected:
        raise RuntimeError('full discovery partition lost or duplicated cases')
    _cancelled.clear()
    before = time.monotonic()
    print(f'Full tooling: {len(inventory)} cases, {limit} children', flush=True)
    with tempfile.TemporaryDirectory() as directory:
        _result_dir = directory
        with ThreadPoolExecutor(max_workers=limit) as pool:
            try:
                results = list(pool.map(run_job, jobs))
            finally:
                cancel_pending_and_reap_workers()
    errors = sum(len(result['errors']) for result in results)
    failures = sum(len(result['failures']) for result in results)
    skipped = [pair for result in results for pair in result['skipped']]
    for result in results:
        for case_id, elapsed in result['elapsed'].items():
            print(f'case {case_id}: {elapsed:.3f}s')
        for case_id, detail in result['failures'] + result['errors']:
            print(f'FAIL {case_id}\n{detail}')
    for case_id, reason in skipped:
        print(f'SKIP {case_id}: {reason}')
        if ci and case_id not in OPTIONAL_SKIPS:
            errors += 1
            print(f'unexpected CI skip: {case_id}: {reason}')
    print(f'\nRan {sum(result["ran"] for result in results)} tests in {time.monotonic() - before:.3f}s')
    if errors or failures:
        print(f'FAILED (failures={failures}, errors={errors}, skipped={len(skipped)})')
        return 1
    print(f'OK (skipped={len(skipped)})' if skipped else 'OK')
    return 0



def shell_source_lines(contents):
    """Keep source locations while separating shell commands from heredoc data."""
    delimiter = None
    for number, line in enumerate(contents.splitlines(keepends=True), 1):
        if delimiter is not None:
            end, quoted = delimiter
            if line.strip() == end:
                delimiter = None
            yield number, '', '' if quoted else line
            continue
        if line.lstrip().startswith('#'):
            yield number, '', ''
            continue
        matches = list(re.finditer(r"<<-?\s*(['\"]?)([A-Za-z_]\w*)\1", line))
        if len(matches) > 1:
            raise ValueError(f'line {number}: unsupported multiple heredocs')
        if matches:
            delimiter = (matches[0][2], bool(matches[0][1]))
        yield number, line, line


def reject_arithmetic_commands(contents, source):
    # Conservative source guard: balanced arithmetic, including multiline forms.
    # No shell expansion or evaluation is used to decide the TERM hazard.
    masked = ''.join(arithmetic if arithmetic else '\n'
                     for _, _, arithmetic in shell_source_lines(contents))
    for match in re.finditer(r'\$\(\(', masked):
        depth = 2
        index = match.end()
        while index < len(masked) and depth:
            if masked[index] == '\\':
                index += 2
                continue
            if (masked.startswith('$(', index) and not masked.startswith('$((', index)) or masked[index] == '`':
                line = masked.count('\n', 0, match.start()) + 1
                raise ValueError(f'{source}:{line}: command substitution inside arithmetic expansion')
            if masked[index] == '(':
                depth += 1
            elif masked[index] == ')':
                depth -= 1
            index += 1


def assert_static_coverage(root: Path, patterns: Sequence[str]) -> None:
    """Check the bounded lifecycle source contract without importing or executing it."""
    root = root.resolve()

    def require(path, source, line):
        try:
            relative = path.resolve().relative_to(root).as_posix()
        except ValueError:
            raise ValueError(f'{source}:{line}: repository path escapes root: {path}')
        if not any(fnmatch.fnmatchcase(relative, pattern) for pattern in patterns):
            raise ValueError(f'{source}:{line}: full route does not cover {relative}')
        if not path.is_file():
            raise ValueError(f'{source}:{line}: missing source {relative}')
        return relative

    entry = root / 'docs/materials/scripts/optic-settling-smoke.sh'
    visited, active = set(), set()

    def shell(path, source='lifecycle entry', line=1):
        relative = require(path, source, line)
        if relative in active:
            raise ValueError(f'{source}:{line}: shell source cycle at {relative}')
        if relative in visited:
            return
        active.add(relative)
        contents = path.read_text()
        reject_arithmetic_commands(contents, relative)
        for number, text, _ in shell_source_lines(contents):
            if re.match(r'^\s*(?:\.|source)\s+', text):
                operands = shlex.split(text, comments=True)
                if len(operands) != 2:
                    raise ValueError(f'{relative}:{number}: unsupported source statement')
                operand = operands[1]
                # Sourcing retains the driver's HERE and the fixture's repository cwd.
                for prefix, base in (('$HERE/', entry.parent), ('${HERE}/', entry.parent),
                                     ('$ROOT/', root), ('${ROOT}/', root)):
                    if operand.startswith(prefix):
                        operand = str(base / operand[len(prefix):])
                        break
                if any(character in operand for character in '$`*?[]'):
                    raise ValueError(f'{relative}:{number}: unsupported source operand {operand}')
                target = Path(operand)
                shell(target if target.is_absolute() else root / target, relative, number)
            if path == entry:
                # Helpers' tool fallbacks are stubbed by lifecycle fixtures (CAPTURE_META).
                for match in re.finditer(r'\$(?:ROOT|\{ROOT\})/(tools/[\w./-]+)', text):
                    require(root / match[1], relative, number)
        active.remove(relative)
        visited.add(relative)

    shell(entry)
    visited_python = set()

    def module_path(module, source, line):
        if module == 'tools' and not (root / 'tools/__init__.py').exists():
            return None  # existing namespace root
        base = root.joinpath(*module.split('.'))
        candidates = [candidate for candidate in (base.with_suffix('.py'), base / '__init__.py')
                      if candidate.is_file()]
        if len(candidates) != 1:
            raise ValueError(f'{source}:{line}: unresolved or ambiguous tools import {module}')
        return candidates[0]

    def python(path, source='covered Python', line=1):
        relative = require(path, source, line)
        if relative in visited_python:
            return
        visited_python.add(relative)
        try:
            tree = ast.parse(path.read_text(), filename=relative)
        except SyntaxError as error:
            raise ValueError(f'{relative}:{error.lineno}: {error.msg}') from error
        parts = list(Path(relative).with_suffix('').parts)
        package = parts[:-1]
        for node in ast.walk(tree):
            imports = []
            if isinstance(node, ast.Import):
                imports = [alias.name for alias in node.names if alias.name == 'tools' or alias.name.startswith('tools.')]
            elif isinstance(node, ast.ImportFrom):
                module = node.module or ''
                if node.level:
                    if node.level > len(package):
                        raise ValueError(f'{relative}:{node.lineno}: unresolved relative tools import')
                    module = '.'.join(package[:len(package) - node.level + 1] + ([module] if module else []))
                if module == 'tools':
                    imports = ['tools.' + alias.name for alias in node.names]
                elif module.startswith('tools.'):
                    imports = [module]
                    # Imported attributes need no extra module; package submodules do.
                    for alias in node.names:
                        child = module + '.' + alias.name
                        base = root.joinpath(*child.split('.'))
                        if base.with_suffix('.py').is_file() or (base / '__init__.py').is_file():
                            imports.append(child)
            for module in imports:
                target = module_path(module, relative, node.lineno)
                if target is not None:
                    python(target, relative, node.lineno)

        classes = [node for node in tree.body if isinstance(node, ast.ClassDef) and
                   node.name in LIFECYCLE_CLASSES]
        if not classes:
            return
        globals_ = [node for node in tree.body if isinstance(node, (ast.Assign, ast.AnnAssign))]
        for cls in classes:
            aliases = {}
            nodes = list(ast.walk(cls))

            def repo_path(node):
                if ast.unparse(node) == 'Path(__file__).resolve().parents[1]':
                    return ''
                if isinstance(node, (ast.Name, ast.Attribute)):
                    if (isinstance(node, ast.Attribute) and node.attr in ('parent', 'parents')
                            and repo_path(node.value) is not None):
                        raise ValueError(f'{relative}:{node.lineno}: unsupported repository path construction')
                    return aliases.get(ast.unparse(node))
                if isinstance(node, ast.BinOp) and isinstance(node.op, ast.Div):
                    left = repo_path(node.left)
                    if left is not None:
                        if not isinstance(node.right, ast.Constant) or not isinstance(node.right.value, str):
                            raise ValueError(f'{relative}:{node.lineno}: dynamic repository path')
                        return str(Path(left) / node.right.value)
                    if any((isinstance(child, ast.Name) and child.id == '__file__') or
                           (isinstance(child, (ast.Name, ast.Attribute)) and ast.unparse(child) in aliases)
                           for child in ast.walk(node.left)):
                        raise ValueError(f'{relative}:{node.lineno}: unsupported repository root')
                if isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute):
                    if node.func.attr in ('joinpath', 'with_name', 'with_suffix', 'resolve', 'absolute', 'expanduser', 'relative_to') and repo_path(node.func.value) is not None:
                        raise ValueError(f'{relative}:{node.lineno}: unsupported repository path construction')
                return None

            assignments = globals_ + [node for node in nodes if isinstance(node, (ast.Assign, ast.AnnAssign))]
            for _ in range(len(assignments) + 1):
                before = dict(aliases)
                for assignment in assignments:
                    if assignment.value is None:
                        continue
                    value = repo_path(assignment.value)
                    if value is None and any(isinstance(child, ast.Name) and child.id == '__file__'
                                             for child in ast.walk(assignment.value)):
                        raise ValueError(f'{relative}:{assignment.lineno}: unsupported repository root')
                    if value is not None:
                        targets = assignment.targets if isinstance(assignment, ast.Assign) else [assignment.target]
                        for target in targets:
                            aliases[ast.unparse(target)] = value
                if aliases == before:
                    break
            parents = {id(child): parent for parent in nodes for child in ast.iter_child_nodes(parent)}
            for node in nodes:
                parent = parents.get(id(node))
                if isinstance(parent, ast.BinOp) and isinstance(parent.op, ast.Div):
                    continue  # check the complete path, not intermediate directories
                value = repo_path(node)
                if value:
                    require(root / value, relative, node.lineno)

    seeds = {path for pattern in patterns for path in root.glob(pattern)
             if path.suffix == '.py' and path.parts[len(root.parts):len(root.parts) + 1] == ('tools',)}
    # Selected class source files are entry points even if accidentally omitted.
    seeds.update(root / ('tools/' + name) for name in ('test_optic_settling.py', 'test_vt_lib.py'))
    for path in sorted(seeds):
        python(path)


def check_paths():
    patterns = subprocess.check_output(['just', '--evaluate', 'tooling_full_paths'], text=True).split()
    assert_static_coverage(Path.cwd(), patterns)

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    choices = parser.add_mutually_exclusive_group(required=True)
    choices.add_argument('--full', action='store_true')
    choices.add_argument('--check-paths', action='store_true')
    choices.add_argument('--worker', metavar='RESULT', help=argparse.SUPPRESS)
    parser.add_argument('--ci', action='store_true')
    parser.add_argument('ids', nargs='*', help=argparse.SUPPRESS)
    args = parser.parse_args()
    if args.worker:
        return worker(args.worker, args.ids)
    signal.signal(signal.SIGTERM, interrupt)
    signal.signal(signal.SIGINT, interrupt)
    try:
        if args.check_paths:
            check_paths()
            return 0
        return run_full(args.ci)
    except (ValueError, RuntimeError, KeyboardInterrupt) as error:
        print(f'FAILED: {error}', file=sys.stderr)
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
