---
id: material-3fcba2
title: "Glass noise layers: N stacked grain generators with gain, type, and seed scale each"
status: doing
priority: 2
size: m
complexity: mid
process: planned
owner: materials-26.04
created: 2026-09-09T03:03:30Z
updated: 2026-10-06T09:08:53Z
started: 2026-10-06T09:08:29Z
depends: [material-cf32e5]
parent: material-3aa1f2
tags: [material, noise]
---

Prism goal prism-85f63a wants several noise devices stacked on one material. Extend the glass noise node to a small fixed number of layers (four, after the impulse fan-out precedent): packed vec4 uniforms for gain, type, and scale, an unrolled constant-bound loop in material.frag, and a per-layer seed scale or offset so identical types do not coincide. Single-node configs must render byte-identical. Record the cost of stacked fine layers (nine hashes per fragment per layer).

## Notes

- 2026-09-12T19:24:31Z (materials-26.04): Complexity mid: Four layers, packed uniforms, a bounded shader loop, and single-node pixel identity are specified; the existing noise optic localizes the work. Layer decoding, inheritance, seed separation, and lightness composition still require bounded implementation choices and capture verification.
- 2026-10-06T09:08:29Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:931ef8b7-a925-41a6-983b-bb8509d7d7d0","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-06T09:08:52Z (materials-26.04): Process planned: the body predates the noise site attribute (material-cf32e5) and the pipeline schema prism vendors. Open design choices: KDL shape for repeated noise nodes and their inherit/include merge, how a multi-instance parameter appears in pipeline.json's ParamSpec/stage ownership (prism-85f63a's flat-bus question), layer x site interaction ahead of material-829590, seed separation, and composition order of white/fine vs lightness layers.
