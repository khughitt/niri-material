# Static lifecycle source guard

Real source check and synthetic recursive shell/class/Python dependency controls passed. Arithmetic fixture was red (ValueError not raised), then green with a named helper line. Unsupported root construction controls were red, then green. Fast-mode focused controls execute outside the skipped classes. Full suite: 305 cases, two optional skips, 28.464 s. Direct --check-paths succeeds without discovering tests or importing media bindings.

```text
.F.E
======================================================================
ERROR: test_subject_paths_are_full_routed (tools.test_optic_settling.LifecycleCoverageTests.test_subject_paths_are_full_routed)
----------------------------------------------------------------------
Traceback (most recent call last):
  File "tools/test_optic_settling.py", line 757, in test_subject_paths_are_full_routed
    assert_static_coverage(root, patterns)
    ~~~~~~~~~~~~~~~~~~~~~~^^^^^^^^^^^^^^^^
  File "tools/tooling_tests.py", line 282, in assert_static_coverage
    shell(entry)
    ~~~~~^^^^^^^
  File "tools/tooling_tests.py", line 264, in shell
    raise ValueError(f'{relative}:{number}: unsupported source statement')
ValueError: docs/materials/scripts/optic-settling-smoke.sh:153: unsupported source statement

======================================================================
FAIL: test_nested_command_substitution_in_arithmetic_is_rejected_in_helpers (tools.test_tooling_tests.StaticCoverageTests.test_nested_command_substitution_in_arithmetic_is_rejected_in_helpers)
----------------------------------------------------------------------
Traceback (most recent call last):
  File "tools/test_tooling_tests.py", line 212, in test_nested_command_substitution_in_arithmetic_is_rejected_in_helpers
    with self.assertRaisesRegex(ValueError, 'nested-lib.sh:2.*command substitution.*arithmetic'):
         ~~~~~~~~~~~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
AssertionError: ValueError not raised

----------------------------------------------------------------------
Ran 4 tests in 0.037s

FAILED (failures=1, errors=1)
error: recipe `test-one` failed on line 56 with exit code 1

```

```text
FF
======================================================================
FAIL: test_missing_source_cycle_dynamic_operand_and_missing_module_fail (tools.test_tooling_tests.StaticCoverageTests.test_missing_source_cycle_dynamic_operand_and_missing_module_fail) (diagnostic='unsupported')
----------------------------------------------------------------------
Traceback (most recent call last):
  File "tools/test_tooling_tests.py", line 207, in test_missing_source_cycle_dynamic_operand_and_missing_module_fail
    with self.assertRaisesRegex(ValueError, diagnostic):
         ~~~~~~~~~~~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^
AssertionError: ValueError not raised

======================================================================
FAIL: test_missing_source_cycle_dynamic_operand_and_missing_module_fail (tools.test_tooling_tests.StaticCoverageTests.test_missing_source_cycle_dynamic_operand_and_missing_module_fail) (diagnostic='unsupported')
----------------------------------------------------------------------
Traceback (most recent call last):
  File "tools/test_tooling_tests.py", line 207, in test_missing_source_cycle_dynamic_operand_and_missing_module_fail
    with self.assertRaisesRegex(ValueError, diagnostic):
         ~~~~~~~~~~~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^
AssertionError: ValueError not raised

----------------------------------------------------------------------
Ran 1 test in 0.022s

FAILED (failures=2)
error: recipe `test-one` failed on line 56 with exit code 1

```
