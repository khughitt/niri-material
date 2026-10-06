---
id: material-6f606b
title: "Signals: sources, model and glass responses"
status: todo
priority: 2
lane: true
created: 2026-10-02T23:07:45Z
updated: 2026-10-03T16:48:43Z
depends: []
tags: [signals]
source: docs/notes/2026-10-02-workstreams-brief.md
agent: claude-code/claude-opus-5-5
---

Connect signal sources, the store and IPC model, and glass responses, with preparation that can continue while captures wait for a quiet host. First milestone: the workspace replay finding (material-c1330b, now done) and terminal transport and attribution (material-07bac9, needs a nested compositor).

The signal pipeline (source -> store/IPC -> response) is code and design work that needs no quiet host, so it is the lane to push when captures are blocked. Done when its three goals (material-9b8bf9 sources, material-b5cbd6 model, material-0a4093 responses) close.
