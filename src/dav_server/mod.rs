use crate::AppState;
use crate::dav_server::get_handler::get_handler;
use crate::dav_server::propfind_handler::propfind_handler;
use axum::extract;
use axum::extract::{Request, State};
use axum::http::{HeaderValue, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use std::path;
use std::path::PathBuf;
use webdav_meta::methods::PROPFIND;

mod get_handler;
mod propfind_handler;

pub async fn webdav_handler(
    method: Method,
    path: Option<extract::Path<PathBuf>>,
    State(app_state): State<AppState>,
    req: Request,
) -> Response {
    let path = match path {
        Some(extract::Path(path)) => path,
        None => path::Path::new("/").into(),
    };

    let mut resp = match method {
        Method::GET => get_handler(req, path, app_state).await,
        _ if method == PROPFIND.as_ref() => propfind_handler(req, path, app_state).await,
        _ => StatusCode::METHOD_NOT_ALLOWED.into_response(),
    };
    resp.headers_mut()
        .append("dav", HeaderValue::from_static("1"));

    resp
}
