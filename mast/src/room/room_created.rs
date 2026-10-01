#[derive(Debug, Clone, PartialEq)]
pub struct RoomCreated {
    pub room: uuid::Uuid,
    pub board: uuid::Uuid,
}

pub fn boards_of(events: &[RoomCreated], room: uuid::Uuid) -> Option<Vec<uuid::Uuid>> {
    events
        .iter()
        .find(|created| created.room == room)
        .map(|created| vec![created.board])
}

#[cfg(test)]
mod tests {
    #[test]
    fn when_boards_of_with_created_event_of_the_room_then_returns_its_board() {
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

        assert_eq!(
            super::boards_of(
                &[
                    super::RoomCreated {
                        room: other_room,
                        board: other_board,
                    },
                    super::RoomCreated { room, board },
                ],
                room,
            ),
            Some(vec![board])
        );
    }

    #[test]
    fn when_boards_of_without_created_event_of_the_room_then_returns_none() {
        let mut bytes = [0u8; 48];
        std::io::Read::read_exact(
            &mut std::fs::File::open("/dev/urandom").unwrap(),
            &mut bytes,
        )
        .unwrap();
        let room = uuid::Builder::from_random_bytes(bytes[..16].try_into().unwrap()).into_uuid();
        let other_room =
            uuid::Builder::from_random_bytes(bytes[16..32].try_into().unwrap()).into_uuid();
        let board = uuid::Builder::from_random_bytes(bytes[32..].try_into().unwrap()).into_uuid();

        assert_eq!(
            super::boards_of(
                &[super::RoomCreated {
                    room: other_room,
                    board,
                }],
                room,
            ),
            None
        );
    }
}
