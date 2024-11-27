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
        let file_contents = fs::read_to_string("parth-doc-data.json").expect("Couldn't read file");
        let data: Vec<FileRow> =
            serde_json::from_str(&file_contents).expect("Json not formatted well");

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
            let mut current_id = datum.file.id;
            loop {
                let row = all_files.get(&current_id).unwrap();
                let current_size = folder_sizes
                    .get(&row.file.parent)
                    .copied()
                    .unwrap_or_default();
                folder_sizes.insert(row.file.parent, size + current_size);
                if current_id == root {
                    break;
                }
                current_id = row.file.parent;
            }
        }

        Self {
            current_root: root,
            all_files,
            folder_sizes,
        }
    }



    pub struct Layers {
        layers: Vec<Vec<BetterNode>>
    }

    pub struct BetterNode {
        id: Uuid,
        name: String,
        portion: f64,
    }

    // populate Vec<Node>
    // write some basic tests at the bottom and fix any mistakes in here
    // all we're trying to accomplish by wed 4th is a very reliable data abstraction
    pub fn get_children(&self, id: &Uuid) -> Vec<Node> {
        let total_size = *self.folder_sizes.get(id).unwrap() as f64;
        let children = self
            .all_files
            .values()
            .filter(|f| f.file.parent == *id) // todo handle root
            .map(|f| Node {
                id: f.file.id,
                name: f.file.name.clone(),
                portion: f.size as f64 / total_size,
                children: todo!(),
            });
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::Data;

    #[test]
    fn init() {
        Data::init();
    }
}
