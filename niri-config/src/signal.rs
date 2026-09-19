use std::time::Duration;

use crate::utils::MergeWith;

/// Global motion policy for material signals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SignalMotionPolicy {
    #[default]
    Full,
    Reduced,
    Off,
}

impl std::str::FromStr for SignalMotionPolicy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "full" => Ok(Self::Full),
            "reduced" => Ok(Self::Reduced),
            "off" => Ok(Self::Off),
            _ => Err(format!("unknown signal motion policy: {s}")),
        }
    }
}

/// The top-level `signal { }` block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Signal {
    pub motion: SignalMotionPolicy,
    /// Input-idle threshold for sustained attention motion; zero disables the gate.
    pub idle_after: Duration,
}

impl Default for Signal {
    fn default() -> Self {
        Self {
            motion: SignalMotionPolicy::default(),
            idle_after: Duration::from_millis(30_000),
        }
    }
}

#[derive(knuffel::Decode, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SignalPart {
    #[knuffel(child, unwrap(argument, str))]
    pub motion: Option<SignalMotionPolicy>,
    #[knuffel(child, unwrap(argument))]
    pub idle_after_ms: Option<u32>,
}

impl SignalPart {
    pub fn validate(&self) -> Result<(), String> {
        if self.idle_after_ms.is_some_and(|ms| ms > 3_600_000) {
            return Err(String::from("idle-after-ms must be at most 3600000"));
        }
        Ok(())
    }
}

impl MergeWith<SignalPart> for Signal {
    fn merge_with(&mut self, part: &SignalPart) {
        if let Some(motion) = part.motion {
            self.motion = motion;
        }
        if let Some(ms) = part.idle_after_ms {
            self.idle_after = Duration::from_millis(u64::from(ms));
        }
    }
}
