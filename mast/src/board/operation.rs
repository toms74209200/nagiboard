#[derive(Debug, Clone, PartialEq)]
pub enum Operation {
    AddNote {
        id: keel::note_id::NoteId,
        note_type: keel::note_type::NoteType,
        text: String,
        x: f64,
        y: f64,
    },
    MoveNotes(crate::non_empty::NonEmpty<Position>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Position {
    pub id: keel::note_id::NoteId,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Edit(EditKind);

#[derive(Debug, Clone, PartialEq)]
pub enum EditKind {
    NoteAdded {
        id: keel::board::Assigned,
        note_type: keel::note_type::NoteType,
        text: String,
        x: f64,
        y: f64,
    },
    NotesMoved(crate::non_empty::NonEmpty<Position>),
}

impl Edit {
    pub fn parse(board: &keel::board::Board, operation: Operation) -> Option<Self> {
        match operation {
            Operation::AddNote {
                id,
                note_type,
                text,
                x,
                y,
            } => Some(Self(EditKind::NoteAdded {
                id: board.assign_note_id(id, note_type),
                note_type,
                text,
                x,
                y,
            })),
            Operation::MoveNotes(positions) => positions
                .iter()
                .all(|p| board.notes().iter().any(|n| n.id() == &p.id))
                .then_some(Self(EditKind::NotesMoved(positions))),
        }
    }

    pub fn kind(&self) -> &EditKind {
        &self.0
    }

    pub fn apply(&self, board: &mut keel::board::Board) {
        match &self.0 {
            EditKind::NoteAdded {
                id,
                note_type,
                text,
                x,
                y,
            } => {
                board.add_note(id.id().clone(), *note_type, text.clone(), *x, *y);
            }
            EditKind::NotesMoved(positions) => {
                for p in positions.iter() {
                    board.move_note(&p.id, p.x, p.y);
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Outcome {
    Applied,
    Reassigned(keel::note_id::NoteId),
    Skipped,
}

impl Outcome {
    pub fn of(edit: Option<&Edit>) -> Self {
        match edit.map(Edit::kind) {
            Some(EditKind::NoteAdded {
                id: keel::board::Assigned::Reassigned(id),
                ..
            }) => Outcome::Reassigned(id.clone()),
            Some(_) => Outcome::Applied,
            None => Outcome::Skipped,
        }
    }
}

#[cfg(test)]
mod tests {
    fn random_id() -> keel::note_id::NoteId {
        keel::note_id::NoteId::parse(&random_string::generate_random_string(
            8,
            &[random_string::CharacterType::Lowercase],
            "",
            &mut std::fs::File::open("/dev/urandom").unwrap(),
        ))
        .unwrap()
    }

    fn add_note(id: keel::note_id::NoteId) -> super::Operation {
        super::Operation::AddNote {
            id,
            note_type: keel::note_type::NoteType::Event,
            text: String::new(),
            x: 0.0,
            y: 0.0,
        }
    }

    fn move_notes(ids: &[&keel::note_id::NoteId]) -> super::Operation {
        super::Operation::MoveNotes(
            crate::non_empty::NonEmpty::parse(
                ids.iter()
                    .map(|&id| super::Position {
                        id: id.clone(),
                        x: 1.0,
                        y: 2.0,
                    })
                    .collect(),
            )
            .unwrap(),
        )
    }

    #[test]
    fn when_parse_add_note_with_free_id_then_returns_edit_with_the_id_as_proposed() {
        let id = random_id();

        let edit = super::Edit::parse(&keel::board::Board::default(), add_note(id.clone()));

        assert!(matches!(
            edit.as_ref().map(super::Edit::kind),
            Some(super::EditKind::NoteAdded { id: keel::board::Assigned::AsProposed(i), .. }) if *i == id
        ));
        assert_eq!(super::Outcome::of(edit.as_ref()), super::Outcome::Applied);
    }

    #[test]
    fn when_parse_add_note_with_taken_id_then_returns_edit_with_reassigned_id() {
        let id = random_id();
        let mut board = keel::board::Board::default();
        board.add_note(
            id.clone(),
            keel::note_type::NoteType::Command,
            String::new(),
            0.0,
            0.0,
        );

        let edit = super::Edit::parse(&board, add_note(id));

        assert_eq!(
            super::Outcome::of(edit.as_ref()),
            super::Outcome::Reassigned(keel::note_id::NoteId::parse("e1").unwrap())
        );
    }

    #[test]
    fn when_parse_move_notes_of_existing_notes_then_returns_edit_that_sets_their_positions() {
        let id = random_id();
        let mut board = keel::board::Board::default();
        board.add_note(
            id.clone(),
            keel::note_type::NoteType::Event,
            String::new(),
            0.0,
            0.0,
        );

        let edit = super::Edit::parse(&board, move_notes(&[&id]));
        if let Some(edit) = &edit {
            edit.apply(&mut board);
        }

        assert_eq!(super::Outcome::of(edit.as_ref()), super::Outcome::Applied);
        assert!(matches!(board.notes(), [n] if n.x() == 1.0 && n.y() == 2.0));
    }

    #[test]
    fn when_parse_move_notes_with_a_missing_note_then_returns_none() {
        let id = random_id();
        let mut board = keel::board::Board::default();
        board.add_note(
            id.clone(),
            keel::note_type::NoteType::Event,
            String::new(),
            0.0,
            0.0,
        );

        let edit = super::Edit::parse(&board, move_notes(&[&id, &random_id()]));

        assert_eq!(edit, None);
        assert_eq!(super::Outcome::of(edit.as_ref()), super::Outcome::Skipped);
    }

    #[test]
    fn when_apply_note_added_then_adds_the_note_with_the_assigned_id() {
        let id = random_id();
        let mut board = keel::board::Board::default();
        board.add_note(
            id.clone(),
            keel::note_type::NoteType::Event,
            String::new(),
            0.0,
            0.0,
        );
        let edit = super::Edit::parse(&board, add_note(id)).unwrap();

        edit.apply(&mut board);

        assert!(matches!(board.notes(), [_, n] if n.id().as_str() == "e1"));
    }
}
