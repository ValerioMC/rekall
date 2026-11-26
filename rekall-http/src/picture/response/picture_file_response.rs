use axum::http::{header, HeaderValue};
use axum::response::{IntoResponse, Response};
use rekall_console::picture::PictureFileView;
use rekall_core::picture::PictureFormat;

/// An SVG opened directly is a document: this keeps its scripts from running.
const SVG_POLICY: &str = "default-src 'none'; style-src 'unsafe-inline'; img-src data:; sandbox";

/// A picture's bytes. Its name carries a hash of its content, so the browser keeps it for good.
pub struct PictureFileResponse {
    view: PictureFileView,
}

impl From<PictureFileView> for PictureFileResponse {
    fn from(view: PictureFileView) -> Self {
        Self { view }
    }
}

impl IntoResponse for PictureFileResponse {
    fn into_response(self) -> Response {
        let format = self.view.format;
        let headers = [
            (header::CONTENT_TYPE, HeaderValue::from_static(format.content_type())),
            (header::CACHE_CONTROL, HeaderValue::from_static("public, max-age=31536000, immutable")),
            (header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff")),
            (header::HeaderName::from_static("cross-origin-resource-policy"), HeaderValue::from_static("same-origin")),
        ];
        let mut response = (headers, self.view.bytes).into_response();
        if format == PictureFormat::Svg {
            response.headers_mut().insert(header::CONTENT_SECURITY_POLICY, HeaderValue::from_static(SVG_POLICY));
        }
        response
    }
}
