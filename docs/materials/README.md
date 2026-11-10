# Material documentation

The typed references for the material system. Each states its type and what it
covers in its first paragraph. Upstream niri's own documentation is in `../wiki/`.

## Using materials

- [Material configuration](material-config.md): every `material` and `glass`
  parameter with type, default and range, the optics, presets, window rules and
  validation errors.
- [Material signals](signals.md): how a window signal reaches the glass, the
  `response` blocks, the motion policy and the redraw cost.
- [Material motion](motion.md): the ring of light, focus and attention motion,
  the jelly and follow lag, and the rigid tilt.
- [Shell remap](shell-remap.md): recoloring a shell's opaque layer surfaces to
  match a glass material.
- [Material performance guide](performance.md): where GPU, CPU, memory and power
  go, which parameters are expensive, and the levers that reduce cost.

## How it renders

- [Render pipeline](render-pipeline.md): what runs in what order when a glass
  window is drawn, and which parameter acts at which stage. Read before any
  rendering change.
- [Glass optics](glass-optics.md): what the glass shader computes at a fragment.
- [Pipeline schema](pipeline-schema.md): the glass pipeline as data
  (`resources/materials/pipeline.json`), format version 1.
- [Adding an optic](adding-an-optic.md): the recipe for a new pipeline stage.

## Measuring

- [Captures](captures.md): the `capture.json` record, the lanes, the gates and
  the harnesses under `scripts/capture/`.
- [Capture host setup](capture-host-setup.md): what a host needs before it can
  run the capture lanes.

## The fork

- [Upstream divergence](upstream-divergence.md): what this fork changes relative
  to upstream, the seam inventory, and the rebase procedure.

## Data files

- `upstream-baseline.toml`: the upstream baseline the divergence report
  compares against.
- `upstream-conflicts.toml`: the acknowledged expected conflicts for a rebase.
