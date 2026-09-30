#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Created {
        room: uuid::Uuid,
        board: keel::board::Board,
    },
}
