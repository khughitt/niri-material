# Noctalia glass cohesion and workspace project hue

2026-10-16. Design for `material-62347d` under `material-55380c`. Status:
accepted for planning 2026-10-17 (review round 3). Scope brief:
`docs/notes/2026-10-14-noctalia-material-brief.md`.

## Intent

The owner's words (2026-10-15): beyond a static variant of the material, the
shell surfaces should take the hue of the workspace's active coding-agent
terminal, the familiar hue its glass already carries, so the whole workspace,
not only the terminal, reinforces the project being worked on.

The owner's verdict (2026-10-16): column C of the `material-b156e7` sheet
looks viable. In C the bar and panels share the glass's darkness and hue: the
surface family is moved to the glass face's color, the bar and attached panels
are at 0.95 opacity, and floating panels are solid.

What this design delivers:

1. **The look.** The live bar and panels read as C does, and keep reading that
   way with no manual step when the wallpaper changes or the glass changes
   within the supported range (dark glass, section 2). The shell matches the
   glass over a reference backdrop. The terminal brightens over bright
   wallpaper and the opaque shell does not follow it.
2. **The project hue.** Each output's bar and panels take the hue of the
   project its active workspace is working on, and change with it.

Success: the owner judges a nested sheet in which the new mechanism's neutral
state is indistinguishable from C, and its hued states read as the project's
hue without hurting legibility. On the live desktop, switching between
workspaces of two projects changes the shell's hue with a crossfade. Nothing
else on the desktop changes: GTK, Qt, terminal and editor colors do not move.

Assumptions (not the owner's words, open to correction in review):

- "Familiar-based hue" means the accent niri already folds into
  `Window.signal`, the same value the window's glass shows, whichever source
  wrote it (brief, Constraints).
- The surfaces in scope are the bar, the attached panels and the floating
  panels. OSDs, toasts and the dock can opt in later through the same rule.
- The hue means hue only. The agent's level, motion and impulses stay on the
  window.

## What changed since the brief

When I reported C's verdict, I proposed carrying the hue in C's custom palette:
switch the palette whenever the held hue changes. Reading Noctalia 5.2.1's
source and the live configuration rules that out, and with it any design that
keeps C's look in a Noctalia palette:

- **A palette change fans out across the desktop.** Every change to the
  resolved palette runs every enabled template, then fires `colors_changed`
  (`src/app/application_services.cpp`: `setResolvedCallback` →
  `TemplateApplyService::apply` → `HookKind::ColorsChanged`). The live
  configuration enables about 17 templates: GTK 3 and 4, Qt, niri, ghostty,
  kitty, btop, zathura, bat, yazi, nvim with a post hook, and the Claude,
  Codex, ohai, lsd, fastfetch and Prism outputs. `colors_changed` then runs
  `prism apply niri`, which reloads niri's config. A hue change per workspace
  switch would rewrite and reload all of that every time.
- **C's palette restyles more than the shell.** C's custom palette also feeds
  those templates, so moving its surface family to glass black would darken
  GTK and Qt applications and every themed tool, not just the bar and panels.
  The owner judged the bar and panels, not a desktop-wide restyle.
- **A custom palette closes a feedback loop.** Under Prism's default
  `glass.tintSource = noctalia`, the glass's attenuation color is computed from
  Noctalia's surface (`integrations/niri/render.js`, `sourceColors`). Once the
  surface is computed from the glass, each depends on the other. The live
  source is `familiar`, but the default is not.
- **Palettes are not hot-reloaded.** Noctalia watches its config directory and
  settings file but not `palettes/`, so a regenerated palette also needs
  `noctalia msg config-reload`, which runs every reload callback.

Two findings point the other way:

- **The glass face is predictable.** On the flat face, what the glass shows is
  `linearToSrgb(B · T · (1 − f0) + 0.15 · f0)`, where `T` is the attenuation
  color raised to `thickness / attenuation-distance` and `f0` is Schlick's
  reflectance at the configured IOR (`main.frag`: reflection is zero on the
  face and the facing term vanishes there). With the live `terminal-glass` and
  a neutral backdrop `B` of 0.2 (linear), this gives 8,10,10. `material-b156e7`
  measured 8,11,10. The backdrop still matters: the same glass computes to
  7,7,7 over black and 9,17,18 over white, and lighter glass diverges much
  further (section 2).
- **niri already recolors client pixels.** `ClippedSurfaceRenderElement`
  wraps a client's `WaylandSurfaceRenderElement` in a custom texture program.
  A layer surface can be wrapped the same way.

## Decision

niri recolors the Noctalia surfaces itself. A layer rule names Noctalia's
surface family and a glass material. niri remaps pixels in that family to the
glass face's color, which it computes from the material, plus the project hue
held for the output. Noctalia keeps its wallpaper palette and needs no new
hook. Its only change is three static settings: bar opacity, panel mode and
corner radius.

Rejected:

- **C's custom palette, switched for each hue.** It fans out across the whole
  desktop on every change, restyles more than the shell, and loops under
  Prism's default tint source (see above).
- **C's palette kept static, with niri drawing the hue beneath the client.**
  At C's 0.95 opacity, a tint under the client shows through at most 5%.
  Lowering the opacity until the hue shows lets the bright wallpaper through
  as well, which undoes the darkness the owner approved, and the optics would
  have to be retuned and judged again. The template fan-out problem also
  remains for the static palette.
- **Hue only on plugin widgets.** This works today but leaves the built-in bar
  and panels neutral, which is narrower than the request.
- **An upstream Noctalia runtime palette override.** It would be clean, but it
  isn't ours to ship. This design doesn't need it, so proposing it stays the
  owner's call.

Cost of the decision: a client-pixel shader on a few layer surfaces, and
remapping by color proximity rather than by exact palette role. The second is
the main risk, so the owner's sheet gates the design (Verification).

## Design

### 1. Static Noctalia settings

These three settings belong to C, live in Noctalia's settings layer
(`settings.toml`), and change once at rollout. Changing them runs no
templates:

- `[bar.default] background_opacity = 0.95`. Attached panels follow the bar.
- `[shell.panel] transparency_mode = "solid"`.
- `[shell] corner_radius_scale = 1.6`, so panel corners match the windows'
  20 px (12 px × 1.05 UI scale × 1.6).

Noctalia's palette stays `source = "wallpaper"` with its live scheme.

### 2. Glass face model

A pure function in `src/render_helpers/material/` computes the face color of a
`ResolvedGlass`:

```
T    = clamp(attenuation-color, 0.001, 1) ^ (thickness / attenuation-distance)
f0   = ((ior − 1) / (ior + 1))²
face = linearToSrgb(B_ref · T · (1 − f0) + 0.15 · f0),   B_ref = 0.2 (neutral, linear)
```

The two constants mirror `main.frag`'s flat-face terms. A test checks the
shader source for them, as `ring.rs` already does for the ring constants, so
changing the shader without updating the model fails. Noise and saturation
are left out: the noise is zero-mean, and saturation leaves a neutral backdrop
unchanged.

**A reference swatch, not the live glass.** The result is the face over
`B_ref`. The real terminal shows its actual backdrop through the glass, and the
opaque shell can't follow that, so the match holds only where the backdrop
term is small. Glass is supported only when no channel transmits more than
1%:

```
max(T) · (1 − f0) ≤ 0.01
```

Each channel of the transmitted light is the backdrop's channel times that
channel of `T · (1 − f0)`. So under this check, every backdrop, colored or
neutral, passes at most 1% of each channel and of its luminance through the
face. A neutral measure such as `Y(T)` can't promise that. For example,
`#000080` at thickness 30, distance 10 and IOR 1.28 has `Y(T)` under 1%, yet
passes 12.5% of a blue backdrop. The max-channel check rejects it.

**Lightness error within the range.** The error grows as the specular floor
`0.15 · f0` shrinks, so it depends on IOR. niri accepts IOR in [1, 3]. For
neutral backdrops between 0.05 and 0.5 (linear), the face's offset from the
swatch at the 1% bound is:

| IOR | OKLab L offset at backdrop 0.05 / 0.5 |
| --- | --- |
| 1.0 (worst case) | −0.0466 / +0.04501 |
| 1.1 | −0.038 / +0.042 |
| 1.28 (live) | −0.022 / +0.032 |
| 1.5 | −0.013 / +0.022 |
| 3.0 | −0.004 / +0.008 |

The guaranteed bound is the worst row: within −0.047 and +0.04501 OKLab L
across the supported range. That case is a neutral `T` at the bound with IOR 1
(for example `#191919`, thickness 20, distance 10: −0.046 / +0.044). The live
glass is at −0.007 / +0.013. The worst case is established numerically, not
assumed: a test samples accepted configurations across colors, distances,
thicknesses and IORs, and checks every sample against the guaranteed bound
(Verification).

| Glass | max(T) · (1 − f0) | Face over black / 0.2 / white |
| --- | --- | --- |
| live `terminal-glass` | 0.004 | 7,7,7 / 8,10,10 / 9,17,18 |
| same color, distance 20 | 0.035, out of range | 7,7,7 / 13,24,24 / 28,53,54 |
| `#dfe8ff` (Prism's default tint) at distance 30, live thickness and IOR | 0.98, out of range | 7,7,7 / 116,118,123 / 238,243,254 |

**Outside the range.** niri rejects a `surface-remap` naming out-of-range
glass at config load:

```
surface-remap: material "<name>" transmits <max(T)·(1−f0)> in its most open channel;
the shell match supports dark glass up to 0.01
```

Through Prism, this makes the apply fail with that message. The fix is to
darken the glass or turn off `shell.remap` (section 6). Rejecting the rule
means a Prism glass edit that crosses the range fails loudly, instead of the
shell quietly turning a light color under dark text.

### 3. Remap

**Inputs.** The rule gives three source colors, Noctalia's `surface`,
`surface_container` and `outline_variant`, together called the family. The
target is the face color from section 2. The target family follows the
`noctalia-glass-palette.py` rule:

- the surface takes the target's L, C and h;
- the container and outline keep their L and C offsets from the source
  surface, and take the target's hue.

A hue (section 4) only changes the target's C and h.

**Per pixel.** The program runs on the layer surface's texture, before the
texture is composited:

1. Unpremultiply the pixel and convert it to OKLab.
2. Measure its distance `d` to the polyline from surface through container to
   outline in OKLab, and project the pixel onto that polyline.
3. Interpolate the anchors' offsets (target anchor minus source anchor) at the
   projected point, and apply them with weight
   `w = 1 − smoothstep(r0, r1, d)`.
4. Convert back, clamp to gamut, and premultiply.

**What this does to Noctalia's other colors:**

- Exact anchor pixels reproduce `noctalia-glass-palette.py`'s output.
- Low-chroma blends near the family, such as hover overlays of `on_surface` at
  low alpha, move with it, much as they would under C's palette.
- Hover and accent overlays drawn at low alpha over the surface (up to about
  25%) sit 0.002 to 0.015 from the polyline, so they move with the family, as
  they would under C's palette.
- Text and icons (`on_surface`, `on_surface_variant`, L ≥ 0.8), accents
  (`primary` and the rest) and `error` sit 0.12 to 0.55 away and stay
  unchanged.
- Client shadows unpremultiply to black, 0.19 away from the surface anchor,
  so they are untouched.
- The chip text colors cannot be separated from the overlays.
  `on_secondary` sits 0.015 from the polyline and `on_primary` and
  `on_tertiary` 0.04, overlapping the overlays' range. They may move
  partially. They are dark text on light chips, and the shift (at most 0.05
  darker, hue toward the glass) keeps their contrast. The sheet judges it.

The radii `r0` and `r1` are constants (starting values 0.008 and 0.02).
Tests pin them against the live wallpaper's palette and two other m3-content
palettes:

- every family anchor at weight 1;
- `on_surface`, `on_surface_variant`, the accents, `error` and black at
  weight 0;
- surface overlays of `on_surface` and `primary` at up to 16% alpha at
  weight ≥ 0.5.

**Per-frame uniforms.** niri computes these on the CPU: the three source
anchors and their offsets in OKLab, the two radii, and the current target
after the crossfade. The shader does no color science beyond the OKLab
conversions.

### 4. Project hue

**Resolver.** A pure, per-workspace function in `src/layout/` decides which
hue each workspace holds:

- A workspace's active window settles once it has stayed active for the hold
  dwell, 1000 ms by default. The binding owns this timer; it is not the window
  switcher's debounce, and focus timestamps play no part (brief, review
  fix 1).
- When a settled window carries an accent, the workspace records it as its
  hue source, keyed by window id.
- The workspace shows the recorded window's current accent. When that accent
  changes, the shell follows. The record is cleared when the window closes,
  leaves the workspace or loses its accent, and the workspace goes neutral.
- A settled window without an accent does not replace the record. That is the
  hold: after agent A, a long look at a browser keeps A's hue, and a 300 ms
  pass over agent B on the way somewhere else does not count.
- A workspace with no record is neutral, meaning the static look of section 3
  with the face color unhued.

**Per output.** An output shows its active workspace's hue. This comes from
`Monitor::active_window()`, which doesn't depend on keyboard focus, so an open
panel or an unfocused output keeps its hue (brief, `material-6b1c0b`).

- During a workspace switch, the output blends the current target families of
  the two workspaces it shows, by the fractional
  `Monitor::workspace_render_idx()`, in OKLab.
- The overview shows the active workspace's hue.
- Every layer on an output reads that one blended point, so the whole output
  shows the same hue at the same moment.

**Targets come first.** A hue state resolves to a target family before any
blending: the surface, container and outline target colors in OKLab.

- *Neutral:* the face color, and the container and outline at their offsets
  (section 3).
- *Hued:* the target surface keeps the face's L, takes the accent's OKLCH hue,
  and takes chroma `hue-chroma`. The container and outline follow at their
  offsets.

Each workspace's crossfade, and the switch blend, interpolate these target
families linearly in OKLab and never touch the accents. Linear OKLab
interpolation is continuous at every fraction: red to cyan passes through a
near-neutral family, not across an undefined hue, and neutral to hued fades
the chroma. Accents are never interpolated in RGB and then reduced to a hue:
that path snaps at the gray midpoint (the round-1 review computed `#130505` at
49.9% and `#000d0d` at 50.1%).

Because a dark face shows almost no hue of its own (section 2), the hue isn't
taken from the glass. `hue-chroma` is an art-directed strength, and the owner
picks it at the sheet gate.

**Crossfade.** Each workspace's resolver state owns one crossfade over target
families. When the workspace's target changes, a fade starts from the current
interpolated family, so an interrupted fade continues from where it is. The
duration and curve are `animations.material-signal`, the same setting the
tile's accent crossfade uses, so the shell and the terminal's glass fade
together. The tile's `SignalCrossfade` isn't reused: it interpolates accents,
and the shell must interpolate targets. While any fade or switch blend runs on
an output, its remapped layers count as animating.

### 5. Configuration

The layer rule gets a new node:

```kdl
layer-rule {
    match namespace=r#"^noctalia-(bar-[^"]+|attached-panel|panel)$"#
    surface-remap {
        surface "#151310"      // Noctalia surface
        container "#221f1c"    // Noctalia surface_container
        outline "#4d453c"      // Noctalia outline_variant
        material "terminal-glass"
        hue-chroma 0.03        // 0: static look, no project hue
    }
}
```

- Every node is complete on its own. `surface`, `container`, `outline` and
  `material` are required in each node, and a missing one fails config load.
  The other config-load failures:
  - an unknown material, with the existing `unknown material:` error;
  - glass outside the supported range (section 2);
  - a source `surface` lighter than OKLab L 0.4, because the remap supports a
    dark Noctalia scheme.
- `hue-chroma` is in [0, 0.1] and defaults to 0.
- The remap applies to the layer surface and its popups. Noctalia's menus and
  tooltips draw on the same family, and keeping them neutral would read as a
  seam.
- When several matching rules set `surface-remap`, the last one replaces the
  whole node, as `opacity` does. Fields never merge across rules. So whatever
  rules match a layer at runtime, the node it gets is one that was validated
  at load, and no runtime failure path exists.

### 6. Prism contract

Prism writes this rule into `prism.kdl`. It already rewrites that file on every
`colors_changed` and every glass change, so the existing hook carries
wallpaper and glass changes, and nothing new can loop. The changes are in
Prism's repository:

- **Template.** Prism's Noctalia template (`noctalia-palette.template`) adds
  `surface_container` and `outline_variant` beside `primary` and `surface`.
- **Rule output.** The niri sink emits one complete `surface-remap` node only
  while both `glass.enabled` and a new parameter `shell.remap` (default on)
  are on, whatever the glass's tint source.
  - When `glass.enabled` is off, Prism already leaves out the
    `terminal-glass` definition. The remap rule is left out with it, and so is
    the remap's need for Noctalia's palette, so the existing Glass toggle keeps
    working without touching `shell.remap`.
  - The emitted node carries:
  - the three family colors from Noctalia's resolved palette;
  - the name of the active glass material;
  - `hue-chroma` from a new Prism parameter (proposed `shell.hueChroma`).

  Prism owns every field, so no other file needs to add to the node.
- **Light mode.** When Noctalia's resolved surface is lighter than OKLab
  L 0.4, the sink omits the rule and its apply report says why:
  `shell remap off: Noctalia is in light mode`. Switching to light mode then
  leaves the shell as Noctalia draws it, rather than failing the whole apply.
- **Missing palette.** When the rule is to be emitted and the palette file
  is missing, the apply fails the same way it does today for the `noctalia`
  tint source, with the existing fix hint.
- **Glass out of range.** The apply fails with niri's message (section 2).

### 7. Rendering integration

**Layers.**
- A `SurfaceRemapRenderElement` wraps each `WaylandSurfaceRenderElement` that
  `MappedLayer::render_normal` and `render_popups` produce for a rule with
  `surface-remap`, following `ClippedSurfaceRenderElement`.
- Its program is a new `surface_remap.frag` beside `clipped_surface.frag`.
- Block-out keeps drawing its solid color, unremapped.

**Placement in the frame.**
- The remap acts on client pixels before compositing. It is not a material
  stage and does not change `docs/materials/render-pipeline.md`'s pass order;
  that document gains one row in section 1 saying so.
- Background effects behind the layer are unchanged.
- Captures and screencasts see the remapped shell, as the screen does.

**Damage.**
- The wrapper's commit counter folds in a fingerprint of its uniforms, so a
  target or hue change damages the surface and a steady state costs nothing
  extra.
- A running crossfade counts in `MappedLayer::are_animations_ongoing`, so
  frames keep coming through the fade.
- `Niri::update_render_elements` passes each output's resolved point to its
  layers.

**Scanout.** A wrapped surface is no longer a direct-scanout candidate. Top
layer surfaces with partial alpha seldom are anyway.

### 8. Owner semantics

The brief left these questions to the owner. The spec adopted the brief's
lean, and the owner kept all four at acceptance (2026-10-17):

- **Active window has no accent:** the workspace holds the hue of the last
  accented window that settled there, or goes neutral if there is none.
- **Brief visits:** they don't count. A window counts once it has been active
  for 1000 ms.
- **Hue only, or the agent's level too:** hue only.
- **Surfaces:** the bar, attached panels and floating panels. OSDs, toasts and
  the dock can opt in through the rule.

## Error handling

- Configuration errors fail config load with the node and value named:
  - an unknown material;
  - a missing field in any `surface-remap` node;
  - `hue-chroma` out of range;
  - glass outside the supported range;
  - a light source surface.

  There is no fallback to an unremapped shell, and no runtime failure path,
  because nodes are replaced whole (section 5).
- Prism omits the rule in light mode and reports it (section 6).
- Prism refuses to emit the rule without Noctalia's palette, and says how to
  produce it.
- A window id the resolver no longer finds clears its record. Clearing is the
  defined transition to neutral, not a fallback.

## Verification

**Unit tests (Rust, `just test-one`):**
- The face model:
  - against `material-b156e7`'s measurement: the live `terminal-glass` within
    two levels of 8,11,10;
  - the table in section 2: black, `B_ref` and white for the live glass and
    for distance 20;
  - the range check accepts the live glass;
  - it rejects distance 20, the `#dfe8ff` light glass, and `#000080` at
    thickness 30, distance 10, IOR 1.28 (passes the neutral `Y(T)` test, but
    a blue backdrop gets 12.5% through), each with the section 2 message;
  - `#191919` at thickness 20, distance 10, IOR 1 is accepted, and its
    offsets (−0.046 / +0.044) fall inside the guaranteed bound;
  - the IOR table in section 2, row by row;
  - a sampled sweep of accepted configurations (colors, thicknesses,
    distances, IOR 1 to 3): every sample's face over neutral backdrops from
    0.05 to 0.5 stays within −0.047 and +0.04501 L of its swatch, and over
    saturated red, green and blue backdrops passes at most 1% of each
    channel.
- A check of the model's constants against `main.frag`.
- The remap's CPU mirror reproduces `noctalia-glass-palette.py`'s anchors
  within one level.
- The radii, on three palettes, with the weights section 3 lists.
- Black and white are unchanged.
- The resolver:
  - settle after the dwell;
  - a brief visit doesn't count;
  - the hold across a non-accent window;
  - clearing on close, move or lost accent;
  - the switch blend;
  - keyboard focus on a panel leaves the hue unchanged.
- The crossfade over target families:
  - red to cyan sampled at many fractions around 50% changes by less than
    0.01 OKLab between neighbouring samples (no snap);
  - neutral to hued and back;
  - an interrupted fade continues from its current point.
- Config:
  - parsing `surface-remap`;
  - each load error;
  - two matching rules, where the later complete node replaces the earlier
    one whole.
- The damage fingerprint changes with the hue and holds at rest.

**Prism tests (in the Prism task):**
- `glass.enabled` on and `shell.remap` on: one complete node naming
  `terminal-glass`.
- `glass.enabled` off: no rule, no remap palette requirement. A missing
  palette file doesn't fail the apply unless the `noctalia` tint source needs
  it.
- `shell.remap` off: no rule.
- Light mode: no rule, and the report line.
- The emitted fragment passes `niri validate` for each case above.

**Nested sheet (owner gate).** `docs/materials/scripts/noctalia-surfaces.sh`
gains a `remap` case, with Noctalia on its wallpaper palette and the rule set,
and two hued cases with a nested agent window carrying an accent set by
`SetWindowSignal`. One sheet holds:

| Column | Shows | Question |
| --- | --- | --- |
| C | the matched palette, as approved | |
| R | remap, neutral | does R read the same as C? |
| H1 | `hue-chroma` 0.015 | is the hue present but quiet? |
| H2 | `hue-chroma` 0.03 | which strength reads as the project, legibly? |

Rows are the bar, the control center and the launcher. The owner answers each
column's question, and the pick sets `hue-chroma`'s Prism default. A
transition strip (switching between two accented workspaces, frames sampled
across the crossfade) shows the fade.

**Live rollout (host-use, separately authorized).**
1. Install niri.
2. Apply Prism.
3. The owner changes the three Noctalia settings.
4. A 30-second check: switch between two agent workspaces with the control
   center open.

## Out of scope

- Slab optics (bevel, refraction, ring) on layer surfaces: optical treatment 2
  in the brief. The verdict on C says cohesion doesn't need them.
- Publishing the per-output resolved hue over IPC for plugins. It can follow
  when a plugin wants it.
- Noctalia surfaces other than the bar and panels, beyond opting them in by
  rule.
- `dots-6bfb8d`'s static restyle overlaps the three Noctalia settings. That
  task should take them from this design rather than choose its own.

## Decomposition

One plan in this repository, covering:

- the face model;
- the remap (math, shader, element, config);
- the resolver and per-output crossfade;
- the sheet case and the sheet.

A Prism task covers section 6 (template, rule output, parameter). The niri side
is tested on literal colors and doesn't wait on it. The live rollout needs
both, plus the owner's settings change.
