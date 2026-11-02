#[derive(Debug, PartialEq)]
pub struct Snapshot {
    pub seq: crate::board::board_edited::Seq,
    pub dsl: String,
}

pub fn get_room_events(
    room_events: &std::sync::Mutex<Vec<super::room_created::RoomCreated>>,
    board_events: &std::sync::Mutex<Vec<crate::board::board_created::BoardCreated>>,
    board_edited_events: &std::sync::Mutex<Vec<crate::board::board_edited::BoardEdited>>,
    board_edited_senders: &std::sync::Mutex<
        std::collections::HashMap<
            uuid::Uuid,
            tokio::sync::broadcast::Sender<crate::board::board_edited::BoardEdited>,
        >,
    >,
    room: uuid::Uuid,
) -> Result<
    Option<(
        Snapshot,
        impl futures_util::Stream<Item = crate::board::board_edited::BoardEdited>
        + Send
        + Sync
        + 'static,
    )>,
    String,
> {
    let Some(board) = room_events
        .lock()
        .map_err(|e| e.to_string())
        .map(|events| super::room_created::boards_of(&events, room))?
        .and_then(|boards| boards.first().copied())
    else {
        return Ok(None);
    };
    let receiver = board_edited_senders
        .lock()
        .map_err(|e| e.to_string())?
        .entry(board)
        .or_insert_with(|| tokio::sync::broadcast::channel(256).0)
        .subscribe();
    let created: Vec<_> = board_events
        .lock()
        .map_err(|e| e.to_string())?
        .iter()
        .filter(|c| c.board == board)
        .cloned()
        .collect();
    let edited: Vec<_> = board_edited_events
        .lock()
        .map_err(|e| e.to_string())?
        .iter()
        .filter(|e| e.board() == board)
        .cloned()
        .collect();
    let content = crate::board::board_content::BoardContent::of(&created, &edited, board)
        .ok_or_else(|| format!("board {board} is not created"))?;
    let seq = content.seq();
    Ok(Some((
        Snapshot {
            seq,
            dsl: keel::dsl::serialize(content.content()),
        },
        futures_util::stream::unfold(receiver, move |mut receiver| async move {
            loop {
                match receiver.recv().await {
                    Ok(edited) if edited.seq() > seq => return Some((edited, receiver)),
                    Ok(_) => continue,
                    Err(_) => return None,
                }
            }
        }),
    )))
}

#[cfg(all(test, feature = "medium"))]
mod medium_tests {
    #[test]
    fn when_get_room_events_with_unknown_room_then_returns_none() {
        let mut bytes = [0u8; 48];
        std::io::Read::read_exact(
            &mut std::fs::File::open("/dev/urandom").unwrap(),
            &mut bytes,
        )
        .unwrap();
        let room = uuid::Builder::from_random_bytes(bytes[..16].try_into().unwrap()).into_uuid();
        let board = uuid::Builder::from_random_bytes(bytes[16..32].try_into().unwrap()).into_uuid();
        let other_room =
            uuid::Builder::from_random_bytes(bytes[32..].try_into().unwrap()).into_uuid();
        let senders = std::sync::Mutex::default();

        let result = super::get_room_events(
            &std::sync::Mutex::new(vec![super::super::room_created::RoomCreated {
                room,
                board,
            }]),
            &std::sync::Mutex::new(vec![crate::board::board_created::BoardCreated {
                board,
                content: keel::dsl::parse("event e1 \"\" @ 0,0\n").board,
            }]),
            &std::sync::Mutex::new(Vec::new()),
            &senders,
            other_room,
        );

        assert!(matches!(result, Ok(None)));
        assert!(senders.lock().unwrap().is_empty());
    }

    #[test]
    fn when_get_room_events_after_edits_then_snapshot_has_the_edits_applied() {
        let mut bytes = [0u8; 32];
        std::io::Read::read_exact(
            &mut std::fs::File::open("/dev/urandom").unwrap(),
            &mut bytes,
        )
        .unwrap();
        let room = uuid::Builder::from_random_bytes(bytes[..16].try_into().unwrap()).into_uuid();
        let board = uuid::Builder::from_random_bytes(bytes[16..].try_into().unwrap()).into_uuid();
        let room_events = std::sync::Mutex::new(vec![super::super::room_created::RoomCreated {
            room,
            board,
        }]);
        let board_events = std::sync::Mutex::new(vec![crate::board::board_created::BoardCreated {
            board,
            content: keel::dsl::parse("event e1 \"\" @ 0,0\n").board,
        }]);
        let board_edited_events = std::sync::Mutex::new(Vec::new());
        let senders = std::sync::Mutex::default();
        super::super::post_room_ops::post_room_ops(
            &room_events,
            &board_events,
            &board_edited_events,
            &senders,
            room,
            crate::non_empty::NonEmpty::parse(vec![crate::board::operation::Operation::MoveNotes(
                crate::non_empty::NonEmpty::parse(vec![crate::board::operation::Position {
                    id: keel::note_id::NoteId::parse("e1").unwrap(),
                    x: 8.0,
                    y: 0.0,
                }])
                .unwrap(),
            )])
            .unwrap(),
        )
        .unwrap();

        let (snapshot, _) = super::get_room_events(
            &room_events,
            &board_events,
            &board_edited_events,
            &senders,
            room,
        )
        .unwrap()
        .unwrap();

        assert_eq!(
            (snapshot.seq.value(), snapshot.dsl.as_str()),
            (1, "# eventstorming v1\n\nevent     e1   \"\" @ 8,0\n")
        );
    }

    #[tokio::test]
    async fn when_ops_are_posted_after_get_room_events_then_the_stream_yields_them_in_order() {
        let mut bytes = [0u8; 32];
        std::io::Read::read_exact(
            &mut std::fs::File::open("/dev/urandom").unwrap(),
            &mut bytes,
        )
        .unwrap();
        let room = uuid::Builder::from_random_bytes(bytes[..16].try_into().unwrap()).into_uuid();
        let board = uuid::Builder::from_random_bytes(bytes[16..].try_into().unwrap()).into_uuid();
        let room_events = std::sync::Mutex::new(vec![super::super::room_created::RoomCreated {
            room,
            board,
        }]);
        let board_events = std::sync::Mutex::new(vec![crate::board::board_created::BoardCreated {
            board,
            content: keel::dsl::parse("event e1 \"\" @ 0,0\n").board,
        }]);
        let board_edited_events = std::sync::Mutex::new(Vec::new());
        let senders = std::sync::Mutex::default();
        let (_, stream) = super::get_room_events(
            &room_events,
            &board_events,
            &board_edited_events,
            &senders,
            room,
        )
        .unwrap()
        .unwrap();
        let mut stream = std::pin::pin!(stream);

        for x in [1.0, 2.0] {
            super::super::post_room_ops::post_room_ops(
                &room_events,
                &board_events,
                &board_edited_events,
                &senders,
                room,
                crate::non_empty::NonEmpty::parse(vec![
                    crate::board::operation::Operation::MoveNotes(
                        crate::non_empty::NonEmpty::parse(vec![
                            crate::board::operation::Position {
                                id: keel::note_id::NoteId::parse("e1").unwrap(),
                                x,
                                y: 0.0,
                            },
                        ])
                        .unwrap(),
                    ),
                ])
                .unwrap(),
            )
            .unwrap();
        }

        let first = futures_util::StreamExt::next(&mut stream).await.unwrap();
        let second = futures_util::StreamExt::next(&mut stream).await.unwrap();
        assert_eq!((first.seq().value(), second.seq().value()), (1, 2));
    }

    #[tokio::test]
    async fn when_ops_are_posted_to_other_room_then_the_stream_does_not_yield_them() {
        let mut bytes = [0u8; 64];
        std::io::Read::read_exact(
            &mut std::fs::File::open("/dev/urandom").unwrap(),
            &mut bytes,
        )
        .unwrap();
        let room = uuid::Builder::from_random_bytes(bytes[..16].try_into().unwrap()).into_uuid();
        let board = uuid::Builder::from_random_bytes(bytes[16..32].try_into().unwrap()).into_uuid();
        let other_room =
            uuid::Builder::from_random_bytes(bytes[32..48].try_into().unwrap()).into_uuid();
        let other_board =
            uuid::Builder::from_random_bytes(bytes[48..].try_into().unwrap()).into_uuid();
        let room_events = std::sync::Mutex::new(vec![
            super::super::room_created::RoomCreated { room, board },
            super::super::room_created::RoomCreated {
                room: other_room,
                board: other_board,
            },
        ]);
        let board_events = std::sync::Mutex::new(vec![
            crate::board::board_created::BoardCreated {
                board,
                content: keel::dsl::parse("event e1 \"\" @ 0,0\n").board,
            },
            crate::board::board_created::BoardCreated {
                board: other_board,
                content: keel::dsl::parse("event e1 \"\" @ 0,0\n").board,
            },
        ]);
        let board_edited_events = std::sync::Mutex::new(Vec::new());
        let senders = std::sync::Mutex::default();
        let (_, stream) = super::get_room_events(
            &room_events,
            &board_events,
            &board_edited_events,
            &senders,
            room,
        )
        .unwrap()
        .unwrap();
        let mut stream = std::pin::pin!(stream);

        for (room, x) in [(other_room, 1.0), (room, 2.0)] {
            super::super::post_room_ops::post_room_ops(
                &room_events,
                &board_events,
                &board_edited_events,
                &senders,
                room,
                crate::non_empty::NonEmpty::parse(vec![
                    crate::board::operation::Operation::MoveNotes(
                        crate::non_empty::NonEmpty::parse(vec![
                            crate::board::operation::Position {
                                id: keel::note_id::NoteId::parse("e1").unwrap(),
                                x,
                                y: 0.0,
                            },
                        ])
                        .unwrap(),
                    ),
                ])
                .unwrap(),
            )
            .unwrap();
        }

        let next = futures_util::StreamExt::next(&mut stream).await.unwrap();
        assert_eq!((next.board(), next.seq().value()), (board, 1));
    }
}
