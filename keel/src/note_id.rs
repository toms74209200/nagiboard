use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NoteId(String);

impl NoteId {
    pub fn parse(value: &str) -> Option<Self> {
        (value.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
            && value
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-'))
        .then(|| Self(value.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for NoteId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use random_string::CharacterType::{Lowercase, Numeric, Uppercase};
    use random_string::generate_random_string;
    use std::fs::File;

    #[test]
    fn when_parse_with_letters_and_digits_then_returns_the_id() {
        let mut urandom = File::open("/dev/urandom").unwrap();
        let id = generate_random_string(4, &[Lowercase, Uppercase], "", &mut urandom)
            + &generate_random_string(4, &[Numeric], "", &mut urandom);
        assert_eq!(NoteId::parse(&id).unwrap().as_str(), id);
    }

    #[test]
    fn when_parse_with_underscore_start_and_hyphen_then_returns_the_id() {
        let mut urandom = File::open("/dev/urandom").unwrap();
        let id = format!(
            "_{}-{}",
            generate_random_string(4, &[Lowercase, Uppercase], "", &mut urandom),
            generate_random_string(4, &[Numeric], "", &mut urandom)
        );
        assert_eq!(NoteId::parse(&id).unwrap().as_str(), id);
    }

    #[test]
    fn when_parse_with_digit_start_then_returns_none() {
        let mut urandom = File::open("/dev/urandom").unwrap();
        let id = generate_random_string(4, &[Numeric], "", &mut urandom)
            + &generate_random_string(4, &[Lowercase, Uppercase], "", &mut urandom);
        assert_eq!(NoteId::parse(&id), None);
    }

    #[test]
    fn when_parse_with_empty_string_then_returns_none() {
        assert_eq!(NoteId::parse(""), None);
    }

    #[test]
    fn when_parse_with_whitespace_then_returns_none() {
        let mut urandom = File::open("/dev/urandom").unwrap();
        let id = format!(
            "{} {}",
            generate_random_string(4, &[Lowercase, Uppercase], "", &mut urandom),
            generate_random_string(4, &[Numeric], "", &mut urandom)
        );
        assert_eq!(NoteId::parse(&id), None);
    }

    #[test]
    fn when_parse_with_quote_then_returns_none() {
        let mut urandom = File::open("/dev/urandom").unwrap();
        let id = format!(
            "{}\"{}",
            generate_random_string(4, &[Lowercase, Uppercase], "", &mut urandom),
            generate_random_string(4, &[Numeric], "", &mut urandom)
        );
        assert_eq!(NoteId::parse(&id), None);
    }

    #[test]
    fn when_parse_with_non_ascii_letter_then_returns_none() {
        let mut urandom = File::open("/dev/urandom").unwrap();
        let id = format!(
            "é{}",
            generate_random_string(4, &[Numeric], "", &mut urandom)
        );
        assert_eq!(NoteId::parse(&id), None);
    }
}
