/// Whether the write is a first insert or an update, which Hibernate named in its message.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Persist,
    Update,
}
