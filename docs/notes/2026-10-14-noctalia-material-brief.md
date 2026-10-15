# Material cohesion on Noctalia surfaces

2026-10-14; focused hue added 2026-10-15. Scope handoff for `material-62347d`;
not an approved design.

## Problem

Make Noctalia's bar and panels feel part of the same material as the windows.
The source asks for cohesion, without choosing a surface treatment or renderer
extension. Start with one representative panel; the bar is the second consumer.

Focused hue (owner, 2026-10-15): beyond a static variant of the material, the
shell surfaces should take the hue of the workspace's active coding-agent
terminal, the familiar identity hue its glass already shows, so the whole
workspace reinforces the project being worked on.

## Current behaviour and evidence

- At main checkout `d9914514`, `niri-config/src/layer_rule.rs` exposes
  background effects, opacity, shadows and corner geometry, but no material
  reference. `src/layer/mapped.rs` renders surface pixels, shadows and
  `background_effect::render_for_tile`; it has no `MaterialState` or material
  render element. Window material rules therefore cannot simply select a bar.
- `docs/wiki/Configuration:-Layer-Rules.md` documents blur, noise, saturation,
  namespace matching and separate popup overrides. These are an existing route
  for a first comparison, not evidence that they reproduce the window glass.
  `BackgroundEffectRule` (`niri-config/src/appearance.rs`) has xray, blur,
  noise and saturation, and no tint.
- Read-only `niri msg --json layers` on 2026-10-14 reported
  `noctalia-bar-default` on the top layer and `noctalia-wallpaper` on the
  background layer. No Noctalia panel was open; its namespace, surface geometry
  and transparency are unverified.
- The original capture is
  `mindful:thought:d6cae16e5bd867ec390c14ab866e7619`: use material on bar,
  panel and other Noctalia surfaces for cohesion. It links the `familiar`
  thought. No overlapping open research was found in the local task corpus.
- The hue already reaches niri. `familiar-niri watch` writes each agent
  terminal's `familiar` signal slot with its session's identity hue as the
  accent (familiar's `integrations/niri/signal.js` and `docs/surfaces.md`).
  niri folds the slots into `Window.signal`, published by `niri msg --json
  windows` and as `WindowSignalChanged`. `Workspace.active_window_id` and
  `WorkspaceActiveWindowChanged` name each workspace's active window
  (`niri-ipc/src/lib.rs`, `docs/wiki/IPC.md`). Only the window's own glass
  consumes the accent (`accent "ring"`, `accent-tint`, with a crossfade).
- niri renders layers per output in `Niri::render_layer_normal`
  (`src/niri.rs`), where the layout's active workspace for that output is in
  reach. `MappedLayer` holds no signal or accent state.
- The installed shell is Noctalia 5.2.1, a native C++ shell with Luau plugins
  (Prism's and wali's panels declare plugin API 22), not the Quickshell
  version. Its palette is generated from the wallpaper (`[theme] source =
  "wallpaper"` in the Noctalia config), and its `colors_changed` hook runs
  `prism apply niri`.
- Noctalia's v5.2.1 plugin runtime (upstream
  `docs/user/plugins/development/runtime-api.mdx` and `declarative-ui.mdx`)
  lets a plugin read palette roles (`getColor`) and stream a command's stdout
  (`runStream`, so `niri msg --json event-stream`); UI color props accept
  `#rrggbb`. Plugins cannot write config or the palette. `noctalia msg
  color-scheme-set` persists the palette choice to settings, so built-in bar
  and panel chrome only changes color through a persisted palette switch.

## Constraints

- Keep wallpaper and unrelated shell surfaces outside the first treatment.
- Establish the actual panel's namespace, buffer extent, alpha and background
  painting before choosing a shader footprint; a surface rectangle may contain
  empty space around the visible panel. Compositor effects cannot be assumed
  visible through an opaque client background.
- Preserve client text, input regions, exclusive zones, popup placement,
  output scaling, capture blocking and damage behaviour. Any renderer design
  must follow `docs/materials/render-pipeline.md`.
- Research can inspect source and saved configuration without using the desktop.
  A rendered proof uses a nested compositor (`nested`); opening panels or
  changing the live desktop needs separate host-use authorization. Appearance
  review needs `owner`; settled cost evidence needs `quiet` if later required.
- The shell follows the window's folded signal accent, never familiar's
  catalog: whether that hue comes from familiar's identity slots or tasks'
  project colour is open in `ops-df0842` (with `tui-76e406`, `fam-053b0d`),
  and the shell should show whatever hue the glass shows.
- An open panel takes keyboard focus, so no window is `is_focused`, while
  `active_window_id` holds (the rule `prism-c0500d` relies on). The binding
  therefore reads the output's active workspace and its active window, so
  opening a panel keeps the hue.
- Bars belong to outputs, not workspaces: an output shows its active
  workspace's hue. Workspace switches, the overview, and workspaces without an
  agent window need defined behaviour.
- A palette rewrite per focus change is ruled out: it persists to settings,
  regenerates templates, and through `colors_changed` reapplies niri's
  material configuration on every focus change.
- Related open work: `ops-500adb` (one glass across niri, terminals, editor and
  shell, naming the familiar hue as a per-project tint) is the umbrella;
  `dots-6bfb8d` restyles the bar, launcher and settings panel statically and
  overlaps Alternative 1.

## Alternatives

1. **Existing layer effects plus Noctalia styling (current first experiment).**
   Test one panel with matched tint, transparency, blur and geometry. This is
   the smallest way to find out which cues supply cohesion. It cannot follow a
   hue: layer effects have no tint and the palette is static.
2. **Opt-in material rendering for layer surfaces.** Reuse the window material
   system if the comparison needs its slab, edge or interior optics. This needs
   a reviewed design for surface footprint, client composition and lifecycle.
   The focused hue fits here: the layer's accent binds to its output's active
   window and `accent-tint` and the crossfade carry it, one optical model in
   one hue. It adds per-layer accent state and damage on focus changes
   (`material-6b1c0b` traces both).
3. **Client-side imitation of the glass.** Consider only if the required shape
   is unavailable to the compositor; it introduces a separate optical model.
4. **Plugin-side hue.** A Noctalia plugin widget, or Prism's panel, streams
   niri's events and colors its own elements with the active window's accent.
   This works at plugin API 22 with no niri change, but it cannot reach
   built-in bar chrome or built-in panels, and the hue sits on the shell
   rather than in its glass.

Prefer the bounded comparison before a renderer extension. No visual choice
or claim of adequate cohesion has been made. For the hue, the lean is 2 if the
comparison justifies a layer material, with 4 as a cheap early look at whether
a hued shell reads as cohesive. The semantics lean, for the owner to confirm:
hue only (level, motion and impulses stay on the window); an output shows its
active workspace's project hue, held while a non-agent window is active there
(the most recently active window with an accent) and neutral on a workspace
with none; changes crossfade and follow a workspace switch.

## Unanswered questions

- Which panel offers a small, representative target, and what does its surface
  expose? The feasibility follow-up answers from Noctalia source/configuration
  and, if available, a nested instance.
- Do existing effects supply enough cohesion, or are slab optics essential?
  The owner judges a labelled comparison after that follow-up.
- If a material pass is needed, can it use a known visible panel rectangle and
  preserve shell behaviour? Source tracing answers the feasibility; design
  follows only if the first comparison justifies it.
- Can niri bind a layer surface's accent to its output's active window
  (state, crossfade, damage, workspace switch, overview) without a new optical
  model? `material-6b1c0b` answers from source tracing.
- What does a workspace show when its active window has no accent: the last
  agent hue, neutral, or strict focus? Hue only, or also the agent's level? And
  which surfaces follow the hue: bar, panels, launcher, OSDs? The owner
  decides, after the static comparison; the lean is above.

## Proposed decomposition

- Keep `material-62347d` as an idea until feasibility establishes a concrete
  surface and an experiment the owner can judge.
- `material-55380c` is the child goal within the existing material-composition
  lane `material-3aa1f2`.
- `material-e27dbc` identifies one panel and specifies the smallest comparison
  (P2, small, mid complexity, direct, `nested`). `material-62347d` depends on
  it. Its completion records the finding here and uses `--no-reviewed` to
  readmit that idea before the result branch merges. Its comparison may add a
  hued arm with a fixed familiar hex in a custom scheme, if that is cheap.
- `material-6b1c0b` traces the compositor hue binding (P2, small, mid
  complexity, direct). `material-62347d` depends on it and wakes on its
  finding.
- File implementation or renderer design only after the findings; do not add
  a general shell-material framework in this scope pass. A plugin-side probe
  (Alternative 4) waits for the owner's semantics answer.
