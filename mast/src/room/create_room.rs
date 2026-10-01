const MAX_DSL_BYTES: usize = 1 << 20;

pub fn create_room(
    board_events: &std::sync::Mutex<Vec<crate::board::board_created::BoardCreated>>,
    room_events: &std::sync::Mutex<Vec<super::room_created::RoomCreated>>,
    data: &str,
) -> Result<Result<uuid::Uuid, Option<Vec<keel::dsl::Diagnostic>>>, String> {
    match crate::base64url::decode(data)
        .and_then(|bytes| crate::deflate::inflate(&bytes, MAX_DSL_BYTES))
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .ok_or(None)
        .and_then(|dsl| crate::board::dsl::board_from_dsl(&dsl).map_err(Some))
    {
        Ok(content) => {
            let board = crate::uuid_v4::generate()?;
            board_events
                .lock()
                .map_err(|e| e.to_string())?
                .push(crate::board::board_created::BoardCreated { board, content });
            let room = crate::uuid_v4::generate()?;
            room_events
                .lock()
                .map_err(|e| e.to_string())?
                .push(super::room_created::RoomCreated { room, board });
            Ok(Ok(room))
        }
        Err(diagnostics) => Ok(Err(diagnostics)),
    }
}

#[cfg(all(test, feature = "medium"))]
mod medium_tests {
    #[test]
    fn when_create_room_with_data_of_board_then_returns_id_of_room_associated_with_the_board() {
        let board_events = std::sync::Mutex::new(Vec::new());
        let room_events = std::sync::Mutex::new(Vec::new());

        let result = super::create_room(
            &board_events,
            &room_events,
            "HYxLCoQwEAX3fYqHbnWRVsSVCHOA8Qoh9oiQD2Sic_3p-BZvU0W1kFti-ZaUwxkP3IbKWbzgnXfJ-Pj0I7JOMeqs0Wtel_pBcoMV49zxwORSCDbuSt2jbN46QaqRarGZOuZRUwb9og79AQ",
        );

        let Ok(Ok(id)) = result else {
            panic!("expected a created room, got {result:?}");
        };
        let boards_created = board_events.lock().unwrap();
        let [crate::board::board_created::BoardCreated { board, content }] =
            boards_created.as_slice()
        else {
            panic!("expected a created board, got {boards_created:?}");
        };
        assert!(matches!(
            content.notes(),
            [actor, command] if actor.text() == "Customer" && command.text() == "Place order"
        ));
        assert_eq!(
            room_events.lock().unwrap().as_slice(),
            [super::super::room_created::RoomCreated {
                room: id,
                board: *board,
            }]
        );
    }

    #[test]
    fn when_create_room_twice_then_returns_different_ids() {
        let board_events = std::sync::Mutex::new(Vec::new());
        let room_events = std::sync::Mutex::new(Vec::new());

        let first = super::create_room(
            &board_events,
            &room_events,
            "HYxLCoQwEAX3fYqHbnWRVsSVCHOA8Qoh9oiQD2Sic_3p-BZvU0W1kFti-ZaUwxkP3IbKWbzgnXfJ-Pj0I7JOMeqs0Wtel_pBcoMV49zxwORSCDbuSt2jbN46QaqRarGZOuZRUwb9og79AQ",
        );
        let second = super::create_room(
            &board_events,
            &room_events,
            "HYxLCoQwEAX3fYqHbnWRVsSVCHOA8Qoh9oiQD2Sic_3p-BZvU0W1kFti-ZaUwxkP3IbKWbzgnXfJ-Pj0I7JOMeqs0Wtel_pBcoMV49zxwORSCDbuSt2jbN46QaqRarGZOuZRUwb9og79AQ",
        );

        let (Ok(Ok(first)), Ok(Ok(second))) = (first, second) else {
            panic!("expected two created rooms");
        };
        assert_ne!(first, second);
        assert_eq!(board_events.lock().unwrap().len(), 2);
        assert_eq!(room_events.lock().unwrap().len(), 2);
    }

    #[test]
    fn when_create_room_with_undecodable_data_then_returns_none() {
        let board_events = std::sync::Mutex::new(Vec::new());
        let room_events = std::sync::Mutex::new(Vec::new());
        let data = random_string::generate_random_string(
            16,
            &[random_string::CharacterType::Lowercase],
            "",
            &mut std::fs::File::open("/dev/urandom").unwrap(),
        ) + "+";

        let result = super::create_room(&board_events, &room_events, &data);

        assert_eq!(result, Ok(Err(None)));
        assert!(board_events.lock().unwrap().is_empty());
        assert!(room_events.lock().unwrap().is_empty());
    }

    #[test]
    fn when_create_room_with_unexpressible_line_then_returns_its_diagnostics() {
        let board_events = std::sync::Mutex::new(Vec::new());
        let room_events = std::sync::Mutex::new(Vec::new());

        let result =
            super::create_room(&board_events, &room_events, "Ky7JTM6uVCg2VFBSUnBQMNAx4AIA");

        assert_eq!(
            result,
            Ok(Err(Some(vec![keel::dsl::Diagnostic {
                line: 1,
                kind: keel::dsl::DiagnosticKind::UnknownNoteType("sticky".to_string()),
            }])))
        );
        assert!(board_events.lock().unwrap().is_empty());
        assert!(room_events.lock().unwrap().is_empty());
    }
}
