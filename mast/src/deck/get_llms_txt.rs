pub fn get_llms_txt(dir: &std::path::Path) -> Result<String, String> {
    std::fs::read_to_string(dir.join("llms.txt")).map_err(|e| e.to_string())
}

#[cfg(all(test, feature = "medium"))]
mod medium_tests {
    #[test]
    fn when_get_llms_txt_with_dir_of_the_app_then_returns_its_llms_txt() {
        let name = random_string::generate_random_string(
            16,
            &[random_string::CharacterType::Lowercase],
            "",
            &mut std::fs::File::open("/dev/urandom").unwrap(),
        );
        let dir = std::env::temp_dir().join(&name);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("llms.txt"), format!("# {name}\n")).unwrap();

        let result = super::get_llms_txt(&dir);

        std::fs::remove_dir_all(&dir).unwrap();
        assert_eq!(result, Ok(format!("# {name}\n")));
    }

    #[test]
    fn when_get_llms_txt_with_dir_without_llms_txt_then_returns_error() {
        let name = random_string::generate_random_string(
            16,
            &[random_string::CharacterType::Lowercase],
            "",
            &mut std::fs::File::open("/dev/urandom").unwrap(),
        );
        let dir = std::env::temp_dir().join(&name);
        std::fs::create_dir_all(&dir).unwrap();

        let result = super::get_llms_txt(&dir);

        std::fs::remove_dir_all(&dir).unwrap();
        assert!(result.is_err());
    }
}
