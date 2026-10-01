#[derive(Debug, Clone, PartialEq)]
pub struct BoardCreated {
    pub board: uuid::Uuid,
    pub content: keel::board::Board,
}

pub fn content_of(events: &[BoardCreated], board: uuid::Uuid) -> Option<keel::board::Board> {
    events
        .iter()
        .find(|created| created.board == board)
        .map(|created| created.content.clone())
}

#[cfg(test)]
mod tests {
    #[test]
    fn when_content_of_with_created_event_of_the_board_then_returns_its_content() {
        let mut bytes = [0u8; 32];
        std::io::Read::read_exact(
            &mut std::fs::File::open("/dev/urandom").unwrap(),
            &mut bytes,
        )
        .unwrap();
        let board = uuid::Builder::from_random_bytes(bytes[..16].try_into().unwrap()).into_uuid();
        let other = uuid::Builder::from_random_bytes(bytes[16..].try_into().unwrap()).into_uuid();
        let text = random_string::generate_random_string(
            16,
            &[random_string::CharacterType::Lowercase],
            "",
            &mut std::fs::File::open("/dev/urandom").unwrap(),
        );
        let content = keel::dsl::parse(&format!("event e1 \"{text}\" @ 0,0\n")).board;

        assert_eq!(
            super::content_of(
                &[
                    super::BoardCreated {
                        board: other,
                        content: keel::board::Board::default(),
                    },
                    super::BoardCreated {
                        board,
                        content: content.clone(),
                    },
                ],
                board,
            ),
            Some(content)
        );
    }

    #[test]
    fn when_content_of_without_created_event_of_the_board_then_returns_none() {
        let mut bytes = [0u8; 32];
        std::io::Read::read_exact(
            &mut std::fs::File::open("/dev/urandom").unwrap(),
            &mut bytes,
        )
        .unwrap();
        let board = uuid::Builder::from_random_bytes(bytes[..16].try_into().unwrap()).into_uuid();
        let other = uuid::Builder::from_random_bytes(bytes[16..].try_into().unwrap()).into_uuid();

        assert_eq!(
            super::content_of(
                &[super::BoardCreated {
                    board: other,
                    content: keel::board::Board::default(),
                }],
                board,
            ),
            None
        );
    }
}
