pub fn get_board(
    events: &std::sync::Mutex<Vec<super::board_created::BoardCreated>>,
    board: uuid::Uuid,
) -> Result<Option<String>, String> {
    events
        .lock()
        .map(|events| {
            super::board_created::content_of(&events, board)
                .map(|content| keel::dsl::serialize(&content))
        })
        .map_err(|e| e.to_string())
}

#[cfg(all(test, feature = "medium"))]
mod medium_tests {
    #[test]
    fn when_get_board_with_id_of_created_board_then_returns_its_content_as_dsl() {
        let mut bytes = [0u8; 16];
        std::io::Read::read_exact(
            &mut std::fs::File::open("/dev/urandom").unwrap(),
            &mut bytes,
        )
        .unwrap();
        let board = uuid::Builder::from_random_bytes(bytes).into_uuid();
        let text = random_string::generate_random_string(
            16,
            &[random_string::CharacterType::Lowercase],
            "",
            &mut std::fs::File::open("/dev/urandom").unwrap(),
        );
        let events = std::sync::Mutex::new(vec![super::super::board_created::BoardCreated {
            board,
            content: keel::dsl::parse(&format!("event e1 \"{text}\" @ 0,0\n")).board,
        }]);

        let result = super::get_board(&events, board);

        assert_eq!(
            result,
            Ok(Some(format!(
                "# eventstorming v1\n\nevent     e1   \"{text}\" @ 0,0\n"
            )))
        );
    }

    #[test]
    fn when_get_board_with_id_of_no_board_then_returns_none() {
        let mut bytes = [0u8; 16];
        std::io::Read::read_exact(
            &mut std::fs::File::open("/dev/urandom").unwrap(),
            &mut bytes,
        )
        .unwrap();
        let events = std::sync::Mutex::new(Vec::new());

        let result = super::get_board(&events, uuid::Builder::from_random_bytes(bytes).into_uuid());

        assert_eq!(result, Ok(None));
    }
}
