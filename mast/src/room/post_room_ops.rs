#[derive(Debug, PartialEq)]
pub enum PostRoomOpsError {
    RoomNotFound,
    Internal(String),
}

pub fn post_room_ops(
    room_events: &std::sync::Mutex<Vec<super::room_created::RoomCreated>>,
    board_events: &std::sync::Mutex<Vec<crate::board::board_created::BoardCreated>>,
    board_edited_events: &std::sync::Mutex<Vec<crate::board::board_edited::BoardEdited>>,
    room: uuid::Uuid,
    operations: crate::non_empty::NonEmpty<crate::board::operation::Operation>,
) -> Result<
    (
        crate::board::board_edited::Seq,
        Vec<crate::board::operation::Outcome>,
    ),
    PostRoomOpsError,
> {
    let board = room_events
        .lock()
        .map_err(|e| PostRoomOpsError::Internal(e.to_string()))
        .map(|events| super::room_created::boards_of(&events, room))?
        .and_then(|boards| boards.first().copied())
        .ok_or(PostRoomOpsError::RoomNotFound)?;
    let created = board_events
        .lock()
        .map_err(|e| PostRoomOpsError::Internal(e.to_string()))?;
    let mut edited = board_edited_events
        .lock()
        .map_err(|e| PostRoomOpsError::Internal(e.to_string()))?;
    let content = crate::board::board_content::BoardContent::of(&created, &edited, board)
        .ok_or_else(|| PostRoomOpsError::Internal(format!("board {board} is not created")))?;
    let (event, outcomes) =
        crate::board::board_edited::BoardEdited::derive(&content, operations)
            .map_err(|_| PostRoomOpsError::Internal(format!("board {board} has run out of seq")))?;
    let seq = event.as_ref().map_or(content.seq(), |e| e.seq());
    edited.extend(event);
    Ok((seq, outcomes))
}

#[cfg(all(test, feature = "medium"))]
mod medium_tests {
    fn created_room() -> (
        std::sync::Mutex<Vec<super::super::room_created::RoomCreated>>,
        std::sync::Mutex<Vec<crate::board::board_created::BoardCreated>>,
        uuid::Uuid,
        uuid::Uuid,
    ) {
        let mut bytes = [0u8; 32];
        std::io::Read::read_exact(
            &mut std::fs::File::open("/dev/urandom").unwrap(),
            &mut bytes,
        )
        .unwrap();
        let room = uuid::Builder::from_random_bytes(bytes[..16].try_into().unwrap()).into_uuid();
        let board = uuid::Builder::from_random_bytes(bytes[16..].try_into().unwrap()).into_uuid();
        (
            std::sync::Mutex::new(vec![super::super::room_created::RoomCreated {
                room,
                board,
            }]),
            std::sync::Mutex::new(vec![crate::board::board_created::BoardCreated {
                board,
                content: keel::dsl::parse("event e1 \"\" @ 0,0\n").board,
            }]),
            room,
            board,
        )
    }

    fn add_note(id: &str) -> crate::board::operation::Operation {
        crate::board::operation::Operation::AddNote {
            id: keel::note_id::NoteId::parse(id).unwrap(),
            note_type: keel::note_type::NoteType::Event,
            text: String::new(),
            x: 0.0,
            y: 0.0,
        }
    }

    fn move_note(id: &str) -> crate::board::operation::Operation {
        crate::board::operation::Operation::MoveNotes(
            crate::non_empty::NonEmpty::parse(vec![crate::board::operation::Position {
                id: keel::note_id::NoteId::parse(id).unwrap(),
                x: 0.0,
                y: 0.0,
            }])
            .unwrap(),
        )
    }

    fn ops(
        operations: Vec<crate::board::operation::Operation>,
    ) -> crate::non_empty::NonEmpty<crate::board::operation::Operation> {
        crate::non_empty::NonEmpty::parse(operations).unwrap()
    }

    #[test]
    fn when_post_room_ops_to_created_room_then_records_the_edit_and_returns_the_outcomes() {
        let (room_events, board_events, room, board) = created_room();
        let board_edited_events = std::sync::Mutex::new(Vec::new());
        let id = random_string::generate_random_string(
            8,
            &[random_string::CharacterType::Lowercase],
            "",
            &mut std::fs::File::open("/dev/urandom").unwrap(),
        );

        let result = super::post_room_ops(
            &room_events,
            &board_events,
            &board_edited_events,
            room,
            ops(vec![add_note(&id), add_note("e1")]),
        );

        assert_eq!(
            result.map(|(seq, outcomes)| (seq.value(), outcomes)),
            Ok((
                1,
                vec![
                    crate::board::operation::Outcome::Applied,
                    crate::board::operation::Outcome::Reassigned(
                        keel::note_id::NoteId::parse("e2").unwrap()
                    ),
                ]
            ))
        );
        let content = crate::board::board_content::BoardContent::of(
            &board_events.lock().unwrap(),
            &board_edited_events.lock().unwrap(),
            board,
        )
        .unwrap();
        assert_eq!(
            content
                .content()
                .notes()
                .iter()
                .map(|n| n.id().as_str())
                .collect::<Vec<_>>(),
            ["e1", id.as_str(), "e2"]
        );
    }

    #[test]
    fn when_post_room_ops_twice_then_gives_the_second_edit_the_next_seq_on_the_first() {
        let (room_events, board_events, room, _) = created_room();
        let board_edited_events = std::sync::Mutex::new(Vec::new());

        let first = super::post_room_ops(
            &room_events,
            &board_events,
            &board_edited_events,
            room,
            ops(vec![add_note("e1")]),
        );
        let second = super::post_room_ops(
            &room_events,
            &board_events,
            &board_edited_events,
            room,
            ops(vec![add_note("e1")]),
        );

        assert_eq!(
            first.map(|(seq, outcomes)| (seq.value(), outcomes)),
            Ok((
                1,
                vec![crate::board::operation::Outcome::Reassigned(
                    keel::note_id::NoteId::parse("e2").unwrap()
                )]
            ))
        );
        assert_eq!(
            second.map(|(seq, outcomes)| (seq.value(), outcomes)),
            Ok((
                2,
                vec![crate::board::operation::Outcome::Reassigned(
                    keel::note_id::NoteId::parse("e3").unwrap()
                )]
            ))
        );
    }

    #[test]
    fn when_post_room_ops_with_only_impossible_operations_then_records_nothing_and_keeps_the_seq() {
        let (room_events, board_events, room, _) = created_room();
        let board_edited_events = std::sync::Mutex::new(Vec::new());
        let id = random_string::generate_random_string(
            8,
            &[random_string::CharacterType::Lowercase],
            "",
            &mut std::fs::File::open("/dev/urandom").unwrap(),
        );

        let result = super::post_room_ops(
            &room_events,
            &board_events,
            &board_edited_events,
            room,
            ops(vec![move_note(&id)]),
        );

        assert_eq!(
            result.map(|(seq, outcomes)| (seq.value(), outcomes)),
            Ok((0, vec![crate::board::operation::Outcome::Skipped]))
        );
        assert!(board_edited_events.lock().unwrap().is_empty());
    }

    #[test]
    fn when_post_room_ops_to_no_room_then_returns_room_not_found() {
        let (room_events, board_events, _, _) = created_room();
        let board_edited_events = std::sync::Mutex::new(Vec::new());
        let mut bytes = [0u8; 16];
        std::io::Read::read_exact(
            &mut std::fs::File::open("/dev/urandom").unwrap(),
            &mut bytes,
        )
        .unwrap();

        let result = super::post_room_ops(
            &room_events,
            &board_events,
            &board_edited_events,
            uuid::Builder::from_random_bytes(bytes).into_uuid(),
            ops(vec![add_note("e1")]),
        );

        assert_eq!(result, Err(super::PostRoomOpsError::RoomNotFound));
        assert!(board_edited_events.lock().unwrap().is_empty());
    }
}
