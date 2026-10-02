---
id: material-3a17b8
title: "Shared frozen-clock render helpers for material tests, with a focus-gain step"
status: todo
priority: 3
size: s
complexity: low
process: direct
created: 2026-10-02T00:54:22Z
updated: 2026-10-02T00:54:22Z
depends: []
tags: [harness, testing]
agent: claude-code/claude-opus-5-5
---

ring_look.rs borrows set_time/render_at/diff from ring_pair.rs through pub(super), so one test module depends on another. Move them to a shared src/tests helper module, and add a focus_gain(f, at) helper that does it the way ring_look found works: focus_left, update_keyboard_focus, refresh_window_states, refresh_layout, refresh_window_rules, with no client roundtrip or refresh_and_flush_clients. Niri::redraw re-stamps the clock from the frame clock, and the tile stamps the beam's start from now_unadjusted on its next update_render_elements, so either one stamps the gain at real time and the comet reads as elapsed 0 forever. Also document that is-active rules treat a lone window as active from the moment it maps, so a gain needs a second window.
