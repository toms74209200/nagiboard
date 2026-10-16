pub fn get_deck_asset(dir: &std::path::Path, name: &str) -> Result<Option<String>, String> {
    match std::fs::read_to_string(dir.join("assets").join(name)) {
        Ok(asset) => Ok(Some(asset)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

#[cfg(all(test, feature = "medium"))]
mod medium_tests {
    #[test]
    fn when_get_deck_asset_with_name_of_asset_of_the_app_then_returns_it() {
        let name = random_string::generate_random_string(
            16,
            &[random_string::CharacterType::Lowercase],
            "",
            &mut std::fs::File::open("/dev/urandom").unwrap(),
        );
        let dir = std::env::temp_dir().join(&name);
        std::fs::create_dir_all(dir.join("assets")).unwrap();
        std::fs::write(
            dir.join("assets").join(format!("{name}.js")),
            format!("console.log(\"{name}\")"),
        )
        .unwrap();

        let result = super::get_deck_asset(&dir, &format!("{name}.js"));

        std::fs::remove_dir_all(&dir).unwrap();
        assert_eq!(result, Ok(Some(format!("console.log(\"{name}\")"))));
    }

    #[test]
    fn when_get_deck_asset_with_name_of_no_asset_then_returns_none() {
        let name = random_string::generate_random_string(
            16,
            &[random_string::CharacterType::Lowercase],
            "",
            &mut std::fs::File::open("/dev/urandom").unwrap(),
        );
        let dir = std::env::temp_dir().join(&name);
        std::fs::create_dir_all(dir.join("assets")).unwrap();
        std::fs::write(dir.join(format!("{name}.js")), &name).unwrap();

        let result = super::get_deck_asset(&dir, &format!("{name}.js"));

        std::fs::remove_dir_all(&dir).unwrap();
        assert_eq!(result, Ok(None));
    }
}
