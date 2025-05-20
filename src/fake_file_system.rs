use anyhow::Context;
use axum::http;
use axum::http::StatusCode;
use std::collections::HashMap;
use std::ops::{Add, Deref};
use std::path::{Component, Path, PathBuf};
use std::str::FromStr;
use urlencoding::encode;
use webdav_meta::xml;
use webdav_meta::xml::nonempty::NonEmpty;

pub struct FakeFilesystem {
    files: HashMap<PathBuf, Node>,
}

impl FakeFilesystem {
    /// Creates a new file system with a folder at its root.
    pub fn new_with_root() -> FakeFilesystem {
        let root_dir = Node::Folder(Folder {
            name: "".to_string(),
        });
        let mut map: HashMap<PathBuf, Node> = HashMap::new();
        map.insert(PathBuf::from("/"), root_dir);
        FakeFilesystem { files: map }
    }

    pub fn remove_node(&mut self, path: &Path) {
        let to_delete = self
            .files
            .keys()
            .filter(|k| k.starts_with(path))
            .cloned()
            .collect::<Vec<_>>();
        for path_to_delete in to_delete {
            self.files.remove(&path_to_delete);
        }
    }

    pub fn read_node(&self, path: &Path) -> Option<&Node> {
        self.files.get(path)
    }

    pub fn read_dir(&self, path: &Path) -> Option<Vec<(PathBuf, &Node)>> {
        if let Some(Node::Folder(_)) = self.files.get(path) {
            let children = self.children(path);
            return Some(
                children
                    .into_iter()
                    .filter_map(|child| self.read_node(&child).and_then(|node| Some((child, node))))
                    .collect(),
            );
        }
        None
    }

    fn children(&self, path: &Path) -> Vec<PathBuf> {
        self.files
            .keys()
            .filter(|file_path| file_path.parent() == Some(path))
            .map(|file_path| file_path.to_path_buf())
            .collect()
    }

    pub fn add_node(&mut self, path: &Path, node: Node) {
        self.files.insert(path.to_owned(), node);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub enum Node {
    File(File),
    Folder(Folder),
}

impl Node {
    pub fn to_propstat_response(&self, path: &Path) -> anyhow::Result<xml::elements::Response> {
        let components = path
            .components()
            .filter_map(|component| match component {
                Component::Prefix(_) => None,
                Component::RootDir => None,
                Component::CurDir => None,
                Component::ParentDir => None,
                Component::Normal(component_part) => {
                    Some(encode(component_part.to_str().unwrap()).to_string())
                }
            })
            .collect::<Vec<_>>();

        let (uri, properties) = match self {
            Node::File(file) => {
                let uri = http::Uri::builder()
                    .path_and_query(format!("/{}", components.join("/")))
                    .build()
                    .context("couldn't build URI")?;
                let properties = xml::elements::Properties::new()
                    .with(xml::properties::ResourceType::empty())
                    .with(xml::properties::ContentLength(file.size as u64));
                (uri, properties)
            }
            Node::Folder(_) => {
                let uri = http::Uri::builder()
                    .path_and_query(format!("/{}/", components.join("/")))
                    .build()
                    .context("couldn't build URI")?;
                let properties = xml::elements::Properties::new()
                    .with(xml::properties::ResourceType::collection());
                (uri, properties)
            }
        };
        Ok(xml::elements::Response::Propstat {
            href: xml::elements::Href(uri),
            propstat: NonEmpty::new(xml::elements::Propstat {
                prop: properties,
                status: xml::elements::Status(StatusCode::OK),
                responsedescription: None,
            }),
            responsedescription: None,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct File {
    pub(crate) name: String,
    pub(crate) size: i64,
    pub(crate) download_details: (i64, i64),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Folder {
    pub(crate) name: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    mod read_node {
        use super::*;
        use assert_unordered::assert_eq_unordered_sort;

        #[test]
        fn it_reads_root() {
            let fs = FakeFilesystem::new_with_root();
            assert_eq!(
                fs.read_node(&PathBuf::from("/")),
                Some(&Node::Folder(Folder {
                    name: "".to_string()
                }))
            );
        }

        #[test]
        fn it_reads_children() {
            let mut fs = FakeFilesystem::new_with_root();
            let folder = Node::Folder(Folder {
                name: "hello".to_string(),
            });
            let file = Node::File(File {
                name: "hello.txt".to_string(),
                size: 1200,
                download_details: (-1, -1),
            });
            fs.files.insert(PathBuf::from("/hello"), folder.clone());
            fs.files.insert(PathBuf::from("/hello.txt"), file.clone());
            assert_eq_unordered_sort!(
                fs.read_dir(&PathBuf::from("/")),
                Some(vec![
                    (PathBuf::from("/hello.txt"), &file),
                    (PathBuf::from("/hello"), &folder)
                ])
            );
        }

        #[test]
        fn it_reads_nested_children() {
            let mut fs = FakeFilesystem::new_with_root();
            let folder = Node::Folder(Folder {
                name: "hello".to_string(),
            });
            let file = Node::File(File {
                name: "hello.txt".to_string(),
                size: 1200,
                download_details: (-1, -1),
            });
            fs.files.insert(PathBuf::from("/hello"), folder.clone());
            fs.files
                .insert(PathBuf::from("/hello/hello.txt"), file.clone());
            assert_eq_unordered_sort!(
                fs.read_dir(&PathBuf::from("/hello")),
                Some(vec![(PathBuf::from("/hello/hello.txt"), &file)])
            );
        }
    }
}
