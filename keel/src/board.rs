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
