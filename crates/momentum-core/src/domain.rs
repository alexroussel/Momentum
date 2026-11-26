//! Domain enums persisted in SQLite. String representations are stable: they are stored in the
//! database and used by the CSV/JSON export, so never rename a variant's `as_str` value.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("unknown {kind} value: {value}")]
pub struct ParseEnumError {
    pub kind: &'static str,
    pub value: String,
}

macro_rules! string_enum {
    ($(#[$meta:meta])* $name:ident, $kind:literal, { $($variant:ident => $s:literal),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[cfg_attr(feature = "specta", derive(specta::Type))]
        pub enum $name {
            $(#[serde(rename = $s)] $variant),+
        }

        impl $name {
            pub const ALL: &'static [$name] = &[$($name::$variant),+];

            pub fn as_str(self) -> &'static str {
                match self { $($name::$variant => $s),+ }
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl FromStr for $name {
            type Err = ParseEnumError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $($s => Ok($name::$variant),)+
                    _ => Err(ParseEnumError { kind: $kind, value: s.to_owned() }),
                }
            }
        }
    };
}

string_enum!(
    /// Lifecycle of a category.
    CategoryStatus, "category status", { Active => "active", Archived => "archived" }
);

string_enum!(
    /// What a recorded session was.
    SessionKind, "session kind", { Focus => "focus", Break => "break", LongBreak => "long_break" }
);

string_enum!(
    /// How a session ended (names follow the Session export format for compatibility).
    SessionResult, "session result", {
        FullFocus => "fullFocus",
        PartialFocus => "partialFocus",
        EndEarly => "endEarly",
        Abandoned => "abandoned",
    }
);

string_enum!(
    /// Self-reported mood chosen on the reflection screen.
    Mood, "mood", { Distracted => "distracted", Neutral => "neutral", Focused => "focused" }
);

string_enum!(
    /// UI theme preference.
    Theme, "theme", { Dark => "dark", Light => "light", System => "system" }
);

/// The 12 preset category colors (see docs/DESIGN.md §1.2).
pub const CATEGORY_PALETTE: [(&str, &str); 12] = [
    ("coral", "#E5484D"),
    ("violet", "#7C3AED"),
    ("teal", "#14B8A6"),
    ("amber", "#F59E0B"),
    ("blue", "#3B82F6"),
    ("green", "#22C55E"),
    ("pink", "#EC4899"),
    ("orange", "#F97316"),
    ("indigo", "#6366F1"),
    ("cyan", "#06B6D4"),
    ("lime", "#84CC16"),
    ("graphite", "#8B8D98"),
];

/// Returns true if `s` is a `#RRGGBB` hex color.
pub fn is_hex_color(s: &str) -> bool {
    s.len() == 7 && s.starts_with('#') && s[1..].chars().all(|c| c.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enums_round_trip_through_strings() {
        for r in SessionResult::ALL {
            assert_eq!(r.as_str().parse::<SessionResult>().unwrap(), *r);
        }
        for k in SessionKind::ALL {
            assert_eq!(k.as_str().parse::<SessionKind>().unwrap(), *k);
        }
        assert!("nope".parse::<Mood>().is_err());
    }

    #[test]
    fn serde_uses_stable_names() {
        assert_eq!(
            serde_json::to_string(&SessionResult::FullFocus).unwrap(),
            "\"fullFocus\""
        );
        assert_eq!(
            serde_json::to_string(&SessionKind::LongBreak).unwrap(),
            "\"long_break\""
        );
    }

    #[test]
    fn palette_is_valid() {
        assert!(CATEGORY_PALETTE.iter().all(|(_, hex)| is_hex_color(hex)));
        assert!(!is_hex_color("E5484D"));
        assert!(!is_hex_color("#GGGGGG"));
    }
}
