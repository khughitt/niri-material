---
id: material-ec9f9d
title: Compute pkgver instead of hand-editing it
status: todo
priority: 3
size: s
created: 2026-09-08T15:13:04Z
updated: 2026-09-08T15:13:04Z
depends: []
tags: [packaging]
---

Found bringing europa (laptop) up to date on 2026-09-08 after ~3 weeks. The release procedure rewrites pkgver and the source commit by hand; recomputing the count from the previous pin disagreed with a historical pin by 3. Outcome: a recipe derives pkgver and the source pin from the commit being packaged.
