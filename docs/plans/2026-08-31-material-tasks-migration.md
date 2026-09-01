# Material Tasks migration ledger

**Status:** complete 2026-09-01; historical/superseded. The reviewed migration
commits were fast-forwarded into stable Material, the canonical registry maps
`material` to that checkout, and there are no deferred foreign dependencies. The
implemented migration sequence runs through `8047b6ca14ec1e2a0760a79f5d9d4883a9fc2519`;
the central portfolio record owns the resulting SHA of this ledger-only traceability
correction.

## Scope and evidence

| Field | Value |
| --- | --- |
| Initial stable source | `materials-26.04` at `7e94d71af195d5f5062d9b51ec80bca513af8ae3` |
| First integrated stable head (historical snapshot) | `e4a982b82f55cae55c8fc8fc9a492f4ebb0b5849` |
| Migration implementation range | `b559a4c32d2e022107fd6048ced1f658429bbbcb` through `8047b6ca14ec1e2a0760a79f5d9d4883a9fc2519`: `b559a4c32d2e022107fd6048ced1f658429bbbcb`, `d064120f43fe15d8021f2dbdb4b469eba5da4710`, review fix `e4a982b82f55cae55c8fc8fc9a492f4ebb0b5849`, integration record `02caf4dbbb129a0baf8160ac27c584e18f24c235`, portability fix `28423687fdcc627ed8f6bb501742650478e0318e`, and late evidence fix `8047b6ca14ec1e2a0760a79f5d9d4883a9fc2519` |
| Tasks source | `b943419c0e37b947a0ca1814f416ff61259f4d8a` |
| Audit date | 2026-09-01 |
| Project prefix | `material` |
| Authority roots | `~/d/niri-material`, `docs/materials/**`, production code/tests/config, and the preserved `debug/overview-drag-frost` worktree |
| Companion evidence | `~/d/niri-experiments` physical/parity results and `~/d/prism` ownership/config records, inspected read-only |

The audit read every path under `docs/materials/**`, the Material-relevant wiki
exceptions, root claims, the implementation and tests governing those claims,
and the active five-file debug diff. The 83 tracked wiki files are inherited
upstream documentation; the strict MkDocs build validates their current link
surface without turning them into Material task authority.

## Git state inspected

The stable and migration-worktree rows below are historical snapshots captured during
integration; the complete implementation range is recorded above.

| Checkout or branch | Inspected state | Decision |
| --- | --- | --- |
| `~/d/niri-material`, `materials-26.04` (historical first-integration snapshot) | Clean at first integrated head `e4a982b82f55cae55c8fc8fc9a492f4ebb0b5849`, six commits ahead of `origin/materials-26.04` | Stable authority after the reviewed first fast-forward from `7e94d71af195d5f5062d9b51ec80bca513af8ae3`. |
| `chore/tasks-migration-material` (historical ledger-closeout snapshot) | Isolated worktree at `e4a982b82f55cae55c8fc8fc9a492f4ebb0b5849` before the ledger-only closeout | Retained for the independent ledger review and second fast-forward. |
| `debug/overview-drag-frost` | `048814893b9bc4924927468ce1f539b7764d942b`; only the five expected tracked files are dirty; no staged or untracked files | Preserve byte-for-byte; active task owner is `debug/overview-drag-frost`. |
| `patched-26.04` | `5e53b949`, tracking `origin/patched-26.04` | Historical production base; no task outcome. |

The preserved active-work fingerprint is:

| Path | SHA-256 |
| --- | --- |
| `niri-config/src/lib.rs` | `2b72b4b61013d9d2fa10aa144d4bdb366224a7d49cc022ae2a8914940f5fed1d` |
| `src/layout/tile.rs` | `f91f4f507aa0447892988f5077cc580399205c4d4cdc9fadfd1fec2f9b7861fe` |
| `src/render_helpers/effect_buffer.rs` | `2dffd6ef66b9e577c0f4cacac99959d7ac8a0118a5d01797f3d2791e30c8a53c` |
| `src/render_helpers/material.rs` | `479e32e4c1d677027ac13d525cbc21e59aeffc77cc868fcd033c5972448e9dc4` |
| `src/render_helpers/xray.rs` | `4b7c3d68fd90ee97406d0f7baa500a2c82a1c87d368396afbfd12becaea4a0eb` |

## Document classification

| Path | Classification | Reason |
| --- | --- | --- |
| `docs/wiki/**` | `historical/superseded` | Inherited upstream wiki; unchanged except listed per-file exceptions. |
| `docs build tooling` | `authority/current` | `docs/.gitignore`, `docs/hooks/**`, `docs/mkdocs.yaml`, `docs/pyproject.toml`, and `docs/uv.lock`; verified by the strict docs gate. |
| `docs/materials/2026-08-22-repository-split.md` | `historical/superseded` | Implemented repository-boundary design. |
| `docs/materials/2026-08-22-v1-design.md` | `authority/current` | Accepted architecture and explicit post-v1 roadmap; source for the roughness outcome. |
| `docs/materials/2026-08-24-glass-config-surface-design.md` | `historical/superseded` | Implemented configuration-surface design. |
| `docs/materials/2026-08-24-v1-parity-design.md` | `historical/superseded` | Completed frozen-reference parity evidence; corroborates roughness scope. |
| `docs/materials/2026-08-25-v1-reference-static-preflight-design.md` | `historical/superseded` | Completed parity preflight evidence. |
| `docs/materials/2026-08-27-v1-drm-acceptance-design.md` | `authority/current` | Accepted physical harness contract and source for the remaining default-preserves-v1 rerun. |
| `docs/materials/2026-08-28-v1-daily-driver-rollout-design.md` | `authority/current` | Current deployment and burn-in verdict; corrected stale inner claims. |
| `docs/materials/2026-08-29-material-backdrop-blur-design.md` | `active delivery` | Current blur design, active frost investigation, DRM deferral, and noise/saturation question. |
| `docs/materials/2026-08-29-material-backdrop-blur-evidence.md` | `historical/superseded` | Completed nested verification evidence; retains the DRM deferral. |
| `docs/materials/README.md` | `authority/current` | Current Material index and status surface; reconciled with burn-in and the active defect. |
| `docs/materials/material-config.md` | `authority/current` | Current user-facing material configuration reference. |
| `docs/materials/plans/2026-08-22-repository-migration.md` | `historical/superseded` | Completed repository migration procedure. |
| `docs/materials/plans/2026-08-22-slice0.md` | `historical/superseded` | Executed renderer-seam procedure; unchecked boxes are historical. |
| `docs/materials/plans/2026-08-22-slice1.md` | `historical/superseded` | Completed config-and-assignment procedure. |
| `docs/materials/plans/2026-08-23-slice2.md` | `historical/superseded` | Completed shader/composition procedure. |
| `docs/materials/plans/2026-08-24-glass-config-surface.md` | `historical/superseded` | Executed configuration-surface procedure. |
| `docs/materials/plans/2026-08-24-slice3.md` | `historical/superseded` | Completed overview-crop procedure; latent unused `niri_size` mismatch has no consumer and no task. |
| `docs/materials/plans/2026-08-24-v1-parity.md` | `historical/superseded` | Executed parity procedure. |
| `docs/materials/plans/2026-08-26-v1-reference-static-preflight.md` | `historical/superseded` | Executed preflight and recapture procedure. |
| `docs/materials/plans/2026-08-27-v1-drm-acceptance.md` | `historical/superseded` | Executed v1 acceptance procedure; reusable harness remains current through its design. |
| `docs/materials/plans/2026-08-28-v1-daily-driver-rollout.md` | `historical/superseded` | Executed deployment procedure; status header now makes its unchecked steps unambiguously historical. |
| `docs/superpowers/plans/2026-08-29-material-backdrop-blur.md` | `historical/superseded` | Fully checked implementation/nested-verification plan; its separately deferred DRM run remains a candidate. |
| `docs/plans/2026-08-31-material-tasks-migration.md` | `historical/superseded` | Completed audit and migration ledger; live outcomes are tracked in the Material task store. |
| `README.md` | `authority/current` | Upstream project entry point; contains no Material-specific delivery claim. |
| `.agents/AGENTS.md` | `authority/current` | Current repository Tasks workflow guidance. |

There are no changed or Material-authored wiki exceptions. Material-relevant
`docs/wiki/Window-Effects.md`, `docs/wiki/Configuration:-Miscellaneous.md`, and
`docs/wiki/Configuration:-Window-Rules.md` accurately describe the inherited
background-effect surface, not the fork-only material surface.

## Drift corrections

| Claim | Evidence | Correction | Outward grep result |
| --- | --- | --- | --- |
| README said daily-driver deployment and two-stage burn-in were incomplete. | Commit `7020776e` is an ancestor and records roughly 13 hours, two cold starts, clean scoped journal review, and operator PASS. | README now records the passed deployment/burn-in and the separately accepted cosmetic defect. | Remaining `burn-in` future tense is confined to explicitly historical procedures or foreign Prism records. |
| The rollout plan header still said burn-in was in progress and named the superseded `7f6e69c3` package. | The deployed backdrop-blur source is `52f74f10`; commit `7020776e` records the final verdict. | Header now names `26.04.r133.g52f74f10-1`, the PASS evidence, and historical checkbox semantics. | The plan's unchecked steps remain immutable procedure, not current progress. |
| The rollout design's verdict section still said the rollout remained in burn-in. | Its own status header and ancestor `7020776e` say burn-in passed. | Verdict now records when deployment completed and separates the accepted cosmetic defect. | No current Material surface still calls deployment incomplete. |
| The rollout design attributed the drag observation primarily to opacity. | Four instrumented rounds ruled opacity out and proved dragged/stationary elements bind identical buffers and textures. | The design now points to the sampling-geometry investigation. | The active blur design and README agree on the narrowed cause. |
| The blur design's older defect paragraph said fallback versus sharp sampling was unknown. | Current instrumentation proves material readiness and the same blurred GL textures for both consumers. | The paragraph now records the sampling-geometry boundary and ruled-out causes. | The only remaining `not yet established` phrase concerns an irrelevant `Unchanged` buffer state already identified as a red herring. |
| Prism's native-sink and debug-backdrop docs still say burn-in is in progress. | Material commit `7020776e` is later evidence; Prism owns those files and Task 3 audits that repository next. | No cross-repository mutation in this task. | Recorded below as an externally owned, already completed outcome; it is not a Material task or blocker. |

## Candidate outcomes

| Outcome | Evidence | Sources | Active state | Size | Proposed status | Blockers | Disposition | Task ID |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Fix frost loss while interactively dragging a material window | Four live probe rounds narrowed the defect to sampling geometry; active five-file instrumentation is preserved. | `docs/materials/2026-08-29-material-backdrop-blur-design.md`; `debug/overview-drag-frost` diff | Branch and worktree active; owner `debug/overview-drag-frost` | `m` | `doing` | None | `create` | `material-e88df7` |
| Verify backdrop blur preserves v1 defaults on physical DRM | Nested verification passed, but the design and evidence explicitly defer byte-identical paired settled frames to the existing DRM harness. | `docs/materials/2026-08-29-material-backdrop-blur-design.md`; `docs/materials/2026-08-29-material-backdrop-blur-evidence.md`; `docs/materials/2026-08-27-v1-drm-acceptance-design.md` | Unstarted | `m` | `todo` | None | `create` | `material-ce3315` |
| Add damage-aware mipmapped backdrop storage and glass roughness | Accepted v1 design and DRM design call this the first post-v1 follow-up; parity design calls roughness the highest-value missing parameter. Current code has no mip/prefiltered storage or roughness field. | `docs/materials/2026-08-22-v1-design.md`; `docs/materials/2026-08-24-v1-parity-design.md`; `docs/materials/2026-08-27-v1-drm-acceptance-design.md`; `src/render_helpers/effect_buffer.rs` | Unstarted | `l` | `todo` | None | `create` | `material-c854bd` |
| Decide how noise and saturation compose with glass | The blur design identifies a real unresolved control-surface question and two measurements that require confirmation before implementation. | `docs/materials/2026-08-29-material-backdrop-blur-design.md`; `src/render_helpers/background_effect.rs`; `src/render_helpers/shaders/postprocess.frag` | Unscoped design question | `s` | `idea` | None | `create` | `material-cad932` |
| Daily-driver package deployment and burn-in | `52f74f10` is an ancestor; `7020776e` records deployment, two cold starts, journal review, and operator PASS. | `docs/materials/2026-08-28-v1-daily-driver-rollout-design.md` | Complete | — | — | — | `no task` | — |
| Prism owns and emits the native material definition | Prism `main` at `d20111c2` contains the renderer, manifest, definition, and tests; Material records the 2026-08-29 ownership handoff. | `~/d/prism/integrations/niri/**`; `~/d/prism/defs/glass.yaml`; rollout design | Complete; externally owned | — | — | — | `no task` | — |
| Correct Prism's stale burn-in prose | Material's completed burn-in evidence supersedes the foreign prose, but Prism documentation is outside this repository's migration write scope. | `~/d/prism/docs/superpowers/**`; Material `7020776e` | Pending Prism audit, externally owned | — | — | — | `no Material task`; Task 3 migration concern | — |
| Per-material blur strength, materials on layer surfaces, per-workspace assignment, user GLSL, true meshes, and other v1 exclusions | Each is explicitly a non-goal, unproven need, or broad deferred roadmap item; none has current delivery evidence. | Accepted designs and non-goals | Speculative | — | — | — | `no task` | — |
| Fix the unused cropped `niri_size` uniform | No shader reads it; the executed Slice 3 plan explicitly applies YAGNI until a consumer exists. | `docs/materials/plans/2026-08-24-slice3.md`; shader grep | Latent, no consumer | — | — | — | `no task` | — |

### Reviewed task fields

The literal task controls and CLI-created records use these reviewed values.

#### Overview drag frost

- Title: `Fix frost loss during interactive material-window drag`
- Status: `doing`
- Owner: `debug/overview-drag-frost`
- Priority: `2`
- Size: `m`
- Tags: `migration`, `bug`
- Body: `Outcome: frosted backdrop sampling remains visually stable while a material window is interactively dragged between workspaces in overview and normal scrolling. Acceptance evidence: remove temporary FROSTDBG instrumentation; add the smallest runnable regression check for the corrected sampling geometry; pass the source-neutral Rust gates; and record a focused live verification showing the dragged and stationary elements retain equivalent frost without fallback, sharp-texture selection, or sampling-coordinate divergence. Sources: docs/materials/2026-08-29-material-backdrop-blur-design.md and the preserved debug/overview-drag-frost five-file diff. Uncertainty: four probe rounds isolate the fault to sampling geometry passed to two elements reading identical blurred textures, but the exact bad coordinate or scale is not yet identified.`

#### Default-preserves-v1 DRM regression

- Title: `Verify backdrop blur preserves v1 defaults on physical DRM`
- Status: `todo`
- Owner: none
- Priority: `2`
- Size: `m`
- Tags: `migration`, `acceptance`, `hardware`
- Body: `Outcome: a build carrying backdrop-blur proves that the default-off material path preserves the accepted v1 appearance on the real DRM/NVIDIA path. Acceptance evidence: run the existing physical DRM harness scenarios unchanged; require byte-identical paired settled frames and all applicable machine and operator gates; record the pinned source, binary, environment, capture hashes, observations, cleanup, and reconciled verdict. Sources: docs/materials/2026-08-29-material-backdrop-blur-design.md, docs/materials/2026-08-29-material-backdrop-blur-evidence.md, and docs/materials/2026-08-27-v1-drm-acceptance-design.md. Uncertainty: nested verification proves opt-out equivalence on Weston but cannot establish physical scanout or the existing paired-frame contract.`

#### Mipmapped backdrop and roughness

- Title: `Add mipmapped backdrop storage and glass roughness`
- Status: `todo`
- Owner: none
- Priority: `2`
- Size: `l`
- Tags: `migration`, `feature`, `rendering`
- Body: `Outcome: glass exposes a meaningful roughness control backed by damage-aware, buffer-owned mipmapped or prefiltered background and backdrop storage rather than an inert shader knob. Acceptance evidence: define and review the config and renderer contract; verify damage, cache invalidation, per-target isolation, default appearance, and bounded allocation/performance behavior with runnable tests and focused visual evidence; update the material reference. Sources: docs/materials/2026-08-22-v1-design.md, docs/materials/2026-08-24-v1-parity-design.md, and docs/materials/2026-08-27-v1-drm-acceptance-design.md. Uncertainty: the accepted designs establish this as the first post-v1 follow-up and roughness as the highest-value missing parameter, but do not choose mip generation, filtering, or cache policy.`

#### Noise and saturation

- Title: `Decide how noise and saturation compose with glass`
- Status: `idea`
- Owner: none
- Priority: `2`
- Size: `s`
- Tags: `migration`, `design`, `rendering`
- Body: `Outcome: settle whether and where global blur noise and saturation should apply to glass transmission, with enough evidence to scope implementation or reject it. Acceptance evidence: confirm on a current build whether blur.offset at one pass is observable and whether saturation 0 reaches grayscale; trace the existing background-effect postprocess and material shader paths; record the chosen control surface, compositing order, defaults, and compatibility impact before code changes. Sources: docs/materials/2026-08-29-material-backdrop-blur-design.md, src/render_helpers/background_effect.rs, and src/render_helpers/shaders/postprocess.frag. Uncertainty: the follow-up is real, but current evidence does not establish whether the observations are material integration gaps, independent blur bugs, or intended behavior.`

No candidate has an already-resolvable blocker in the six baseline projects.

## Deferred foreign dependencies

None. No Material task depends on a not-yet-created Prism task. Prism's stale
burn-in prose is a foreign migration concern, not a delivery blocker for any
Material outcome.

## Verification

| Command | Result | Commit containing record |
| --- | --- | --- |
| `sha256sum --check /tmp/tasks-material-active-worktree.sha256` | PASS for all five preserved paths before audit and after worktree creation. | `b559a4c32d2e022107fd6048ced1f658429bbbcb` |
| `cargo test --all --exclude niri-visual-tests -- --nocapture` | PASS: 249 niri, 49 niri-config, 1 wiki parse, 3 niri-ipc, and 1 doctest; zero failures. | `b559a4c32d2e022107fd6048ced1f658429bbbcb` |
| `cargo clippy --all --all-targets` | PASS with pre-existing warnings and no denial/error. | `b559a4c32d2e022107fd6048ced1f658429bbbcb` |
| `cargo fmt --all -- --check` | Environmental baseline exception: stable rustfmt 1.9.0 from Rust 1.98 exits 1 on untouched `src/protocols/foreign_toplevel.rs`; the repository and CI require nightly rustfmt, which is unavailable on this host. The stable checkout fails identically and both copies hash to `41891c1ce0b3e4e5c59f51db9f82009ab2a677b12f50a8a2d9b60e443652ce50`; no source-format change is included. | `b559a4c32d2e022107fd6048ced1f658429bbbcb` |
| `(cd docs && uv sync --locked --all-extras --dev && uv run mkdocs build)` | PASS; strict documentation build completed. | `b559a4c32d2e022107fd6048ced1f658429bbbcb` |
| NUL-safe exact document coverage comparison | PASS after task creation: 114 actual paths equal 114 classified paths (111 tracked docs, this ledger, root README, and `.agents/AGENTS.md`). | Tasks-store commit `d064120f43fe15d8021f2dbdb4b469eba5da4710` |
| `tasks check`, `tasks prime`, and `tasks ready` under the temporary seven-project registry | PASS: zero errors or warnings; counts are 1 idea, 2 todo, 1 doing; ready contains the two todo outcomes. | Tasks-store commit `d064120f43fe15d8021f2dbdb4b469eba5da4710` |
| First stable fast-forward | PASS from `7e94d71af195d5f5062d9b51ec80bca513af8ae3` to `e4a982b82f55cae55c8fc8fc9a492f4ebb0b5849`, integrating exactly `b559a4c32d2e022107fd6048ced1f658429bbbcb`, `d064120f43fe15d8021f2dbdb4b469eba5da4710`, and `e4a982b82f55cae55c8fc8fc9a492f4ebb0b5849`. | Integration closeout `02caf4dbbb129a0baf8160ac27c584e18f24c235` |
| Stable `cargo test --all --exclude niri-visual-tests -- --nocapture` | PASS: 249 niri, 49 niri-config, 1 wiki parse, 3 niri-ipc, and 1 doctest; zero failures. | Integration closeout `02caf4dbbb129a0baf8160ac27c584e18f24c235` |
| Stable `cargo clippy --all --all-targets` | PASS with the same pre-existing non-denied warnings. | Integration closeout `02caf4dbbb129a0baf8160ac27c584e18f24c235` |
| Stable `cargo fmt --all -- --check` | Unchanged environmental baseline exception: stable rustfmt 1.9.0 exits 1 only on untouched `src/protocols/foreign_toplevel.rs`; output and stderr are byte-identical to the pre-integration stable baseline and the source still hashes to `41891c1ce0b3e4e5c59f51db9f82009ab2a677b12f50a8a2d9b60e443652ce50`. | Integration closeout `02caf4dbbb129a0baf8160ac27c584e18f24c235` |
| Stable strict MkDocs build | PASS under the required shared docs environment. | Integration closeout `02caf4dbbb129a0baf8160ac27c584e18f24c235` |
| Canonical `tasks init --prefix material`, run twice | PASS twice with identical JSON: prefix `material`, root set to the stable Material checkout, and no warnings. | Integration closeout `02caf4dbbb129a0baf8160ac27c584e18f24c235` |
| Canonical `tasks check`, `tasks prime`, and `tasks ready` | PASS: zero errors or warnings; prefix `material`; counts are 1 idea, 2 todo, 1 doing; ready contains `material-ce3315` and `material-c854bd`. | Integration closeout `02caf4dbbb129a0baf8160ac27c584e18f24c235` |
| Post-integration active-work fingerprint and Git-state gate | PASS for all five preserved checksums and exactly the five expected dirty paths, with no staged or untracked files. | Integration closeout `02caf4dbbb129a0baf8160ac27c584e18f24c235` |
