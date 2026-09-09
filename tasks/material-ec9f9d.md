---
id: material-ec9f9d
title: Compute pkgver instead of hand-editing it
status: done
priority: 3
size: s
owner: materials-26.04
created: 2026-09-08T15:13:04Z
updated: 2026-09-09T09:34:13Z
depends: []
tags: [packaging]
---

Found bringing europa (laptop) up to date on 2026-09-08 after ~3 weeks. The release procedure rewrites pkgver and the source commit by hand; recomputing the count from the previous pin disagreed with a historical pin by 3. Outcome: a recipe derives pkgver and the source pin from the commit being packaged.

## Notes

- 2026-09-09T09:34:13Z (materials-26.04): The hand-written counts disagree with any rule: r278/r252/r169 against 273/247/167 counted from patched_commit. r<N> is now defined as the commits this fork carries over patched_commit in upstream-baseline.toml, the only ancestor a rewritten history can be counted from -- git describe finds no tag ancestor at all. The pin stays on 5dbe182d, deliberately; only its version string is corrected, from r278 to r273.
- 2026-09-09T09:34:13Z (materials-26.04): just package-pin <commit> derives pkgver, the source= commit and NIRI_BUILD_COMMIT from the commit being packaged, and check refuses a PKGBUILD whose three values disagree.
