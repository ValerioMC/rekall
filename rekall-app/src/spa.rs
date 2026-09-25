//! The console: `rekall-ui/dist` served as static files, built into the binary the way the jar
//! carried it under `classpath:/static`, and `SinglePageApplicationRouting`'s forwards so a
//! refresh on a client-side route gets the application rather than a 404. A path nothing answers
//! is Spring Boot's 404; a write to one is its 405, since only the resource handler was left.

use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::{header, HeaderValue, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use rekall_api::error::framework;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../rekall-ui/dist"]
#[allow_missing = true]
struct Bundle;

/// Where the files come from: the bundle built in, or a folder named by `rekall.ui.dist`.
#[derive(Clone)]
pub struct Assets {
    folder: Option<Arc<PathBuf>>,
}

impl Assets {
    pub fn new(folder: Option<PathBuf>) -> Self {
        Self { folder: folder.map(Arc::new) }
    }

    fn read(&self, relative: &str) -> Option<Vec<u8>> {
        if relative.is_empty() || relative.ends_with('/') {
            return None;
        }
        match &self.folder {
            Some(folder) => {
                let safe = Path::new(relative);
                if safe.components().any(|c| !matches!(c, Component::Normal(_))) {
                    return None;
                }
                let file = folder.join(safe);
                file.is_file().then(|| std::fs::read(file).ok()).flatten()
            }
            None => Bundle::get(relative).map(|file| file.data.into_owned()),
        }
    }

    /// Whether there is a console to serve at all.
    pub fn has_index(&self) -> bool {
        self.read("index.html").is_some()
    }
}

const FORWARDED: [&str; 10] = ["/", "/projects", "/companies", "/tasks", "/search", "/calendar", "/report", "/projects/", "/companies/", "/tasks/"];
const FORWARDED_TREES: [&str; 3] = ["/projects/{*rest}", "/companies/{*rest}", "/tasks/{*rest}"];

pub fn routes(assets: Assets) -> Router {
    let mut router = Router::new();
    for path in FORWARDED.into_iter().chain(FORWARDED_TREES) {
        router = router.route(path, get(index));
    }
    router.fallback(resource).with_state(assets)
}

async fn index(State(assets): State<Assets>) -> Response {
    serve(&assets, "index.html").unwrap_or_else(|| framework(StatusCode::NOT_FOUND))
}

async fn resource(State(assets): State<Assets>, request: Request) -> Response {
    if request.method() != Method::GET && request.method() != Method::HEAD {
        let mut refused = framework(StatusCode::METHOD_NOT_ALLOWED);
        refused.headers_mut().insert(header::ALLOW, HeaderValue::from_static("GET,HEAD"));
        return refused;
    }
    let path = request.uri().path();
    let Ok(decoded) = percent_decode(path.trim_start_matches('/')) else {
        return framework(StatusCode::NOT_FOUND);
    };
    serve(&assets, &decoded).unwrap_or_else(|| framework(StatusCode::NOT_FOUND))
}

fn serve(assets: &Assets, relative: &str) -> Option<Response> {
    let bytes = assets.read(relative)?;
    let mime = mime_guess::from_path(relative).first_or_octet_stream();
    let mut response = Body::from(bytes).into_response();
    if let Ok(value) = HeaderValue::from_str(mime.as_ref()) {
        response.headers_mut().insert(header::CONTENT_TYPE, value);
    }
    response.headers_mut().insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));
    Some(response)
}

fn percent_decode(text: &str) -> Result<String, ()> {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let hex = std::str::from_utf8(bytes.get(i + 1..i + 3).ok_or(())?).map_err(|_| ())?;
            out.push(u8::from_str_radix(hex, 16).map_err(|_| ())?);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).map_err(|_| ())
}

