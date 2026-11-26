use rekall_kernel::Id;

/// The record a picture is inserted into, which decides the folder it is kept in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PictureOwner {
    Task(Id),
    /// The blueprint of the project with this id.
    Blueprint(Id),
    /// The section with this id.
    Page(Id),
    /// The component with this id.
    Component(Id),
}
