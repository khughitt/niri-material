---
id: material-2a38d4
title: Link IPC usage documentation from the crate README
status: todo
priority: 3
size: xs
complexity: low
process: direct
created: 2026-10-24T13:48:45Z
updated: 2026-10-24T13:48:45Z
depends: []
tags: [docs]
source: material-10a999
agent: codex
---

Why: niri-ipc/README.md identifies the crate and gives exact-version dependency guidance but has no route to connection/request/event-stream usage or API reference. niri-ipc/src/lib.rs:1-52 already explains NIRI_SOCKET, socket::Socket, framing and separate event/request sockets.

Done: add a short route from the README to the existing API/protocol documentation rather than duplicate the protocol. Preserve the exact-version compatibility warning and verify the link and normal documentation checks.
