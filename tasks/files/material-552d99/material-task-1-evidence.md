# Journal TERM fix receipts

Original native regression on Ubuntu fails with exit 2 and Bash EOF parser diagnostic; split substitutions preserve 143. Host Bash 5.3 is not red evidence. Ten consecutive real locked-session runs per environment below include production PID/partial-evidence/VT/lock assertions.

## task-1-red.log

```text
F
======================================================================
FAIL: test_term_during_journal_end_preserves_exit_and_cleanup (tools.test_optic_settling.JournalSignalTests.test_term_during_journal_end_preserves_exit_and_cleanup)
----------------------------------------------------------------------
Traceback (most recent call last):
  File "/repo/tools/test_optic_settling.py", line 761, in test_term_during_journal_end_preserves_exit_and_cleanup
    self.assertEqual(run.returncode, 143, run.stderr)
AssertionError: 2 != 143 : environment: trap: line 2: unexpected EOF while looking for matching `)'


----------------------------------------------------------------------
Ran 1 test in 0.006s

FAILED (failures=1)
error: recipe `test-one` failed on line 55 with exit code 1
```

## task-1-host.log

```text
...
----------------------------------------------------------------------
Ran 3 tests in 6.877s

OK
{"iteration": 1, "seconds": 6.513615010000649, "exit": 0, "stdout": ".\n----------------------------------------------------------------------\nRan 1 test in 5.735s\n\nOK\n", "stderr": ""}
{"iteration": 2, "seconds": 6.453462339995895, "exit": 0, "stdout": ".\n----------------------------------------------------------------------\nRan 1 test in 5.690s\n\nOK\n", "stderr": ""}
{"iteration": 3, "seconds": 6.592670318990713, "exit": 0, "stdout": ".\n----------------------------------------------------------------------\nRan 1 test in 5.778s\n\nOK\n", "stderr": ""}
{"iteration": 4, "seconds": 6.520764037006302, "exit": 0, "stdout": ".\n----------------------------------------------------------------------\nRan 1 test in 5.742s\n\nOK\n", "stderr": ""}
{"iteration": 5, "seconds": 6.523763771998347, "exit": 0, "stdout": ".\n----------------------------------------------------------------------\nRan 1 test in 5.735s\n\nOK\n", "stderr": ""}
{"iteration": 6, "seconds": 6.620907357995748, "exit": 0, "stdout": ".\n----------------------------------------------------------------------\nRan 1 test in 5.823s\n\nOK\n", "stderr": ""}
{"iteration": 7, "seconds": 6.61877604899928, "exit": 0, "stdout": ".\n----------------------------------------------------------------------\nRan 1 test in 5.849s\n\nOK\n", "stderr": ""}
{"iteration": 8, "seconds": 6.715323601005366, "exit": 0, "stdout": ".\n----------------------------------------------------------------------\nRan 1 test in 5.942s\n\nOK\n", "stderr": ""}
{"iteration": 9, "seconds": 6.529033812999842, "exit": 0, "stdout": ".\n----------------------------------------------------------------------\nRan 1 test in 5.734s\n\nOK\n", "stderr": ""}
{"iteration": 10, "seconds": 6.620366911010933, "exit": 0, "stdout": ".\n----------------------------------------------------------------------\nRan 1 test in 5.801s\n\nOK\n", "stderr": ""}
```

## task-1-ubuntu.log

```text
...
----------------------------------------------------------------------
Ran 3 tests in 5.289s

OK
{"iteration": 1, "seconds": 4.1418237580073765, "exit": 0, "stdout": ".\n----------------------------------------------------------------------\nRan 1 test in 3.996s\n\nOK\n", "stderr": ""}
{"iteration": 2, "seconds": 4.191879606994917, "exit": 0, "stdout": ".\n----------------------------------------------------------------------\nRan 1 test in 4.042s\n\nOK\n", "stderr": ""}
{"iteration": 3, "seconds": 4.162153326004045, "exit": 0, "stdout": ".\n----------------------------------------------------------------------\nRan 1 test in 4.027s\n\nOK\n", "stderr": ""}
{"iteration": 4, "seconds": 4.286598842008971, "exit": 0, "stdout": ".\n----------------------------------------------------------------------\nRan 1 test in 4.154s\n\nOK\n", "stderr": ""}
{"iteration": 5, "seconds": 4.133360350999283, "exit": 0, "stdout": ".\n----------------------------------------------------------------------\nRan 1 test in 3.984s\n\nOK\n", "stderr": ""}
{"iteration": 6, "seconds": 4.132514643002651, "exit": 0, "stdout": ".\n----------------------------------------------------------------------\nRan 1 test in 3.988s\n\nOK\n", "stderr": ""}
{"iteration": 7, "seconds": 4.160724778994336, "exit": 0, "stdout": ".\n----------------------------------------------------------------------\nRan 1 test in 4.026s\n\nOK\n", "stderr": ""}
{"iteration": 8, "seconds": 4.251153136996436, "exit": 0, "stdout": ".\n----------------------------------------------------------------------\nRan 1 test in 4.108s\n\nOK\n", "stderr": ""}
{"iteration": 9, "seconds": 4.099037471998599, "exit": 0, "stdout": ".\n----------------------------------------------------------------------\nRan 1 test in 3.960s\n\nOK\n", "stderr": ""}
{"iteration": 10, "seconds": 4.18340724700829, "exit": 0, "stdout": ".\n----------------------------------------------------------------------\nRan 1 test in 4.035s\n\nOK\n", "stderr": ""}
```

## task-1-focus-final.log

```text
..
----------------------------------------------------------------------
Ran 2 tests in 5.697s

OK
```

