# Material signals acceptance evidence

**Status:** passed 2026-09-04 on `feat/material-signals`. The accepted
implementation is `663202b1`; `aaae42a4` pins the Arch package to that source.

This run covers the unit and fixture suites, nested IPC behavior, redraw
cadence and suppression, GPU cost, package contents, and a ten-minute physical
DRM check.

## Unit and fixture suites

| Command | Result |
|---|---|
| `cargo test -p niri-ipc` | 4 unit tests and 1 doctest passed |
| `cargo test -p niri-config` | 63 unit tests and 1 integration test passed |
| `cargo test --bin niri` | command passed; this target contains no tests |
| `cargo test --lib` | 300 fixture and unit tests passed |
| `cargo test -p niri-visual-tests --no-run` | visual test target compiled |

The Arch package build repeated the full check and passed 300 niri tests, 63
config tests, one wiki integration test, four IPC tests, and compiled the
visual test target. Final review added regression coverage for hidden tabs,
offscreen columns, culled workspaces, and duplicate `response=` properties.

## Nested IPC round trip

`material-signals-smoke.sh ipc` exited zero with `ipc: OK (29 events
recorded)`. The retained stream contained the expected Demand set, live and
expired Done impulse states, clear to `null`, and Demand-to-Quiet TTL decay.
Every invalid request failed without changing the folded signal or appending
an event. The run cleaned up its compositor, Weston unit and socket, and
runtime directory.

Retained artifact directory: `signals-2647273-1788412743`.

## GLES redraw behavior

`material-signals-smoke.sh cases` exited zero with `cases: OK`.

| Case | Measured redraws | Acceptance |
|---|---:|---|
| quiet ring | 0 | pass: zero |
| Demand + Pulse | 533 (26.6/s) | pass: 540 +/-15% |
| focused Demand + Pulse | 0 | pass: zero |
| Demand + Breathe | 160 (8.0/s) | pass: 160 +/-15% |
| Demand + Flash | 319 (15.9/s) | pass: 320 +/-15% |
| ten Breathe windows | 160 (8.0/s) | pass: within 10% of one window |
| ten Flash windows | 319 (15.9/s) | pass: within 10% of one window |
| inactive workspace | 0 | pass: zero |
| hidden tab | 0 | pass: zero |
| offscreen column | 0 | pass: zero |
| motion off | 6 | pass: zero in final 14 s |
| reduced Flash | 533 (26.6/s) | pass: within 15% of Pulse |
| attention response `none` | 0 | pass: zero |
| impulse response `none` | 6 | pass: zero in final 14 s |
| Done pulse | 93 in burst | pass: at least 30; zero in final 14 s |
| slowdown 3 | 533 (26.6/s) | pass: within 10% of Pulse |

The run retained 16 nonempty Tracy traces and matching capture logs and CPU
exports. All nested services, sockets, and processes were absent after
cleanup. Retained artifact directory: `signals-3219722-1788427980`.

An exact-source nested visual smoke also exited zero and produced five
distinct screenshots. Retained artifact directory:
`signals-2078145-1788484085`.

## GPU comparison

The host was in active use, and initial reference-first trials showed temporal
variation larger than the 10% gate. The accepted harness therefore rotates
the order by round: reference/default/ring, default/ring/reference, then
ring/reference/default. Fixtures, three captures per path, 20-28 second
window, 14 samples per capture, medians, and the one-sided 10% regression
ceiling remain unchanged.

Three complete balanced trials passed individually:

| Trial | Reference medians (ns) | Default medians (ns) | Ring/rim medians (ns) | Gate |
|---|---|---|---|---|
| 1 | 4096, 4096, 5120 | 3072, 2048, 4096 | 5632, 3072, 4608 | pass |
| 2 | 4096, 5120, 5120 | 7168, 5120, 5120 | 3072, 5120, 5120 | pass |
| 3 | 3072, 5120, 5120 | 5120, 5120, 4096 | 4096, 5120, 4096 | pass |

Across the 126 samples per path, 20% trimmed means were 4581.05 ns for the
base reference, 4783.16 ns for this build's unsignaled default path, and
4459.79 ns for Demand/Static ring plus rim orbit. The default path was 4.41%
above reference, within the 10% ceiling; ring/rim was 6.76% below this build's
default in this sample. The current binary SHA-256 was
`4be54a7002468cdd21693593a8636d5c2bd5b6563d14c559b7449e668aa434eb`
in every trial. Cleanup leaked no service, socket, worktree, or process.

These GPU captures used `38d506f2`. The accepted-source changes after that
commit do not touch shader files: they correct CPU-side visibility scheduling,
reject duplicate configuration properties, complete visual-test call sites,
and pin the final package. The exact-source visual and package checks above
cover those changes.

Retained artifact directories:

- `signals-377360-1788446987`
- `signals-445392-1788447687`
- `signals-483279-1788448189`

## Package and physical DRM

The installed package is `niri-material 26.04.r215.g663202b1-1`. Its archive
contains the same 27 entries and 11 files as the installed package and has
SHA-256
`3e154ef32b03d8b021ec207f3b2539dc611705c570c7be961183edb0fc979c2e`.
The installed `/usr/bin/niri` has SHA-256
`23098121e5b24e39741a7a1d211229807446d39b7e4fc6ea2fb50a809408ec4a`.

After an operator-controlled install and graphical-session restart, both the
CLI and live compositor reported `niri 26.04 (663202b1)`. From 21:28:31 to
21:38:32 US/Eastern, an unfocused floating window held a Demand + Pulse signal
with accent `#e5a33c`. The actively used desktop moved away from its workspace;
eleven samples at one-minute intervals confirmed that the hidden window
remained unfocused, the signal remained intact, and the compositor remained
responsive on the accepted build. Returning the window to the active workspace
preserved the signal and resumed the onscreen Pulse. The compositor journal
contained only normal configuration reloads during the interval, with no
warning, error, or panic. Clearing the `physical-smoke-final` source restored
the window's signal to `null`, and the temporary client was removed.
