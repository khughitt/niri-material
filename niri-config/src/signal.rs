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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Signal {
    pub motion: SignalMotionPolicy,
}

#[derive(knuffel::Decode, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SignalPart {
    #[knuffel(child, unwrap(argument, str))]
    pub motion: Option<SignalMotionPolicy>,
}

impl MergeWith<SignalPart> for Signal {
    fn merge_with(&mut self, part: &SignalPart) {
        if let Some(motion) = part.motion {
            self.motion = motion;
        }
    }
}
