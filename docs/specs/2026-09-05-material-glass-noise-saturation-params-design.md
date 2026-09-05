# Glass noise and saturation parameters: design

**Status:** implemented and merged into `materials-26.04` at `6377515f`
(native piece `material-1293e8`), `just check` and `just test` passing, nested GLES evidence
in
[`2026-09-05-material-glass-noise-saturation-params-evidence.md`](../materials/2026-09-05-material-glass-noise-saturation-params-evidence.md).
Hub goal `prism-63dd45`; the Prism piece `prism-d0d4cb` follows once this
build is installed.

**Task:** `material-1293e8`

## Context

Native commit `7c702e58` composes `noise` and `saturation` in `material.frag`
after the glass optics, but the only source for those two values is the global
`blur` block, and only while backdrop blur is effective
([`2026-09-02-material-noise-saturation-design.md`](2026-09-02-material-noise-saturation-design.md)).
That design listed glass-specific knobs as a non-goal because nothing needed
them yet.

Prism now renders two materials per terminal set, `terminal-glass` and
`terminal-glass-inactive`, assigned by `is-active`
([`2026-09-04-focus-glass-spike.md`](../materials/2026-09-04-focus-glass-spike.md)).
Every optic that makes an unfocused pane recede is a matrix row with a focused
and an unfocused value. Noise and saturation are natural rows on that matrix
and cannot be expressed today: the global blur block is one value per
compositor, and Prism ships `backdrop-blur false` by default, so its glass
receives neither effect at all.

The per-window `background-effect { noise; saturation }` route was considered
and rejected. Prism pins that pass inert for material windows on purpose. It
would compose a second image beneath the material instead of after its optics,
and the backdrop-blur design already rejected restoring it.

## Decision

`glass { }` gains two optional parameters:

| Parameter | Type | Range | Omitted |
| --- | --- | --- | --- |
| `noise` | float | 0–1 | inherit |
| `saturation` | float | 0–3 | inherit |

A written value is a material optic. It renders as written regardless of
`backdrop-blur` and regardless of `blur { off }`. The global blur switch turns
off blur; these two parameters are not blur.

An omitted value keeps today's behaviour exactly:

```text
effective backdrop blur = glass.backdrop-blur && !blur.off
noise      = glass.noise      ?? (effective ? blur.noise      : 0)
saturation = glass.saturation ?? (effective ? blur.saturation : 1)
```

The ranges are the meaningful ones for the shader, not the loose `0–1000`
the blur block accepts. `noise` is a centered amplitude added in sRGB units;
`saturation` is the `mix(gray, color, s)` factor, where 1 is neutral and
values above 1 oversaturate. Out-of-range values are parse errors like every
other glass parameter.

## Configuration contract

`niri_config::material::Glass` gains `noise: Option<FloatOrInt<0, 1>>` and
`saturation: Option<FloatOrInt<0, 3>>` as `child, unwrap(argument)` fields,
matching `roughness`.

`ResolvedGlass` gains `noise: Option<f64>` and `saturation: Option<f64>`.
Every other resolved field holds a final value because every other parameter
has a fixed default. These two do not: their default is "inherit", which is
only decidable once the global blur block is known, and that happens in the
layout, not in the config crate. `Option` is the honest representation of
that; `resolve` copies the written value through and leaves omission as
`None`. Introducing an enum for a two-state default would be ceremony.

The material grammar in `docs/materials/material-config.md` documents both
parameters in the table and rewrites the inheritance paragraph to distinguish
the written case from the omitted case.

## Renderer ownership

`resolve_material` in `src/layout/tile.rs` already computes the effective
backdrop-blur gate and the `(noise, saturation)` pair for
`MaterialRenderConfig`. It becomes the single place that applies the
`??` rule above. Nothing downstream changes: `MaterialState`,
`apply_resolved`, the commit-counter damage on a changed pair, the two
uniforms and the shader's neutral branches all consume the same
`MaterialRenderConfig` they consume today.

No shader change. No `EffectBuffer`, roughness pyramid, blur or postprocess
change. No new texture, pass or allocation.

## Prism integration

Prism always writes both parameters into both of its material definitions, so
its glass no longer depends on inheritance. The exact definitions, panel rows
and defaults are Prism's spec. The native contract Prism relies on is only the
table above and the "written value always applies" rule.

The generated fragment fails `niri validate` on a build without these
parameters. Prism's piece therefore depends on this one landing and being
installed first.

## Documentation updates when implementation lands

- `docs/materials/material-config.md`: table rows and the inheritance
  paragraph.
- `docs/specs/2026-09-02-material-noise-saturation-design.md`: a status line
  noting that the "no glass-specific parameters" decision is superseded here.
  The rest of that document stays as the record of the composition it
  designed.
- `docs/materials/2026-08-29-material-backdrop-blur-design.md`: one sentence
  in the noise-and-saturation follow-up pointing here.
- `docs/materials/README.md`: index entries for this spec and its evidence.
- This document's status header, and the task ids above once filed.

## Verification

GPU-free:

- config parse accepts `noise 0.02` and `saturation 0.85`, rejects
  `noise 1.5` and `saturation 4`, and resolves omission to `None`;
- `resolve_material` resolves each parameter independently. The matrix
  covers, against a non-neutral global block (`noise 0.02`,
  `saturation 1.5`): both written; both omitted with backdrop blur effective
  (the global pair); both omitted with it off, and with `blur { off }`
  (`(0, 1)`); noise written and saturation omitted; saturation written and
  noise omitted; and explicit neutral values (`noise 0`, `saturation 1`)
  overriding the non-neutral globals while backdrop blur is effective. A
  written value must survive `backdrop-blur false` and `blur { off }` in every
  case. An implementation that treats the pair as all-or-nothing must fail
  this matrix;
- `just check` and `just test` pass. They are the repository's required
  entry points and enforce suite selection, timing records and the commit
  checks; bare `cargo test` does not;
- Prism's suite passes against the new grammar (recorded on its side).

Nested GLES smoke on the headless Weston unit, never the desktop session:

- a material with `backdrop-blur false`, `saturation 0` renders a text-free
  glass ROI grayscale;
- the same material with `noise 0.5` measurably raises ROI variance against
  `noise 0`;
- a material with both omitted and `backdrop-blur false` is byte-identical to
  its pre-change capture.

The smoke reuses the capture and metric scripts retained from
`2026-09-02-material-noise-saturation-evidence.md` where they still apply, and
records its result as `docs/materials/2026-09-05-material-glass-noise-saturation-params-evidence.md`.

## Alternatives rejected

### Per-window background-effect

Revives the pass Prism deliberately pins off, composes beneath the glass rather
than after it, and was already rejected by the backdrop-blur design.

### Gate written values on backdrop blur

Keeps the frosted-look framing but makes the new sliders inert until the user
enables Frosted backdrop, which Prism ships off. A written parameter that
silently does nothing violates the repository's explicit-over-defensive rule.

### Mirror the blur block's 0–1000 ranges

Consistency with the blur block buys nothing; the shader has no meaningful
behaviour beyond the tighter ranges, and the parser exists to reject
meaningless values.

## Non-goals

- Changing what the global blur block means or how the background effect
  resolves it.
- Additional noise types (Prism idea `prism-d6b600`).
- Layer-surface materials.
- Performance instrumentation for two scalar uniforms that already exist.
