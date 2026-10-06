# Render anomalies: attribute causes before choosing fixes

Scope pass: 2026-10-06. Handoff, not an approved renderer design.
Goal: `material-7ff4bc`, an ordinary child of the evidence-instruments lane.

## Problem

Resolve two remaining rendering observations: recurring GL errors
(`material-c8732f`) and a small dark-channel lift under blurred backdrop
grain (`material-d1171f`). Both need attribution, but no evidence links
their causes. Keep their investigations independent.

## Current behaviour and evidence

- The original GL report records repeated `GL_INVALID_VALUE` / "Size and/or
  offset out of range" messages on 2026-09-05. A read-only journal check
  found the same signature four times on 2026-10-06, at 15:53:19, 17:48:47,
  18:24:55 and 19:11:45 UTC. The emitting compositor's startup log reports
  version 26.04 (`310b4e30`); this is a version label, not a verified binary
  hash/source snapshot. The narrow final-error interval contains only the
  GL callback message, with no caller or action attribution. IPC lookup from
  this shell failed because its exported socket path was unavailable.
- `git ls-tree 310b4e30 -- src/render_helpers/grain.rs` contains no grain
  pass file. Current HEAD does contain it. Do not attribute the installed
  error to newer code merely because that code also uses raw GL. The current
  grain regressions check incomplete framebuffers, draw errors and cleanup;
  they establish neither the historical error's cause nor its resolution.
- [Noise placement evidence](../materials/2026-10-05-noise-placement-evidence.md#nested-smoke)
  (`07b2c196`, `be099a65`) records blue +1 code on 44,066 of 80,000 face
  pixels: mean +0.55, blue spread 0.50 codes, red/green unchanged. The
  blurred roughness-1 residue is mostly DC; the sharp roughness-1 control
  has zero residue. Clipping fits the sign, but blur rounding is not excluded.
- `src/render_helpers/shaders/material/noise.frag::noise_source` explicitly
  clamps straight grained color to [0, 1], then premultiplies. The grain
  target in `src/render_helpers/effect_buffer.rs` is `Abgr8888`, as required
  by the [accepted placement design](../specs/2026-10-05-noise-placement-design.md)
  §4. A wider target alone would not remove the explicit clamp. Existing
  `src/tests/noise_site.rs` renders deterministic pixels with frozen clocks;
  `glass-noise-site-smoke.sh::signed_diff` was corrected to site minus zero
  (`ff8e4a99`). Neither existing check isolates the lift's mechanism.

## Constraints

Read [render-pipeline.md](../materials/render-pipeline.md) before renderer
changes. Preserve alpha, opaque-client identity, neutral output and cache
invalidation. No speculative GL guards, callback suppression or production
format changes follow from this pass. Pin build/renderer provenance and keep
channel bias distinct from residual grain. Start with one case through its
verdict before a longer matrix. Prefer in-process controls; nested reproduction
uses the declared `nested` need. Timing/power runs require separate quiet
evidence; no desktop replay, service restart or host pointer change is authorized
by these investigations. The pending noise-layers branch is unnecessary.

## Alternatives

1. **Current lean: bounded attribution first.** Preserve the GL recurrence
   record, find a call/argument-level reproducer, and isolate grain clipping
   versus quantization with flat-field GPU controls and an offline reference.
2. Add size clamps or widen the grain format now. Defer: the offending GL
   call is unknown; the clamp/format/blur chain and driver/memory implications
   need evidence and a reviewed contract before a change.
3. Accept the small color bias as documented 8-bit behavior. This may be
   reasonable once its cause and bounds are established and the owner judges
   the look; the existing grey grain-floor assertion is not that acceptance.

## Unanswered questions

- Which call, arguments and stimulus produce the GL error? `material-86c841`
  traces the pinned renderer and attempts one minimal isolated reproduction.
  Its result may instead report ranked candidates and the next falsifiable check.
- Does clipping alone explain the blue lift, or does downstream rounding
  contribute? `material-aaa714` compares dark/midtone/upper-bound controls,
  zero/backdrop/glass, blur and roughness, with a signed offline reference.
- Is a supported bias acceptable, or worth a clamp/format change? Reuse
  `material-674d4e` for owner judgement; format support, cost and the new
  color/alpha contract remain questions for any subsequently justified design.

## Proposed decomposition

- `material-c8732f` and `material-d1171f`: **briefed**, with their original
  bodies preserved and previously absent parents set to `material-7ff4bc`.
- `material-86c841`: P2 / s / high / direct, `nested` need; bounded GL
  provenance/caller attribution. It wakes `material-c8732f` with a finding
  note and updates this brief in the same result commit. No renderer fix.
- `material-aaa714`: P2 / s / mid / direct; deterministic grain-bias control
  study using existing fixtures, with no quiet-host benchmark or format change.
  It wakes `material-d1171f` with findings and updates this brief in the same
  result commit. Existing owner-look work is retained, not duplicated.
- Both follow-ups belong to the shared goal under the existing instruments
  lane. A separate fix or design is filed only when its cause and check are
  established; no new renderer subsystem or lane is needed.
