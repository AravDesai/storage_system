use std::{collections::HashMap, fs};
//model::file::File
use lb_rs::{shared::file::File, shared::file_metadata::FileType, Uuid};
use serde::Deserialize;

#[derive(Debug)]
pub struct Data {
    current_root: Uuid,
    all_files: HashMap<Uuid, FileRow>,
    folder_sizes: HashMap<Uuid, u64>,
}

#[derive(PartialEq, Debug)]
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
    pub fn from_file() {
        let file_contents = fs::read_to_string("parth-doc-data.json").expect("Couldn't read file");
        let data: Vec<FileRow> =
            serde_json::from_str(&file_contents).expect("Json not formatted well");
        Self::init(data);
    }

    pub fn init(data: Vec<FileRow>) -> Self {
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
            let mut size = 0;
            if datum.file.is_document() {
                size = datum.size;
            }
            let mut current_id = datum.file.id;
            loop {
                let row = all_files.get(&current_id).unwrap();
                let current_size = folder_sizes
                    .get(&row.file.parent)
                    .copied()
                    .unwrap_or_default();
                if current_id == root {
                    break;
                }
                folder_sizes.insert(row.file.parent, size + current_size);
                size += current_size;
                current_id = row.file.parent;
            }
        }

        Self {
            current_root: root,
            all_files,
            folder_sizes,
        }
    }

    // pub struct Layers {
    //     layers: Vec<Vec<BetterNode>>
    // }

    // pub struct BetterNode {
    //     id: Uuid,
    //     name: String,
    //     portion: f64,
    // }

    // populate Vec<Node>
    // write some basic tests at the bottom and fix any mistakes in here
    // all we're trying to accomplish by wed 4th is a very reliable data abstraction
    pub fn get_children(&self, id: &Uuid) -> Vec<Node> {
        if !self.all_files.get(id).unwrap().file.is_folder() {
            return vec![];
        }
        println!("ID: {}", id);
        let total_size = *self.folder_sizes.get(id).unwrap() as f64;
        let children = self
            .all_files
            .values()
            .filter(|f| f.file.parent == *id && f.file.id != *id)
            .map(|f| Node {
                id: f.file.id,
                name: f.file.name.clone(),
                portion: f.size as f64 / total_size,
                children: self.get_children(&f.file.id),
            });
        let mut gathered_children = vec![];
        for child in children.into_iter() {
            gathered_children.push(child);
        }
        return gathered_children;
    }
}

#[cfg(test)]
mod test {
    use std::fs;

    use eframe::egui::TextBuffer;
    use lb_rs::Uuid;

    use crate::data::{FileRow, Node};

    use super::Data;

    //Test init: Root and two files. Verify order of files
    //Test get_children(): Root and two files
    //another variant of both of the above with a deeply nested folder structure
    //empty list
    //only one file
    //root missing

    #[test]
    fn init_root_two_files() {
        let file_contents =
            fs::read_to_string("./test_files/root_two_files.json").expect("Couldn't read file");
        let data: Vec<FileRow> =
            serde_json::from_str(&file_contents).expect("Json not formatted well");
        let hold = Data::init(data);
        let expected_root = Uuid::parse_str("8cac2286-87d0-4df3-b6f7-5c86c4fa928c").unwrap();
        assert_eq!(hold.current_root, expected_root);
        let root_size = (hold.folder_sizes.get(&expected_root)).unwrap().clone();
        assert_eq!(root_size, 1400);
    }

    //This test fails sometimes because the json is not parsed in order
    #[test]
    fn get_children_root_two_files() {
        let file_contents =
            fs::read_to_string("./test_files/root_two_files.json").expect("Couldn't read file");
        let data: Vec<FileRow> =
            serde_json::from_str(&file_contents).expect("Json not formatted well");
        let hold = Data::init(data);
        let actual_children = Data::get_children(&hold, &hold.current_root);
        let expected_children = vec![
            Node {
                id: Uuid::parse_str("1c890596-1df9-4638-b0c1-ec77fdaa7a49").unwrap(),
                name: "file1".to_string(),
                portion: 800.0/1400.0,
                children: vec![],
            },
            Node {
                id: Uuid::parse_str("9b052bca-50b4-47b1-8f6a-8a51e3310d86").unwrap(),
                name: "file2".to_string(),
                portion: 600.0/1400.0,
                children: vec![],
            },
        ];
        assert_eq!(actual_children, expected_children);
    }
}
