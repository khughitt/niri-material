# Portable full tooling validation

Old GIO registration fails on Ubuntu with an AttributeError; established register_object passes all six consumer cases on both environments. Empty merge-tree result regressions fail independently for statuses 0 and 1 before the guard. Focused 80-case host gate passes; Ubuntu pilot (journal, lock, consumer, argv and Cargo isolation), all report/consumer cases and guarded 308-case full run pass with exactly five optional skips (24.093 s). Host full: 308 cases, two optional skips (28.269 s).

## Negative controls

```json
[
  {
    "mode": "--required-skip",
    "exit": 1,
    "seconds": 3.281762457001605,
    "diagnostic": "control missing required binding"
  },
  {
    "mode": "--dynamic-skip",
    "exit": 1,
    "seconds": 26.942087818999426,
    "diagnostic": "unexpected CI skip"
  },
  {
    "mode": "--fail-lifecycle",
    "exit": 1,
    "seconds": 26.733619844992063,
    "diagnostic": "control lifecycle assertion"
  }
]
```

## task-4-consumer-red.log

```text
Traceback (most recent call last):
  File "/repo/tools/fake_screencast.py", line 35, in on_bus
    connection.register_object_with_closures2(path, info.interfaces[iface], handle, None, None)
    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
AttributeError: 'DBusConnection' object has no attribute 'register_object_with_closures2'
F
======================================================================
FAIL: test_startup_sampling_and_summary (tools.test_screencast_consumer.ConsumerEndToEndTests.test_startup_sampling_and_summary)
----------------------------------------------------------------------
Traceback (most recent call last):
  File "/repo/tools/test_screencast_consumer.py", line 185, in test_startup_sampling_and_summary
    self.assertEqual(consumer.stdout.readline().strip(), 'ready 7')
AssertionError: '' != 'ready 7'
+ ready 7


----------------------------------------------------------------------
Ran 1 test in 0.257s

FAILED (failures=1)
error: recipe `test-one` failed on line 56 with exit code 1
```

## task-4-red.log

```text
FFF
======================================================================
FAIL: test_empty_result_tree_is_an_error_for_both_protocol_statuses (tools.test_upstream_report.Conflicts.test_empty_result_tree_is_an_error_for_both_protocol_statuses) (status=0)
----------------------------------------------------------------------
Traceback (most recent call last):
  File "tools/test_upstream_report.py", line 529, in test_empty_result_tree_is_an_error_for_both_protocol_statuses
    with self.assertRaisesRegex(report.ReportError, 'invalid ref diagnostic'):
         ~~~~~~~~~~~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
AssertionError: ReportError not raised

======================================================================
FAIL: test_empty_result_tree_is_an_error_for_both_protocol_statuses (tools.test_upstream_report.Conflicts.test_empty_result_tree_is_an_error_for_both_protocol_statuses) (status=1)
----------------------------------------------------------------------
Traceback (most recent call last):
  File "tools/test_upstream_report.py", line 529, in test_empty_result_tree_is_an_error_for_both_protocol_statuses
    with self.assertRaisesRegex(report.ReportError, 'invalid ref diagnostic'):
         ~~~~~~~~~~~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
AssertionError: ReportError not raised

======================================================================
FAIL: test_focused_recipe_preserves_arguments_and_counts_stderr (tools.test_gates.Gates.test_focused_recipe_preserves_arguments_and_counts_stderr)
----------------------------------------------------------------------
Traceback (most recent call last):
  File "tools/test_gates.py", line 103, in test_focused_recipe_preserves_arguments_and_counts_stderr
    self.assertEqual(result.returncode, 0, result.stderr)
    ~~~~~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
AssertionError: 127 != 0 : tt: host-budget: command not found
error: recipe `test-one` failed on line 56 with exit code 127


----------------------------------------------------------------------
Ran 2 tests in 0.117s

FAILED (failures=3)
error: recipe `test-one` failed on line 56 with exit code 1
```

## task-4-forward-red.log

```text
F
======================================================================
FAIL: test_focused_recipe_records_failure_through_host_budget (tools.test_gates.Gates.test_focused_recipe_records_failure_through_host_budget)
----------------------------------------------------------------------
Traceback (most recent call last):
  File "tools/test_gates.py", line 122, in test_focused_recipe_records_failure_through_host_budget
    self.assertEqual((record['exit'], record['tests']), (7, 1))
    ~~~~~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
AssertionError: Tuples differ: (127, None) != (7, 1)

First differing element 0:
127
7

- (127, None)
+ (7, 1)

----------------------------------------------------------------------
Ran 1 test in 0.096s

FAILED (failures=1)
error: recipe `test-one` failed on line 56 with exit code 1
```

