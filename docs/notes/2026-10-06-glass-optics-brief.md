# Glass optics: use the existing light path before extending it

Scope pass: 2026-10-06; updated 2026-10-10 for the edge-cohesion spike. Handoff, not an
approved design. Existing lane: `material-6062fd`.

## Problem

Make dark glass edges, the travelling ring and terminal content read as one
optical material. Lane ideas cover edge grain, ring embedding, content
depth, ring brightness, alternating pane offsets and rim width on low-ior
glass (`material-295e10`); `material-d0518d` (focus light beyond the beam,
unparented) waits on the same edge and tilt results.

## Current behaviour and evidence

- `src/render_helpers/shaders/material/main.frag` refracts the ring's landing
  point at 20% slab thickness, scatters it with roughness, attenuates its
  light and spills the moving beam onto the chamfer. Ring embedding is a
  residual appearance question, not an absent light path.
- Film-site grain (`fa8a447e`, merged) runs through `noise_post` after
  attenuation and additive light, before coverage and client compositing.
  It reaches glint and ring light even on dark glass. Glass-site grain
  affects transmission. Film grain is neither bevel-local microtexture nor
  a perturbation of surface normals.
- Opaque client pixels still return immediately. Current film grain cannot
  implement shared film over client content; `main.frag` composites
  `win + (1 - win.a) * glass`. The [v1 contract](../materials/2026-08-22-v1-design.md)
  preserves opaque content.
- `material-1d70db` shipped `ring-rest` at `7d240563`; gain still defaults
  to 1. `material-519eeb` pins the owner's gain-1.2 profile in
  `src/tests/ring_look.rs`. Profile acceptance does not decide the stock default.
- `material-be611b` merged (`23a2e37b`): height-field bevel, reflection and
  edge highlight. Its pick was judged on a look whose bevel transmits; the
  live active look (thickness 31.2, `#4c563a` over 9 px) transmits ~2e-4
  across the whole bevel, so the edge reads as a dark frame
  (`material-f4143a` note, 2026-10-09).
- 2026-10-09 diagnosis: at rest the ring's bevel spill is zero, since
  `main.frag` scales it by `moving` alone, so the line never touches the
  edge (`material-58a1a1`). Glint and reflection ride raw Schlick, whose f0
  is 1.5 % at ior 1.28 and lights one or two pixels (`material-295e10`).
  `material-f72d5c` (edge light) now proposes a rim profile in place of raw
  Schlick for the reflection optic. Spike `material-ac926f` dumped both
  probes; its sheet is parked for the owner's verdict.

## Constraints

Follow [render-pipeline.md](../materials/render-pipeline.md). Preserve
premultiplied alpha, default/neutral output, the accepted ring reference and
settled redraw gating. Use in-process frozen-clock GLES fixtures, without
taking the live desktop. The single existing noise setting can compare
glass and film; pending noise layers are unnecessary. Content depth and
inactive desaturation (`material-987655`, owned by another goal) need one
coordinated opt-in content boundary. Keep existing parents and source material.

## Alternatives

1. **Compare existing controls first (current lean for edge grain and ring
   embedding).** Test film versus glass grain on the accepted bevel; add
   surface-only microtexture or a new ring model only if a concrete gap remains.
2. Extend bevel-local reflected light or normals immediately. This could
   provide directional surface texture that film cannot, but its need is
   unestablished until the existing-control comparison and owner judgement.
3. For content treatment, design explicit opt-in depth (G2, the idea's current
   lean) against shared film/tone (G1) and deferring content treatment. G3
   halation and G4 sidechain require an additional blurred client source;
   defer that bundle. Current film placement cannot resolve the client-content
   contract.

## Unanswered questions

- Do film grain and existing ring optics look sufficient on the accepted
  bevel? `material-f4143a` produces controlled evidence; the owner judges
  appearance. Additional clocks or optics are not presumed necessary.
- Which content treatment preserves text readability, and which client,
  popup and capture surfaces should opt in? `material-d257d9` frames the
  contract for written design and plan review by the owner. Its acceptance
  tests must distinguish transparent glass from opaque client text.
- Does the spike's sheet read as one slab? Only the owner can say
  (`material-ac926f`). A yes starts `material-f72d5c` and `material-58a1a1`;
  a no drops both.
- Should the glint share the edge light's rim weight? `material-295e10`
  asked for a rim width on both glint and reflection; `material-f72d5c`'s
  design covers reflection and is where the glint decision belongs.
- `material-2592a2`: after daily use, retain native ring-glow 1 or raise it
  to 1.2? Only the owner can judge the default. Recommendation: keep stock
  1 and the accepted profile override unless normal use establishes a stock
  visibility problem.

## Proposed decomposition

- `material-f4143a`: P2 / s / mid / direct, child of the existing optics
  lane; depends on the accepted merge of `material-be611b`. It wakes
  `material-1aa3af` and `material-4e3e9c` with result notes in the same
  commit. Both remain briefed ideas. Its matched sheet must include the
  live active and inactive looks.
- `material-ac926f` (spike, parked on the owner) gates `material-f72d5c`
  (edge light, planned) and `material-58a1a1` (resting spill, direct).
  58a1a1 is the residual gap `material-4e3e9c` named; noise and frost
  interaction still waits on `material-f4143a`.
- `material-295e10`: proposed drop into `material-f72d5c`, whose design takes
  the glint's rim weight too.
- `material-d0518d` is shelved until the edge light and the tilt
  (`material-abc08c`) are seen on the live desktop.
- `material-d257d9`: P2 / m / high / planned, child of the same lane;
  written spec and plan reviews precede implementation. It wakes
  `material-7f5751` and related `material-987655` without changing their
  parents. Design can begin before the bevel merge; depth implementation cannot.
- `material-2592a2` remains an idea with one Open questions heading and its
  existing owner need. No extra gain-control implementation task.
- `material-933a8b` is shelved. Wake after the bevel is accepted and merged
  and the owner requests a concrete alternating-column look; then settle
  column insertion/reorder and floating-window semantics. General
  zebra-striping controls stay deferred.
