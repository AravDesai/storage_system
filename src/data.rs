use std::{collections::HashMap, fs};

use lb_rs::{model::file::File, Uuid};
use serde::Deserialize;

pub struct Data {
    current_root: Uuid,
    all_files: HashMap<Uuid, FileRow>,
    folder_sizes: HashMap<Uuid, u64>,
}

pub struct Node {
    pub id: Uuid,
    pub name: String,
    pub portion: f64,
    pub children: Vec<Node>,
}

#[derive(Debug, Deserialize, Clone, Hash, PartialEq, Eq)]
pub struct FileRow {
    pub file: File,
    pub size: u64,
}

impl Data {
    pub fn init() -> Self {
        let json_info = fs::read_to_string("parth-doc-data.json").expect("Couldn't read file");
        let data: Vec<FileRow> = serde_json::from_str(&json_info).expect("Json not formatted well");

        let mut all_files = HashMap::new();
        let mut root = Uuid::nil();
        for datum in data.clone() {
            if datum.file.id == datum.file.parent {
                root = datum.file.id;
            }
            all_files.insert(datum.file.id, datum);
        }

        let mut folder_sizes = HashMap::new();
        for datum in data {
            let size = datum.size;
            let current_id = datum.file.id;
            loop {
                let row = all_files.get(&current_id).unwrap();
                let current_size = folder_sizes.get(&row.file.parent).copied().unwrap_or_default();
                folder_sizes.insert(row.file.parent, size + current_size);
                if current_id == root {
                    break;
                }
                let current_id = row.file.parent;
            }
        }

        Self {
            current_root: root,
            all_files,
            folder_sizes,
        }
    }

    pub fn set_root(&mut self, id: Uuid) {
        self.current_root = id;
    }

    pub fn get_children(&self) -> Vec<Node> {
        // let overall_size = self.folder_sizes.get(&self.current_root);
        todo!()
    }
}
