use crate::AppState;
use crate::fake_file_system::Node;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use headers::HeaderMapExt;
use std::path::PathBuf;
use tracing::error;
use webdav_meta::headers::Depth;
use webdav_meta::xml::IntoXml;
use webdav_meta::xml::elements::Multistatus;

pub(super) async fn propfind_handler(req: Request, path: PathBuf, app_state: AppState) -> Response {
    let depth = match req.headers().typed_get::<Depth>() {
        Some(Depth::Infinity) | None => Depth::Infinity,
        Some(d) => d,
    };
    let new_path = {
        let path_str = path.to_string_lossy();
        let mut normalized = String::new();

        // Add leading slash if missing
        if !path_str.starts_with('/') {
            normalized.push('/');
        }

        normalized.push_str(&path_str);

        // Remove trailing slash if present (but preserve root path "/")
        if normalized.len() > 1 && normalized.ends_with('/') {
            normalized.pop();
        }

        PathBuf::from(normalized)
    };

    let fs = app_state.fake_file_system.lock().unwrap();

    if let Some(node) = fs.read_node(&new_path) {
        match node {
            Node::File(_) => {
                let responses = node.to_propstat_response(&new_path).unwrap();
                let response = Multistatus {
                    responsedescription: None,
                    response: vec![responses],
                };
                response.into_xml().unwrap().into_response()
            }
            Node::Folder(_) => match depth {
                Depth::Zero => {
                    let responses = node.to_propstat_response(&new_path).unwrap();
                    let response = Multistatus {
                        responsedescription: None,
                        response: vec![responses],
                    };
                    response.into_xml().unwrap().into_response()
                }
                Depth::One | Depth::Infinity => {
                    let folder_response = node.to_propstat_response(&new_path).unwrap();
                    let dir = fs.read_dir(&new_path).unwrap();
                    let mut dir_children = dir
                        .into_iter()
                        .map(|(child_path, child)| child.to_propstat_response(&child_path).unwrap())
                        .collect::<Vec<_>>();
                    dir_children.push(folder_response);
                    let response = Multistatus {
                        responsedescription: None,
                        response: dir_children,
                    };
                    let xml_response = response.into_xml().unwrap();
                    xml_response.into_response()
                }
            },
        }
    } else {
        error!("node not found for path {}", path.display());
        StatusCode::NOT_FOUND.into_response()
    }
}
