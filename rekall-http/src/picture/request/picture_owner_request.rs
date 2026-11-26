use rekall_console::picture::PictureOwner;
use rekall_kernel::Id;
use serde::Deserialize;

use crate::picture::request::picture_owner_kind::PictureOwnerKind;

/// A blueprint is named by its project's id.
#[derive(Debug, Deserialize)]
pub struct PictureOwnerRequest {
    pub kind: PictureOwnerKind,
    pub id: Id,
}

impl From<PictureOwnerRequest> for PictureOwner {
    fn from(request: PictureOwnerRequest) -> Self {
        match request.kind {
            PictureOwnerKind::Task => Self::Task(request.id),
            PictureOwnerKind::Blueprint => Self::Blueprint(request.id),
            PictureOwnerKind::Page => Self::Page(request.id),
            PictureOwnerKind::Component => Self::Component(request.id),
        }
    }
}
