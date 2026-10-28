pub fn get_board(
    board_events: &std::sync::Mutex<Vec<super::board_created::BoardCreated>>,
    board_edited_events: &std::sync::Mutex<Vec<super::board_edited::BoardEdited>>,
    board: uuid::Uuid,
) -> Result<Option<String>, String> {
    let created = board_events.lock().map_err(|e| e.to_string())?;
    let edited = board_edited_events.lock().map_err(|e| e.to_string())?;
    Ok(
        super::board_content::BoardContent::of(&created, &edited, board)
            .map(|content| keel::dsl::serialize(content.content())),
    )
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

        let result = super::get_board(&events, &std::sync::Mutex::new(Vec::new()), board);

        assert_eq!(
            result,
            Ok(Some(format!(
                "# eventstorming v1\n\nevent     e1   \"{text}\" @ 0,0\n"
            )))
        );
    }

    #[test]
    fn when_get_board_with_id_of_edited_board_then_returns_its_content_with_the_edits_applied() {
        let mut bytes = [0u8; 16];
        std::io::Read::read_exact(
            &mut std::fs::File::open("/dev/urandom").unwrap(),
            &mut bytes,
        )
        .unwrap();
        let board = uuid::Builder::from_random_bytes(bytes).into_uuid();
        let created = vec![super::super::board_created::BoardCreated {
            board,
            content: keel::dsl::parse("event e1 \"\" @ 0,0\n").board,
        }];
        let (event, _) = super::super::board_edited::BoardEdited::derive(
            &super::super::board_content::BoardContent::of(&created, &[], board).unwrap(),
            crate::non_empty::NonEmpty::parse(vec![super::super::operation::Operation::MoveNotes(
                crate::non_empty::NonEmpty::parse(vec![super::super::operation::Position {
                    id: keel::note_id::NoteId::parse("e1").unwrap(),
                    x: 16.0,
                    y: 32.0,
                }])
                .unwrap(),
            )])
            .unwrap(),
        )
        .unwrap();
        let events = std::sync::Mutex::new(created);
        let edited = std::sync::Mutex::new(event.into_iter().collect());

        let result = super::get_board(&events, &edited, board);

        assert_eq!(
            result,
            Ok(Some(
                "# eventstorming v1\n\nevent     e1   \"\" @ 16,32\n".to_string()
            ))
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

        let result = super::get_board(
            &events,
            &std::sync::Mutex::new(Vec::new()),
            uuid::Builder::from_random_bytes(bytes).into_uuid(),
        );

        assert_eq!(result, Ok(None));
    }
}
