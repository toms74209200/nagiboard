#[derive(Debug, Clone, PartialEq)]
pub struct NonEmpty<T>(Vec<T>);

impl<T> NonEmpty<T> {
    pub fn parse(items: Vec<T>) -> Option<Self> {
        (!items.is_empty()).then_some(Self(items))
    }

    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.0.iter()
    }
}

impl<T> IntoIterator for NonEmpty<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn when_parse_with_items_then_keeps_them_in_order() {
        let items: Vec<u8> = random_string::generate_random_string(
            8,
            &[random_string::CharacterType::Lowercase],
            "",
            &mut std::fs::File::open("/dev/urandom").unwrap(),
        )
        .into_bytes();

        let parsed = super::NonEmpty::parse(items.clone());

        assert_eq!(
            parsed.map(|p| p.into_iter().collect::<Vec<_>>()),
            Some(items)
        );
    }

    #[test]
    fn when_parse_without_items_then_returns_none() {
        assert_eq!(super::NonEmpty::<u8>::parse(Vec::new()), None);
    }
}
