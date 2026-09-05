use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt, str::FromStr};

/// The fixed reaction vocabulary, stored by name so the database never contains raw emoji.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Emoji {
    Alien,
    Crab,
    Heart,
    Fire,
    Eyes,
    Party,
}

impl Emoji {
    pub const ALL: [Self; 6] = [
        Self::Alien,
        Self::Crab,
        Self::Heart,
        Self::Fire,
        Self::Eyes,
        Self::Party,
    ];

    pub const fn glyph(self) -> &'static str {
        match self {
            Self::Alien => "👽",
            Self::Crab => "🦀",
            Self::Heart => "❤️",
            Self::Fire => "🔥",
            Self::Eyes => "👀",
            Self::Party => "🎉",
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Alien => "alien",
            Self::Crab => "crab",
            Self::Heart => "heart",
            Self::Fire => "fire",
            Self::Eyes => "eyes",
            Self::Party => "party",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParseEmojiError;

impl fmt::Display for ParseEmojiError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("unknown emoji name")
    }
}

impl std::error::Error for ParseEmojiError {}

impl FromStr for Emoji {
    type Err = ParseEmojiError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "alien" => Ok(Self::Alien),
            "crab" => Ok(Self::Crab),
            "heart" => Ok(Self::Heart),
            "fire" => Ok(Self::Fire),
            "eyes" => Ok(Self::Eyes),
            "party" => Ok(Self::Party),
            _ => Err(ParseEmojiError),
        }
    }
}

impl fmt::Display for Emoji {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.glyph())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReactionTarget {
    Post { slug: String },
    Comment(CommentId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReactionCount {
    pub emoji: Emoji,
    pub count: u32,
    /// Whether the current viewer left this reaction.
    pub reacted: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reactions {
    pub post: Vec<ReactionCount>,
    pub comments: HashMap<CommentId, Vec<ReactionCount>>,
}

use super::CommentId;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn persisted_emoji_names_remain_compatible() {
        for (emoji, stored) in [
            (Emoji::Alien, "alien"),
            (Emoji::Crab, "crab"),
            (Emoji::Heart, "heart"),
            (Emoji::Fire, "fire"),
            (Emoji::Eyes, "eyes"),
            (Emoji::Party, "party"),
        ] {
            assert_eq!(emoji.name(), stored);
            assert_eq!(stored.parse::<Emoji>(), Ok(emoji));
        }
    }

    #[test]
    fn serialized_emoji_names_remain_compatible() {
        // The server-function wire format uses enum names, not lowercase database names.
        for (emoji, json) in [
            (Emoji::Alien, r#""Alien""#),
            (Emoji::Crab, r#""Crab""#),
            (Emoji::Heart, r#""Heart""#),
            (Emoji::Fire, r#""Fire""#),
            (Emoji::Eyes, r#""Eyes""#),
            (Emoji::Party, r#""Party""#),
        ] {
            assert_eq!(serde_json::to_string(&emoji).unwrap(), json);
            assert_eq!(serde_json::from_str::<Emoji>(json).unwrap(), emoji);
        }
    }

    #[test]
    fn unknown_emoji_names_are_rejected() {
        assert!("👽".parse::<Emoji>().is_err());
        assert!("unknown".parse::<Emoji>().is_err());
        assert!(serde_json::from_str::<Emoji>(r#""Unknown""#).is_err());
    }

    #[test]
    fn emoji_display_uses_glyphs() {
        assert_eq!(Emoji::Alien.to_string(), "👽");
        assert_eq!(Emoji::Heart.to_string(), "❤️");
    }
}
