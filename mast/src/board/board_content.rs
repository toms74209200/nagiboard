#[derive(Debug, Clone, PartialEq)]
pub struct BoardContent {
    board: uuid::Uuid,
    seq: super::board_edited::Seq,
    content: keel::board::Board,
}

impl BoardContent {
    pub fn of(
        created: &[super::board_created::BoardCreated],
        edited: &[super::board_edited::BoardEdited],
        board: uuid::Uuid,
    ) -> Option<Self> {
        super::board_created::content_of(created, board).map(|content| {
            edited.iter().filter(|e| e.board() == board).fold(
                Self {
                    board,
                    seq: super::board_edited::Seq::INITIAL,
                    content,
                },
                |mut current, e| {
                    for edit in e.edits().iter() {
                        edit.apply(&mut current.content);
                    }
                    current.seq = e.seq();
                    current
                },
            )
        })
    }

    pub fn board(&self) -> uuid::Uuid {
        self.board
    }

    pub fn seq(&self) -> super::board_edited::Seq {
        self.seq
    }

    pub fn content(&self) -> &keel::board::Board {
        &self.content
    }
}

#[cfg(test)]
mod tests {
    fn random_board() -> uuid::Uuid {
        let mut bytes = [0u8; 16];
        std::io::Read::read_exact(
            &mut std::fs::File::open("/dev/urandom").unwrap(),
            &mut bytes,
        )
        .unwrap();
        uuid::Builder::from_random_bytes(bytes).into_uuid()
    }

    fn move_e1(x: f64) -> crate::non_empty::NonEmpty<super::super::operation::Operation> {
        crate::non_empty::NonEmpty::parse(vec![super::super::operation::Operation::MoveNotes(
            crate::non_empty::NonEmpty::parse(vec![super::super::operation::Position {
                id: keel::note_id::NoteId::parse("e1").unwrap(),
                x,
                y: 0.0,
            }])
            .unwrap(),
        )])
        .unwrap()
    }

    #[test]
    fn when_of_with_edits_of_the_board_then_returns_created_content_with_them_applied_and_the_last_seq()
     {
        let board = random_board();
        let other = random_board();
        let created = [
            super::super::board_created::BoardCreated {
                board,
                content: keel::dsl::parse("event e1 \"\" @ 0,0\n").board,
            },
            super::super::board_created::BoardCreated {
                board: other,
                content: keel::dsl::parse("event e1 \"\" @ 0,0\n").board,
            },
        ];
        let mut edited = Vec::new();
        for (target, x) in [(board, 1.0), (other, 3.0), (board, 2.0)] {
            let current = super::BoardContent::of(&created, &edited, target).unwrap();
            let (event, _) =
                super::super::board_edited::BoardEdited::derive(&current, move_e1(x)).unwrap();
            edited.extend(event);
        }

        let content = super::BoardContent::of(&created, &edited, board);

        assert!(matches!(
            content,
            Some(c) if c.seq().value() == 2 && matches!(c.content().notes(), [n] if n.x() == 2.0)
        ));
    }

    #[test]
    fn when_of_without_edits_then_returns_created_content_with_initial_seq() {
        let board = random_board();
        let created = [super::super::board_created::BoardCreated {
            board,
            content: keel::dsl::parse("event e1 \"\" @ 0,0\n").board,
        }];

        let content = super::BoardContent::of(&created, &[], board);

        assert_eq!(
            content.map(|c| (c.seq(), c.content().clone())),
            Some((
                super::super::board_edited::Seq::INITIAL,
                created[0].content.clone()
            ))
        );
    }

    #[test]
    fn when_of_without_created_event_of_the_board_then_returns_none() {
        assert_eq!(super::BoardContent::of(&[], &[], random_board()), None);
    }
}
