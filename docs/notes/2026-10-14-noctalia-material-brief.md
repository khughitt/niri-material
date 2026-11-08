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
  background layer. No Noctalia panel was open then.
- Surfaces, from the Noctalia v5.2.1 source and a nested instance
  (`material-e27dbc`, 2026-10-15):

  | Surface | Namespace, layer | Buffer vs visible body | Background | Blur request |
  | --- | --- | --- | --- | --- |
  | Bar | `noctalia-bar-<name>` (`-default` here), top | larger: full width less `margin_ends`, plus shadow pad and corner flare; input is the bar rectangle | `surface` × `[bar.<name>] background_opacity` (live 0.70, a settings override of the config's 0.93) | `ext-background-effect` in the bar's exact shape while `compositor_blur` is on (default) |
  | Attached panel: control center, wallpaper, session, and plugin panels with `placement = "attached"` (Prism's) | `noctalia-attached-panel`, the bar's layer, exclusive keyboard | padded 26 px each side and 16 px below; overlaps the bar by 1 px; input is the body | `surface` × the **bar's** opacity | always: the body, concave at the bar |
  | Floating panel: launcher, clipboard, polkit, plugin panels by default | `noctalia-panel`, top or overlay (the launcher is overlay) | padded 14 px, 16 px below; input is the body | `surface` × `[shell.panel] transparency_mode`: solid 1.0, soft 0.80, glass 0.55 (live: glass) | always: the rounded body |
  | OSD, toasts, dock | `noctalia-osd`, `noctalia-notification`, `noctalia-dock` | card plus padding | own `background_opacity` (defaults 0.97, 0.97, 0.88) | yes: rounded cards |

  Every open panel adds a full-output, transparent
  `noctalia-panel-click-shield` beneath it. Corners and shadows are drawn
  client-side, and no surface sets an opaque region. The nested listing
  matched the source: the bar and the control center (`noctalia-attached-panel`)
  on top, the launcher (`noctalia-panel`) on overlay.
- The live niri config already matches these namespaces with
  `background-effect { xray false; }`, so the bar and panels get real blur in
  the client's exact shape today. Layer effects can add noise and saturation
  to it; nothing in them reads as a slab.
- The nested comparison (`docs/materials/scripts/noctalia-surfaces.sh`; its
  sheet is attached to `material-e27dbc`) sets the live baseline against a
  treated case: every panel at 0.55, panel corners at the windows' 20 px, and
  the glass's noise 0.03 and saturation 0.95 on the shell's blur. Over the
  dark glass terminal and the wallpaper's dark upper half the change is
  small: bar pixels move by 5 to 10 levels, while the corners now visibly
  match the windows. Bevel, refraction, edge light, ring and hue are absent
  from both.
- The owner judged hue and darkness the main difference (2026-10-16), and a
  third case, `matched` (`material-b156e7`; sheet attached there), closes it
  with Noctalia settings alone. A custom palette keeps the wallpaper palette's
  accents and text and moves the surface family to the glass's rendered
  color in OKLCH (`docs/materials/scripts/noctalia-glass-palette.py`):
  surface `#151310` to `#090a0a`, cards `#221f1c` to `#141616`, outlines
  `#4d453c` to `#333d3d`. The bar and attached panels go to 0.95 opacity and
  floating panels to `solid`. Measured against the glass at 8,11,10, the
  control center reads 10,10,10 and the launcher 9,10,10 (baseline 23 to
  28); the bar over the bright wall reads 18,17,16, the 5% of backdrop it
  still passes.
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
- The compositor binding is feasible (`material-6b1c0b`, source trace at
  `ae238683`). `Monitor::active_window()` returns the active workspace's
  active window from layout state, independent of keyboard focus and of
  which monitor is focused, so an open panel or an unfocused second output
  keeps its hue. `Niri::update_render_elements` already walks each output's
  layers with mutable access beside `self.layout`, the same per-frame point
  where a tile advances its accent crossfade (`Tile::signal_for_frame`).
  Redraws are already requested: every signal write calls
  `queue_redraw_all`, and focus, active-window and workspace-switch changes
  redraw their output. What a layer lacks: a crossfade (the tile's
  `SignalCrossfade` and `CrossfadePoint` are private to
  `src/layout/tile.rs`), coverage in `MappedLayer::are_animations_ongoing`
  (today only `baba-is-float`) so frames continue through a fade, and
  fingerprint-gated damage on whatever draws the hue.
  `Monitor::workspace_render_idx()` gives the fractional index a switch
  blend would follow.
- Focus timestamps cannot carry a hold rule (brief review, 2026-10-15).
  `get_focus_timestamp()` is the window switcher's recency stamp: on a
  keyboard focus change it is written only after `recent-windows {
  debounce-ms }` (default 750) passes without focus moving on, or at once on
  keyboard input (`src/niri.rs` focus handling, `mru_apply_keyboard_commit`).
  After agent A, a brief visit to agent B, then a browser, B's stamp is
  still older than A's, so a timestamp rule holds A's hue; the rule would
  also shift whenever the switcher's debounce is retuned. A hold needs its
  own per-workspace record of active-window changes.
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
- Bar and panel buffers carry padding, shadow and flare around the visible
  body. A shader footprint follows the shape Noctalia already sends through
  `ext-background-effect`, not the buffer.
- Anything niri draws under a Noctalia surface (blur, slab or tint) shows
  through only (1 − opacity) of it: 0.30 behind the bar and attached panels at
  live settings, 0.45 behind floating glass panels. Built-in panels offer only
  solid, soft and glass (1.0, 0.80, 0.55), and attached panels follow the bar.
  On the live desktop the bar's opacity sits in Noctalia's settings layer,
  which overrides `config.toml`, so a rollout changes it there.
- A custom palette is static: the live palette follows the wallpaper
  (`source = "wallpaper"`), and the glass color is Prism's. A live matched
  palette has to be regenerated whenever either changes, from the hooks that
  already run on `wallpaper_changed` and `colors_changed` (switching to the
  regenerated palette fires `colors_changed` again, so that path must not
  loop), and the glass's
  rendered color depends on its backdrop (it passes a few percent), so the
  target is a measured or modelled value, not the attenuation color itself.
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

Two choices, made separately. The comparison chooses the optical treatment;
the focused hue needs a delivery route whichever treatment it chooses, so
"existing effects are sufficient" settles the optics, not the hue.

Optical treatment:

1. **Existing layer effects plus Noctalia styling (current first experiment).**
   Test one panel with matched tint, transparency, blur and geometry. This is
   the smallest way to find out which cues supply cohesion. On its own it
   cannot follow a hue: layer effects have no tint and the palette is static.
2. **Opt-in material rendering for layer surfaces.** Reuse the window material
   system if the comparison needs its slab, edge or interior optics. This needs
   a reviewed design for surface footprint, client composition and lifecycle.
3. **Client-side imitation of the glass.** Consider only if the required shape
   is unavailable to the compositor; it introduces a separate optical model.

Hue delivery:

- **A. Compositor tint under the client.** niri tints the layer's backdrop
  with the bound accent, beneath the client's translucent background: a tint
  term beside the existing background effects if 1 wins, `accent-tint` if 2
  wins. It shows only where the client background is translucent, the same
  limit as every compositor effect. It is the only route to built-in bar
  chrome and built-in panels: Noctalia 5.2.1 and 5.3.0 give plugins no
  palette write. The binding's smallest change set: a pure per-monitor
  resolver (hold record, switch blend) with a layout test; one crossfade per
  output in `OutputState`, so every layer on an output shows the same hue
  (the tile's crossfade types moved to a shared module); the resolved point
  passed to `MappedLayer::update_render_elements`; the fade counted in
  `are_animations_ongoing`; and a layer-rule opt-in.
- **B. Plugin-side hue.** A Noctalia plugin widget, or Prism's panel, streams
  niri's events and colors its own elements with the active window's accent.
  This works at plugin API 22 with no niri change, but it cannot reach
  built-in bar chrome or built-in panels, and the hue sits on the shell
  rather than in its glass. If niri later publishes the per-output resolved
  hue over IPC, plugins would show the same hue and hold rule as the glass
  instead of re-deriving them from the event stream.
- **C. A Noctalia runtime palette override.** A non-persisted override of the
  palette roles would recolor built-in chrome client-side. Neither 5.2.1 nor
  5.3.0 has one; it would be an upstream feature, and proposing it is the
  owner's action.

Design (2026-10-16, accepted 2026-10-17; plan docs/plans/2026-10-17-noctalia-glass-cohesion.md): `docs/specs/2026-10-16-noctalia-glass-cohesion-design.md`.
After the owner chose C, it takes neither A nor a palette route. niri remaps
the shell's surface family to the glass face and the held hue, because a
Noctalia palette change runs every template and `colors_changed`.

Prefer the bounded comparison before a renderer extension. No visual choice
or claim of adequate cohesion has been made. For the hue, the lean is A,
carried by whichever optical treatment the comparison picks, with B as a
cheap early look at whether a hued shell reads as cohesive. The semantics
lean, for the owner to confirm: hue only (level, motion and impulses stay on
the window); an output shows its active workspace's project hue; while a
window without an accent is active there, the workspace holds the hue of
the last accented window that settled as its active window, and shows
neutral when it has none; changes crossfade and follow a workspace switch.
A window settles once it stays active for a dwell the binding owns, not the
window switcher's debounce, recorded when the workspace's active window
changes: keyboard navigation through a column on the way elsewhere does not
recolor the shell.

## Unanswered questions

- Answered by `material-e27dbc`: the control center is the representative
  panel. It shares `noctalia-attached-panel` with Prism's panel and every
  attached plugin panel, and takes the bar's opacity, so the bar and attached
  panels are one treatment; the launcher (`noctalia-panel`) stands for the
  floating panels.
- Do existing effects supply enough cohesion, or are slab optics essential?
  The owner judges the attached sheet (cells A1 to B3). That verdict picks the
  optical treatment only; hue delivery follows either way.
- Answered by `material-e27dbc`: a material pass can use the visible body.
  Noctalia sends its exact shape through `ext-background-effect`, and drawing
  only beneath the client leaves text, input regions, exclusive zones and
  popups to Noctalia.
- Answered by `material-6b1c0b`: niri can bind a layer surface's accent to
  its output's active window with existing layout state and redraw triggers;
  the hold needs its own record, not focus timestamps. The drawing belongs to
  the hue-delivery design, whichever optical treatment wins.
- What does a workspace show when its active window has no accent: the last
  agent hue, neutral, or strict focus? Does a brief visit count, or only a
  window that stays active for a dwell? Hue only, or also the agent's level?
  And which surfaces follow the hue: bar, panels, launcher, OSDs? The owner
  decides, after the static comparison; the lean is above.

## Proposed decomposition

- Keep `material-62347d` as an idea until feasibility establishes a concrete
  surface and an experiment the owner can judge.
- `material-55380c` is the child goal within the existing material-composition
  lane `material-3aa1f2`.
- `material-e27dbc` identified the panel, ran the comparison nested and
  attached its sheet (done 2026-10-15; see the evidence above). It added no
  hued arm: Noctalia's generated palette uses token names (`surface`) while a
  custom palette takes `mSurface`-style keys, so the arm needed a transcoded
  palette, and it would only have previewed client-side tinting, not
  Alternative A. Next, the owner judges the sheet's cells A1 to B3 and answers
  the hue questions above; the design follows that verdict.
- `material-6b1c0b` traced the compositor hue binding (done 2026-10-15; see
  the evidence above) and readmitted `material-62347d` with its finding. Its
  recommendation to fold the binding into a layer-material design is
  superseded: after the comparison, the hue-delivery design (A plus the
  binding) follows whichever optical treatment wins, unless the owner drops
  the hue requirement.
- File implementation or renderer design only after the findings; do not add
  a general shell-material framework in this scope pass. A plugin-side probe
  (Alternative 4) waits for the owner's semantics answer.
