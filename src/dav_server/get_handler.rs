use crate::AppState;
use crate::fake_file_system::Node;
use axum::body::Body;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::path::PathBuf;

pub(super) async fn get_handler(req: Request, path: PathBuf, app_state: AppState) -> Response {
    let normalized_path = {
        let path_str = path.to_string_lossy();
        let mut normalized = String::new();
        if !path_str.starts_with('/') {
            normalized.push('/');
        }
        normalized.push_str(&path_str);
        if normalized.len() > 1 && normalized.ends_with('/') {
            normalized.pop();
        }
        PathBuf::from(normalized)
    };

    // Get the file from fake filesystem
    let node = {
        let fs = app_state.fake_file_system.lock().unwrap();
        fs.read_node(&normalized_path).cloned()
    };

    if let Some(node) = node {
        match node {
            Node::File(file) => {
                let range_header = req.headers().get("Range").cloned();
                let mut request_builder = app_state.reqwest_client.get(&file.download_url);

                if let Some(range) = range_header {
                    request_builder = request_builder.header("Range", range);
                }

                let reqwest_response = request_builder.send().await.unwrap();

                let status = reqwest_response.status();
                let headers = reqwest_response.headers().clone();
                let stream = reqwest_response.bytes_stream();

                let mut response_builder = Response::builder().status(status);

                for (name, value) in headers.iter() {
                    if name != "transfer-encoding" {
                        response_builder = response_builder.header(name, value);
                    }
                }

                response_builder
                    .body(Body::from_stream(stream))
                    .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
            }
            Node::Folder(_) => StatusCode::FORBIDDEN.into_response(),
        }
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}
