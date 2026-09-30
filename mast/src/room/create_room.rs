const MAX_DSL_BYTES: usize = 1 << 20;

pub fn create_room(
    events: &super::room_events::RoomEvents,
    data: &str,
) -> Result<Result<uuid::Uuid, Option<Vec<keel::dsl::Diagnostic>>>, String> {
    match crate::base64url::decode(data)
        .and_then(|bytes| crate::deflate::inflate(&bytes, MAX_DSL_BYTES))
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .ok_or(None)
        .and_then(|dsl| super::board::board_from_dsl(&dsl).map_err(Some))
    {
        Ok(board) => {
            let room = super::room_events::new_room_id()?;
            events.append(super::event::Event::Created { room, board })?;
            Ok(Ok(room))
        }
        Err(diagnostics) => Ok(Err(diagnostics)),
    }
}

#[cfg(all(test, feature = "medium"))]
mod medium_tests {
    #[test]
    fn when_create_room_with_data_of_board_then_returns_id_of_the_created_room() {
        let events = super::super::room_events::RoomEvents::default();

        let result = super::create_room(
            &events,
            "HYxLCoQwEAX3fYqHbnWRVsSVCHOA8Qoh9oiQD2Sic_3p-BZvU0W1kFti-ZaUwxkP3IbKWbzgnXfJ-Pj0I7JOMeqs0Wtel_pBcoMV49zxwORSCDbuSt2jbN46QaqRarGZOuZRUwb9og79AQ",
        );

        let Ok(Ok(id)) = result else {
            panic!("expected a created room, got {result:?}");
        };
        assert!(matches!(
            events.log.lock().unwrap().as_slice(),
            [super::super::event::Event::Created { room, board }] if *room == id && matches!(
                board.notes(),
                [actor, command] if actor.text() == "Customer" && command.text() == "Place order"
            )
        ));
    }

    #[test]
    fn when_create_room_twice_then_returns_different_ids() {
        let events = super::super::room_events::RoomEvents::default();

        let first = super::create_room(
            &events,
            "HYxLCoQwEAX3fYqHbnWRVsSVCHOA8Qoh9oiQD2Sic_3p-BZvU0W1kFti-ZaUwxkP3IbKWbzgnXfJ-Pj0I7JOMeqs0Wtel_pBcoMV49zxwORSCDbuSt2jbN46QaqRarGZOuZRUwb9og79AQ",
        );
        let second = super::create_room(
            &events,
            "HYxLCoQwEAX3fYqHbnWRVsSVCHOA8Qoh9oiQD2Sic_3p-BZvU0W1kFti-ZaUwxkP3IbKWbzgnXfJ-Pj0I7JOMeqs0Wtel_pBcoMV49zxwORSCDbuSt2jbN46QaqRarGZOuZRUwb9og79AQ",
        );

        let (Ok(Ok(first)), Ok(Ok(second))) = (first, second) else {
            panic!("expected two created rooms");
        };
        assert_ne!(first, second);
        assert_eq!(events.log.lock().unwrap().len(), 2);
    }

    #[test]
    fn when_create_room_with_undecodable_data_then_returns_none() {
        let events = super::super::room_events::RoomEvents::default();
        let data = random_string::generate_random_string(
            16,
            &[random_string::CharacterType::Lowercase],
            "",
            &mut std::fs::File::open("/dev/urandom").unwrap(),
        ) + "+";

        let result = super::create_room(&events, &data);

        assert_eq!(result, Ok(Err(None)));
        assert!(events.log.lock().unwrap().is_empty());
    }

    #[test]
    fn when_create_room_with_unexpressible_line_then_returns_its_diagnostics() {
        let events = super::super::room_events::RoomEvents::default();

        let result = super::create_room(&events, "Ky7JTM6uVCg2VFBSUnBQMNAx4AIA");

        assert_eq!(
            result,
            Ok(Err(Some(vec![keel::dsl::Diagnostic {
                line: 1,
                kind: keel::dsl::DiagnosticKind::UnknownNoteType("sticky".to_string()),
            }])))
        );
        assert!(events.log.lock().unwrap().is_empty());
    }
}
