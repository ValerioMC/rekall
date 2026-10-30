/// Whether the steps a session writes are ready to work or wait on the staging shelf for a person.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Staging {
    Ready,
    Draft,
}
