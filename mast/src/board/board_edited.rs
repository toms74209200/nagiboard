#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Seq(u32);

impl Seq {
    pub const INITIAL: Seq = Seq(0);

    fn next(self) -> Option<Seq> {
        self.0.checked_add(1).map(Seq)
    }

    pub fn value(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoardEdited {
    board: uuid::Uuid,
    seq: Seq,
    edits: crate::non_empty::NonEmpty<super::operation::Edit>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeqExhausted;

impl BoardEdited {
    pub fn derive(
        content: &super::board_content::BoardContent,
        operations: crate::non_empty::NonEmpty<super::operation::Operation>,
    ) -> Result<(Option<Self>, Vec<super::operation::Outcome>), SeqExhausted> {
        let mut board = content.content().clone();
        let (edits, outcomes): (Vec<_>, Vec<_>) = operations
            .into_iter()
            .map(|operation| {
                let edit = super::operation::Edit::parse(&board, operation);
                if let Some(edit) = &edit {
                    edit.apply(&mut board);
                }
                let outcome = super::operation::Outcome::of(edit.as_ref());
                (edit, outcome)
            })
            .unzip();
        let event = crate::non_empty::NonEmpty::parse(edits.into_iter().flatten().collect())
            .map(|edits| {
                content.seq().next().ok_or(SeqExhausted).map(|seq| Self {
                    board: content.board(),
                    seq,
                    edits,
                })
            })
            .transpose()?;
        Ok((event, outcomes))
    }

    pub fn board(&self) -> uuid::Uuid {
        self.board
    }

    pub fn seq(&self) -> Seq {
        self.seq
    }

    pub fn edits(&self) -> &crate::non_empty::NonEmpty<super::operation::Edit> {
        &self.edits
    }
}

#[cfg(test)]
mod tests {
    fn content() -> super::super::board_content::BoardContent {
        let mut bytes = [0u8; 16];
        std::io::Read::read_exact(
            &mut std::fs::File::open("/dev/urandom").unwrap(),
            &mut bytes,
        )
        .unwrap();
        let board = uuid::Builder::from_random_bytes(bytes).into_uuid();
        super::super::board_content::BoardContent::of(
            &[super::super::board_created::BoardCreated {
                board,
                content: keel::dsl::parse("event e1 \"\" @ 0,0\n").board,
            }],
            &[],
            board,
        )
        .unwrap()
    }

    fn add_note(id: &str) -> super::super::operation::Operation {
        super::super::operation::Operation::AddNote {
            id: keel::note_id::NoteId::parse(id).unwrap(),
            note_type: keel::note_type::NoteType::Event,
            text: String::new(),
            x: 0.0,
            y: 0.0,
        }
    }

    fn move_note(id: &str) -> super::super::operation::Operation {
        super::super::operation::Operation::MoveNotes(
            crate::non_empty::NonEmpty::parse(vec![super::super::operation::Position {
                id: keel::note_id::NoteId::parse(id).unwrap(),
                x: 1.0,
                y: 2.0,
            }])
            .unwrap(),
        )
    }

    #[test]
    fn when_derive_with_possible_operations_then_returns_event_with_next_seq_and_their_outcomes() {
        let content = content();
        let id = random_string::generate_random_string(
            8,
            &[random_string::CharacterType::Lowercase],
            "",
            &mut std::fs::File::open("/dev/urandom").unwrap(),
        );

        let (event, outcomes) = super::BoardEdited::derive(
            &content,
            crate::non_empty::NonEmpty::parse(vec![add_note("e1"), move_note("e2"), add_note(&id)])
                .unwrap(),
        )
        .unwrap();

        assert_eq!(
            outcomes,
            [
                super::super::operation::Outcome::Reassigned(
                    keel::note_id::NoteId::parse("e2").unwrap()
                ),
                super::super::operation::Outcome::Applied,
                super::super::operation::Outcome::Applied,
            ]
        );
        assert!(matches!(
            event,
            Some(e) if e.board() == content.board() && e.seq() == super::Seq(1) && e.edits().iter().count() == 3
        ));
    }

    #[test]
    fn when_derive_with_only_impossible_operations_then_returns_no_event_and_skipped_outcomes() {
        let id = random_string::generate_random_string(
            8,
            &[random_string::CharacterType::Lowercase],
            "",
            &mut std::fs::File::open("/dev/urandom").unwrap(),
        );

        let result = super::BoardEdited::derive(
            &content(),
            crate::non_empty::NonEmpty::parse(vec![move_note(&id)]).unwrap(),
        );

        assert_eq!(
            result,
            Ok((None, vec![super::super::operation::Outcome::Skipped]))
        );
    }

    #[test]
    fn when_derive_on_edited_content_then_gives_the_event_the_seq_after_it() {
        let content = content();
        let (first, _) = super::BoardEdited::derive(
            &content,
            crate::non_empty::NonEmpty::parse(vec![move_note("e1")]).unwrap(),
        )
        .unwrap();
        let edited = super::super::board_content::BoardContent::of(
            &[super::super::board_created::BoardCreated {
                board: content.board(),
                content: keel::dsl::parse("event e1 \"\" @ 0,0\n").board,
            }],
            &first.into_iter().collect::<Vec<_>>(),
            content.board(),
        )
        .unwrap();

        let (second, _) = super::BoardEdited::derive(
            &edited,
            crate::non_empty::NonEmpty::parse(vec![move_note("e1")]).unwrap(),
        )
        .unwrap();

        assert!(matches!(second, Some(e) if e.seq() == super::Seq(2)));
    }
}
