//! Per-window signal store (design §1). Pure: no timers, no renderer.

use std::collections::VecDeque;
use std::time::Duration;

use niri_config::Color;
use niri_ipc::{ImpulseKind, SignalLevel, SignalMotion};

pub const MAX_EXTERNAL_SLOTS: usize = 16;
pub const MAX_SOURCE_LEN: usize = 64;
pub const MAX_TAG_LEN: usize = 256;
pub const MAX_IMPULSES: usize = 4;
pub const MAX_TTL: Duration = Duration::from_secs(86_400);
pub const IMPULSE_LIFETIME: Duration = Duration::from_millis(1500);
pub const NATIVE_SOURCE: &str = "niri";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Expiry {
    pub at: Duration,
    pub after_level: SignalLevel,
    pub after_motion: SignalMotion,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Slot {
    pub accent: Option<Color>,
    pub level: SignalLevel,
    pub motion: SignalMotion,
    pub tag: Option<String>,
    pub expires: Option<Expiry>,
    pub until_focus: bool,
    pub written_at: Duration,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Impulse {
    pub source: String,
    pub kind: ImpulseKind,
    pub accent: Option<Color>,
    pub at: Duration,
}

impl Impulse {
    pub fn expires_at(&self) -> Duration {
        self.at + IMPULSE_LIFETIME
    }
}

/// Validated input for `set`.
#[derive(Debug, Clone, PartialEq)]
pub struct SetSlot {
    pub accent: Option<Color>,
    pub level: SignalLevel,
    pub motion: SignalMotion,
    pub tag: Option<String>,
    pub expiry: Option<Expiry>,
    pub until_focus: bool,
}

impl SetSlot {
    /// Turns the IPC ttl/after flags into an `Expiry` relative to `now`.
    pub fn expiry_from_ttl(
        ttl: Option<Duration>,
        after_level: Option<SignalLevel>,
        after_motion: Option<SignalMotion>,
        now: Duration,
    ) -> Result<Option<Expiry>, SignalError> {
        Self::validate_ttl(ttl, after_level, after_motion)?;
        Ok(ttl.map(|ttl| Expiry {
            at: now + ttl,
            after_level: after_level.unwrap_or(SignalLevel::Quiet),
            after_motion: after_motion.unwrap_or(SignalMotion::Static),
        }))
    }

    pub fn validate_ttl(
        ttl: Option<Duration>,
        after_level: Option<SignalLevel>,
        after_motion: Option<SignalMotion>,
    ) -> Result<(), SignalError> {
        match ttl {
            Some(ttl) if ttl > MAX_TTL => Err(SignalError::TtlTooLong),
            Some(_) if after_level.is_none() && after_motion.is_none() => {
                Err(SignalError::TtlWithoutAfter)
            }
            None if after_level.is_some() || after_motion.is_some() => {
                Err(SignalError::AfterWithoutTtl)
            }
            _ => Ok(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalError {
    ReservedSource,
    SourceTooLong,
    TagTooLong,
    TooManySlots,
    UnknownSource,
    TtlTooLong,
    TtlWithoutAfter,
    AfterWithoutTtl,
    BadColor,
}

impl std::fmt::Display for SignalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::ReservedSource => "source name \"niri\" is reserved",
            Self::SourceTooLong => "source name longer than 64 bytes",
            Self::TagTooLong => "tag longer than 256 bytes",
            Self::TooManySlots => "window already has 16 signal sources",
            Self::UnknownSource => "no signal slot for this source on this window",
            Self::TtlTooLong => "ttl-ms exceeds 86400000",
            Self::TtlWithoutAfter => "ttl-ms requires after-level or after-motion",
            Self::AfterWithoutTtl => "after-level and after-motion require ttl-ms",
            Self::BadColor => "accent must be #rrggbb or #rrggbbaa",
        };
        f.write_str(s)
    }
}

/// The fold result (design §1).
#[derive(Debug, Clone, PartialEq)]
pub struct Folded {
    pub level: SignalLevel,
    pub motion: SignalMotion,
    pub accent: Option<Color>,
    pub tag: Option<String>,
    pub sources: Vec<String>,
    pub impulses: Vec<Impulse>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct WindowSignals {
    /// Kept sorted by (written_at, name).
    slots: Vec<(String, Slot)>,
    impulses: VecDeque<Impulse>,
}

pub fn parse_accent(s: &str) -> Result<Color, SignalError> {
    let hex = s.strip_prefix('#').ok_or(SignalError::BadColor)?;
    if !(hex.len() == 6 || hex.len() == 8) || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(SignalError::BadColor);
    }
    let byte = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).map_err(|_| SignalError::BadColor);
    Ok(Color::from_rgba8_unpremul(
        byte(0)?,
        byte(2)?,
        byte(4)?,
        0xff,
    ))
}

pub fn accent_hex(c: Color) -> String {
    let q = |v: f32| (v.clamp(0., 1.) * 255.).round() as u8;
    format!("#{:02x}{:02x}{:02x}", q(c.r), q(c.g), q(c.b))
}

impl WindowSignals {
    fn sort(&mut self) {
        self.slots
            .sort_by(|(an, a), (bn, b)| a.written_at.cmp(&b.written_at).then_with(|| an.cmp(bn)));
    }

    fn slot_mut(&mut self, source: &str) -> Option<&mut Slot> {
        self.slots
            .iter_mut()
            .find(|(n, _)| n == source)
            .map(|(_, s)| s)
    }

    fn validate_external(source: &str, tag: Option<&str>) -> Result<(), SignalError> {
        if source == NATIVE_SOURCE {
            return Err(SignalError::ReservedSource);
        }
        if source.len() > MAX_SOURCE_LEN {
            return Err(SignalError::SourceTooLong);
        }
        if tag.is_some_and(|t| t.len() > MAX_TAG_LEN) {
            return Err(SignalError::TagTooLong);
        }
        Ok(())
    }

    pub fn set(&mut self, source: &str, set: SetSlot, now: Duration) -> Result<(), SignalError> {
        Self::validate_external(source, set.tag.as_deref())?;
        let exists = self.slots.iter().any(|(n, _)| n == source);
        let external = self
            .slots
            .iter()
            .filter(|(n, _)| n != NATIVE_SOURCE)
            .count();
        if !exists && external >= MAX_EXTERNAL_SLOTS {
            return Err(SignalError::TooManySlots);
        }
        let slot = Slot {
            accent: set.accent,
            level: set.level,
            motion: set.motion,
            tag: set.tag,
            expires: set.expiry,
            until_focus: set.until_focus,
            written_at: now,
        };
        match self.slot_mut(source) {
            Some(existing) => *existing = slot,
            None => self.slots.push((source.to_owned(), slot)),
        }
        self.sort();
        Ok(())
    }

    pub fn pulse(
        &mut self,
        source: &str,
        kind: ImpulseKind,
        accent: Option<Color>,
        now: Duration,
    ) -> Result<(), SignalError> {
        Self::validate_external(source, None)?;
        if !self.slots.iter().any(|(n, _)| n == source) {
            return Err(SignalError::UnknownSource);
        }
        self.push_impulse(Impulse {
            source: source.to_owned(),
            kind,
            accent,
            at: now,
        });
        Ok(())
    }

    fn push_impulse(&mut self, impulse: Impulse) {
        if self.impulses.len() >= MAX_IMPULSES {
            self.impulses.pop_front();
        }
        self.impulses.push_back(impulse);
    }

    pub fn clear(&mut self, source: &str) -> Result<(), SignalError> {
        Self::validate_external(source, None)?;
        let before = self.slots.len();
        self.slots.retain(|(n, _)| n != source);
        if self.slots.len() == before {
            return Err(SignalError::UnknownSource);
        }
        self.impulses.retain(|i| i.source != source);
        Ok(())
    }

    /// Native slot exists iff urgent (design §1).
    pub fn set_native_urgent(&mut self, urgent: bool, now: Duration) -> bool {
        let exists = self.slots.iter().any(|(n, _)| n == NATIVE_SOURCE);
        match (urgent, exists) {
            (true, false) => {
                self.slots.push((
                    NATIVE_SOURCE.to_owned(),
                    Slot {
                        accent: None,
                        level: SignalLevel::Demand,
                        motion: SignalMotion::Pulse,
                        tag: None,
                        expires: None,
                        until_focus: false,
                        written_at: now,
                    },
                ));
                self.sort();
                self.push_impulse(Impulse {
                    source: NATIVE_SOURCE.to_owned(),
                    kind: ImpulseKind::Ping,
                    accent: None,
                    at: now,
                });
                true
            }
            (false, true) => {
                self.slots.retain(|(n, _)| n != NATIVE_SOURCE);
                self.impulses.retain(|i| i.source != NATIVE_SOURCE);
                true
            }
            _ => false,
        }
    }

    pub fn on_focus(&mut self) -> bool {
        let mut changed = false;
        for (_, slot) in &mut self.slots {
            if slot.until_focus {
                let (level, motion) = slot
                    .expires
                    .map_or((SignalLevel::Quiet, SignalMotion::Static), |e| {
                        (e.after_level, e.after_motion)
                    });
                changed |= slot.level != level || slot.motion != motion || slot.expires.is_some();
                slot.level = level;
                slot.motion = motion;
                slot.expires = None;
                slot.until_focus = false;
            }
        }
        changed
    }

    pub fn advance(&mut self, now: Duration) -> bool {
        let mut changed = false;
        for (_, slot) in &mut self.slots {
            if let Some(e) = slot.expires {
                if now >= e.at {
                    slot.level = e.after_level;
                    slot.motion = e.after_motion;
                    slot.expires = None;
                    slot.until_focus = false;
                    changed = true;
                }
            }
        }
        let before = self.impulses.len();
        self.impulses.retain(|i| now < i.expires_at());
        changed |= self.impulses.len() != before;
        changed
    }

    pub fn next_deadline(&self) -> Option<Duration> {
        let slots = self
            .slots
            .iter()
            .filter_map(|(_, s)| s.expires.map(|e| e.at));
        let impulses = self.impulses.iter().map(Impulse::expires_at);
        slots.chain(impulses).min()
    }

    pub fn fold(&self, now: Duration) -> Option<Folded> {
        if self.slots.is_empty() {
            return None;
        }
        let effective = |s: &Slot| match s.expires {
            Some(e) if now >= e.at => (e.after_level, e.after_motion),
            _ => (s.level, s.motion),
        };
        let max_level = self.slots.iter().map(|(_, s)| effective(s).0).max()?;
        let (_, winner) = self
            .slots
            .iter()
            .rev()
            .find(|(_, s)| effective(s).0 == max_level)?;
        let accent = winner
            .accent
            .or_else(|| self.slots.iter().find_map(|(_, s)| s.accent));
        let tag = winner
            .tag
            .clone()
            .or_else(|| self.slots.iter().find_map(|(_, s)| s.tag.clone()));
        Some(Folded {
            level: max_level,
            motion: effective(winner).1,
            accent,
            tag,
            sources: self.slots.iter().map(|(n, _)| n.clone()).collect(),
            impulses: self
                .impulses
                .iter()
                .filter(|i| now < i.expires_at())
                .cloned()
                .collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use niri_ipc::{ImpulseKind, SignalLevel as L, SignalMotion as M};

    fn ms(v: u64) -> Duration {
        Duration::from_millis(v)
    }

    fn set(level: L, motion: M) -> SetSlot {
        SetSlot {
            accent: None,
            level,
            motion,
            tag: None,
            expiry: None,
            until_focus: false,
        }
    }

    #[test]
    fn level_is_max_and_motion_follows_winner() {
        let mut s = WindowSignals::default();
        s.set(
            "a",
            SetSlot {
                motion: M::Breathe,
                ..set(L::Active, M::Breathe)
            },
            ms(1),
        )
        .unwrap();
        s.set("b", set(L::Demand, M::Flash), ms(2)).unwrap();
        let f = s.fold(ms(2)).unwrap();
        assert_eq!(f.level, L::Demand);
        assert_eq!(f.motion, M::Flash);
        assert_eq!(f.sources, vec!["a", "b"]);
    }

    #[test]
    fn ties_go_to_latest_written() {
        let mut s = WindowSignals::default();
        s.set("a", set(L::Notice, M::Breathe), ms(1)).unwrap();
        s.set("b", set(L::Notice, M::Pulse), ms(2)).unwrap();
        assert_eq!(s.fold(ms(2)).unwrap().motion, M::Pulse);
    }

    #[test]
    fn accent_and_tag_select_independently() {
        let mut s = WindowSignals::default();
        let accent = parse_accent("#e5a33c").unwrap();
        s.set(
            "a",
            SetSlot {
                accent: Some(accent),
                ..set(L::Quiet, M::Static)
            },
            ms(1),
        )
        .unwrap();
        s.set(
            "b",
            SetSlot {
                tag: Some(String::from("t")),
                ..set(L::Demand, M::Pulse)
            },
            ms(2),
        )
        .unwrap();
        let f = s.fold(ms(2)).unwrap();
        assert_eq!(f.accent, Some(accent));
        assert_eq!(f.tag.as_deref(), Some("t"));
    }

    #[test]
    fn native_slot_exists_iff_urgent_and_pings_on_rise() {
        let mut s = WindowSignals::default();
        assert!(s.set_native_urgent(true, ms(5)));
        let f = s.fold(ms(5)).unwrap();
        assert_eq!((f.level, f.motion), (L::Demand, M::Pulse));
        assert_eq!(f.impulses.len(), 1);
        assert_eq!(f.impulses[0].kind, ImpulseKind::Ping);
        assert!(s.set_native_urgent(false, ms(6)));
        assert!(s.fold(ms(6)).is_none());
    }

    #[test]
    fn external_requests_cannot_touch_the_native_slot() {
        let mut s = WindowSignals::default();
        s.set_native_urgent(true, ms(0));
        assert_eq!(
            s.pulse("niri", ImpulseKind::Done, None, ms(1)),
            Err(SignalError::ReservedSource)
        );
        assert_eq!(s.clear("niri"), Err(SignalError::ReservedSource));
        assert_eq!(s.fold(ms(1)).unwrap().sources, vec!["niri"]);
    }

    #[test]
    fn reserved_source_and_bounds_are_rejected_without_mutation() {
        let mut s = WindowSignals::default();
        assert_eq!(
            s.set("niri", set(L::Quiet, M::Static), ms(1)),
            Err(SignalError::ReservedSource)
        );
        let long = "x".repeat(MAX_SOURCE_LEN + 1);
        assert_eq!(
            s.set(&long, set(L::Quiet, M::Static), ms(1)),
            Err(SignalError::SourceTooLong)
        );
        let tag = Some("t".repeat(MAX_TAG_LEN + 1));
        assert_eq!(
            s.set(
                "a",
                SetSlot {
                    tag,
                    ..set(L::Quiet, M::Static)
                },
                ms(1)
            ),
            Err(SignalError::TagTooLong)
        );
        for i in 0..MAX_EXTERNAL_SLOTS {
            s.set(&format!("s{i}"), set(L::Quiet, M::Static), ms(i as u64))
                .unwrap();
        }
        assert_eq!(
            s.set("one-more", set(L::Quiet, M::Static), ms(99)),
            Err(SignalError::TooManySlots)
        );
        assert_eq!(s.fold(ms(99)).unwrap().sources.len(), MAX_EXTERNAL_SLOTS);
    }

    #[test]
    fn ttl_is_capped_and_after_requires_ttl() {
        let mut s = WindowSignals::default();
        assert_eq!(
            SetSlot::validate_ttl(Some(MAX_TTL + ms(1)), Some(L::Quiet), None),
            Err(SignalError::TtlTooLong)
        );
        assert_eq!(
            SetSlot::validate_ttl(None, Some(L::Quiet), None),
            Err(SignalError::AfterWithoutTtl)
        );
        assert!(SetSlot::validate_ttl(Some(ms(10)), None, None).is_err());
        assert!(s.set("a", set(L::Quiet, M::Static), ms(1)).is_ok());
    }

    #[test]
    fn slot_decays_to_after_and_keeps_accent() {
        let mut s = WindowSignals::default();
        let accent = parse_accent("#112233").unwrap();
        let expiry = Some(Expiry {
            at: ms(1000),
            after_level: L::Quiet,
            after_motion: M::Breathe,
        });
        s.set(
            "a",
            SetSlot {
                accent: Some(accent),
                expiry,
                ..set(L::Demand, M::Pulse)
            },
            ms(0),
        )
        .unwrap();
        assert_eq!(s.next_deadline(), Some(ms(1000)));
        assert_eq!(s.fold(ms(999)).unwrap().level, L::Demand);
        assert_eq!(s.fold(ms(1000)).unwrap().level, L::Quiet);
        assert!(!s.advance(ms(999)));
        assert!(s.advance(ms(1000)));
        let f = s.fold(ms(1000)).unwrap();
        assert_eq!(
            (f.level, f.motion, f.accent),
            (L::Quiet, M::Breathe, Some(accent))
        );
        assert_eq!(s.next_deadline(), None);
    }

    #[test]
    fn expiry_then_focus_demotes_once_in_either_order() {
        let expiry = Some(Expiry {
            at: ms(1000),
            after_level: L::Notice,
            after_motion: M::Breathe,
        });
        let mut s = WindowSignals::default();
        s.set(
            "a",
            SetSlot {
                until_focus: true,
                expiry,
                ..set(L::Demand, M::Pulse)
            },
            ms(0),
        )
        .unwrap();
        assert!(s.advance(ms(1000)));
        assert!(!s.on_focus());
        let f = s.fold(ms(1000)).unwrap();
        assert_eq!((f.level, f.motion), (L::Notice, M::Breathe));
        let mut s = WindowSignals::default();
        s.set(
            "a",
            SetSlot {
                until_focus: true,
                expiry,
                ..set(L::Demand, M::Pulse)
            },
            ms(0),
        )
        .unwrap();
        assert!(s.on_focus());
        assert_eq!(s.next_deadline(), None);
        assert!(!s.advance(ms(1000)));
        let f = s.fold(ms(1000)).unwrap();
        assert_eq!((f.level, f.motion), (L::Notice, M::Breathe));
    }

    #[test]
    fn until_focus_demotes_only_asking_slots() {
        let mut s = WindowSignals::default();
        s.set(
            "a",
            SetSlot {
                until_focus: true,
                ..set(L::Demand, M::Pulse)
            },
            ms(0),
        )
        .unwrap();
        s.set("b", set(L::Notice, M::Breathe), ms(1)).unwrap();
        assert!(s.on_focus());
        let f = s.fold(ms(2)).unwrap();
        assert_eq!((f.level, f.motion), (L::Notice, M::Breathe));
    }

    #[test]
    fn pulse_requires_slot_and_fifth_evicts_oldest() {
        let mut s = WindowSignals::default();
        assert_eq!(
            s.pulse("a", ImpulseKind::Done, None, ms(0)),
            Err(SignalError::UnknownSource)
        );
        s.set("a", set(L::Quiet, M::Static), ms(0)).unwrap();
        for i in 0..5u64 {
            s.pulse("a", ImpulseKind::Done, None, ms(i)).unwrap();
        }
        let f = s.fold(ms(4)).unwrap();
        assert_eq!(f.impulses.len(), MAX_IMPULSES);
        assert_eq!(f.impulses[0].at, ms(1));
        assert_eq!(s.next_deadline(), Some(ms(1) + IMPULSE_LIFETIME));
        assert_eq!(s.fold(ms(1) + IMPULSE_LIFETIME).unwrap().impulses.len(), 3);
        assert!(s.advance(ms(1) + IMPULSE_LIFETIME));
        assert_eq!(s.fold(ms(1) + IMPULSE_LIFETIME).unwrap().impulses.len(), 3);
    }

    #[test]
    fn clear_drops_slot_and_its_impulses() {
        let mut s = WindowSignals::default();
        s.set("a", set(L::Quiet, M::Static), ms(0)).unwrap();
        s.pulse("a", ImpulseKind::Ping, None, ms(0)).unwrap();
        assert_eq!(s.clear("b"), Err(SignalError::UnknownSource));
        s.clear("a").unwrap();
        assert!(s.fold(ms(0)).is_none());
    }

    #[test]
    fn accent_parses_and_normalizes() {
        assert_eq!(accent_hex(parse_accent("#E5A33Cff").unwrap()), "#e5a33c");
        assert_eq!(parse_accent("nope"), Err(SignalError::BadColor));
    }
}
