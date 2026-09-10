---
id: material-6b4184
title: Fix pixel comparison verdict in the optics plan
status: done
priority: 2
size: xs
owner: feat/material-397fcb
created: 2026-09-10T10:44:06Z
updated: 2026-09-10T10:45:04Z
depends: []
parent: material-397fcb
tags: [docs]
---

Use ImageMagick compare exit status so identical pixels pass even when AE text includes a normalized value; verify identical, differing, missing, and corrupt image cases.

## Notes

- 2026-09-10T10:45:04Z (feat/material-397fcb): Plan comparison now uses ImageMagick exit status; identical pixels pass, differing/missing/corrupt images fail. Verified all four cases through just and tools/tt.
