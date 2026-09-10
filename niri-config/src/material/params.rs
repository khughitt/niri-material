//! Parameter metadata: the facts the docs table and the parser share.
//!
//! Design: `docs/specs/2026-09-10-material-optics-design.md` §6.

use crate::appearance::Color;
use crate::FloatOrInt;

use super::{Milli, Positive, ResolvedGlass};

/// The parse-time bounds of a scalar parameter type, as the type declares
/// them. The docs table and the parser test read these so a range can only
/// be written once, on the type.
pub trait Bounded {
    const MIN: f64;
    const MAX: f64;
    /// Whether `MIN` itself is rejected (`Positive`).
    const MIN_EXCLUSIVE: bool = false;
}

impl<const MIN: i32, const MAX: i32> Bounded for FloatOrInt<MIN, MAX> {
    const MIN: f64 = MIN as f64;
    const MAX: f64 = MAX as f64;
}

impl<const MIN_MILLI: i32, const MAX_MILLI: i32> Bounded for Milli<MIN_MILLI, MAX_MILLI> {
    const MIN: f64 = MIN_MILLI as f64 / 1000.;
    const MAX: f64 = MAX_MILLI as f64 / 1000.;
}

impl<const MAX: i32> Bounded for Positive<MAX> {
    const MIN: f64 = 0.;
    const MAX: f64 = MAX as f64;
    const MIN_EXCLUSIVE: bool = true;
}

/// What kind of value a parameter takes, with its default and bounds taken
/// from the resolved type and the bound type rather than typed by hand.
#[derive(Debug, Clone, PartialEq)]
pub enum ParamKind {
    Float {
        default: f64,
        min: f64,
        max: f64,
        min_exclusive: bool,
        unit: &'static str,
    },
    /// A float whose omitted value inherits (noise and saturation).
    FloatOrInherit {
        min: f64,
        max: f64,
    },
    Color {
        default: Color,
    },
    Bool {
        default: bool,
    },
    Enum {
        default: &'static str,
        variants: &'static [&'static str],
    },
}

impl ParamKind {
    pub fn float<B: Bounded>(default: f64, unit: &'static str) -> Self {
        Self::Float {
            default,
            min: B::MIN,
            max: B::MAX,
            min_exclusive: B::MIN_EXCLUSIVE,
            unit,
        }
    }

    pub fn inherit<B: Bounded>() -> Self {
        Self::FloatOrInherit {
            min: B::MIN,
            max: B::MAX,
        }
    }
}

/// One parameter as the docs table and the parser test see it.
pub struct ParamSpec {
    /// The node as the table names it: `"noise"`, `"noise type="`,
    /// `"distortion scale="`. Words are rendered in separate backticks.
    pub node: &'static str,
    pub kind: ParamKind,
    /// The KDL line inside `glass { }` that sets this parameter to a value.
    pub write: fn(&str) -> String,
    /// The resolved value, for `Float` and `FloatOrInherit` kinds.
    pub read: Option<fn(&ResolvedGlass) -> Option<f64>>,
}

/// A number as the table prints it: integers without a decimal point, a
/// typographic minus.
fn num(v: f64) -> String {
    let s = if v.fract() == 0. {
        format!("{}", v as i64)
    } else {
        format!("{v}")
    };
    s.replace('-', "\u{2212}")
}

fn range(min: f64, max: f64, min_exclusive: bool) -> String {
    if min_exclusive {
        format!("> {} through {}", num(min), num(max))
    } else {
        format!("{}–{}", num(min), num(max))
    }
}

fn color_hex(color: &Color) -> String {
    let [r, g, b, _] = color.to_array_unpremul();
    let channel = |c: f32| (c * 255.).round() as u8;
    format!("#{:02x}{:02x}{:02x}", channel(r), channel(g), channel(b))
}

/// Renders the Markdown parameter table `material-config.md` carries.
pub fn render_param_table(specs: &[ParamSpec]) -> String {
    let mut out = String::from(
        "| Parameter | Type | Default | Range | Unit |\n| --- | --- | --- | --- | --- |\n",
    );
    for spec in specs {
        let node = spec
            .node
            .split(' ')
            .map(|word| format!("`{word}`"))
            .collect::<Vec<_>>()
            .join(" ");
        let (ty, default, range, unit) = match &spec.kind {
            ParamKind::Float {
                default,
                min,
                max,
                min_exclusive,
                unit,
            } => (
                String::from("float"),
                num(*default),
                self::range(*min, *max, *min_exclusive),
                String::from(*unit),
            ),
            ParamKind::FloatOrInherit { min, max } => (
                String::from("float"),
                String::from("inherit"),
                self::range(*min, *max, false),
                String::from("—"),
            ),
            ParamKind::Color { default } => (
                String::from("color"),
                format!("`{}`", color_hex(default)),
                String::from("any color"),
                String::from("—"),
            ),
            ParamKind::Bool { default } => (
                String::from("bool"),
                default.to_string(),
                String::from("true / false"),
                String::from("—"),
            ),
            ParamKind::Enum { default, variants } => (
                variants
                    .iter()
                    .map(|v| format!("`{v}`"))
                    .collect::<Vec<_>>()
                    .join(" / "),
                format!("`{default}`"),
                String::from("—"),
                String::from("—"),
            ),
        };
        out.push_str(&format!(
            "| {node} | {ty} | {default} | {range} | {unit} |\n"
        ));
    }
    out
}

#[cfg(test)]
#[allow(clippy::assertions_on_constants)]
mod tests {
    use super::*;
    use crate::material::{Milli, Positive};
    use crate::FloatOrInt;

    #[test]
    fn bounds_follow_the_type_parameters() {
        assert_eq!(<FloatOrInt<1, 3> as Bounded>::MIN, 1.);
        assert_eq!(<FloatOrInt<1, 3> as Bounded>::MAX, 3.);
        assert!(!<FloatOrInt<1, 3> as Bounded>::MIN_EXCLUSIVE);
        assert_eq!(<Milli<0, 20> as Bounded>::MAX, 0.02);
        assert_eq!(<Positive<65535> as Bounded>::MIN, 0.);
        assert!(<Positive<65535> as Bounded>::MIN_EXCLUSIVE);
    }

    #[test]
    fn the_table_renders_one_row_per_spec_in_order() {
        let specs = [
            ParamSpec {
                node: "ior",
                kind: ParamKind::float::<FloatOrInt<1, 3>>(1.5, "—"),
                write: |v| format!("ior {v}"),
                read: Some(|g| Some(g.ior)),
            },
            ParamSpec {
                node: "offset-x",
                kind: ParamKind::float::<FloatOrInt<-64, 64>>(6., "logical px"),
                write: |v| format!("offset-x {v}"),
                read: Some(|g| Some(g.offset_x)),
            },
            ParamSpec {
                node: "attenuation-distance",
                kind: ParamKind::float::<Positive<65535>>(60., "logical px"),
                write: |v| format!("attenuation-distance {v}"),
                read: Some(|g| Some(g.attenuation_distance)),
            },
            ParamSpec {
                node: "noise type=",
                kind: ParamKind::Enum {
                    default: "white",
                    variants: &["white", "fine", "lightness"],
                },
                write: |v| format!("noise 0.5 type=\"{v}\""),
                read: None,
            },
        ];
        let table = render_param_table(&specs);
        let expected = "\
| Parameter | Type | Default | Range | Unit |
| --- | --- | --- | --- | --- |
| `ior` | float | 1.5 | 1–3 | — |
| `offset-x` | float | 6 | −64–64 | logical px |
| `attenuation-distance` | float | 60 | > 0 through 65535 | logical px |
| `noise` `type=` | `white` / `fine` / `lightness` | `white` | — | — |
";
        assert_eq!(table, expected);
    }
}
