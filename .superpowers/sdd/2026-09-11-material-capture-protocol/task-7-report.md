# Task 7 report: optic smoke capture metadata adoption

## Result

The shared optic smoke library now records capture identity, settles before every nested
launch, releases capture ownership during cleanup without masking the run's result, and
hashes every artifact except the manifest itself. Aurora and iridescence preflight
immediately after sourcing, identify their binaries and pinned scene configuration after
building, and require the caller to supply `CAPTURE_TASK`.

## TDD and checks

- Red: the focused suite reported four planned adoption failures; added execution checks
  also failed for missing capture functions and missing cleanup release.
- Green: `tools.test_glass_optic_smoke` passed 8 tests; `bash -n` passed for the library
  and both entry scripts.
- Hook: `just check` is run by the commit hook and recorded below after commit.
- Commit: this report lands with `feat(material): optic smokes record captures through capture-meta`.

## Review

All callers of the shared library and `start_nested` were traced. Preflight remains in
the two native entry scripts so the offline prepare consumer can source the library, and
the shared launch path performs every settle. No build or hardware capture was run.

Concerns: none.
