# Frozen-reference static preflight: design

**Status:** accepted 2026-08-25; not implemented
**Parent design:** `docs/materials/2026-08-24-v1-parity-design.md`

## Goal

Make the one-time v1 parity gate fail on an invalid frozen reference before
rebuilding or capturing the old native surface. Static metrics must measure a
pane selected independently of the parameter under test.

This remains purpose-built evidence for the pinned v1 reference. It is not a
general visual-test harness, a CI job, or a reusable region-of-interest API.

## Context

The first parity run passed capture integrity but failed all combined static
rows. A reference-only diagnostic spike isolated one analyzer failure:

- duplicate frames were identical;
- the default-to-attenuation response covered 99.5% of a 403 by 378 pane;
- the reference registration nevertheless accepted only one diagnostic-grid
  block;
- that block was on the displaced bevel, so the flat-face filter rejected it;
- direct measurements within geometry-derived insets from 16 through 40
  pixels produced stable attenuation signals.

The response mask itself was not too small. The reference-only
phase-equivalent source search left the displacement field with insufficient
spatial support, while the analyzer allowed the one-block field to proceed.
Using an attenuation variant to select its own measurement region was also a
circular dependency even if that registration had succeeded.

## Decision

Capture an unrendered static source for both implementations and derive a
rectangular pane region from the source-to-default response. Register both
implementations against their captured source. Use the full rectangle for
static displacement and confinement metrics, and its fixed 32-pixel inset for
flat-face attenuation.

Run the frozen reference's nine static rows as a preflight. All nine must pass
before the old native surface is rebuilt or either implementation is captured
for final evidence. A passing preflight is deliberately recaptured during the
full run; no resumable cross-run artifact handoff is added.

## Capture and manifest contract

The static capture order is:

1. Capture `source` duplicates with the diagnostic background visible and the
   material pane absent.
2. Map the same pane geometry and capture `default` duplicates.
3. Capture the nine existing static variants and their return gates.

For each duplicate, the capture computes the exact changed-pixel bounding
rectangle from `source` to `default`. Both rectangles must be nonempty and
identical. The accepted rectangle is recorded per implementation as:

```json
"static_roi": { "x": 0, "y": 0, "width": 1, "height": 1 }
```

The values above show the schema, not fixed coordinates. All fields are
integers derived for that run.

`static.source` becomes mandatory for reference as it already is for native.
The reference-only `static_source` phase-equivalence metadata and special
registration path are removed. The final manifest keeps exact keys and adds
one `static_roi` object to each implementation.

## Analyzer contract

The analyzer validates before producing semantic rows:

- `static_roi` has exactly `x`, `y`, `width`, and `height`;
- every value is an integer, dimensions are positive, and the rectangle is
  within the captured frame;
- the 32-pixel inset leaves a nonempty rectangle;
- duplicate captures remain below the existing 0.1% drift limit;
- source and default duplicates do not reuse files;
- every registered field used by an oracle contains at least nine accepted
  diagnostic-grid blocks.

Static displacement uses the existing registration algorithm with
`static.source` and a mask filled from `static_roi`. Confinement rejects any
variant response outside that rectangle. Attenuation uses the rectangle with
32 pixels removed from each edge; it no longer depends on displacement-field
flatness or an attenuation-derived response mask.

The analyzer gains a reference-static mode that reads the intermediate
reference manifest and emits only the nine static rows. It preserves the
existing exit convention:

- `0`: all rows pass;
- `1`: valid evidence with at least one failed oracle;
- `2`: invalid or incomplete evidence.

Either nonzero result stops the procedure before native build or capture.

## Artifact lifecycle

Host-local zsh environment sets `NIRI_MATERIAL_WORK_ROOT` to the requested
non-synchronized SSD location. Preflight artifacts, final capture artifacts,
and the dedicated old-surface Cargo target must be children of that root.
Project code and documentation use the variable rather than embedding a
machine-specific path.

The execution procedure creates each preflight in a unique temporary
directory. A passing preflight is deleted immediately. A failing preflight is
retained with its printed path while it is the active diagnostic artifact,
then deleted once a later run supersedes it. After final evidence is distilled
and committed, its raw capture is deleted and the dedicated old-surface target
is cleaned with Cargo.

Normal process cleanup remains the replay script's responsibility. It stops
owned compositor, Quickshell, terminal, wallpaper, and anchor processes and
removes their runtime directory on both success and failure.

## Verification

Implementation extends the existing replay and analyzer self-checks rather
than adding another test framework. The checks cover:

- accepted ROI schema and fixed-inset attenuation;
- empty, out-of-bounds, and duplicate-disagreeing rectangles;
- rejection of a field with fewer than nine accepted blocks;
- reference preflight success and both failure exit paths;
- proof that failed preflight cannot reach a native step.

After those checks pass, execution requires a real 9/9 frozen-reference
preflight. Only then may the old native surface be rebuilt and the full parity
capture rerun. Result and design status documents are updated from the new
committed evidence, followed by a user-facing documentation grep for the old
acceptance claim.

## Alternatives rejected

**Hard-coded pane coordinates:** fewer captures, but the evidence would depend
on host placement rather than prove the measured pane for each run.

**Morph the attenuation response mask:** a smaller analyzer diff, but the
parameter under test would still select its own measurement region.

**General ROI or resumable-capture framework:** unnecessary for this frozen,
one-time gate. Recapturing after preflight is simpler and keeps failed
diagnostics separate from final evidence.
