---
id: material-ad1780
title: Focused glass edges render opaque while unfocused edges stay transparent (glass6)
status: todo
priority: 2
size: xs
complexity: low
process: direct
needs: [owner]
created: 2026-10-02T19:21:48Z
updated: 2026-10-15T18:31:50Z
depends: [material-f72d5c]
parent: material-6062fd
tags: [quick-add]
source: "mindful:thought:6258221f80b07e869303a0449ee939cc"
agent: claude-code/claude-opus-5-5
---

Why: the owner's report (mindful thought:6258221f80b07e869303a0449ee939cc, 2026-10-02): with the glass6 setting, unfocused window edges look right (transparent, with fringing) but focused edges look opaque. Asked: what causes it (tint distance or colour, or something else), and how to keep the edge's dynamic range when focused. Near: material-be611b (realistic edges), material-48b514 (edge tint, done).

Established (2026-10-09, notes below; do not re-derive): not a tint colour. Stage 4's attenuation path h / -t.z is physically right (the refracted ray exits the back), but at the active look's thickness/attenuation-distance ratio of about 3.5 the whole bevel transmits about 2e-4, while the inactive look's ratio of about 1.7 still passes light. What real dark glass has and the model lacks is piped light at the rim. The fix is the edge-light term in material-f72d5c (source: this task), which follows the spike material-ac926f.

Remaining: this task is the owner's acceptance of that fix. Once material-f72d5c lands, render the glass6 looks focused and unfocused and have the owner judge whether the focused edge keeps its dynamic range. If material-f72d5c is abandoned or does not restore it, the remaining lever is the active look's thickness/attenuation-distance ratio, which is a Prism look decision for the owner: file it then rather than here.

Done: the owner's verdict recorded on this task, and either the symptom closed or a follow-up filed for the ratio lever.

## Notes

- 2026-10-09T23:29:33Z (materials-26.04): Cause found 2026-10-09: not a tint colour but the attenuation path on the bevel. rayPath = h / max(-t.z, 0.25) lengthens the path as the structural normal tilts, cancelling the height-field thinning; at the active look's thickness/attenuation-distance ratio 3.5 the whole bevel transmits ~2e-4, while the inactive look's ratio 1.7 still passes light. Fix candidate: clip the attenuation path at the slab's side wall (light near the silhouette exits through the side), which brightens the edge toward the rim without lightening the face. Spiked and then implemented under the sibling tasks filed today.
- 2026-10-09T23:32:48Z (materials-26.04): Correction 2026-10-09: the path formula is right (the refracted ray exits the back). The real gap is piped light at the rim; the candidate is the edge-light term in material-f72d5c. The thickness/attenuation ratio finding stands.
