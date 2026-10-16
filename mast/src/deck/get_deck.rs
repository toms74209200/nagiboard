pub fn get_deck(dir: &std::path::Path) -> Result<String, String> {
    std::fs::read_to_string(dir.join("index.html")).map_err(|e| e.to_string())
}

#[cfg(all(test, feature = "medium"))]
mod medium_tests {
    #[test]
    fn when_get_deck_with_dir_of_the_app_then_returns_its_index_html() {
        let name = random_string::generate_random_string(
            16,
            &[random_string::CharacterType::Lowercase],
            "",
            &mut std::fs::File::open("/dev/urandom").unwrap(),
        );
        let dir = std::env::temp_dir().join(&name);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("index.html"),
            format!("<!doctype html><title>{name}</title>"),
        )
        .unwrap();

        let result = super::get_deck(&dir);

        std::fs::remove_dir_all(&dir).unwrap();
        assert_eq!(result, Ok(format!("<!doctype html><title>{name}</title>")));
    }

    #[test]
    fn when_get_deck_with_dir_without_index_html_then_returns_error() {
        let name = random_string::generate_random_string(
            16,
            &[random_string::CharacterType::Lowercase],
            "",
            &mut std::fs::File::open("/dev/urandom").unwrap(),
        );
        let dir = std::env::temp_dir().join(&name);
        std::fs::create_dir_all(&dir).unwrap();

        let result = super::get_deck(&dir);

        std::fs::remove_dir_all(&dir).unwrap();
        assert!(result.is_err());
    }
}
