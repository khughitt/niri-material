//! Where the rigid tilt's swing comes from (docs/materials/motion.md#swing-origin): the last
//! focus an update pass observed. The `Layout` observes the one
//! globally focused window, on its monitor, once per pass that visits it.

use smithay::output::Output;
use smithay::utils::{Logical, Point};

use super::monitor::Monitor;
use super::LayoutElement;
use crate::render_helpers::material::view::{direction_between, SwingFrom};

/// The last observed focus: the window, its output, and its tile's centre in
/// output-local logical coordinates.
#[derive(Debug, Clone, PartialEq)]
pub struct FocusOrigin<Id, O> {
    pub window: Id,
    pub output: O,
    pub centre: Point<f64, Logical>,
}

/// What observing `window` on `output` at `centre` means for the swing:
/// `None` when the focus is the origin's window, otherwise where the new
/// focus swings from.
pub fn swing_from<Id: PartialEq, O: PartialEq>(
    origin: Option<&FocusOrigin<Id, O>>,
    window: &Id,
    output: &O,
    centre: Point<f64, Logical>,
) -> Option<SwingFrom> {
    match origin {
        Some(origin) if origin.window == *window => None,
        Some(origin) if origin.output == *output => Some(direction_between(
            [origin.centre.x, origin.centre.y],
            [centre.x, centre.y],
        )),
        _ => Some(SwingFrom::Fallback),
    }
}

/// One observation on the focused window's monitor, after its overview
/// state is synced: start the swing on a changed focus, then record the
/// focus as the origin. A focused window whose workspace is culled off the
/// output leaves the origin untouched.
///
/// A screencopy rendered without damage can run this pass after an
/// input-driven focus change but before `refresh()` re-resolves window
/// rules, so the swing is started, and the change consumed, under the
/// material the old is-focused rules chose. Accepted, not guarded.
pub fn observe<W: LayoutElement>(
    origin: &mut Option<FocusOrigin<W::Id, Output>>,
    mon: &mut Monitor<W>,
    window: &W::Id,
) {
    let Some(centre) = mon.tile_centre_on_output(window) else {
        return;
    };
    if let Some(from) = swing_from(origin.as_ref(), window, &mon.output, centre) {
        if let Some(tile) = mon.tile_mut(window) {
            tile.start_view_swing(from);
        }
    }
    *origin = Some(FocusOrigin {
        window: window.clone(),
        output: mon.output.clone(),
        centre,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(x: f64, y: f64) -> Point<f64, Logical> {
        Point::from((x, y))
    }

    fn origin(window: u32, output: &'static str, x: f64) -> FocusOrigin<u32, &'static str> {
        FocusOrigin {
            window,
            output,
            centre: at(x, 100.),
        }
    }

    #[test]
    fn no_origin_falls_back() {
        assert_eq!(
            swing_from::<u32, &str>(None, &1, &"a", at(0., 0.)),
            Some(SwingFrom::Fallback)
        );
    }

    #[test]
    fn the_same_window_starts_nothing() {
        let o = origin(1, "a", 100.);
        assert_eq!(swing_from(Some(&o), &1, &"a", at(900., 100.)), None);
    }

    #[test]
    fn another_window_on_the_same_output_swings_from_the_origin() {
        let o = origin(1, "a", 100.);
        assert_eq!(
            swing_from(Some(&o), &2, &"a", at(900., 100.)),
            Some(SwingFrom::Origin([1., 0.]))
        );
    }

    #[test]
    fn another_output_or_a_coincident_centre_falls_back() {
        let o = origin(1, "a", 100.);
        assert_eq!(
            swing_from(Some(&o), &2, &"b", at(900., 100.)),
            Some(SwingFrom::Fallback)
        );
        assert_eq!(
            swing_from(Some(&o), &2, &"a", at(100.4, 100.)),
            Some(SwingFrom::Fallback)
        );
    }
}
