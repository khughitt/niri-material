# Noise Layers Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let a material write the glass `noise` node up to four times, each node a layer with its own amount, `type=`, `site=` and a new `scale=` (grain cell size), with single-node configs rendering byte-identical to today.

**Architecture:** The config decodes repeated `noise` children into four resolved slots and checks backdrop agreement over ordered lists of backdrop layers. The noise optic uploads four `vec4` uniforms (amount, type, site, scale; one component per slot); `noise.frag` gains a per-layer grain value (today's per-pixel hash at scale 1, a Hermite lattice divided by its own covariance-correct deviation above it) and applies the slots in order at each hook. The backdrop grain pass takes the same uniforms from `GrainOptions`. The schema gains the `noise scale=` parameter; nothing else in its shape changes.

**Tech Stack:** Rust (niri fork, smithay GLES renderer), knuffel KDL config, GLSL ES 1.00, cargo-nextest through `just test-one` / `just test-fast`, ImageMagick 7, headless Weston, Tracy 0.13.1 (retained tools), Python 3 stdlib, `tasks` CLI.

**Spec:** `docs/specs/2026-10-06-noise-layers-design.md` (this worktree, `.worktrees/material-3fcba2`). Section numbers below (§3, §4, ...) refer to it. It builds on `docs/specs/2026-10-05-noise-placement-design.md` (the sites) and `docs/materials/render-pipeline.md` (read it before touching the shaders).

**Status:** accepted for native execution 2026-10-06 after plan review round 4 (codex: accept, no findings; the real nextest `FAIL` line format stays an execution check, and an unexpected format fails closed). Round 3 (codex, 2026-10-06): the mutation harness aborts before editing when its backup fails, deletes the backup only after a verified restore, and counts a mutation as caught only on log evidence that the named test alone failed at the expected assertion; every exit path was simulated while planning. Round 2 (codex, 2026-10-06): mutation demonstrations restore a saved copy through one harness, since Task 2's run precedes its commit; the order test's configs are raw strings. Round 1 (codex, 2026-10-06): the smoke's `bins()` checks `identify` and the helper's status and clears stale values; the coefficient pin reads code, not comments, with mutation demonstrations; the cost cleanup test follows `stop_walls` and `cleanup_cost` into the lib; a fixed-seed, hook-ordered reference checks each site's slot order, with reversal demonstrations; a visible lattice blocks the merge unless fixed or the owner explicitly accepts deferral. Execution: native (owner, 2026-10-06).

**Execution:** native/inline 2026-10-06. Tasks 1–3 committed and verified; Tasks 4–5 scripts and offline checks prepared. The capture preflight refused CPU/load/GPU quietness before any cells ran. Baseline release build prepared separately; smoke/cost evidence and the owner’s contact-sheet review remain before Task 6.

## Global Constraints

- Branch `material-3fcba2` in `.worktrees/material-3fcba2`, on `materials-26.04`. Baseline commit `4a8b2072` (the task-start commit).
- Never call a test runner directly: `just test-one <nextest args>` for one test, `just test-fast` before every commit, `just check` before every commit, `just gate` before the merge.
- Commit order in this repository: `git add <files>` → `just upstream-report --stage` → `git add docs/materials/upstream-divergence.md` → `git commit`. Conventional commit subjects ending in `(material-3fcba2)`; no AI attribution of any kind.
- Each task is a child task of `material-3fcba2`: Task 1 `material-ae3a26`, Task 2 `material-cb80f7`, Task 3 `material-d184da`, Task 4 `material-a4d874` (needs `quiet`), Task 5 `material-2f2af4` (needs `quiet`), Task 6 `material-41d052`. `tasks start <id>` before its first step, `tasks done <id> "<one line>"` staged in its last commit.
- `NOISE_LAYERS = 4`. A fifth node fails with `material <name>: at most 4 noise layers, found <n>` (§3).
- `scale=` is `FloatOrInt<1, 16>`, omitted 1, in physical output pixels at every site (§3).
- Seeds: slot `k` adds `O_0 = (47, 113)`, `O_1 = (1301, 2659)`, `O_2 = (3709, 977)`, `O_3 = (2381, 3917)`. Glass and film layers use their material slot; backdrop layers use their position in the backdrop list (§3).
- Fine-lattice norm: `wᵀ C w = Σ w_c² - (1/3)(w00 w10 + w00 w01 + w10 w11 + w01 w11) - (7/18)(w00 w11 + w10 w01)`, literals `0.33333333` and `0.38888889` in `noise.frag`; white uses `Σ w_c²` (§4).
- A config with at most one `noise` node and no `scale=` renders byte-identical to `4a8b2072` at every site, including the inherited case (§1, §7.2 assertion 2). Every single-layer path keeps today's operation order: grain value, times amount, added in encoding; lightness keeps its direct linear return at the glass site.
- Backdrop agreement compares the ordered list of backdrop layers' `(amount, kind, scale)`; the error is `materials "<a>" and "<b>" place different noise at the backdrop (<list a>, <list b>); the backdrop is one texture per output`, a list printed as `[0.3 fine, 0.1 white scale 4]` (scale printed only when not 1) (§3).
- Schema version stays 1; `resources/materials/pipeline.json` and the `material-config.md` table are regenerated, never hand-edited; no multiplicity field (§6).
- No live desktop: in-process tests are headless; the capture lanes are nested Weston (`capture_preflight headless`). A preflight that refuses on host load parks the run `--reason quiet` (see Tasks 4 and 5).

## Review Focus

Five inputs the spec implies but its listed tests do not exercise; each has a test in the task named.

1. **Five `noise` nodes in one material, one of them at the backdrop.** Validation emits its error, but the invalid material is still pushed into the table and reaches the post-include backdrop check, which resolves it. A person expects a config error, not a compositor panic. Test `five_noise_layers_refuse_the_config_without_a_panic`, Task 1.
2. **Two materials with the same backdrop layers interleaved with different glass and film layers.** Agreement is about the backdrop list only; a person expects these to agree. Test `backdrop_agreement_compares_ordered_backdrop_lists`, Task 1.
3. **A reload that changes only one layer's `scale=`.** The glass must re-render: the material commit has to advance. Test `noise_scale_change_advances_the_commit_in_place`, Task 1.
4. **Amount-0 layers carrying other properties** (`noise 0 type="lightness" scale=8 site="film"`). A person muting a layer expects no visible change, whatever else the node says. Test `amount_zero_layers_are_neutral_with_any_properties`, Task 3.
5. **A non-integer `scale=`** (`scale=2.5`). Lattice cells then straddle pixels; a person expects grain of the same strength, not a seam or a weaker field. Test `a_fractional_scale_keeps_the_deviation`, Task 3.

## Mutation harness

Tasks 2 and 3 demonstrate that a test catches a defect by editing `noise.frag`, running the test, and restoring the file. Task 2's demonstrations run before its rewrite is committed, so `git checkout` would restore the old single-layer shader: every demonstration goes through this harness instead. It refuses to edit anything unless its backup was written and compares equal; it restores that exact copy on every exit path, including a failing or interrupted test run, and deletes the backup only after the restored file compares equal (otherwise it exits 3 and prints where the original is). A demonstration counts as caught only with evidence from the runner's log: exactly one test selected (`Starting 1 test across`), the summary `1 test run: 0 passed, 1 failed`, a `FAIL` line ending in the named test, and the expected assertion text. A build error, a runner failure (exit 127), a different test or a different assertion exits 4 with the log kept; a passing test exits 1. Create it once per session in a scratch directory (it is never committed):

```bash
M=$(mktemp -d)
cat > "$M/mutate.py" <<'PY'
"""mutate.py <noise.frag> <edit>: applies one named edit in place."""
import re
import sys

path, name = sys.argv[1], sys.argv[2]
text = open(path).read()


def reverse_calls(text, marker):
    """Reverses the order of the four one-line calls containing `marker`;
    each call keeps its own components and seed."""
    lines = text.split("\n")
    where = [i for i, line in enumerate(lines) if marker in line]
    assert len(where) == 4, f"{marker}: found {len(where)} calls"
    calls = [lines[i] for i in where]
    for i, line in zip(where, reversed(calls)):
        lines[i] = line
    return "\n".join(lines)


EDITS = {
    "adjacent": lambda t: t.replace("norm -= 0.33333333 *", "norm -= 0.33333300 *"),
    "diagonal": lambda t: t.replace("+ 0.38888889 * (w00 * w11", "+ 0.38888800 * (w00 * w11"),
    "no-correction": lambda t: re.sub(r"\n[ \t]*norm -= 0\.33333333.*?;", "", t, flags=re.S),
    "reverse-glass": lambda t: reverse_calls(t, "noiseBehindLayer(v, isLinear,"),
    "reverse-film": lambda t: reverse_calls(t, "encoded = noisePostLayer("),
    "reverse-backdrop": lambda t: reverse_calls(t, "straight = noiseSourceLayer("),
}
mutated = EDITS[name](text)
assert mutated != text, f"{name}: the edit changed nothing"
open(path, "w").write(mutated)
PY
cat > "$M/mutation.sh" <<'SH'
#!/usr/bin/env bash
# mutation.sh <edit> <test name> <expected failure, an extended regex>:
# saves noise.frag as it is now, applies one named edit, runs one test, and
# restores the saved copy on every exit path.
# Exit 0: the named test ran alone and failed with the expected message.
# 1: it passed, so the mutation survived. 2: the backup or the edit failed;
# noise.frag is untouched or restored. 3: restoring failed; the original is
# kept at the path printed. 4: the runner failed some other way (a build
# error, no test or another test selected, another assertion); its log is
# kept at the path printed.
set -u
[ $# -eq 3 ] || { echo "usage: mutation.sh <edit> <test> <expected regex>" >&2; exit 2; }
F=src/render_helpers/shaders/material/noise.frag
[ -s "$F" ] || { echo "mutation: $F is missing or empty" >&2; exit 2; }
SAVED=$(mktemp) || { echo "mutation: cannot create a backup file; nothing changed" >&2; exit 2; }
if ! cp "$F" "$SAVED" || ! cmp -s "$F" "$SAVED"; then
    rm -f "$SAVED"
    echo "mutation: backup of $F failed; nothing changed" >&2
    exit 2
fi
restore() {
    local rc=$?
    if cp "$SAVED" "$F" && cmp -s "$SAVED" "$F"; then
        rm -f "$SAVED"
        exit "$rc"
    fi
    echo "mutation: RESTORE FAILED; the original noise.frag is kept at $SAVED" >&2
    exit 3
}
trap restore EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
python3 -I "$(dirname "$0")/mutate.py" "$F" "$1" || exit 2
LOG=$(mktemp) || exit 2
just test-one -p niri "$2" > "$LOG" 2>&1
status=$?
selected_one() { grep -Eq '^ *Starting 1 test across' "$LOG"; }
if [ "$status" -eq 0 ] && selected_one && grep -Eq 'Summary \[.*\] 1 test run: 1 passed' "$LOG"; then
    rm -f "$LOG"
    echo "MUTATION SURVIVED: $1 ($2 passed)" >&2
    exit 1
fi
if [ "$status" -ne 0 ] && selected_one \
    && grep -Eq 'Summary \[.*\] 1 test run: 0 passed, 1 failed' "$LOG" \
    && grep -Eq "FAIL .*::$2\$" "$LOG" \
    && grep -Eq -- "$3" "$LOG"; then
    echo "caught: $1 ($2: $(grep -Eo -- "$3" "$LOG" | head -1))"
    rm -f "$LOG"
    exit 0
fi
echo "mutation: $2 did not fail as expected under $1 (runner exit $status); log kept at $LOG" >&2
exit 4
SH
chmod +x "$M/mutation.sh"
```

Run it from `.worktrees/material-3fcba2`. Around each group of demonstrations, keep a reference copy and compare afterwards: `cp src/render_helpers/shaders/material/noise.frag "$M/before.frag"` first, `cmp src/render_helpers/shaders/material/noise.frag "$M/before.frag"` last (expected: no output).

---

### Task 1: Config: repeated noise nodes, `scale=`, list agreement, the parameter row

**Files:**
- Modify: `niri-config/src/material/optics/noise.rs` (the node, resolution, validation, backdrop agreement, `params()`)
- Modify: `niri-config/src/material/mod.rs:524` (`Glass.noise` becomes a `Vec`), `:789` (`resolve` call), `:803` (`Material::validate`)
- Modify: `niri-config/src/lib.rs` (re-exports near line 60; tests listed in Step 1)
- Modify: `niri-config/src/material/pipeline.rs:313,430-434,534` (the three noise stages own/read `noise scale=`)
- Modify: `src/render_helpers/material/optics/noise.rs` (interim: slot 0 drives the existing float uniforms)
- Modify: `src/render_helpers/grain.rs:23-30` (interim: `From<BackdropGrain>` takes the first backdrop layer)
- Modify: `src/render_helpers/material/optics/mod.rs:207`, `src/layout/tile.rs:2808,2852`, `src/render_helpers/material/mod.rs:1325,1416` (literals and field writes)
- Regenerate: `docs/materials/material-config.md` (table between `<!-- params:begin -->` and `<!-- params:end -->`), `resources/materials/pipeline.json`

This task is green on its own: the renderer keeps its float uniforms and reads slot 0 only, which is exactly today's behaviour for every config that parses today. Layers 1 to 3 and `scale=` are decoded and validated but not yet rendered; Task 2 renders them.

**Interfaces:**
- Produces (in `niri_config::material::optics::noise`, re-exported from `niri_config`): `pub const NOISE_LAYERS: usize = 4`; `Noise { amount, kind, site, scale: Option<FloatOrInt<1, 16>> }`; `pub struct ResolvedNoiseLayer { pub amount: f64, pub kind: NoiseType, pub site: NoiseSite, pub scale: f64 }`; `pub struct ResolvedNoise { pub layers: [Option<ResolvedNoiseLayer>; NOISE_LAYERS] }` with `ResolvedNoise::single(amount: f64, kind: NoiseType, site: NoiseSite) -> Self` and `ResolvedNoise::is_omitted(&self) -> bool`; `pub fn resolve(nodes: &[Noise]) -> ResolvedNoise`; `pub fn validate(nodes: &[Noise]) -> Result<(), String>`; `pub struct BackdropLayer { pub amount: f64, pub kind: NoiseType, pub scale: f64 }`; `pub struct BackdropGrain { pub layers: [Option<BackdropLayer>; NOISE_LAYERS] }` with `Display` (`[0.3 fine, 0.1 white scale 4]`); `pub fn backdrop_grain(materials: &[Material]) -> Result<Option<BackdropGrain>, String>`; `Config::backdrop_grain(&self) -> Option<BackdropGrain>` (unchanged signature). `Glass.noise: Vec<Noise>`. `ParamSpec` node `"noise scale="`.
- Consumed by: Task 2 (slots, `NOISE_LAYERS`, `BackdropGrain` layers), Task 3 (configs).

- [x] **Step 1: Write the failing config tests**

In `niri-config/src/lib.rs` (its test module imports through `use super::*`, so Step 3's re-exports reach it), replace these existing tests with the versions below (same names unless stated), and add the new ones after `glass_noise_site_rejects_an_unknown_value`:

```rust
    #[test]
    fn glass_noise_and_saturation_parse_as_written() {
        let parsed = do_parse(
            r##"
            material "frost" {
                glass {
                    noise 0.02
                    saturation 0.85
                }
            }
            "##,
        );
        let glass = parsed.materials[0].resolve().glass;
        assert_eq!(glass.noise.layers[0].map(|l| l.amount), Some(0.02));
        assert_eq!(glass.saturation.amount, Some(0.85));
    }

    #[test]
    fn glass_noise_and_saturation_resolve_independently() {
        let noise_only = do_parse(r##"material "frost" { glass { noise 0.5; }; }"##);
        let glass = noise_only.materials[0].resolve().glass;
        assert_eq!(glass.noise.layers[0].map(|l| l.amount), Some(0.5));
        assert_eq!(glass.saturation.amount, None);

        let saturation_only = do_parse(r##"material "frost" { glass { saturation 0; }; }"##);
        let glass = saturation_only.materials[0].resolve().glass;
        assert!(glass.noise.is_omitted());
        assert_eq!(glass.saturation.amount, Some(0.));
    }

    #[test]
    fn noise_resolves_through_its_optic() {
        let written = do_parse(r##"material "frost" { glass { noise 0.5 type="fine"; }; }"##);
        assert_eq!(
            written.materials[0].resolve().glass.noise,
            ResolvedNoise::single(0.5, NoiseType::Fine, NoiseSite::Glass)
        );
        let omitted = do_parse(r##"material "frost" { glass {}; }"##);
        assert_eq!(
            omitted.materials[0].resolve().glass.noise,
            ResolvedNoise::default()
        );
        assert!(ResolvedNoise::default().is_omitted());
    }

    #[test]
    fn glass_noise_and_saturation_default_to_inherit() {
        let d = ResolvedGlass::default();
        assert!(d.noise.is_omitted());
        assert_eq!(d.saturation.amount, None);
    }

    #[test]
    fn glass_noise_type_parses_each_value() {
        for (written, expected) in [
            ("white", NoiseType::White),
            ("fine", NoiseType::Fine),
            ("lightness", NoiseType::Lightness),
        ] {
            let parsed = do_parse(&format!(
                "material \"frost\" {{ glass {{ noise 0.02 type=\"{written}\"; }}; }}\n"
            ));
            let layer = parsed.materials[0].resolve().glass.noise.layers[0].unwrap();
            assert_eq!(layer.amount, 0.02, "{written}");
            assert_eq!(layer.kind, expected, "{written}");
        }
    }

    #[test]
    fn glass_noise_site_parses_each_value_and_defaults_to_glass() {
        for (written, expected) in [
            ("glass", NoiseSite::Glass),
            ("backdrop", NoiseSite::Backdrop),
            ("film", NoiseSite::Film),
        ] {
            let parsed = do_parse(&format!(
                "material \"frost\" {{ glass {{ noise 0.3 type=\"fine\" site=\"{written}\"; }}; }}\n"
            ));
            let layer = parsed.materials[0].resolve().glass.noise.layers[0].unwrap();
            assert_eq!(layer.site, expected, "{written}");
            assert_eq!(layer.amount, 0.3, "{written}");
            assert_eq!(layer.kind, NoiseType::Fine, "{written}");
        }
        let parsed = do_parse("material \"frost\" { glass { noise 0.3; }; }\n");
        assert_eq!(
            parsed.materials[0].resolve().glass.noise.layers[0].unwrap().site,
            NoiseSite::Glass
        );
        let parsed = do_parse("material \"frost\" { glass { }; }\n");
        assert_eq!(
            parsed.materials[0].resolve().glass.noise,
            ResolvedNoise::default()
        );
    }

    #[test]
    fn an_omitted_noise_type_resolves_to_white_and_keeps_the_amount_rule() {
        let written = do_parse(r##"material "frost" { glass { noise 0.5; }; }"##);
        let layer = written.materials[0].resolve().glass.noise.layers[0].unwrap();
        assert_eq!(layer.amount, 0.5);
        assert_eq!(layer.kind, NoiseType::White);

        let omitted = do_parse(r##"material "frost" { glass {}; }"##);
        assert!(omitted.materials[0].resolve().glass.noise.is_omitted());
        assert!(ResolvedGlass::default().noise.is_omitted());
    }

    fn backdrop(amount: f64, kind: NoiseType, scale: f64) -> Option<BackdropLayer> {
        Some(BackdropLayer {
            amount,
            kind,
            scale,
        })
    }

    /// One material, parsed alone so that two disagreeing ones can meet in
    /// `backdrop_grain` without the config refusing them first.
    fn material(text: &str) -> Material {
        do_parse(text).materials.remove(0)
    }

    #[test]
    fn backdrop_grain_disagreement_names_both_materials() {
        let a = material("material \"a\" { glass { noise 0.3 type=\"fine\" site=\"backdrop\"; }; }\n");
        let b = material("material \"b\" { glass { noise 0.1 type=\"fine\" site=\"backdrop\"; }; }\n");
        assert_eq!(
            crate::material::optics::noise::backdrop_grain(&[a.clone(), b]),
            Err(String::from(
                "materials \"a\" and \"b\" place different noise at the backdrop \
                 ([0.3 fine], [0.1 fine]); the backdrop is one texture per output"
            ))
        );
        let c = material("material \"c\" { glass { noise 0.3 type=\"white\" site=\"backdrop\"; }; }\n");
        assert_eq!(
            crate::material::optics::noise::backdrop_grain(&[a, c]),
            Err(String::from(
                "materials \"a\" and \"c\" place different noise at the backdrop \
                 ([0.3 fine], [0.3 white]); the backdrop is one texture per output"
            ))
        );
        // The decoded config refuses the pair; miette wraps long messages,
        // so only a short fragment is asserted here.
        let err = do_parse_err(
            "material \"a\" { glass { noise 0.3 type=\"fine\" site=\"backdrop\"; }; }\n\
             material \"b\" { glass { noise 0.1 type=\"fine\" site=\"backdrop\"; }; }\n",
        );
        assert!(err.contains("place different noise"), "{err}");
    }

    #[test]
    fn backdrop_grain_agreement_is_order_independent() {
        for order in [["a", "b"], ["b", "a"]] {
            let parsed = do_parse(&format!(
                "material \"{}\" {{ glass {{ noise 0.3 type=\"fine\" site=\"backdrop\"; }}; }}\n\
                 material \"{}\" {{ glass {{ noise 0.3 type=\"fine\" site=\"backdrop\"; }}; }}\n\
                 material \"c\" {{ glass {{ noise 0.9 type=\"white\" site=\"glass\"; }}; }}\n\
                 material \"d\" {{ glass {{ noise 0.5 site=\"film\"; }}; }}\n",
                order[0], order[1]
            ));
            assert_eq!(
                parsed.backdrop_grain(),
                Some(BackdropGrain {
                    layers: [backdrop(0.3, NoiseType::Fine, 1.), None, None, None],
                }),
                "{order:?}"
            );
        }
        let parsed = do_parse("material \"c\" { glass { noise 0.9; }; }\n");
        assert_eq!(parsed.backdrop_grain(), None);
    }

    #[test]
    fn glass_noise_layers_parse_in_order_with_their_scale() {
        let parsed = do_parse(
            "material \"film-stock\" { glass {\n\
                 noise 0.04 type=\"lightness\"\n\
                 noise 0.03 type=\"white\" scale=4\n\
                 noise 0.02 type=\"fine\" site=\"film\" scale=2.5\n\
             }; }\n",
        );
        let noise = parsed.materials[0].resolve().glass.noise;
        let layer = |amount, kind, site, scale| {
            Some(ResolvedNoiseLayer {
                amount,
                kind,
                site,
                scale,
            })
        };
        assert_eq!(
            noise.layers,
            [
                layer(0.04, NoiseType::Lightness, NoiseSite::Glass, 1.),
                layer(0.03, NoiseType::White, NoiseSite::Glass, 4.),
                layer(0.02, NoiseType::Fine, NoiseSite::Film, 2.5),
                None,
            ]
        );
        assert!(!noise.is_omitted());
    }

    #[test]
    fn four_noise_layers_parse() {
        let parsed = do_parse(
            "material \"m\" { glass { noise 0.1; noise 0.1; noise 0.1; noise 0.1; }; }\n",
        );
        assert!(parsed.materials[0].resolve().glass.noise.layers[3].is_some());
    }

    #[test]
    fn five_noise_layers_refuse_the_config_without_a_panic() {
        // The invalid material is still pushed into the table and reaches
        // the post-include backdrop check, which resolves it.
        let err = do_parse_err(
            "material \"film-stock\" { glass {\n\
                 noise 0.1 site=\"backdrop\"\nnoise 0.1\nnoise 0.1\nnoise 0.1\nnoise 0.1\n\
             }; }\n",
        );
        assert!(err.contains("at most 4 noise layers, found 5"), "{err}");
        assert_eq!(
            Material {
                name: String::from("film-stock"),
                glass: material(
                    "material \"film-stock\" { glass { noise 0.1; noise 0.1; noise 0.1; noise 0.1; }; }\n"
                )
                .glass,
                responses: Vec::new(),
            }
            .validate(),
            Ok(())
        );
    }

    #[test]
    fn glass_noise_scale_is_bounded_and_defaults_to_one() {
        for value in ["0.5", "16.01", "17"] {
            let err = do_parse_err(&format!(
                "material \"frost\" {{ glass {{ noise 0.3 scale={value}; }}; }}\n"
            ));
            assert!(err.contains("value must be between 1 and 16"), "{value}: {err}");
        }
        for (written, expected) in [("1", 1.), ("16", 16.), ("2.5", 2.5)] {
            let parsed = do_parse(&format!(
                "material \"frost\" {{ glass {{ noise 0.3 scale={written}; }}; }}\n"
            ));
            let layer = parsed.materials[0].resolve().glass.noise.layers[0].unwrap();
            assert_eq!(layer.scale, expected, "{written}");
        }
        let parsed = do_parse("material \"frost\" { glass { noise 0.3; }; }\n");
        assert_eq!(
            parsed.materials[0].resolve().glass.noise.layers[0].unwrap().scale,
            1.
        );
    }

    #[test]
    fn amount_zero_layers_keep_their_slots() {
        let parsed = do_parse("material \"m\" { glass { noise 0; noise 0.3 type=\"fine\"; }; }\n");
        let noise = parsed.materials[0].resolve().glass.noise;
        assert_eq!(noise.layers[0].unwrap().amount, 0.);
        assert_eq!(noise.layers[1].unwrap().amount, 0.3);
        assert!(!noise.is_omitted());
    }

    #[test]
    fn backdrop_agreement_compares_ordered_backdrop_lists() {
        let a = "material \"a\" { glass { noise 0.3 type=\"fine\" site=\"backdrop\"; \
                 noise 0.1 type=\"white\" site=\"backdrop\" scale=4; }; }\n";
        // The same backdrop list between other sites' layers agrees.
        let parsed = do_parse(&format!(
            "{a}material \"b\" {{ glass {{ noise 0.5 site=\"film\"; \
             noise 0.3 type=\"fine\" site=\"backdrop\"; noise 0.2; \
             noise 0.1 type=\"white\" site=\"backdrop\" scale=4; }}; }}\n"
        ));
        assert_eq!(
            parsed.backdrop_grain(),
            Some(BackdropGrain {
                layers: [
                    backdrop(0.3, NoiseType::Fine, 1.),
                    backdrop(0.1, NoiseType::White, 4.),
                    None,
                    None,
                ],
            })
        );
        // Length, order and scale each disagree.
        for (b, listed) in [
            (
                "noise 0.3 type=\"fine\" site=\"backdrop\"",
                "[0.3 fine]",
            ),
            (
                "noise 0.1 type=\"white\" site=\"backdrop\" scale=4; \
                 noise 0.3 type=\"fine\" site=\"backdrop\"",
                "[0.1 white scale 4, 0.3 fine]",
            ),
            (
                "noise 0.3 type=\"fine\" site=\"backdrop\"; \
                 noise 0.1 type=\"white\" site=\"backdrop\" scale=8",
                "[0.3 fine, 0.1 white scale 8]",
            ),
        ] {
            let b = material(&format!("material \"b\" {{ glass {{ {b}; }}; }}\n"));
            assert_eq!(
                crate::material::optics::noise::backdrop_grain(&[material(a), b]),
                Err(format!(
                    "materials \"a\" and \"b\" place different noise at the backdrop \
                     ([0.3 fine, 0.1 white scale 4], {listed}); the backdrop is one \
                     texture per output"
                )),
                "{listed}"
            );
        }
    }
```

`Material::validate` is `pub(crate)` and `Material` derives `Clone`, so both calls above work from the crate's tests.

In `src/render_helpers/material/mod.rs` tests, add after `noise_type_change_advances_the_commit_in_place`:

```rust
    #[test]
    fn noise_scale_change_advances_the_commit_in_place() {
        let mut base = render_config("frost");
        base.material.glass.noise = niri_config::ResolvedNoise::single(
            0.04,
            niri_config::NoiseType::Fine,
            niri_config::NoiseSite::Glass,
        );
        let mut slot = Some(MaterialState::new(base.clone()));
        let id_before = slot.as_ref().unwrap().id().clone();
        let initial_commit = slot.as_ref().unwrap().commit.get();

        let mut changed = base;
        changed.material.glass.noise.layers[0].as_mut().unwrap().scale = 4.;
        assert!(!apply_resolved(&mut slot, Some(&changed)));
        assert_eq!(slot.as_ref().unwrap().id(), &id_before);
        assert_ne!(slot.as_ref().unwrap().commit.get(), initial_commit);
    }
```

- [x] **Step 2: Run them to see them fail**

Run: `just test-one -p niri-config glass_noise`
Expected: compile errors (`ResolvedNoise::single`, `layers`, `ResolvedNoiseLayer`, `BackdropLayer` unknown).

- [x] **Step 3: Implement the config side**

Replace the body of `niri-config/src/material/optics/noise.rs` from the `Noise` struct to the end of `backdrop_grain` with:

```rust
/// How many `noise` nodes one material may write (design
/// `2026-10-06-noise-layers-design.md` §3).
pub const NOISE_LAYERS: usize = 4;

/// `noise <amount> type=<type> site=<site> scale=<px>`, one layer.
///
/// The type, site and scale do nothing while the amount is zero, so they
/// ride the node they depend on, as `distortion` carries `scale=`. An omitted
/// type is `white`, an omitted site `glass` and an omitted scale 1, which
/// render exactly as the node did before the properties existed.
#[derive(knuffel::Decode, Debug, Clone, Copy, PartialEq)]
pub struct Noise {
    #[knuffel(argument)]
    pub amount: FloatOrInt<0, 1>,
    #[knuffel(property(name = "type"), str)]
    pub kind: Option<NoiseType>,
    #[knuffel(property(name = "site"), str)]
    pub site: Option<NoiseSite>,
    /// Grain cell size in physical output pixels; 1 is per-pixel grain.
    #[knuffel(property)]
    pub scale: Option<FloatOrInt<1, 16>>,
}

/// One written layer, every property resolved.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResolvedNoiseLayer {
    pub amount: f64,
    pub kind: NoiseType,
    pub site: NoiseSite,
    pub scale: f64,
}

/// Final noise state: the written layers in config order, then `None`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ResolvedNoise {
    /// All `None` means no node was written. Only then does the inherit rule
    /// apply, in the renderer's optic: slot 0 becomes white grain at the
    /// glass site, scale 1, with the global `blur` block's amount while
    /// backdrop blur is effective and 0 otherwise. A written layer never
    /// inherits.
    pub layers: [Option<ResolvedNoiseLayer>; NOISE_LAYERS],
}

impl ResolvedNoise {
    /// One written layer at scale 1.
    pub fn single(amount: f64, kind: NoiseType, site: NoiseSite) -> Self {
        let mut layers = [None; NOISE_LAYERS];
        layers[0] = Some(ResolvedNoiseLayer {
            amount,
            kind,
            site,
            scale: 1.,
        });
        Self { layers }
    }

    /// Whether no node was written, so slot 0 inherits.
    pub fn is_omitted(&self) -> bool {
        self.layers[0].is_none()
    }
}

/// The written nodes, in order. A rejected material still reaches the
/// post-include backdrop check, so this takes at most `NOISE_LAYERS` nodes
/// rather than assuming `validate` passed; an accepted config never has more.
pub fn resolve(nodes: &[Noise]) -> ResolvedNoise {
    let mut layers = [None; NOISE_LAYERS];
    for (slot, node) in layers.iter_mut().zip(nodes) {
        *slot = Some(ResolvedNoiseLayer {
            amount: node.amount.0,
            kind: node.kind.unwrap_or_default(),
            site: node.site.unwrap_or_default(),
            scale: node.scale.map_or(1., |scale| scale.0),
        });
    }
    ResolvedNoise { layers }
}

pub fn validate(nodes: &[Noise]) -> Result<(), String> {
    if nodes.len() > NOISE_LAYERS {
        return Err(format!(
            "at most {NOISE_LAYERS} noise layers, found {}",
            nodes.len()
        ));
    }
    Ok(())
}

/// One backdrop layer as the output's grain pass applies it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BackdropLayer {
    pub amount: f64,
    pub kind: NoiseType,
    pub scale: f64,
}

/// The grain an output's effect buffer applies: the backdrop-site layers,
/// in order, that every material placing noise there agrees on (design §3).
/// Slot `k` here is the `k`-th backdrop layer, which also picks its seed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BackdropGrain {
    pub layers: [Option<BackdropLayer>; NOISE_LAYERS],
}

impl BackdropGrain {
    fn of(noise: &ResolvedNoise) -> Option<Self> {
        let mut layers = [None; NOISE_LAYERS];
        let placed = noise
            .layers
            .iter()
            .flatten()
            .filter(|layer| layer.site == NoiseSite::Backdrop);
        for (slot, layer) in layers.iter_mut().zip(placed) {
            *slot = Some(BackdropLayer {
                amount: layer.amount,
                kind: layer.kind,
                scale: layer.scale,
            });
        }
        layers[0].is_some().then_some(Self { layers })
    }
}

impl std::fmt::Display for BackdropGrain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[")?;
        for (i, layer) in self.layers.iter().flatten().enumerate() {
            if i > 0 {
                f.write_str(", ")?;
            }
            write!(f, "{} {}", layer.amount, NoiseType::NAMES[layer.kind as usize])?;
            if layer.scale != 1. {
                write!(f, " scale {}", layer.scale)?;
            }
        }
        f.write_str("]")
    }
}

/// The backdrop grain the material table agrees on: `None` when no material
/// places noise there, the agreed list otherwise, and an error naming the
/// first two materials that disagree. The backdrop is one texture per
/// output, so there is no per-material value to fall back on.
pub fn backdrop_grain(
    materials: &[crate::material::Material],
) -> Result<Option<BackdropGrain>, String> {
    let mut agreed: Option<(&str, BackdropGrain)> = None;
    for material in materials {
        let Some(grain) = BackdropGrain::of(&resolve(&material.glass.noise)) else {
            continue;
        };
        match agreed {
            None => agreed = Some((&material.name, grain)),
            Some((first, g)) if g != grain => {
                return Err(format!(
                    "materials \"{first}\" and \"{}\" place different noise at the backdrop \
                     ({g}, {grain}); the backdrop is one texture per output",
                    material.name,
                ));
            }
            Some(_) => {}
        }
    }
    Ok(agreed.map(|(_, grain)| grain))
}
```

Update the module doc comment's first line to `` //! `noise <amount> type=<type> site=<site> scale=<px>`, up to four layers: grain on the shared backdrop, ``. In `params()`, change the `noise` row's read and add the scale row after `noise site=`:

```rust
        ParamSpec {
            node: "noise",
            kind: ParamKind::inherit::<FloatOrInt<0, 1>>(),
            write: |v| format!("noise {v}"),
            read: Some(|g| g.noise.layers[0].map(|layer| layer.amount)),
        },
```

```rust
        ParamSpec {
            node: "noise scale=",
            kind: ParamKind::float::<FloatOrInt<1, 16>>(1., "px"),
            write: |v| format!("noise 0.5 scale={v}"),
            read: Some(|g| Some(g.noise.layers[0].map_or(1., |layer| layer.scale))),
        },
```

In `niri-config/src/material/mod.rs`: `Glass.noise` becomes

```rust
    #[knuffel(children(name = "noise"))]
    pub noise: Vec<optics::noise::Noise>,
```

line 789 becomes `noise: optics::noise::resolve(&g.noise),`, and `Material::validate` gains, right after the aurora check:

```rust
        optics::noise::validate(&self.glass.noise)
            .map_err(|message| format!("material {}: {message}", self.name))?;
```

In `niri-config/src/lib.rs`, the re-export becomes:

```rust
pub use crate::material::optics::noise::{
    BackdropGrain, BackdropLayer, Noise, NoiseSite, NoiseType, ResolvedNoise, ResolvedNoiseLayer,
    NOISE_LAYERS,
};
```

In `niri-config/src/material/pipeline.rs`, the three noise stages (`backdrop-grain`, `noise`, `film-grain`) list `"noise scale="` after `"noise site="` in `reads`, and the `noise` stage also in `owns`:

```rust
            &["noise", "noise type=", "noise site=", "noise scale="],
```

- [x] **Step 4: Keep the renderer on slot 0 (interim)**

`src/render_helpers/material/optics/noise.rs`, `values`:

```rust
    fn values(glass: &ResolvedGlass, ctx: &OpticFrame<'_>) -> Vec<Uniform<'static>> {
        // Interim (plan Task 1): slot 0 only; Task 2 uploads every slot.
        let first = glass.noise.layers[0];
        let inherited = if ctx.backdrop_blur { ctx.blur.noise } else { 0. };
        vec![
            Uniform::new("mat_noise", first.map_or(inherited, |l| l.amount) as f32),
            Uniform::new(
                "mat_noise_type",
                first.map_or(0., |l| l.kind as u8 as f32),
            ),
            Uniform::new(
                "mat_noise_site",
                first.map_or(0., |l| l.site as u8 as f32),
            ),
        ]
    }
```

In its tests, replace each `ResolvedNoise { amount: Some(a), kind: k, site: s }` literal with `ResolvedNoise::single(a, k, s)`.

`src/render_helpers/grain.rs`, interim conversion:

```rust
impl From<BackdropGrain> for GrainOptions {
    // Interim (plan Task 1): the first backdrop layer; Task 2 takes all.
    fn from(grain: BackdropGrain) -> Self {
        let first = grain.layers[0].expect("a BackdropGrain has a first layer");
        Self {
            amount: first.amount as f32,
            kind: first.kind,
        }
    }
}
```

Literals:
- `src/render_helpers/material/optics/mod.rs:207`: `noise: ResolvedNoise::single(0.2, NoiseType::White, NoiseSite::Glass),` and import `NoiseSite, NoiseType` beside `ResolvedNoise`.
- `src/layout/tile.rs:2808`: `noise: noise.map_or_else(niri_config::ResolvedNoise::default, |amount| niri_config::ResolvedNoise::single(amount, niri_config::NoiseType::White, niri_config::NoiseSite::Glass)),`
- `src/layout/tile.rs:2852`: `noise: niri_config::ResolvedNoise::single(0.3, noise_type, niri_config::NoiseSite::Glass),`
- `src/render_helpers/material/mod.rs:1325`: `changed.material.glass.noise = niri_config::ResolvedNoise::single(0.04, niri_config::NoiseType::White, niri_config::NoiseSite::Glass);`
- `src/render_helpers/material/mod.rs:1416` (inside `noise_type_change_advances_the_commit_in_place`): the test starts from a written white layer so the kind alone changes:

```rust
    #[test]
    fn noise_type_change_advances_the_commit_in_place() {
        let mut base = render_config("frost");
        base.material.glass.noise = niri_config::ResolvedNoise::single(
            0.04,
            niri_config::NoiseType::White,
            niri_config::NoiseSite::Glass,
        );
        let mut slot = Some(MaterialState::new(base.clone()));
        let id_before = slot.as_ref().unwrap().id().clone();
        let initial_commit = slot.as_ref().unwrap().commit.get();

        let mut changed = base;
        changed.material.glass.noise.layers[0].as_mut().unwrap().kind =
            niri_config::NoiseType::Fine;
        assert!(!apply_resolved(&mut slot, Some(&changed)));
        assert_eq!(slot.as_ref().unwrap().id(), &id_before);
        assert_ne!(slot.as_ref().unwrap().commit.get(), initial_commit);
    }
```

- [x] **Step 5: Regenerate the table and the schema file**

```bash
MATERIAL_DOCS_UPDATE=1 just test-one -p niri-config material_parameter_table_matches_the_docs
MATERIAL_DOCS_UPDATE=1 just test-one -p niri-config material_pipeline_schema_matches_the_file
git diff --stat docs/materials/material-config.md resources/materials/pipeline.json
```

Expected: the table gains one row, `` | `noise` `scale=` | float | 1 | 1–16 | px | `` (the renderer's column format); `pipeline.json` gains `"noise scale="` in the three noise stages' lists and nothing else.

- [x] **Step 6: Run the tests**

Run: `just test-one -p niri-config noise` then `just test-one -p niri-config material_parameter` then `just test-one -p niri noise` then `just test-fast`.
Expected: all PASS (the `niri` noise tests include `src/tests/noise_site.rs`, whose identities still hold on slot 0).

- [x] **Step 7: Commit**

```bash
tasks done material-ae3a26 "Config: up to four noise nodes with scale=, list agreement at the backdrop, the noise scale= row"
just check
git add niri-config src docs/materials/material-config.md resources/materials/pipeline.json tasks/
just upstream-report --stage
git add docs/materials/upstream-divergence.md
git commit -m "feat(material): decode up to four noise layers with a scale each (material-3fcba2)"
```

---

### Task 2: Four slots in the noise shader and both programs

**Files:**
- Modify: `src/render_helpers/shaders/material/noise.frag` (whole file)
- Modify: `src/render_helpers/material/optics/noise.rs` (four `vec4` uniforms; tests rewritten)
- Modify: `src/render_helpers/grain.rs` (`GrainOptions` holds layers; four-component uniforms; tests)
- Modify: `src/render_helpers/effect_buffer.rs:859-910` (test constructions of `GrainOptions`)
- Modify: `src/render_helpers/shaders/mod.rs:578` (declaration assertion), `:768-776` (`uniform_nodes`)
- Modify: `src/layout/tile.rs` (tests reading `mat_noise` / `mat_noise_type` as floats)

**Interfaces:**
- Consumes: `ResolvedNoise.layers`, `NOISE_LAYERS`, `BackdropGrain.layers` (Task 1).
- Produces: `GrainOptions { pub layers: [Option<GrainLayer>; NOISE_LAYERS] }`, `GrainLayer { pub amount: f32, pub kind: NoiseType, pub scale: f32 }`, `GrainOptions::uniforms(&self) -> [[f32; 4]; 3]` (amounts, kinds, scales), test-only `GrainOptions::one(amount: f32, kind: NoiseType) -> Self`; uniforms `mat_noise`, `mat_noise_type`, `mat_noise_site`, `mat_noise_scale`, all `vec4`. GLSL: `noiseValue(vec2, float type, float scale, vec2 offset)`, `noiseApply(vec3, float grain, float type)`, hooks `noise_behind`, `noise_post`, `noise_source` unchanged in signature.

- [x] **Step 1: Write the failing optic and grain tests**

Replace the `tests` module of `src/render_helpers/material/optics/noise.rs` with:

```rust
#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::time::Duration;

    use niri_config::{
        Blur, NoiseSite, NoiseType, ResolvedGlass, ResolvedNoise, ResolvedNoiseLayer,
    };
    use smithay::backend::renderer::gles::UniformValue;

    use super::*;
    use crate::render_helpers::material::optics::OpticFrame;

    fn frame(backdrop_blur: bool, blur: &Blur) -> OpticFrame<'_> {
        OpticFrame {
            logical_now: Duration::ZERO,
            motion: niri_config::signal::SignalMotionPolicy::Full,
            animations_off: false,
            backdrop_blur,
            blur,
            seed: 0.,
        }
    }

    /// The four uniform vectors: amount, type, site, scale.
    fn vectors(glass: &ResolvedGlass, backdrop_blur: bool, blur: &Blur) -> [[f32; 4]; 4] {
        let values = NoiseOptic::values(glass, &frame(backdrop_blur, blur));
        let names: Vec<_> = values.iter().map(|u| u.name.clone()).collect();
        assert_eq!(
            names,
            ["mat_noise", "mat_noise_type", "mat_noise_site", "mat_noise_scale"]
        );
        let v = |u: &UniformValue| match *u {
            UniformValue::_4f(a, b, c, d) => [a, b, c, d],
            ref other => panic!("{other:?}"),
        };
        [
            v(&values[0].value),
            v(&values[1].value),
            v(&values[2].value),
            v(&values[3].value),
        ]
    }

    fn layer(amount: f64, kind: NoiseType, site: NoiseSite, scale: f64) -> Option<ResolvedNoiseLayer> {
        Some(ResolvedNoiseLayer {
            amount,
            kind,
            site,
            scale,
        })
    }

    #[test]
    fn written_layers_pack_in_config_order() {
        let glass = ResolvedGlass {
            noise: ResolvedNoise {
                layers: [
                    layer(0.04, NoiseType::Lightness, NoiseSite::Glass, 1.),
                    layer(0.03, NoiseType::White, NoiseSite::Glass, 4.),
                    layer(0.02, NoiseType::Fine, NoiseSite::Film, 2.5),
                    None,
                ],
            },
            ..Default::default()
        };
        assert_eq!(
            vectors(&glass, true, &Blur::default()),
            [
                [0.04, 0.03, 0.02, 0.],
                [2., 0., 1., 0.],
                [0., 0., 2., 0.],
                [1., 4., 2.5, 1.],
            ]
        );
    }

    #[test]
    fn omitted_noise_inherits_into_slot_zero_only_while_backdrop_blur_is_effective() {
        let glass = ResolvedGlass::default();
        let blur = Blur {
            noise: 0.02,
            ..Default::default()
        };
        let on = vectors(&glass, true, &blur);
        assert_eq!(on[0], [0.02, 0., 0., 0.]);
        assert_eq!(on[1], [0.; 4]);
        assert_eq!(on[2], [0.; 4]);
        assert_eq!(on[3], [1.; 4]);
        assert_eq!(vectors(&glass, false, &blur)[0], [0.; 4]);
    }

    #[test]
    fn a_written_layer_stops_inheritance_at_every_site() {
        let blur = Blur {
            noise: 0.05,
            ..Default::default()
        };
        for site in [NoiseSite::Glass, NoiseSite::Backdrop, NoiseSite::Film] {
            let glass = ResolvedGlass {
                noise: ResolvedNoise::single(0.2, NoiseType::White, site),
                ..Default::default()
            };
            let v = vectors(&glass, true, &blur);
            assert_eq!(v[0], [0.2, 0., 0., 0.], "{site:?}");
            assert_eq!(v[2], [site as u8 as f32, 0., 0., 0.], "{site:?}");
        }
    }

    /// Correlation of two fine-lattice corners `d` apart, from the high-pass
    /// definition `h(p) - mean(h at p's eight neighbours)`, by enumerating
    /// each corner's hash coefficients (design §4).
    fn fine_corner_correlation(d: (i32, i32)) -> f64 {
        let coefficients = |c: (i32, i32)| {
            let mut m = HashMap::new();
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let w = if (dx, dy) == (0, 0) { 1. } else { -1. / 8. };
                    *m.entry((c.0 + dx, c.1 + dy)).or_insert(0.) += w;
                }
            }
            m
        };
        let cov = |a: (i32, i32), b: (i32, i32)| {
            let (a, b) = (coefficients(a), coefficients(b));
            a.iter()
                .map(|(k, v)| v * b.get(k).copied().unwrap_or(0.))
                .sum::<f64>()
        };
        cov((0, 0), d) / cov((0, 0), (0, 0))
    }

    #[test]
    fn the_fine_norm_coefficients_are_twice_the_corner_correlations() {
        let adjacent = fine_corner_correlation((1, 0));
        let diagonal = fine_corner_correlation((1, 1));
        assert!((adjacent + 1. / 6.).abs() < 1e-12, "{adjacent}");
        assert!((diagonal + 7. / 36.).abs() < 1e-12, "{diagonal}");
        // The comments name the coefficients too, so only code counts, and
        // the whole subtraction must be there, not the bare literals.
        let code = NoiseOptic::GLSL
            .lines()
            .map(|line| line.split("//").next().unwrap())
            .collect::<Vec<_>>()
            .join(" ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        let expected = format!(
            "norm -= {:.8} * (w00 * w10 + w00 * w01 + w10 * w11 + w01 * w11) \
             + {:.8} * (w00 * w11 + w10 * w01);",
            -2. * adjacent,
            -2. * diagonal
        );
        assert!(
            code.contains(&expected),
            "noise.frag's fine branch must subtract the corner covariance: `{expected}`"
        );
    }
}
```

In `src/render_helpers/grain.rs` tests, add:

```rust
    #[test]
    fn options_pack_backdrop_layers_in_order_with_neutral_empty_slots() {
        let grain = niri_config::BackdropGrain {
            layers: [
                Some(niri_config::BackdropLayer {
                    amount: 0.3,
                    kind: NoiseType::Fine,
                    scale: 1.,
                }),
                Some(niri_config::BackdropLayer {
                    amount: 0.1,
                    kind: NoiseType::White,
                    scale: 4.,
                }),
                None,
                None,
            ],
        };
        assert_eq!(
            GrainOptions::from(grain).uniforms(),
            [[0.3, 0.1, 0., 0.], [1., 0., 0., 0.], [1., 4., 1., 1.]]
        );
    }

    #[test]
    fn options_differ_when_any_slot_differs() {
        let mut a = GrainOptions::one(0.3, NoiseType::Fine);
        let b = a;
        assert_eq!(a, b);
        a.layers[1] = Some(GrainLayer {
            amount: 0.1,
            kind: NoiseType::White,
            scale: 4.,
        });
        assert_ne!(a, b);
    }
```

- [x] **Step 2: Run them to see them fail**

Run: `just test-one -p niri -E 'test(noise) | test(grain)'`
Expected: compile errors (`GrainLayer`, `GrainOptions::one`, `uniforms` unknown).

- [x] **Step 3: Write `noise.frag`**

Replace `src/render_helpers/shaders/material/noise.frag` with:

```glsl
// Optic: noise. Up to four layers (design 2026-10-06-noise-layers-design.md),
// one per vec4 component: amount, type (0 white, 1 fine, 2 lightness), site
// (0 glass, 1 backdrop, 2 film; design 2026-10-05-noise-placement-design.md)
// and scale (grain cell in physical pixels; 1 is per-pixel grain).
// noise_behind applies the glass layers (render-pipeline.md stage 3b),
// noise_source the backdrop layers in the effect-program grain pass,
// noise_post the film layers (stage 9); each in slot order. Neutral at
// amount 0. Uses hash12, fineGrain, srgbToLinear, linearToSrgb,
// linearToOklab and oklabToLinear from common.frag.
uniform vec4 mat_noise;
uniform vec4 mat_noise_type;
uniform vec4 mat_noise_site;
uniform vec4 mat_noise_scale;

// Per-slot seed offsets; slot 0 is the single-node original. Glass and film
// layers use their material slot, backdrop layers their place in the
// backdrop list (the effect program receives only those).
const vec2 NOISE_SEED_0 = vec2(47.0, 113.0);
const vec2 NOISE_SEED_1 = vec2(1301.0, 2659.0);
const vec2 NOISE_SEED_2 = vec2(3709.0, 977.0);
const vec2 NOISE_SEED_3 = vec2(2381.0, 3917.0);

// One layer's signed grain before its amount. White is hash12 - 0.5; fine
// and lightness use fineGrain's high-pass. At scale 1 and below, the
// original per-pixel hash. Above it, Hermite-interpolated lattice values
// divided by the interpolation's own deviation sqrt(w^T C w), so the
// grain's deviation is the same at every scale and every position in a
// cell. White corners are independent (C = I). Fine corners share hashes:
// adjacent ones correlate at -1/6 and diagonal ones at -7/36, which the
// norm subtracts as twice those (0.33333333, 0.38888889).
float noiseValue(vec2 fragCoord, float type, float scale, vec2 offset) {
    if (scale <= 1.0)
        return type < 0.5 ? hash12(fragCoord + offset) - 0.5 : fineGrain(fragCoord + offset);
    vec2 p = fragCoord / scale;
    vec2 base = floor(p) + offset;
    vec2 u = fract(p);
    u = u * u * (3.0 - 2.0 * u);
    float w00 = (1.0 - u.x) * (1.0 - u.y);
    float w10 = u.x * (1.0 - u.y);
    float w01 = (1.0 - u.x) * u.y;
    float w11 = u.x * u.y;
    float norm = w00 * w00 + w10 * w10 + w01 * w01 + w11 * w11;
    float g00;
    float g10;
    float g01;
    float g11;
    if (type < 0.5) {
        g00 = hash12(base) - 0.5;
        g10 = hash12(base + vec2(1.0, 0.0)) - 0.5;
        g01 = hash12(base + vec2(0.0, 1.0)) - 0.5;
        g11 = hash12(base + vec2(1.0, 1.0)) - 0.5;
    } else {
        // The 4x4 block of hashes around the cell: block (x, y) is the
        // lattice point base + (x - 1, y - 1), so the corners are block
        // cells 5, 6, 9 and 10, and each corner's 3x3 neighbourhood lies in
        // the block. A corner's fine value is (9 h - its 3x3 sum) / 8,
        // scaled as fineGrain scales.
        float h[16];
        for (int y = 0; y < 4; y++)
            for (int x = 0; x < 4; x++)
                h[y * 4 + x] = hash12(base + vec2(float(x) - 1.0, float(y) - 1.0));
        float k = 0.94280904 / 8.0;
        g00 = (9.0 * h[5] - (h[0] + h[1] + h[2] + h[4] + h[5] + h[6] + h[8] + h[9] + h[10])) * k;
        g10 = (9.0 * h[6] - (h[1] + h[2] + h[3] + h[5] + h[6] + h[7] + h[9] + h[10] + h[11])) * k;
        g01 = (9.0 * h[9] - (h[4] + h[5] + h[6] + h[8] + h[9] + h[10] + h[12] + h[13] + h[14])) * k;
        g11 = (9.0 * h[10] - (h[5] + h[6] + h[7] + h[9] + h[10] + h[11] + h[13] + h[14] + h[15])) * k;
        norm -= 0.33333333 * (w00 * w10 + w00 * w01 + w10 * w11 + w01 * w11)
              + 0.38888889 * (w00 * w11 + w10 * w01);
    }
    return (w00 * g00 + w10 * g10 + w01 * g01 + w11 * g11) / sqrt(norm);
}

// Lightness grain on an encoded colour: Oklab L of the clamped colour moves
// by `grain`; the result is clamped and re-encoded.
vec3 noiseLightness(vec3 encoded, float grain) {
    vec3 lab = linearToOklab(srgbToLinear(clamp(encoded, 0.0, 1.0)));
    lab.x += grain;
    return linearToSrgb(clamp(oklabToLinear(lab), 0.0, 1.0));
}

// A layer's grain on an encoded colour: white and fine add in encoding
// (signed, unclamped); lightness goes through noiseLightness.
vec3 noiseApply(vec3 encoded, float grain, float type) {
    if (type < 1.5)
        return encoded + vec3(grain);
    return noiseLightness(encoded, grain);
}

bool noiseAt(float site) {
    return (mat_noise.x > 0.0 && mat_noise_site.x == site)
        || (mat_noise.y > 0.0 && mat_noise_site.y == site)
        || (mat_noise.z > 0.0 && mat_noise_site.z == site)
        || (mat_noise.w > 0.0 && mat_noise_site.w == site);
}

// One glass layer on the running value: encoded unless isLinear, in which
// case it holds a lightness layer's clamped linear result, re-encoded here
// only because another glass layer follows.
void noiseBehindLayer(inout vec3 v, inout bool isLinear, vec2 fragCoord,
                      float amount, float type, float site, float scale, vec2 offset) {
    if (amount <= 0.0 || site != 0.0)
        return;
    if (isLinear) {
        v = linearToSrgb(v);
        isLinear = false;
    }
    float grain = noiseValue(fragCoord, type, scale, offset) * amount;
    if (type < 1.5) {
        v = v + vec3(grain);
        return;
    }
    // Retain lightness grain's gamut clamp, keeping its linear result.
    vec3 lab = linearToOklab(srgbToLinear(clamp(v, 0.0, 1.0)));
    lab.x += grain;
    v = clamp(oklabToLinear(lab), 0.0, 1.0);
    isLinear = true;
}

// Glass: the averaged linear backdrop, encoded once for the glass layers,
// returned in linear light. One active layer is the pre-layer arithmetic.
vec3 noise_behind(vec3 color, vec2 fragCoord) {
    if (!noiseAt(0.0))
        return color;
    vec3 v = linearToSrgb(color);
    bool isLinear = false;
    noiseBehindLayer(v, isLinear, fragCoord, mat_noise.x, mat_noise_type.x, mat_noise_site.x, mat_noise_scale.x, NOISE_SEED_0);
    noiseBehindLayer(v, isLinear, fragCoord, mat_noise.y, mat_noise_type.y, mat_noise_site.y, mat_noise_scale.y, NOISE_SEED_1);
    noiseBehindLayer(v, isLinear, fragCoord, mat_noise.z, mat_noise_type.z, mat_noise_site.z, mat_noise_scale.z, NOISE_SEED_2);
    noiseBehindLayer(v, isLinear, fragCoord, mat_noise.w, mat_noise_type.w, mat_noise_site.w, mat_noise_scale.w, NOISE_SEED_3);
    return isLinear ? v : srgbToLinear(v);
}

vec3 noisePostLayer(vec3 encoded, vec2 fragCoord, float amount, float type, float site,
                    float scale, vec2 offset) {
    if (amount <= 0.0 || site != 2.0)
        return encoded;
    return noiseApply(encoded, noiseValue(fragCoord, type, scale, offset) * amount, type);
}

// Film: the encoded glass after ring, aurora, glint and sweeps, before the
// coverage multiply.
vec3 noise_post(vec3 encoded, vec2 fragCoord) {
    encoded = noisePostLayer(encoded, fragCoord, mat_noise.x, mat_noise_type.x, mat_noise_site.x, mat_noise_scale.x, NOISE_SEED_0);
    encoded = noisePostLayer(encoded, fragCoord, mat_noise.y, mat_noise_type.y, mat_noise_site.y, mat_noise_scale.y, NOISE_SEED_1);
    encoded = noisePostLayer(encoded, fragCoord, mat_noise.z, mat_noise_type.z, mat_noise_site.z, mat_noise_scale.z, NOISE_SEED_2);
    encoded = noisePostLayer(encoded, fragCoord, mat_noise.w, mat_noise_type.w, mat_noise_site.w, mat_noise_scale.w, NOISE_SEED_3);
    return encoded;
}

vec3 noiseSourceLayer(vec3 straight, vec2 fragCoord, float amount, float type, float scale,
                      vec2 offset) {
    if (amount <= 0.0)
        return straight;
    return noiseApply(straight, noiseValue(fragCoord, type, scale, offset) * amount, type);
}

// Backdrop: one premultiplied texel of the effect buffer, in the effect
// program. The slots hold the agreed backdrop layers in backdrop-list order,
// not a material's slots, and mat_noise_site is not consulted. A transparent
// texel is left alone; the grain lands on the straight colour and is clamped
// once, because the result is stored in an 8-bit premultiplied texture.
vec4 noise_source(vec4 texel, vec2 fragCoord) {
    if (texel.a <= 0.0
        || (mat_noise.x <= 0.0 && mat_noise.y <= 0.0 && mat_noise.z <= 0.0 && mat_noise.w <= 0.0))
        return texel;
    vec3 straight = texel.rgb / texel.a;
    straight = noiseSourceLayer(straight, fragCoord, mat_noise.x, mat_noise_type.x, mat_noise_scale.x, NOISE_SEED_0);
    straight = noiseSourceLayer(straight, fragCoord, mat_noise.y, mat_noise_type.y, mat_noise_scale.y, NOISE_SEED_1);
    straight = noiseSourceLayer(straight, fragCoord, mat_noise.z, mat_noise_type.z, mat_noise_scale.z, NOISE_SEED_2);
    straight = noiseSourceLayer(straight, fragCoord, mat_noise.w, mat_noise_type.w, mat_noise_scale.w, NOISE_SEED_3);
    return vec4(clamp(straight, 0.0, 1.0) * texel.a, texel.a);
}
```

Each single-layer path keeps the old operations in the old order: the old `noise_behind` white branch computed `encoded + (hash12(seed) - 0.5) * mat_noise` and decoded; the new one computes `(hash12(fragCoord + offset) - 0.5) * amount`, adds it as `vec3(grain)` and decodes. The baseline-binary smoke (Task 4) is the byte-level check; if it reports a difference, compare the two compiled paths before changing a tolerance.

- [x] **Step 4: Upload four vectors from the optic**

`src/render_helpers/material/optics/noise.rs`:

```rust
//! `noise`: up to four layers, each at its selected behind/source/post
//! placement, neutral at amount 0. Only slot 0's amount inherits, and only
//! when no node is written; type, site and scale never do.

use niri_config::ResolvedGlass;
use smithay::backend::renderer::gles::{Uniform, UniformType};

use super::{Optic, OpticFrame};

pub struct NoiseOptic;

impl Optic for NoiseOptic {
    const NAME: &'static str = "noise";
    const GLSL: &'static str = include_str!("../../shaders/material/noise.frag");
    const UNIFORMS: &'static [(&'static str, UniformType)] = &[
        ("mat_noise", UniformType::_4f),
        ("mat_noise_type", UniformType::_4f),
        ("mat_noise_site", UniformType::_4f),
        ("mat_noise_scale", UniformType::_4f),
    ];

    fn values(glass: &ResolvedGlass, ctx: &OpticFrame<'_>) -> Vec<Uniform<'static>> {
        let mut amount = [0f32; 4];
        let mut kind = [0f32; 4];
        let mut site = [0f32; 4];
        let mut scale = [1f32; 4];
        if glass.noise.is_omitted() {
            amount[0] = if ctx.backdrop_blur { ctx.blur.noise } else { 0. } as f32;
        }
        for (k, layer) in glass.noise.layers.iter().enumerate() {
            let Some(layer) = layer else { continue };
            amount[k] = layer.amount as f32;
            kind[k] = layer.kind as u8 as f32;
            site[k] = layer.site as u8 as f32;
            scale[k] = layer.scale as f32;
        }
        vec![
            Uniform::new("mat_noise", amount),
            Uniform::new("mat_noise_type", kind),
            Uniform::new("mat_noise_site", site),
            Uniform::new("mat_noise_scale", scale),
        ]
    }
}
```

- [x] **Step 5: Carry the layers through the grain pass**

`src/render_helpers/grain.rs`, replacing `GrainOptions` and its `From`:

```rust
/// One backdrop layer, as the pass needs it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GrainLayer {
    pub amount: f32,
    pub kind: NoiseType,
    pub scale: f32,
}

/// The agreed backdrop layers, in backdrop-list order (slot `k` seeds with
/// `NOISE_SEED_k`). Equality over the whole array is the effect buffer's
/// change detection.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GrainOptions {
    pub layers: [Option<GrainLayer>; NOISE_LAYERS],
}

impl From<BackdropGrain> for GrainOptions {
    fn from(grain: BackdropGrain) -> Self {
        Self {
            layers: grain.layers.map(|layer| {
                layer.map(|layer| GrainLayer {
                    amount: layer.amount as f32,
                    kind: layer.kind,
                    scale: layer.scale as f32,
                })
            }),
        }
    }
}

impl GrainOptions {
    /// Amounts, kinds and scales, one vec4 component per slot; an empty
    /// slot is amount 0 and scale 1, which `noise_source` skips.
    pub fn uniforms(&self) -> [[f32; 4]; 3] {
        let mut out = [[0.; 4], [0.; 4], [1.; 4]];
        for (k, layer) in self.layers.iter().enumerate() {
            if let Some(layer) = layer {
                out[0][k] = layer.amount;
                out[1][k] = layer.kind as u8 as f32;
                out[2][k] = layer.scale;
            }
        }
        out
    }

    /// One layer at scale 1, for tests.
    #[cfg(test)]
    pub(crate) fn one(amount: f32, kind: NoiseType) -> Self {
        let mut layers = [None; NOISE_LAYERS];
        layers[0] = Some(GrainLayer {
            amount,
            kind,
            scale: 1.,
        });
        Self { layers }
    }
}
```

Imports: `use niri_config::{BackdropGrain, NoiseType, NOISE_LAYERS};`. `GrainProgramInner` gains `uniform_scale: ffi::types::GLint`, looked up with `gl.GetUniformLocation(program, c"mat_noise_scale".as_ptr())` beside the other two. In `render`, replace the two `Uniform1f` calls with:

```rust
                let [amount, kind, scale] = options.uniforms();
                gl.Uniform4f(p.uniform_amount, amount[0], amount[1], amount[2], amount[3]);
                gl.Uniform4f(p.uniform_kind, kind[0], kind[1], kind[2], kind[3]);
                gl.Uniform4f(p.uniform_scale, scale[0], scale[1], scale[2], scale[3]);
```

In `grain.rs` and `effect_buffer.rs` tests, replace every `GrainOptions { amount: A, kind: K }` with `GrainOptions::one(A, K)`.

- [x] **Step 6: Update the pins and the tile tests**

`src/render_helpers/shaders/mod.rs:578`: `assert!(source.contains("uniform vec4 mat_noise_site;"));`. In `uniform_nodes`, add `"mat_noise_scale" => vec!["noise scale="],` beside `mat_noise_site`.

`src/layout/tile.rs` tests: add beside `uniform_f32`

```rust
    /// Slot 0 of a four-slot optic uniform (the noise layers).
    fn uniform_slot0(
        uniforms: &[smithay::backend::renderer::gles::Uniform<'static>],
        name: &str,
    ) -> f32 {
        match uniforms.iter().find(|u| u.name == name).unwrap().value {
            smithay::backend::renderer::gles::UniformValue::_4f(x, _, _, _) => x,
            ref other => panic!("{name}: {other:?}"),
        }
    }
```

and replace `uniform_f32(&uniforms, "mat_noise")` and `uniform_f32(&uniforms, "mat_noise_type")` (three calls in the two noise tests) with `uniform_slot0(...)`. `mat_saturation` keeps `uniform_f32`.

- [x] **Step 7: Run the unit tests and the existing pixel identities**

Run: `just test-one -p niri -E 'test(noise) | test(grain) | test(pipeline) | test(uniform)'`
Expected: PASS, including `the_fine_norm_coefficients_are_twice_the_corner_correlations`, every pin in `shaders/mod.rs`, and `src/tests/noise_site.rs` (omitted equals glass, film within one code of glass, backdrop within two codes, amount 0 neutral, damage contract): the single-layer paths are unchanged.

- [x] **Step 7a: Demonstrate that the coefficient pin reads the code**

With the mutation harness (above), on the uncommitted rewrite:

```bash
cp src/render_helpers/shaders/material/noise.frag "$M/before.frag"
T=the_fine_norm_coefficients_are_twice_the_corner_correlations
E="fine branch must subtract the corner covariance"
"$M/mutation.sh" adjacent "$T" "$E"        # 0.33333333 in code becomes 0.33333300; the comment keeps it
"$M/mutation.sh" diagonal "$T" "$E"        # 0.38888889 in code becomes 0.38888800
"$M/mutation.sh" no-correction "$T" "$E"   # the norm -= statement is deleted
cmp src/render_helpers/shaders/material/noise.frag "$M/before.frag"
just test-one -p niri "$T"
```

Expected: `caught:` and exit 0 for all three (any other exit code stops the step: read the message and, for 4, the kept log), `cmp` silent, and the final run PASSES. Record `tasks note material-cb80f7 "mutation: coefficient pin fails for adjacent, diagonal and deleted correction"`.

- [x] **Step 8: Run the fast suite and commit**

```bash
just test-fast
tasks done material-cb80f7 "noise.frag applies four slots with scale=, covariance-correct fine lattice; both programs upload four-slot uniforms"
just check
git add src tasks/
just upstream-report --stage
git add docs/materials/upstream-divergence.md
git commit -m "feat(material): four noise slots with a grain size in the material and grain programs (material-3fcba2)"
```

---

### Task 3: In-process pixel tests

**Files:**
- Create: `src/tests/noise_layers.rs`
- Modify: `src/tests/mod.rs` (add `mod noise_layers;` after `mod noise_site;`)
- Modify: `src/tests/noise_site.rs` (one damage test)

**Interfaces:**
- Consumes: `super::ring_pair::{diff, render_at, set_time}`, `super::fixture::Fixture`, `super::client::LayerConfigureProps` (as `noise_site.rs` does); the configs of Tasks 1 and 2.
- Produces: tests only.

These tests run against the finished shader, so they pass on first run; Step 3 demonstrates once that the per-position check catches the independent-corner norm the spec review found.

- [x] **Step 1: Write the test file**

`src/tests/noise_layers.rs`:

```rust
//! Noise layers (design 2026-10-06-noise-layers-design.md §7.2, §8): slot
//! identities, per-slot application and seeding, independence, and grain
//! size with its normalisation by position in the lattice cell, rendered in
//! process through the headless GLES renderer under a frozen clock.

use std::time::Duration;

use niri_config::Config;
use smithay::reexports::wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_shell_v1::Layer;
use smithay::reexports::wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1::Anchor;
use smithay::utils::{Logical, Point, Rectangle, Size};

use super::client::LayerConfigureProps;
use super::fixture::Fixture;
use super::ring_pair::{diff, render_at, set_time};

// One large window, so each of the 64 position classes at scale 8 holds
// about 16 000 face pixels and a 10 % tolerance spans many standard errors.
const OUT_W: u16 = 1280;
const OUT_H: u16 = 1024;
const W: u16 = 1200;
const H: u16 = 960;
const BEVEL: u16 = 12;
const OFFSET: u16 = 6;
const NONE: &str = "";

fn config(noise: &str) -> Config {
    Config::parse_mem(&format!(
        r##"
        hotkey-overlay {{ skip-at-startup; }}
        layout {{
            gaps 16
            focus-ring {{ off; }}
            border {{ off; }}
            shadow {{ off; }}
            background-color "transparent"
        }}
        animations {{ off; }}
        blur {{ passes 3; offset 3; noise 0; saturation 1; }}
        material "a" {{
            glass {{
                ior 1
                thickness 20
                bevel {BEVEL}
                offset-x {OFFSET}
                offset-y {OFFSET}
                attenuation-color "#ffffff"
                attenuation-distance 60
                chromatic-aberration 0
                anisotropic-blur 0
                distortion 0 scale=0.5
                roughness 0
                backdrop-blur false
                jelly-flex 0
                jelly-ripple 0
                saturation 1
                aurora 0 {{ drift-hz 0; }}
                iridescence 0
                {noise}
            }}
            response "default" {{ focus "none"; accent "none"; ring-beam-speed 0; }}
        }}
        window-rule {{
            material "a"
            background-effect {{ blur false; noise 0; saturation 1; }}
        }}
        "##
    ))
    .unwrap()
}

/// One transparent window over a warm mid-tone background layer that fills
/// the output.
fn fixture() -> Fixture {
    let mut f = Fixture::with_config(config(NONE));
    f.niri_state().backend.headless().add_renderer().unwrap();
    f.add_output(1, (OUT_W, OUT_H));

    let bg = f.add_client();
    let layer = f.client(bg).create_layer(None, Layer::Background, "");
    let surface = layer.surface.clone();
    layer.set_configure_props(LayerConfigureProps {
        anchor: Some(Anchor::Top | Anchor::Left | Anchor::Right),
        size: Some((0, u32::from(OUT_H))),
        exclusive_zone: Some(-1),
        ..Default::default()
    });
    layer.commit();
    f.roundtrip(bg);
    let layer = f.client(bg).layer(&surface);
    layer.attach_new_colored_buffer(140, 115, 90, 255);
    layer.set_size(OUT_W, OUT_H);
    layer.ack_last_and_commit();
    f.double_roundtrip(bg);

    let id = f.add_client();
    let window = f.client(id).create_window();
    let surface = window.surface.clone();
    window.commit();
    f.roundtrip(id);
    let window = f.client(id).window(&surface);
    window.attach_new_shm_buffer(0);
    window.set_size(W, H);
    window.ack_last_and_commit();
    f.double_roundtrip(id);
    f.niri_state().update_keyboard_focus();
    f.double_roundtrip(id);

    set_time(&mut f, Duration::ZERO);
    f.niri_complete_animations();
    f
}

fn reload(f: &mut Fixture, noise: &str) {
    f.niri_state().reload_config(Ok(config(noise)));
    f.niri_state().refresh_and_flush_clients();
}

/// RGBA pixels of the output and the window's flat face in output px.
fn render(f: &mut Fixture) -> (Vec<u8>, Rectangle<i32, Logical>) {
    let pixels = render_at(f, Duration::ZERO);
    let niri = f.niri();
    let (_, _, workspace) = niri.layout.workspaces().next().unwrap();
    let (tile, pos, _) = workspace.tiles_with_render_positions().next().unwrap();
    let rect = Rectangle::new(pos + tile.window_loc(), tile.animated_window_size());
    let inset = f64::from(BEVEL + OFFSET) + 2.;
    let face = Rectangle::new(
        (rect.loc + Point::from((inset, inset))).to_i32_round(),
        (rect.size - Size::from((2. * inset, 2. * inset))).to_i32_round(),
    );
    let output = Rectangle::from_size(Size::from((i32::from(OUT_W), i32::from(OUT_H))));
    assert_eq!(
        face.intersection(output),
        Some(face),
        "the face {face:?} must lie inside the output"
    );
    assert!(
        face.size.w > 1000 && face.size.h > 800,
        "the window did not take its {W}x{H} size: face {face:?}"
    );
    (pixels, face)
}

/// The signed green-channel grain `on - off` over the face, row-major, with
/// each pixel's output coordinates.
fn grain(on: &[u8], off: &[u8], face: Rectangle<i32, Logical>) -> Vec<(i32, i32, f64)> {
    let w = usize::from(OUT_W);
    let mut out = Vec::with_capacity((face.size.w * face.size.h) as usize);
    for y in face.loc.y..face.loc.y + face.size.h {
        for x in face.loc.x..face.loc.x + face.size.w {
            let i = (y as usize * w + x as usize) * 4 + 1;
            out.push((x, y, f64::from(on[i]) - f64::from(off[i])));
        }
    }
    out
}

/// The grain of `noise` against no noise, over the face.
fn grain_of(f: &mut Fixture, noise: &str) -> (Vec<(i32, i32, f64)>, Rectangle<i32, Logical>) {
    reload(f, NONE);
    let (off, face) = render(f);
    reload(f, noise);
    let (on, _) = render(f);
    (grain(&on, &off, face), face)
}

fn sd(values: &[f64]) -> f64 {
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    (values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64).sqrt()
}

fn grain_sd(g: &[(i32, i32, f64)]) -> f64 {
    sd(&g.iter().map(|p| p.2).collect::<Vec<_>>())
}

/// The grain's deviation per position class `(x mod s, y mod s)`: each class
/// holds the pixels at one position inside the lattice cell, whatever the
/// cell's phase or the framebuffer's orientation.
fn position_sds(g: &[(i32, i32, f64)], s: i32) -> Vec<f64> {
    let mut classes = vec![Vec::new(); (s * s) as usize];
    for &(x, y, v) in g {
        classes[(y.rem_euclid(s) * s + x.rem_euclid(s)) as usize].push(v);
    }
    classes.iter().map(|c| sd(c)).collect()
}

/// Deviation of 4x4 block means over the pixels' deviation: about 1/4 for
/// independent pixels, rising toward 1 as the grain coarsens.
fn low_frequency_ratio(g: &[(i32, i32, f64)], face: Rectangle<i32, Logical>) -> f64 {
    let (w, h) = (face.size.w as usize, face.size.h as usize);
    let mut means = Vec::with_capacity((w / 4) * (h / 4));
    for by in 0..h / 4 {
        for bx in 0..w / 4 {
            let mut sum = 0.;
            for dy in 0..4 {
                for dx in 0..4 {
                    sum += g[(by * 4 + dy) * w + bx * 4 + dx].2;
                }
            }
            means.push(sum / 16.);
        }
    }
    sd(&means) / grain_sd(g)
}

#[test]
fn one_layer_equals_the_same_layer_with_empty_slots_after_it() {
    let mut f = fixture();
    for site in ["glass", "backdrop", "film"] {
        for kind in ["white", "fine", "lightness"] {
            let one = format!("noise 0.3 type=\"{kind}\" site=\"{site}\"");
            reload(&mut f, &one);
            let (a, _) = render(&mut f);
            reload(&mut f, &format!("{one}\nnoise 0\nnoise 0\nnoise 0"));
            let (b, _) = render(&mut f);
            assert_eq!(diff(&a, &b), (0, 0), "{kind} at {site}");
        }
    }
}

#[test]
fn written_scale_one_equals_omitted() {
    let mut f = fixture();
    for site in ["glass", "backdrop", "film"] {
        for kind in ["white", "fine", "lightness"] {
            let omitted = format!("noise 0.3 type=\"{kind}\" site=\"{site}\"");
            reload(&mut f, &omitted);
            let (a, _) = render(&mut f);
            reload(&mut f, &format!("{omitted} scale=1"));
            let (b, _) = render(&mut f);
            assert_eq!(diff(&a, &b), (0, 0), "{kind} at {site}");
        }
    }
}

#[test]
fn amount_zero_layers_are_neutral_with_any_properties() {
    let mut f = fixture();
    reload(&mut f, NONE);
    let (none, _) = render(&mut f);
    reload(
        &mut f,
        "noise 0 type=\"lightness\" scale=8 site=\"film\"\n\
         noise 0 type=\"fine\" scale=3 site=\"backdrop\"\n\
         noise 0 type=\"white\" scale=16",
    );
    let (zero, _) = render(&mut f);
    assert_eq!(diff(&none, &zero), (0, 0));
}

#[test]
fn every_slot_is_applied_and_seeded_apart() {
    let mut f = fixture();
    let (slot0, _) = grain_of(&mut f, "noise 0.3 type=\"fine\"");
    let s0 = grain_sd(&slot0);
    assert!(s0 > 5., "slot 0 grain is present: sd {s0:.2}");
    for k in 1..4 {
        let noise = std::iter::repeat("noise 0")
            .take(k)
            .chain(["noise 0.3 type=\"fine\""])
            .collect::<Vec<_>>()
            .join("\n");
        let (g, _) = grain_of(&mut f, &noise);
        let sk = grain_sd(&g);
        assert!(
            (sk / s0 - 1.).abs() < 0.05,
            "slot {k}: sd {sk:.2} vs slot 0 {s0:.2}"
        );
        let difference: Vec<f64> = g.iter().zip(&slot0).map(|(a, b)| a.2 - b.2).collect();
        assert!(
            sd(&difference) >= s0,
            "slot {k} repeats slot 0's pattern: difference sd {:.2}",
            sd(&difference)
        );
    }
}

#[test]
fn two_independent_layers_add_in_quadrature() {
    let mut f = fixture();
    let (one, _) = grain_of(&mut f, "noise 0.2 type=\"fine\"");
    let (two, _) = grain_of(&mut f, "noise 0.2 type=\"fine\"\nnoise 0.2 type=\"fine\"");
    let ratio = grain_sd(&two) / grain_sd(&one);
    assert!(
        (ratio / std::f64::consts::SQRT_2 - 1.).abs() < 0.05,
        "two layers over one: {ratio:.3}, expected about 1.414"
    );
}

#[test]
fn grain_deviation_holds_across_scales_and_cell_positions() {
    let mut f = fixture();
    for kind in ["white", "fine"] {
        let (g1, face) = grain_of(&mut f, &format!("noise 0.3 type=\"{kind}\""));
        let base = grain_sd(&g1);
        let mut previous = low_frequency_ratio(&g1, face);
        for scale in [2, 4, 8] {
            let (g, _) = grain_of(&mut f, &format!("noise 0.3 type=\"{kind}\" scale={scale}"));
            let aggregate = grain_sd(&g);
            assert!(
                (aggregate / base - 1.).abs() < 0.10,
                "{kind} scale {scale}: sd {aggregate:.2} vs scale 1 {base:.2}"
            );
            for (i, class) in position_sds(&g, scale).into_iter().enumerate() {
                let (x, y) = (i as i32 % scale, i as i32 / scale);
                assert!(
                    (class / aggregate - 1.).abs() < 0.10,
                    "{kind} scale {scale}: position ({x}, {y}) has sd {class:.2} vs {aggregate:.2}"
                );
            }
            let ratio = low_frequency_ratio(&g, face);
            assert!(
                ratio > previous,
                "{kind} scale {scale}: low-frequency ratio {ratio:.3} did not rise from {previous:.3}"
            );
            previous = ratio;
        }
    }
}

/// Mean and maximum absolute RGB difference over the face, in codes.
fn face_diff(a: &[u8], b: &[u8], face: Rectangle<i32, Logical>) -> (f64, u8) {
    let w = usize::from(OUT_W);
    let (mut sum, mut max, mut n) = (0u64, 0u8, 0u64);
    for y in face.loc.y..face.loc.y + face.size.h {
        for x in face.loc.x..face.loc.x + face.size.w {
            let i = (y as usize * w + x as usize) * 4;
            for c in 0..3 {
                let d = a[i + c].abs_diff(b[i + c]);
                sum += u64::from(d);
                max = max.max(d);
                n += 1;
            }
        }
    }
    (sum as f64 / n as f64, max)
}

/// Lightness in slot 0, then white in slot 1, with each slot's seed fixed.
/// In the reference the hooks fix the order: the glass hook runs the
/// lightness layer before the film hook adds white. Applied as a stack at
/// one site, the same two layers must give the same pixels, so a shader that
/// applies a site's slots in another order fails here. Swapping the config
/// lines cannot test this, because it swaps the seeds too. At amount 0.5 each
/// on this backdrop, reversing the two moves a pixel by about 1.4 codes on
/// average (a float simulation of the formulas while planning); the glass and
/// film stacks differ from the reference only by float rounding, and the
/// backdrop stack by its 8-bit storage (under half a code on average).
#[test]
fn a_sites_layers_apply_in_slot_order() {
    let mut f = fixture();
    reload(
        &mut f,
        r#"noise 0.5 type="lightness" site="glass"
           noise 0.5 type="white" site="film""#,
    );
    let (reference, face) = render(&mut f);
    for (site, mean_limit, max_limit) in [("glass", 0.1, 1), ("film", 0.1, 1), ("backdrop", 0.7, 2)] {
        reload(
            &mut f,
            &format!(
                r#"noise 0.5 type="lightness" site="{site}"
                   noise 0.5 type="white" site="{site}""#
            ),
        );
        let (stack, _) = render(&mut f);
        let (mean, max) = face_diff(&reference, &stack, face);
        assert!(
            mean <= mean_limit && max <= max_limit,
            "{site} stack against the hook-ordered reference: mean {mean:.3}, max {max} codes"
        );
    }
}

#[test]
fn a_fractional_scale_keeps_the_deviation() {
    let mut f = fixture();
    let (g1, _) = grain_of(&mut f, "noise 0.3 type=\"fine\"");
    let (g, _) = grain_of(&mut f, "noise 0.3 type=\"fine\" scale=2.5");
    let (base, sd25) = (grain_sd(&g1), grain_sd(&g));
    assert!(
        (sd25 / base - 1.).abs() < 0.10,
        "scale 2.5: sd {sd25:.2} vs scale 1 {base:.2}"
    );
}
```

Add `mod noise_layers;` after `mod noise_site;` in `src/tests/mod.rs`.

In `src/tests/noise_site.rs`, after `a_backdrop_only_reload_rerenders_the_unchanged_glass_window`, add a test that changes one backdrop layer's scale (it copies that test's `read` closure, which is local to it):

```rust
#[test]
fn a_backdrop_layer_scale_change_publishes_damage() {
    let look = Look {
        noise_a: "noise 0.2 type=\"fine\" site=\"backdrop\"\n\
                  noise 0.1 type=\"white\" site=\"backdrop\" scale=4",
        noise_b: "noise 0.1 type=\"fine\" site=\"glass\"",
        ..Default::default()
    };
    let mut f = fixture(&look);
    let _ = render(&mut f);
    let read = |f: &mut Fixture| {
        let output = f.niri_output(1);
        let niri = f.niri();
        let buffer = niri.output_state[&output].xray.background
            [RenderTarget::ScreenCapture as usize]
            .borrow()
            .commit();
        let (_, _, workspace) = niri.layout.workspaces().next().unwrap();
        let glass_window = workspace
            .tiles_with_render_positions()
            .find_map(|(tile, _, _)| tile.material().filter(|m| m.material().name == "b"))
            .expect("material b")
            .commit();
        (buffer, glass_window)
    };
    let (buffer_0, window_0) = read(&mut f);
    reload(
        &mut f,
        &Look {
            noise_a: "noise 0.2 type=\"fine\" site=\"backdrop\"\n\
                      noise 0.1 type=\"white\" site=\"backdrop\" scale=8",
            ..look
        },
    );
    let _ = render(&mut f);
    let (buffer_1, window_1) = read(&mut f);
    assert_ne!(buffer_1, buffer_0, "the effect buffer publishes the scale change");
    assert_ne!(
        window_1, window_0,
        "the glass window's fingerprint carries the buffer's commit"
    );
}
```

- [x] **Step 2: Run the new tests**

Run: `just test-one -p niri -E 'test(noise_layers) | test(a_backdrop_layer_scale_change_publishes_damage)'`
Expected: PASS. If `render`'s size assertion fails, the test client did not get its size: read the configure the fixture's layout sent (`default-column-width`) and set `layout { default-column-width { fixed 1200; } }` in `config`, then rerun; do not shrink the window, because the position classes need the samples.

- [x] **Step 3: Demonstrate that the position check catches the independent-corner norm**

With the mutation harness, the fine branch loses its covariance correction, so fine uses `Σ w²` as round 1's spec did:

```bash
cp src/render_helpers/shaders/material/noise.frag "$M/before.frag"
"$M/mutation.sh" no-correction grain_deviation_holds_across_scales_and_cell_positions \
    'fine scale [0-9]+: position \([0-9]+, [0-9]+\) has sd'
cmp src/render_helpers/shaders/material/noise.frag "$M/before.frag"
just test-one -p niri grain_deviation_holds_across_scales_and_cell_positions
```

Expected: `caught: no-correction` and exit 0, the matched text naming a `fine scale <s>: position (x, y)` class (at scale 8 the weakest class is near 0.8 of the aggregate, per spec review round 2's 79.5 %; a smaller scale may fail first). The expected pattern excludes the aggregate assertion's message, so a failure there exits 4 instead of counting; `cmp` silent; the final run PASSES. Record the failing line: `tasks note material-d184da "mutation: independent-corner norm fails <line>"`.

- [x] **Step 3a: Demonstrate that the order check catches reversed application**

With the mutation harness, each hook's four slot calls run in reverse order (each call keeps its own components and seed):

```bash
cp src/render_helpers/shaders/material/noise.frag "$M/before.frag"
"$M/mutation.sh" reverse-glass a_sites_layers_apply_in_slot_order 'glass stack against the hook-ordered reference'
"$M/mutation.sh" reverse-film a_sites_layers_apply_in_slot_order 'film stack against the hook-ordered reference'
"$M/mutation.sh" reverse-backdrop a_sites_layers_apply_in_slot_order 'backdrop stack against the hook-ordered reference'
cmp src/render_helpers/shaders/material/noise.frag "$M/before.frag"
```

Expected: `caught:` and exit 0 three times, the glass and film means near 1.4 codes in the kept output; `cmp` silent. Exit 4 stops the step (read the kept log). If `reverse-backdrop` reports `MUTATION SURVIVED` (exit 1), its 8-bit storage hides the reversal at these amounts: raise all four amounts in the test (reference and stacks) to 0.6 (about 1.8 codes reversed, per the planning simulation), rerun the unmutated test to PASS and all three mutations, and say so in the note. Record `tasks note material-d184da "mutation: reversed glass, film and backdrop application each fail a_sites_layers_apply_in_slot_order: <means>"`, and rerun the test to PASS.

- [x] **Step 4: Fast suite and commit**

```bash
just test-fast
tasks done material-d184da "In-process slot identities, per-slot seeding, quadrature, scale normalisation by cell position, backdrop scale damage"
just check
git add src/tests tasks/
just upstream-report --stage
git add docs/materials/upstream-divergence.md
git commit -m "test(material): noise layer identities, seeding and per-position grain deviation (material-3fcba2)"
```

---

### Task 4: Docs, the nested-Weston smoke and contact sheet, the evidence document

**Files:**
- Modify: `docs/materials/material-config.md` (the noise paragraphs; the table is already generated)
- Modify: `docs/materials/render-pipeline.md` (§1 step 2, §3 rows 3b and 9, §4 noise bullet, §5 table)
- Modify: `docs/materials/scripts/glass-optic-smoke-lib.sh` (gains `metric`, `signed_diff`, `shot_twice`), `docs/materials/scripts/glass-noise-site-smoke.sh` (drops its copies)
- Create: `docs/materials/scripts/glass-noise-layers-smoke.sh`, `docs/materials/scripts/noise-layers-bins.py`
- Create: `docs/materials/<capture-date>-noise-layers-evidence.md`

**Interfaces:**
- Consumes: the lib's `capture_preflight`, `build_binaries`, `capture_identity`, `write_config`, `start_nested`, `spawn_probe`, `calibrate_probe_rect`, `probe_rect`, `shot`, `roi`, `sd`, `ae`, `assert_zero`, `assert_about`, `assert_greater`, `finish`; `$NIRI`, `$WALL`, `$IDLE`, `$OUT`, `PX PY PW PH`.
- Produces: `metric NAME VALUE`, `signed_diff A B OUT`, `shot_twice BINARY NAME` in the lib (moved verbatim); the smoke's `metrics.txt` keys named in Step 4.

- [x] **Step 1: Document layers**

`docs/materials/material-config.md`, replace the paragraph that starts `` `noise` also accepts `site="glass"` `` with:

```markdown
`noise` also accepts `site="glass"` (default), `"backdrop"`, or `"film"`.
`backdrop` grains the output's shared effect-buffer texture before the Kawase
blur and roughness pyramids. Every material placing noise there must agree on
the ordered list of its backdrop layers (amount, type, scale); disagreement
refuses the config and names both materials and both lists (`materials "a" and
"b" place different noise at the backdrop ([0.3 fine], [0.1 fine]) ...`).
Grain applies even with `blur { off }`; a window asking for a blurred
`background-effect` also samples it. `film` grains the finished encoded glass
after ring, aurora, glint and sweeps, on glass coverage only. The site's
softening under blur is measured in `2026-10-05-noise-placement-evidence.md`.

A material may write `noise` up to four times; each node is a layer with its
own amount, `type=`, `site=` and `scale=`, and a fifth node refuses the
config. Each site applies its layers in the order written; sites run backdrop,
glass, film. `scale=` is the grain's cell size in physical output pixels, 1
(the default, per-pixel grain) to 16; above 1 the grain is smooth value noise
normalised so an amount means the same strength at every size. Each layer has
its own grain pattern: glass and film layers by their position among the
material's nodes, backdrop layers by their position among the backdrop layers.
Independent white or fine layers of one type and size at one site, with
nothing clipping, have the variance of one layer of amount `sqrt(a² + b²)` but
not its distribution, so they are close to redundant: layers are for
different types, sites or sizes.
```

and in `### noise`, replace the paragraph after the stage line with:

```markdown
`noise <amount> type=<type> site=<site> scale=<px>`, up to four nodes, grains
the selected carrier; `white`, `fine`, and `lightness` are described above.
Its explicit neutral is amount 0, which keeps the node's slot. When no node is
written, the amount inherits the global `blur` block's `noise` while backdrop
blur is effective and resolves to 0 otherwise, as white grain at the glass
site; a written node never inherits, and type, site and scale never do.
```

`docs/materials/render-pipeline.md`:
- §1 step 2: replace `when a material places noise at `backdrop`, a cached grain pass over that texture` with `when materials place noise layers at `backdrop`, a cached grain pass applying them in order over that texture`.
- §3 row 3b: Stage text becomes `**Behind: noise (glass site).** The layers placed at `glass` (up to four, in the order written; other sites' layers are inert here): encode the averaged backdrop once, add each white or fine layer's grain in encoding or move a lightness layer's Oklab lightness (re-encoding only when another glass layer follows), then return linear light. Grain stays screen-seeded per slot; above `scale=` 1 it is a normalised value-noise lattice of that cell size. Lightness keeps its gamut clamp.`; Parameters `` `noise`, `noise type=`, `noise site=`, `noise scale=`; neutral 0 ``.
- §3 row 9: Stage text becomes `**Post: film grain.** `noise_post` applies the `film` layers in the order written to the encoded glass, over transmitted light, ring, aurora, glint and sweeps, before coverage. The formulas and seeds of 3b act in encoding.`; Parameters as row 3b.
- §4, the `Noise has three material placements.` bullet: append `A material writes up to four `noise` nodes, each a layer with its own site and `scale=`; each site applies its layers in the order written.`
- §5 table: after the `noise` `type=` row add `` | `noise` `scale=` | (pending, prism-85f63a) | source grain pass / 3b / 9 | ``.

- [x] **Step 2: Move the three capture helpers into the lib**

Cut `metric()`, `signed_diff()` and `shot_twice()` (with its comment) from `glass-noise-site-smoke.sh` and paste them verbatim into `glass-optic-smoke-lib.sh` after `mean()`. Run `bash -n` on both files. `glass-noise-site-smoke.sh` still sources the lib before using them, so its behaviour is unchanged.

- [x] **Step 3: Write the position-class helper**

`docs/materials/scripts/noise-layers-bins.py`:

```python
#!/usr/bin/env python3
"""Per-position deviation of a signed grain image (noise layers design §7.2, 5).

Usage: noise-layers-bins.py <raw 16-bit MSB gray> <width> <height> <scale>

Prints aggregate_sd (0..1 units), min_bin_ratio and max_bin_ratio (each
position class's deviation over the aggregate) and lf_ratio (the deviation of
4x4 block means over the pixels'). Pixels are classed by (x mod scale,
y mod scale): one class per position inside the lattice cell, whatever the
crop's phase or the capture's orientation. Exit 2 on malformed input.
"""
import math
import struct
import sys


def sd(values):
    mean = sum(values) / len(values)
    return math.sqrt(sum((v - mean) ** 2 for v in values) / len(values))


def main():
    if len(sys.argv) != 5:
        print(__doc__, file=sys.stderr)
        return 2
    path, w, h, s = sys.argv[1], int(sys.argv[2]), int(sys.argv[3]), int(sys.argv[4])
    with open(path, 'rb') as f:
        data = f.read()
    if len(data) != 2 * w * h or s < 1 or w < 4 or h < 4:
        print(f'{path}: {len(data)} bytes for {w}x{h}, scale {s}', file=sys.stderr)
        return 2
    values = struct.unpack(f'>{w * h}H', data)
    aggregate = sd(values)
    if aggregate == 0:
        print(f'{path}: no grain', file=sys.stderr)
        return 2
    classes = [[] for _ in range(s * s)]
    for y in range(h):
        for x in range(w):
            classes[(y % s) * s + x % s].append(values[y * w + x])
    ratios = [sd(c) / aggregate for c in classes]
    means = [
        sum(values[(by * 4 + dy) * w + bx * 4 + dx] for dy in range(4) for dx in range(4)) / 16
        for by in range(h // 4)
        for bx in range(w // 4)
    ]
    print(f'aggregate_sd={aggregate / 65535:.6f}')
    print(f'min_bin_ratio={min(ratios):.4f}')
    print(f'max_bin_ratio={max(ratios):.4f}')
    print(f'lf_ratio={sd(means) / aggregate:.4f}')
    return 0


if __name__ == '__main__':
    sys.exit(main())
```

Check it on a synthetic image before using it:

```bash
S=$(mktemp -d)
magick -size 256x256 xc:gray50 +noise Uniform -colorspace Gray -depth 16 -endian MSB "gray:$S/u.raw"
python3 -I docs/materials/scripts/noise-layers-bins.py "$S/u.raw" 256 256 8
```

Expected: `min_bin_ratio` and `max_bin_ratio` within 0.9..1.1 and `lf_ratio` near 0.25 (independent pixels).

- [x] **Step 4: Write the smoke**

`docs/materials/scripts/glass-noise-layers-smoke.sh` (mode 0755):

```bash
#!/usr/bin/env bash
# Noise layers smoke (design 2026-10-06-noise-layers-design.md §7.1, §7.2).
# Default: the full matrix and the contact sheet. NOISE_LAYERS_PILOT=1 keeps
# fine grain and scale 8, still exercising all six assertions and the sheet.
# OUT must be fresh; BASE_NIRI pins the 4a8b2072 release binary;
# CAPTURE_TASK and NIRI_MATERIAL_WORK_ROOT are required.
set -eu
BASE_NIRI=${BASE_NIRI:?baseline release binary}
PILOT=${NOISE_LAYERS_PILOT:-0}
case "$PILOT" in 0|1) ;; *) echo "NOISE_LAYERS_PILOT must be 0 or 1" >&2; exit 2 ;; esac
source "$(dirname "$0")/glass-optic-smoke-lib.sh"
trap 'exit 130' INT
trap 'exit 143' TERM
capture_preflight headless
build_binaries
cp "$BASE_NIRI" "$OUT/niri-baseline"
BASE_NIRI=$OUT/niri-baseline
capture_identity --binary "$BASE_NIRI" --config "pilot=$PILOT" --config baseline=4a8b2072
magick -size 1280x720 xc:'rgb(140,115,90)' \
    \( -size 400x300 -seed 11 plasma:fractal \) -gravity southeast -composite "$WALL"
KINDS=(white fine lightness); SITES=(glass backdrop film); SCALE_KINDS=(white fine); SCALES=(2 4 8)
if [ "$PILOT" = 1 ]; then KINDS=(fine); SITES=(glass backdrop film); SCALE_KINDS=(fine); SCALES=(8); fi
BINS=$(dirname "$0")/noise-layers-bins.py
IDENTITY=$'ior 1\nattenuation-color "#ffffff"\nsaturation 1'
TOP_DEFAULT='blur { passes 3; offset 3; noise 0; saturation 1; }'
# The window's face less 60 px a side and 120 px top and bottom: the lattice
# classes need far more samples than the lib's 200x400 face.
wide_roi() { echo "$((PW - 120))x$((PH - 240))+$((PX + 60))+$((PY + 120))"; }
cell() {   # $1 name, $2 binary, $3 glass extra, $4 top extra (optional)
    GLASS_EXTRA="$IDENTITY"$'\n'"$3" TOP_EXTRA=${4:-$TOP_DEFAULT} write_config "$OUT/$1.kdl"
    start_nested "$2" "$OUT/$1.kdl"
    spawn_probe "$2" "$IDLE"
    probe_rect "$2"
    sleep 2
    shot_twice "$2" "$1"
    roi "$1" "$(wide_roi)" face
    stop_nested
}
grain_sd() {   # $1 cell, $2 reference cell; sets METRIC
    signed_diff "$OUT/$1-face.png" "$OUT/$2-face.png" "$OUT/grain-$1.png"
    sd "$OUT/grain-$1.png"
}
# Sets BIN_aggregate_sd, BIN_min_bin_ratio, BIN_max_bin_ratio, BIN_lf_ratio,
# clearing them first so a failed run can never leave the previous cell's.
# identify prints no trailing newline, so its output is captured and checked,
# never piped into `read` (which returns 1 at EOF and trips set -e).
bins() {   # $1 cell (its grain image must exist), $2 scale
    local png=$OUT/grain-$1.png raw=$OUT/grain-$1.raw dims w h out key value name
    unset BIN_aggregate_sd BIN_min_bin_ratio BIN_max_bin_ratio BIN_lf_ratio
    magick "$png" -depth 16 -endian MSB "gray:$raw" || fail "raw export of $1 failed"
    dims=$(magick identify -format '%w %h' "$png") || fail "identify of $1 failed"
    [[ $dims =~ ^([0-9]+)\ ([0-9]+)$ ]] || fail "identify of $1 returned '$dims'"
    w=${BASH_REMATCH[1]}; h=${BASH_REMATCH[2]}
    out=$(python3 -I "$BINS" "$raw" "$w" "$h" "$2") || fail "position classes of $1 failed"
    while IFS='=' read -r key value; do
        case $key in
            aggregate_sd|min_bin_ratio|max_bin_ratio|lf_ratio) ;;
            *) fail "position classes of $1: unexpected line '$key=$value'" ;;
        esac
        is_number "$value" || fail "position classes of $1: $key is '$value'"
        printf -v "BIN_$key" '%s' "$value"
    done <<< "$out"
    for name in BIN_aggregate_sd BIN_min_bin_ratio BIN_max_bin_ratio BIN_lf_ratio; do
        [ -n "${!name:-}" ] || fail "position classes of $1: no ${name#BIN_}"
    done
}
calibrate_probe_rect "$NIRI" 0

cell zero "$NIRI" 'noise 0'
# 2: byte identity against the baseline, one node per kind and site.
for kind in "${KINDS[@]}"; do for site in "${SITES[@]}"; do
    cell "one-$kind-$site"  "$NIRI"      "noise 0.3 type=\"$kind\" site=\"$site\""
    cell "base-$kind-$site" "$BASE_NIRI" "noise 0.3 type=\"$kind\" site=\"$site\""
    ae "$OUT/one-$kind-$site.png" "$OUT/base-$kind-$site.png"
    assert_zero "$kind $site vs baseline" "$METRIC"; metric "${kind}_${site}_baseline_ae" "$METRIC"
done; done
INHERIT_TOP='blur { passes 3; offset 3; noise 0.05; saturation 1; }'
cell inherit      "$NIRI"      'backdrop-blur true' "$INHERIT_TOP"
cell base-inherit "$BASE_NIRI" 'backdrop-blur true' "$INHERIT_TOP"
ae "$OUT/inherit.png" "$OUT/base-inherit.png"
assert_zero "inherited vs baseline" "$METRIC"; metric inherit_baseline_ae "$METRIC"
# 3: scale=1 written equals omitted.
for kind in "${KINDS[@]}"; do
    cell "scale1-$kind" "$NIRI" "noise 0.3 type=\"$kind\" site=\"glass\" scale=1"
    ae "$OUT/scale1-$kind.png" "$OUT/one-$kind-glass.png"
    assert_zero "$kind scale=1 vs omitted" "$METRIC"; metric "${kind}_scale1_ae" "$METRIC"
done
# 4: two independent fine layers add in quadrature.
cell quad-one "$NIRI" 'noise 0.2 type="fine"'
cell quad-two "$NIRI" $'noise 0.2 type="fine"\nnoise 0.2 type="fine"'
grain_sd quad-one zero; one=$METRIC; grain_sd quad-two zero; two=$METRIC
ratio=$(awk -v a="$two" -v b="$one" 'BEGIN { printf "%.4f", a/b }')
metric quadrature_ratio "$ratio"
assert_about "two fine layers over one" "$ratio" 1.41421356 0.05
# 5: deviation by scale and by position in the cell; low frequency rises.
for kind in "${SCALE_KINDS[@]}"; do
    cell "s1-$kind" "$NIRI" "noise 0.3 type=\"$kind\""
    grain_sd "s1-$kind" zero; base=$METRIC; metric "${kind}_s1_sd" "$base"
    bins "s1-$kind" 1; previous=$BIN_lf_ratio; metric "${kind}_s1_lf" "$previous"
    for s in "${SCALES[@]}"; do
        cell "s$s-$kind" "$NIRI" "noise 0.3 type=\"$kind\" scale=$s"
        grain_sd "s$s-$kind" zero; metric "${kind}_s${s}_sd" "$METRIC"
        assert_about "$kind scale $s sd" "$METRIC" "$base" 0.10
        bins "s$s-$kind" "$s"
        metric "${kind}_s${s}_min_bin" "$BIN_min_bin_ratio"; metric "${kind}_s${s}_max_bin" "$BIN_max_bin_ratio"
        metric "${kind}_s${s}_lf" "$BIN_lf_ratio"
        assert_about "$kind scale $s weakest position" "$BIN_min_bin_ratio" 1 0.10
        assert_about "$kind scale $s strongest position" "$BIN_max_bin_ratio" 1 0.10
        assert_greater "$kind scale $s low frequency rises" "$BIN_lf_ratio" "$previous"
        previous=$BIN_lf_ratio
    done
done
# 6: every slot applied and seeded apart.
cell slot0 "$NIRI" 'noise 0.3 type="fine"'
grain_sd slot0 zero; s0=$METRIC
for k in 1 2 3; do
    lines=; for _ in $(seq "$k"); do lines+=$'noise 0\n'; done
    cell "slot$k" "$NIRI" "${lines}noise 0.3 type=\"fine\""
    grain_sd "slot$k" zero; metric "slot${k}_sd" "$METRIC"
    assert_about "slot $k sd" "$METRIC" "$s0" 0.05
    signed_diff "$OUT/slot$k-face.png" "$OUT/slot0-face.png" "$OUT/slot$k-vs-0.png"
    sd "$OUT/slot$k-vs-0.png"; metric "slot${k}_vs_slot0_sd" "$METRIC"
    assert_greater "slot $k differs from slot 0" "$METRIC" "$s0"
done
metric slot0_sd "$s0"

# 7.1: the contact sheet.
SHEET=(s1-fine s8-fine)
sheet_cell() { cell "$1" "$NIRI" "$2"; SHEET+=("$1"); }
if [ "$PILOT" = 0 ]; then
    SHEET=(s1-fine s2-fine s4-fine s8-fine s1-white s2-white s4-white s8-white)
    sheet_cell l1 'noise 0.3 type="lightness"'
    sheet_cell l4 'noise 0.3 type="lightness" scale=4'
fi
sheet_cell stack-white4-fine1 $'noise 0.3 type="white" scale=4\nnoise 0.3 type="fine"'
sheet_cell stack-lightness-film $'noise 0.3 type="lightness"\nnoise 0.3 type="white" site="film"'
sheet_cell four-fine-015 $'noise 0.15 type="fine"\nnoise 0.15 type="fine"\nnoise 0.15 type="fine"\nnoise 0.15 type="fine"'
sheet_cell one-fine-03 'noise 0.3 type="fine"'
args=()
for name in "${SHEET[@]}"; do args+=(-label "$name" "$OUT/$name-face.png"); done
magick montage "${args[@]}" -tile 4x -geometry 360x360+6+6 -background '#202020' -fill white "$OUT/contact-sheet.png" \
    || fail "contact sheet failed"

finish
printf "PASS: noise layers (pilot=%s)\n" "$PILOT"
```

`assert_greater` takes `(label, a, b)` and passes when `a > b`, and `assert_about` takes `(label, value, expected, relative tolerance)`, as in the lib. Run `bash -n docs/materials/scripts/glass-noise-layers-smoke.sh`.

Check `bins()` offline before any capture, with the lib's `fail` and `is_number` and a synthetic grain image, including a failing helper:

```bash
S=$(mktemp -d)
cat > "$S/bins-check.sh" <<'EOF'
set -eu
OUT=$1; BINS=$2
fail() { echo "FAIL: $*" >&2; exit 1; }
is_number() { [[ $1 =~ ^-?[0-9]+([.][0-9]+)?([eE][-+]?[0-9]+)?$ ]]; }
EOF
sed -n '/^# Sets BIN_aggregate_sd/,/^}/p' docs/materials/scripts/glass-noise-layers-smoke.sh >> "$S/bins-check.sh"
printf 'bins cell 8\necho "min=$BIN_min_bin_ratio lf=$BIN_lf_ratio"\n' >> "$S/bins-check.sh"
magick -size 256x256 xc:gray50 +noise Uniform -colorspace Gray "$S/grain-cell.png"
bash "$S/bins-check.sh" "$S" docs/materials/scripts/noise-layers-bins.py
printf '#!/usr/bin/env python3\nimport sys\nsys.exit(3)\n' > "$S/broken.py"
bash "$S/bins-check.sh" "$S" "$S/broken.py"; echo "exit $?"
```

Expected: the first run prints `min=` near 1 and `lf=` near 0.25; the second prints `FAIL: position classes of cell failed` and `exit 1`, with no `BIN_` value printed.

- [ ] **Step 5: Build the baseline and run the pilot**

```bash
git worktree add --detach .worktrees/material-3fcba2-baseline 4a8b2072
( cd .worktrees/material-3fcba2-baseline && just setup && cargo build --release )
```

(Run from the main checkout; `work-link --ensure .worktrees` first, and lock the worktree as the global instructions require.) Then, in `.worktrees/material-3fcba2`:

```bash
OUT=$NIRI_MATERIAL_WORK_ROOT/noise-layers-pilot-$(date +%s) \
BASE_NIRI=<baseline worktree>/target/release/niri \
CAPTURE_TASK=material-a4d874 NOISE_LAYERS_PILOT=1 \
docs/materials/scripts/glass-noise-layers-smoke.sh
```

Expected: `PASS: noise layers (pilot=1)`, about 6 minutes (build 4, cells 2). If `capture_preflight` refuses on host load, record `tasks note material-a4d874 "run: <m> min (est 6, headless); preflight <m>; refused: <cause>"` and park: `tasks park material-a4d874 "Run the noise layers smoke pilot (NOISE_LAYERS_PILOT=1, ~6 min: build 4, cells 2), then the full run (~15 min: build 0, cells 15), then write the evidence document" --reason quiet --waiting-on user --minutes 21 --needs headless`. A refused attempt is analysed (which cells completed, what the preflight measured) before any retry. A failing assertion is a finding: read the cell's metrics before changing code.

- [ ] **Step 6: Run the full smoke**

Same command with `NOISE_LAYERS_PILOT=0` and a fresh `OUT`. Expected: `PASS: noise layers (pilot=0)`, about 15 minutes. Write the `run:` note for every attempt.

- [ ] **Step 7: Write the evidence document**

`docs/materials/<capture-date>-noise-layers-evidence.md` with: the `tools/capture-meta show` output for the pilot and the full run; the full run's `metrics.txt`, grouped by assertion (2 baseline AE, 3 scale=1 AE, 4 quadrature ratio, 5 sd and position classes and low-frequency ratios per kind and scale, 6 slot sds and differences); the contact sheet copied beside it as `<capture-date>-noise-layers-sheet.png`; and an `Owner's look:` line reading `pending`. Attach the sheet to the task: `tasks attach material-3fcba2 <sheet> --caption "Noise layers contact sheet: grain sizes, stacks, four-at-0.15 against one-at-0.3"`.

- [ ] **Step 8: Commit**

```bash
tasks done material-a4d874 "Docs for layers and scale=; nested smoke passes all six assertions against 4a8b2072; evidence and contact sheet"
just check
git add docs tasks/
just upstream-report --stage
git add docs/materials/upstream-divergence.md
git commit -m "docs(materials): noise layers smoke, contact sheet and evidence (material-3fcba2)"
```

---

### Task 5: Cost captures with Tracy

**Files:**
- Modify: `docs/materials/scripts/glass-optic-smoke-lib.sh` (gains the wallpaper-damage helpers), `docs/materials/scripts/noise-placement-cost.sh` (drops its copies)
- Create: `docs/materials/scripts/noise-layers-cost.sh`
- Modify: the evidence document from Task 4 (a Cost section)

**Interfaces:**
- Consumes: the lib's Tracy helpers (`reserve_tracy_port`, `tools_ready`, `capture_bg`, `capture_ready`, `capture_wait`, `csvexport`, `count_last20`), `$NIRI_TRACY`, `probe_rect`.
- Produces: `metrics.txt` keys `<case>_MaterialRenderElement::draw_median_ms`, `<case>_Grain::render_median_ms`, `<case>_glass_area_px`, `<case>_material_ns_per_px_over_none`.

- [x] **Step 1: Move the wallpaper-damage helpers into the lib**

Cut `WALL_PIDS=()`, `wall_count()`, `start_wall()`, `stop_walls()`, `wait_for_wall_removal()` and the comment above `wall_count` (lines 10–43 of `noise-placement-cost.sh` at `4a8b2072`), and `reload_marker()`, `now_ns()`, `sleep_until()` with their comments (lines 84–98), and `cleanup_cost()` with its comment, from `noise-placement-cost.sh` into the lib under a `# --- wallpaper damage` header, verbatim. Each cost script keeps its own `trap cleanup_cost EXIT` line after sourcing the lib. `bash -n` both files.

`tools/test_noise_placement_cost.py` extracts `stop_walls` and `cleanup_cost` from the cost script's text, so it would raise `IndexError` after the move. Point the cleanup test at the lib, which is where both functions now live:

```python
ROOT = Path(__file__).resolve().parent.parent
SCRIPT = ROOT / "docs/materials/scripts/noise-placement-cost.sh"
LIB = ROOT / "docs/materials/scripts/glass-optic-smoke-lib.sh"
```

and in `test_failure_cleanup_reaps_owned_wallpaper_and_preserves_exit_status`, read the functions from the lib while asserting that the script still installs the trap:

```python
        library = LIB.read_text()
        self.assertIn("trap cleanup_cost EXIT", SCRIPT.read_text())
        stop = library.split("stop_walls() {", 1)[1].split("\n}\n", 1)[0]
        cleanup = library.split("cleanup_cost() {", 1)[1].split("\n}\n", 1)[0]
```

(`report_source()` keeps reading the script: the `PYREPORT` block does not move.) Run the focused tooling check:

`just --set one_cmd 'env NIRI_TOOLING_FAST=0 python3 -m unittest' test-one tools.test_noise_placement_cost`

Expected: PASS, all four tests. Then `just --set fast_cmd 'env NIRI_TOOLING_FAST=0 python3 -m tools.tooling_tests --full' test-fast`, since the lib is in `tooling_full_paths`.

- [x] **Step 2: Write the cost script**

`docs/materials/scripts/noise-layers-cost.sh` (mode 0755):

```bash
#!/usr/bin/env bash
# Noise layers Tracy costs (design 2026-10-06-noise-layers-design.md §7.3):
# the material program's GPU span per damaged frame for stacks of fine
# layers at scale 1 and 8, and the backdrop grain pass at scale 1 and 8.
# Each case replaces the wallpaper DAMAGE_STEPS times at 1 Hz; every
# replacement damages the backdrop, which re-renders the glass and reruns the
# grain pass. NOISE_LAYERS_COST_PILOT=1 runs three steps per case.
set -eu
PILOT=${NOISE_LAYERS_COST_PILOT:-0}
case "$PILOT" in 0|1) ;; *) echo "NOISE_LAYERS_COST_PILOT must be 0 or 1" >&2; exit 2 ;; esac
source "$(dirname "$0")/glass-optic-smoke-lib.sh"
trap cleanup_cost EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
capture_preflight headless
build_binaries
capture_identity --config "pilot=$PILOT"
reserve_tracy_port
tools_ready
DAMAGE_STEPS=20
[ "$PILOT" = 0 ] || DAMAGE_STEPS=3
FINE=$'noise 0.15 type="fine"'
declare -A CASES=(
    [none]='noise 0'
    [one-fine-1]='noise 0.3 type="fine"'
    [four-fine-1]="$FINE"$'\n'"$FINE"$'\n'"$FINE"$'\n'"$FINE"
    [four-fine-8]=$(printf '%s scale=8\n' "$FINE" "$FINE" "$FINE" "$FINE")
    [backdrop-fine-1]='noise 0.3 type="fine" site="backdrop"'
    [backdrop-fine-8]='noise 0.3 type="fine" site="backdrop" scale=8'
)
ORDER=(none one-fine-1 four-fine-1 four-fine-8 backdrop-fine-1 backdrop-fine-8)
write_cost_config() {   # $1 path, $2 glass noise lines
    GLASS_EXTRA=$'ior 1\nattenuation-color "#ffffff"\nsaturation 1\n'"$2" \
        TOP_EXTRA='blur { passes 3; offset 3; noise 0; saturation 1; }' write_config "$1.tmp"
    # This script owns and reaps every wallpaper PID.
    sed '/^spawn-at-startup "swaybg" /d' "$1.tmp" > "$1"
    rm "$1.tmp"
}
calibrate_probe_rect "$NIRI" 0
for name in "${ORDER[@]}"; do
    cfg=$OUT/$name.kdl
    write_cost_config "$cfg" "${CASES[$name]}"
    start_nested "$NIRI_TRACY" "$cfg"
    capture_bg "$name"; capture_ready "$name"
    start_wall "$WALL"
    sleep 0.5
    spawn_probe "$NIRI" "$IDLE"
    probe_rect "$NIRI"
    sleep 1
    start=$(now_ns)
    reload_marker "$cfg"
    for i in $(seq 1 "$DAMAGE_STEPS"); do
        magick -size 1280x720 xc:"rgb($((i * 10)),100,120)" "$OUT/$name-wall-$i.png"
        stop_walls; wait_for_wall_removal; start_wall "$OUT/$name-wall-$i.png"
        sleep_until "$((start + i * 1000000000))" 500000000
        reload_marker "$cfg"
    done
    capture_wait
    [ "$(wall_count)" -eq 1 ] || fail "wallpaper layer disappeared during $name"
    stop_walls; stop_nested
    count_last20 "$name" > "$OUT/$name-redraws.txt"
    csvexport --gpu "$OUT/$name.tracy" "$OUT/$name.gpu.csv"
    python3 - "$OUT" "$name" "$DAMAGE_STEPS" "$((PW * PH))" <<'PYREPORT' >> "$OUT/metrics.txt"
import csv, math, pathlib, statistics, sys
root, name, steps, area = pathlib.Path(sys.argv[1]), sys.argv[2], int(sys.argv[3]), int(sys.argv[4])
def zones(path, time_column, duration_column):
    with path.open() as f:
        reader = csv.DictReader(f)
        assert time_column in reader.fieldnames and duration_column in reader.fieldnames, path
        result = [(next(iter(r.values())), float(r[time_column]), float(r[duration_column])) for r in reader]
        assert all(math.isfinite(t) and math.isfinite(d) and t >= 0 and d >= 0 for _, t, d in result), path
        return result
cpu = zones(root / f'{name}.csv', 'ns_since_start', 'exec_time_ns')
gpu = zones(root / f'{name}.gpu.csv', 'Time from start of program', 'GPU execution time')
markers = sorted((t, d) for n, t, d in cpu if n == 'State::reload_config')
assert len(markers) == steps + 1, f'{name}: {len(markers)} reload markers, expected {steps + 1}'
begin, end = markers[0][0], markers[-1][0] + markers[-1][1]
for index, ((lo, _), (hi, _)) in enumerate(zip(markers, markers[1:]), 1):
    events = [n for n, t, _ in cpu if lo <= t < hi]
    assert 'EffectBuffer::sharp_damage' in events, f'{name}: no backdrop damage at step {index}'
for span in ['MaterialRenderElement::draw', 'Grain::render']:
    values = [d for n, t, d in gpu if n == span and begin <= t <= end]
    if span == 'MaterialRenderElement::draw':
        assert len(values) >= steps, f'{name}: {len(values)} material draws for {steps} damages'
    elif name.startswith('backdrop'):
        assert values, f'{name}: no grain pass in the stimulus window'
    print(f'{name}_{span}_gpu_count={len(values)}')
    print(f'{name}_{span}_median_ms={(statistics.median(values) if values else 0) / 1e6:.6f}')
print(f'{name}_glass_area_px={area}')
PYREPORT
done
python3 - "$OUT/metrics.txt" <<'PYDELTA' >> "$OUT/metrics.txt"
import sys
m = dict(line.strip().split('=', 1) for line in open(sys.argv[1]) if '=' in line)
none = float(m['none_MaterialRenderElement::draw_median_ms'])
for name in ['one-fine-1', 'four-fine-1', 'four-fine-8', 'backdrop-fine-1', 'backdrop-fine-8']:
    median = float(m[f'{name}_MaterialRenderElement::draw_median_ms'])
    area = int(m[f'{name}_glass_area_px'])
    print(f'{name}_material_ns_per_px_over_none={(median - none) * 1e6 / area:.4f}')
PYDELTA
finish
printf 'PASS: noise layers cost (pilot=%s)\n' "$PILOT"
```

`bash -n` it. `PW * PH` is the window area; the glass also covers the slab band outside it, so the per-pixel figure is an upper bound and the evidence says so.

- [ ] **Step 3: Pilot, then the full run**

Pilot: `OUT=$NIRI_MATERIAL_WORK_ROOT/noise-layers-cost-pilot-$(date +%s) CAPTURE_TASK=material-2f2af4 NOISE_LAYERS_COST_PILOT=1 docs/materials/scripts/noise-layers-cost.sh`, about 8 minutes (build 4 with Tracy, six cases at about 40 s). Expected `PASS: noise layers cost (pilot=1)`. Then the full run with `NOISE_LAYERS_COST_PILOT=0`, about 10 minutes. On a preflight refusal: `run:` note and `tasks park material-2f2af4 "Run noise-layers-cost.sh pilot (~8 min: build 4, cases 4), then full (~10 min: cases 10), then add the Cost section" --reason quiet --waiting-on user --minutes 18 --needs headless`. Every attempt gets its `run:` note.

- [ ] **Step 4: Report the costs**

Add a `## Cost` section to the evidence document: a table of the six cases' `MaterialRenderElement::draw` and `Grain::render` medians and the per-pixel deltas over `none`, the hash counts of spec §4 beside them (9 per fine layer at scale 1, 16 above it), and one sentence stating that settled glass renders no frames, so these costs are per damaged frame. Report measurements, not verdicts.

- [ ] **Step 5: Commit**

```bash
tasks done material-2f2af4 "Tracy costs of stacked fine layers and the backdrop pass at scale 1 and 8"
just check
git add docs tools/test_noise_placement_cost.py tasks/
just upstream-report --stage
git add docs/materials/upstream-divergence.md
git commit -m "docs(materials): noise layers cost captures (material-3fcba2)"
```

---

### Task 6: Owner's look, gate, whole-branch review, merge, prism hand-off, close

**Files:**
- Modify: the evidence document (`Owner's look:` line), `docs/specs/2026-10-06-noise-layers-design.md` and this plan (status lines)
- Modify (prism): `defs/rack/pipeline.json` (refreshed copy)

- [ ] **Step 1: The owner's look**

Park for the owner: `tasks park material-41d052 "Owner looks at the contact sheet attached to material-3fcba2 (grain sizes, stacks, four at 0.15 against one at 0.3) and judges whether the lattice shows at scale 8; agent records the verdict and continues with the gate" --waiting-on user --reason review`. When the verdict arrives, write it on the evidence document's `Owner's look:` line and note it on the task (`tasks note material-3fcba2 "owner's look: <verdict>"`).

The spec makes a visible lattice a finding that reshapes `scale > 1`, so it gates the merge. If the owner sees the lattice, one of two things happens before Step 2:

- **Fix it on this branch** (the default): amend the spec's §4 with the reshaped lattice (a rotated or jittered lattice; the owner's look names what showed), have that amendment reviewed, change `noiseValue`'s lattice branch, rerun Task 3's tests including both demonstrations, rerun the smoke (Task 4, Steps 5 and 6) and the cost captures (Task 5, Step 3), and ask for the owner's look again.
- **Defer only on the owner's explicit word**: the owner says in so many words that this merge may ship with the lattice. Record it (`tasks note material-3fcba2 "owner's look: lattice visible at scale <s>; owner accepts deferral: <their words>"`) and file the reshape as a task under `material-3aa1f2` with the evidence document's sheet as its source. It is a finding before close, so it carries no `concerns:` note.

- [ ] **Step 2: Gate**

Run: `just gate`. Expected: PASS (full check, Rust tests with doctests).

- [ ] **Step 3: Whole-branch review**

Dispatch one reviewer on the branch against `4a8b2072` with the spec and this plan. Note each round on `material-3fcba2` as `review: impl round <n> — verdict: <revise|accept>; findings: <label> <count>, … | none; reviewer: <harness/model>`. Fix and re-review while Critical or Important findings reproduce, up to five rounds.

- [ ] **Step 4: Merge**

From the main checkout (personal profile: a local merge is the agent's call after an accepted review):

```bash
git merge --no-ff material-3fcba2 -m "merge: noise layers: four grain layers per material with a size each (material-3fcba2)"
just test-fast
```

- [ ] **Step 5: Refresh prism's vendored schema**

In the prism checkout (`tasks root prism-85f63a`), copy `resources/materials/pipeline.json` over `defs/rack/pipeline.json`, run prism's contract test (its AGENTS.md names the recipe), and commit `chore(rack): refresh the vendored pipeline schema for noise scale= (prism-85f63a)` with the rack unchanged.

- [ ] **Step 6: Close**

Mark the spec's and this plan's status lines `implemented <date>`. Then, in one commit on `materials-26.04`:

```bash
tasks done material-41d052 "Owner's look recorded, gate green, review accepted, merged, prism schema refreshed"
tasks done material-3fcba2 "Up to four noise layers per material with type, site and scale=; evidence and costs in docs/materials"
just check
git add docs tasks/
just upstream-report --stage
git add docs/materials/upstream-divergence.md
git commit -m "chore(tasks): close noise layers (material-3fcba2)"
```

Before removing the worktrees: `tt-report`, confirm no host pointer resolves into them (`readlink -f ~/bin/* ~/.local/bin/*`), then `git worktree unlock` and `git worktree remove` for `.worktrees/material-3fcba2` and `.worktrees/material-3fcba2-baseline`.
