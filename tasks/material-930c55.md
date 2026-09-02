---
id: material-930c55
title: "Source: familiar bridge in familiar/integrations/niri"
status: idea
priority: 2
size: m
created: 2026-09-02T12:09:35Z
updated: 2026-09-02T12:09:35Z
depends: [material-a54d89]
tags: [signals, sources]
---

Outcome: familiar-niri watch pushes the intent.json to niri-windows.json join into niri via set/pulse/clear-window-signal, using the mapping in docs/materials/2026-09-02-material-signals-design.md section 2, aggregating several sessions in one window into a single familiar slot (highest urgency wins, most recent impulse wins). Lives in familiar's repository and tracker; this record is the cross-repo pointer. Acceptance: bridge test against a nested niri-material, and no compositor knowledge under familiar's src/.
