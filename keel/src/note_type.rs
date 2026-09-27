#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NoteType {
    Event,
    Command,
    Actor,
    Aggregate,
    Policy,
    ReadModel,
    System,
    Hotspot,
}

impl NoteType {
    const ALL: [NoteType; 8] = [
        NoteType::Event,
        NoteType::Command,
        NoteType::Actor,
        NoteType::Aggregate,
        NoteType::Policy,
        NoteType::ReadModel,
        NoteType::System,
        NoteType::Hotspot,
    ];

    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|t| t.as_str() == value)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            NoteType::Event => "event",
            NoteType::Command => "command",
            NoteType::Actor => "actor",
            NoteType::Aggregate => "aggregate",
            NoteType::Policy => "policy",
            NoteType::ReadModel => "readmodel",
            NoteType::System => "system",
            NoteType::Hotspot => "hotspot",
        }
    }

    pub(crate) fn can_connect_to(self, to: NoteType) -> bool {
        use NoteType::*;
        matches!(
            (self, to),
            (Actor, Command)
                | (Command, Aggregate)
                | (Command, System)
                | (Aggregate, Event)
                | (System, Event)
                | (Event, Policy)
                | (Event, ReadModel)
                | (Policy, Command)
                | (ReadModel, Actor)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn when_parse_with_known_type_then_returns_the_type() {
        assert_eq!(NoteType::parse("event"), Some(NoteType::Event));
    }

    #[test]
    fn when_parse_with_unknown_type_then_returns_none() {
        assert_eq!(NoteType::parse("unknown"), None);
    }

    #[test]
    fn when_parse_with_empty_string_then_returns_none() {
        assert_eq!(NoteType::parse(""), None);
    }

    #[test]
    fn when_parse_then_round_trips_every_type() {
        for t in NoteType::ALL {
            assert_eq!(NoteType::parse(t.as_str()), Some(t));
        }
    }

    #[test]
    fn when_actor_to_command_then_allowed() {
        assert!(NoteType::Actor.can_connect_to(NoteType::Command));
    }

    #[test]
    fn when_command_to_aggregate_then_allowed() {
        assert!(NoteType::Command.can_connect_to(NoteType::Aggregate));
    }

    #[test]
    fn when_event_to_policy_then_allowed() {
        assert!(NoteType::Event.can_connect_to(NoteType::Policy));
    }

    #[test]
    fn when_command_to_actor_then_not_allowed() {
        assert!(!NoteType::Command.can_connect_to(NoteType::Actor));
    }

    #[test]
    fn when_hotspot_to_event_then_not_allowed() {
        assert!(!NoteType::Hotspot.can_connect_to(NoteType::Event));
    }

    #[test]
    fn when_event_to_hotspot_then_not_allowed() {
        assert!(!NoteType::Event.can_connect_to(NoteType::Hotspot));
    }
}
