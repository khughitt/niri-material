# Focus-state glass spike

**Result:** a fully transparent terminal over an `is-active` conditioned
material swap removes the terminal-versus-glass seam, and a dense dark
attenuation keeps the text legible without any terminal background. Run
2026-09-04 against the installed `niri 26.04 (663202b1)` (binary SHA-256
`23098121e5b24e39741a7a1d211229807446d39b7e4fc6ea2fb50a809408ec4a`, the
`feat/material-signals` package) on a 1280 x 720 headless Weston GL host.

Task: `material-cb348e`, piece of ops goal `ops-500adb`.

**The captures below predate the ring of light** (`f8bcb34c`, 2026-09-05);
`663202b1` does not draw it. Re-running the script today puts a ring at every
window edge, which is where this spike's corner crops are, so new captures are
not comparable with these. The script now also sets `animations { off; }` —
without it the ring's drift phase advanced between runs and two runs of the same
case differed by `AE 994` per frame (`material-0af212`).

## Question

Today Prism paints kitty at `background_opacity` 0.98 focused and 0.44
unfocused over the `terminal-glass` material. The opaque cell background ends
inside the slab, so the refracting bevel strip is visible around every
terminal, most obviously at the corners of two adjacent windows. Does setting
the terminal background to 0 and letting the glass carry focus state remove
that seam, and what does the glass then need to keep text readable?

## Method

`scripts/focus-glass-spike.sh` nests the installed compositor under a
headless Weston host, spawns two `kitty --config NONE` windows showing a
static claude-code-like transcript, screenshots the right window focused and
then the left, and crops the inner corners at 3x. The layout, focus ring, and
shadow match the daily-driver config; the backdrop is a daily-driver
wallpaper (a bright tent-and-trees photo, a hard case for legibility).

Cases:

| Case | kitty opacity (unfocused / focused) | Material |
| --- | --- | --- |
| `baseline` | 0.44 / 0.98 | Prism `terminal-glass` both states |
| `zero-same` | 0 / 0 | Prism `terminal-glass` both states |
| `zero-split` | 0 / 0 | Prism glass active; frosted variant inactive |
| `zero-dark` | 0 / 0 | dark glass both states |
| `zero-dark-split` | 0 / 0 | dark glass active; lighter frosted dark inactive |

The dark glass is the Prism block with `attenuation-color "#222436"` (the
terminal background) and `attenuation-distance 30`. Its inactive variant
raises `roughness` to 0.5, lowers `chromatic-aberration` to 0.08 and
`distortion` to 0.10, and relaxes `attenuation-distance` to 70.

## Observations

- `baseline` reproduces the reported seam: the opaque body sits inside the
  slab and the bevel strip, with its chromatic fringe, reads as a border.
- Every zero-opacity case has no seam. The slab, its rounded corners, and the
  bevel read as one object.
- `zero-same` is illegible over the bright backdrop. Prism's current glass
  (`attenuation-distance` 1041, wallpaper-derived orange tint) is close to
  clear; with no terminal background nothing darkens the text field.
- `zero-dark` is legible in both windows and reads as smoked glass. The
  terminal background color moved from kitty into the material.
- `zero-dark-split` keeps the focused window dark and crisp while the
  unfocused one lightens and frosts. Focus is unmistakable with no opacity
  step. The swap flips correctly when focus moves left.
- The swap is a hard cut by construction: `apply_resolved` in
  `src/render_helpers/material.rs` builds a new `MaterialState` when the
  resolved material name changes, so jelly residuals and any other per-state
  dynamics restart. Stills cannot show whether that reads as a pop; that is
  the open question for `material-5a5fff`.

## Consequences

- The seam fix is configuration: kitty at 0 and two materials selected by
  `is-active`. No compositor change is needed for the base treatment.
- The glass must own the background color. Prism's wallpaper-derived
  `attenuation-color` cannot stay the terminal's only backdrop; the active
  material needs the terminal background color at a short distance, which is
  a question for the Prism palette piece.
- Neovim's `transparent` mode and kitty's `transparent_background_colors`
  become the only places an app still paints a background; they need the same
  treatment or they reintroduce the seam.

## Retained artifacts

Captures, generated configs, nested logs, and the comparison montage are
outside the product tree under:

```text
$NIRI_MATERIAL_WORK_ROOT/focus-glass-spike-663202b1
```

`SHA256SUMS` there covers every capture. A rerun of the retained harness from
its committed location reproduced `zero-dark-split` under
`$NIRI_MATERIAL_WORK_ROOT/focus-glass-spike`.
