const MAX_DSL_BYTES: usize = 1 << 20;

#[derive(Debug, PartialEq)]
pub enum CreateRoomError {
    InvalidData(Vec<keel::dsl::Diagnostic>),
    Internal(String),
}

pub fn create_room(
    board_events: &std::sync::Mutex<Vec<crate::board::board_created::BoardCreated>>,
    room_events: &std::sync::Mutex<Vec<super::room_created::RoomCreated>>,
    data: &str,
) -> Result<(uuid::Uuid, String), CreateRoomError> {
    let dsl = crate::base64url::decode(data)
        .and_then(|bytes| crate::deflate::inflate(&bytes, MAX_DSL_BYTES))
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .ok_or(CreateRoomError::InvalidData(Vec::new()))?;
    let content = crate::board::dsl::board_from_dsl(&dsl).map_err(CreateRoomError::InvalidData)?;
    let board = crate::uuid_v4::generate().map_err(CreateRoomError::Internal)?;
    board_events
        .lock()
        .map_err(|e| CreateRoomError::Internal(e.to_string()))?
        .push(crate::board::board_created::BoardCreated { board, content });
    let room = crate::uuid_v4::generate().map_err(CreateRoomError::Internal)?;
    room_events
        .lock()
        .map_err(|e| CreateRoomError::Internal(e.to_string()))?
        .push(super::room_created::RoomCreated { room, board });
    Ok((room, format!("/?room={room}")))
}

#[cfg(all(test, feature = "medium"))]
mod medium_tests {
    #[test]
    fn when_create_room_with_data_of_board_then_returns_id_and_share_url_of_room_associated_with_the_board()
     {
        let board_events = std::sync::Mutex::new(Vec::new());
        let room_events = std::sync::Mutex::new(Vec::new());

        let result = super::create_room(
            &board_events,
            &room_events,
            "HYxLCoQwEAX3fYqHbnWRVsSVCHOA8Qoh9oiQD2Sic_3p-BZvU0W1kFti-ZaUwxkP3IbKWbzgnXfJ-Pj0I7JOMeqs0Wtel_pBcoMV49zxwORSCDbuSt2jbN46QaqRarGZOuZRUwb9og79AQ",
        );

        let Ok((id, share_url)) = result else {
            panic!("expected a created room, got {result:?}");
        };
        assert_eq!(
            share_url,
            "/?room=".to_string() + &id.hyphenated().to_string()
        );
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

        let (Ok((first, _)), Ok((second, _))) = (first, second) else {
            panic!("expected two created rooms");
        };
        assert_ne!(first, second);
        assert_eq!(board_events.lock().unwrap().len(), 2);
        assert_eq!(room_events.lock().unwrap().len(), 2);
    }

    #[test]
    fn when_create_room_with_undecodable_data_then_returns_invalid_data_without_diagnostics() {
        let board_events = std::sync::Mutex::new(Vec::new());
        let room_events = std::sync::Mutex::new(Vec::new());
        let data = random_string::generate_random_string(
            16,
            &[random_string::CharacterType::Lowercase],
            "",
            &mut std::fs::File::open("/dev/urandom").unwrap(),
        ) + "+";

        let result = super::create_room(&board_events, &room_events, &data);

        assert_eq!(result, Err(super::CreateRoomError::InvalidData(Vec::new())));
        assert!(board_events.lock().unwrap().is_empty());
        assert!(room_events.lock().unwrap().is_empty());
    }

    #[test]
    fn when_create_room_with_unexpressible_line_then_returns_invalid_data_with_its_diagnostics() {
        let board_events = std::sync::Mutex::new(Vec::new());
        let room_events = std::sync::Mutex::new(Vec::new());

        let result =
            super::create_room(&board_events, &room_events, "Ky7JTM6uVCg2VFBSUnBQMNAx4AIA");

        assert_eq!(
            result,
            Err(super::CreateRoomError::InvalidData(vec![
                keel::dsl::Diagnostic {
                    line: 1,
                    kind: keel::dsl::DiagnosticKind::UnknownNoteType("sticky".to_string()),
                }
            ]))
        );
        assert!(board_events.lock().unwrap().is_empty());
        assert!(room_events.lock().unwrap().is_empty());
    }
}
