use crate::note_id::NoteId;
use crate::note_type::NoteType;

#[derive(Debug, Clone, PartialEq)]
pub struct Note {
    id: NoteId,
    note_type: NoteType,
    text: String,
    x: f64,
    y: f64,
}

impl Note {
    pub fn id(&self) -> &NoteId {
        &self.id
    }

    pub fn note_type(&self) -> NoteType {
        self.note_type
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Line {
    Solid,
    Dashed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edge {
    from: NoteId,
    to: NoteId,
    line: Line,
}

impl Edge {
    pub fn from(&self) -> &NoteId {
        &self.from
    }

    pub fn to(&self) -> &NoteId {
        &self.to
    }

    pub fn line(&self) -> Line {
        self.line
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Assigned {
    AsProposed(NoteId),
    Reassigned(NoteId),
}

impl Assigned {
    pub fn id(&self) -> &NoteId {
        match self {
            Assigned::AsProposed(id) | Assigned::Reassigned(id) => id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Board {
    title: String,
    notes: Vec<Note>,
    edges: Vec<Edge>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DuplicateNoteId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum EdgeRejected {
    MissingEndpoint,
    Duplicate,
}

impl Board {
    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn notes(&self) -> &[Note] {
        &self.notes
    }

    pub fn edges(&self) -> &[Edge] {
        &self.edges
    }

    pub fn add_note(
        &mut self,
        proposed: NoteId,
        note_type: NoteType,
        text: String,
        x: f64,
        y: f64,
    ) -> Assigned {
        let assigned = self.assign_note_id(proposed, note_type);
        self.notes.push(Note {
            id: assigned.id().clone(),
            note_type,
            text,
            x,
            y,
        });
        assigned
    }

    pub fn move_note(&mut self, id: &NoteId, x: f64, y: f64) {
        if let Some(note) = self.notes.iter_mut().find(|n| &n.id == id) {
            note.x = x;
            note.y = y;
        }
    }

    pub fn assign_note_id(&self, proposed: NoteId, note_type: NoteType) -> Assigned {
        if self.notes.iter().any(|n| n.id == proposed) {
            Assigned::Reassigned(self.next_note_id(note_type))
        } else {
            Assigned::AsProposed(proposed)
        }
    }

    fn next_note_id(&self, note_type: NoteType) -> NoteId {
        let mut number = 1;
        loop {
            let id = NoteId::numbered(note_type, number);
            if self.notes.iter().all(|n| n.id != id) {
                return id;
            }
            number += 1;
        }
    }

    pub(crate) fn set_title(&mut self, title: String) {
        self.title = title;
    }

    pub(crate) fn insert_note(
        &mut self,
        id: NoteId,
        note_type: NoteType,
        text: String,
        x: f64,
        y: f64,
    ) -> Result<(), DuplicateNoteId> {
        if self.notes.iter().any(|n| n.id == id) {
            return Err(DuplicateNoteId);
        }
        self.notes.push(Note {
            id,
            note_type,
            text,
            x,
            y,
        });
        Ok(())
    }

    pub(crate) fn insert_edge(
        &mut self,
        from: &NoteId,
        to: &NoteId,
        requested: Line,
    ) -> Result<Line, EdgeRejected> {
        let find = |id: &NoteId| self.notes.iter().find(|n| &n.id == id);
        let (Some(from), Some(to)) = (find(from), find(to)) else {
            return Err(EdgeRejected::MissingEndpoint);
        };
        if self
            .edges
            .iter()
            .any(|e| e.from == from.id && e.to == to.id)
        {
            return Err(EdgeRejected::Duplicate);
        }
        let line = if from.note_type.can_connect_to(to.note_type) {
            requested
        } else {
            Line::Dashed
        };
        let edge = Edge {
            from: from.id.clone(),
            to: to.id.clone(),
            line,
        };
        self.edges.push(edge);
        Ok(line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use random_string::CharacterType::{Lowercase, Numeric};
    use random_string::generate_random_string;
    use std::fs::File;

    #[test]
    fn when_add_note_with_free_id_then_adds_the_note_with_the_id() {
        let mut urandom = File::open("/dev/urandom").unwrap();
        let id = NoteId::parse(&generate_random_string(8, &[Lowercase], "", &mut urandom)).unwrap();
        let text = generate_random_string(16, &[Lowercase], "", &mut urandom);
        let mut board = Board::default();

        let added = board.add_note(id.clone(), NoteType::Event, text.clone(), 1.0, 2.0);

        assert_eq!(added, Assigned::AsProposed(id.clone()));
        assert_eq!(
            board.notes(),
            [Note {
                id,
                note_type: NoteType::Event,
                text,
                x: 1.0,
                y: 2.0,
            }]
        );
    }

    #[test]
    fn when_add_note_with_taken_id_then_adds_the_note_with_next_free_id_of_its_type() {
        let mut urandom = File::open("/dev/urandom").unwrap();
        let taken =
            NoteId::parse(&generate_random_string(8, &[Lowercase], "", &mut urandom)).unwrap();
        let mut board = Board::default();
        board.add_note(taken.clone(), NoteType::Command, String::new(), 0.0, 0.0);
        board.add_note(
            NoteId::parse("e1").unwrap(),
            NoteType::Event,
            String::new(),
            0.0,
            0.0,
        );

        let added = board.add_note(taken, NoteType::Event, String::new(), 0.0, 0.0);

        assert_eq!(added, Assigned::Reassigned(NoteId::parse("e2").unwrap()));
        assert_eq!(board.notes().len(), 3);
    }

    #[test]
    fn when_move_note_with_id_of_a_note_then_sets_its_position() {
        let mut urandom = File::open("/dev/urandom").unwrap();
        let id = NoteId::parse(&generate_random_string(8, &[Lowercase], "", &mut urandom)).unwrap();
        let x: f64 = generate_random_string(4, &[Numeric], "", &mut urandom)
            .parse()
            .unwrap();
        let y: f64 = generate_random_string(4, &[Numeric], "", &mut urandom)
            .parse()
            .unwrap();
        let mut board = Board::default();
        board.add_note(id.clone(), NoteType::Event, String::new(), 0.0, 0.0);

        board.move_note(&id, x, y);

        assert!(matches!(board.notes(), [n] if n.x() == x && n.y() == y));
    }

    #[test]
    fn when_move_note_with_id_of_no_note_then_leaves_the_board_unchanged() {
        let mut urandom = File::open("/dev/urandom").unwrap();
        let id = NoteId::parse(&generate_random_string(8, &[Lowercase], "", &mut urandom)).unwrap();
        let mut board = Board::default();
        board.add_note(
            NoteId::parse("e1").unwrap(),
            NoteType::Event,
            String::new(),
            0.0,
            0.0,
        );
        let before = board.clone();

        board.move_note(&id, 1.0, 2.0);

        assert_eq!(board, before);
    }
}
