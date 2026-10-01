pub fn get_room_boards(
    events: &std::sync::Mutex<Vec<super::room_created::RoomCreated>>,
    room: uuid::Uuid,
) -> Result<Option<Vec<uuid::Uuid>>, String> {
    events
        .lock()
        .map(|events| super::room_created::boards_of(&events, room))
        .map_err(|e| e.to_string())
}

#[cfg(all(test, feature = "medium"))]
mod medium_tests {
    #[test]
    fn when_get_room_boards_with_id_of_created_room_then_returns_its_board() {
        let mut bytes = [0u8; 32];
        std::io::Read::read_exact(
            &mut std::fs::File::open("/dev/urandom").unwrap(),
            &mut bytes,
        )
        .unwrap();
        let room = uuid::Builder::from_random_bytes(bytes[..16].try_into().unwrap()).into_uuid();
        let board = uuid::Builder::from_random_bytes(bytes[16..].try_into().unwrap()).into_uuid();
        let events = std::sync::Mutex::new(vec![super::super::room_created::RoomCreated {
            room,
            board,
        }]);

        let result = super::get_room_boards(&events, room);

        assert_eq!(result, Ok(Some(vec![board])));
    }

    #[test]
    fn when_get_room_boards_with_id_of_no_room_then_returns_none() {
        let mut bytes = [0u8; 16];
        std::io::Read::read_exact(
            &mut std::fs::File::open("/dev/urandom").unwrap(),
            &mut bytes,
        )
        .unwrap();
        let events = std::sync::Mutex::new(Vec::new());

        let result =
            super::get_room_boards(&events, uuid::Builder::from_random_bytes(bytes).into_uuid());

        assert_eq!(result, Ok(None));
    }
}
