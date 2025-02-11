use std::{collections::HashMap, fs};
//model::file::File
use lb_rs::{shared::file::File, shared::file_metadata::FileType, Uuid};
use serde::Deserialize;

#[derive(Debug)]
pub struct Data {
    current_root: Uuid,
    all_files: HashMap<Uuid, FileRow>,
    folder_sizes: HashMap<Uuid, u64>,
    overall_root: Uuid,
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
        for datum in data.clone() {
            if datum.file.is_folder() {
                folder_sizes.insert(datum.file.id, 1000);
                continue;
            }
        }
        for datum in data {
            if datum.file.is_folder() {
                continue;
            }
            let datum_size = datum.size;
            let mut current_id = datum.file.id;
            loop {
                let row = all_files.get(&current_id).unwrap();
                let mut current_size = folder_sizes
                    .get(&row.file.parent)
                    .copied()
                    .unwrap_or_default();
                current_size += datum_size;
                if current_id == root {
                    break;
                }
                folder_sizes.insert(row.file.parent, current_size);
                current_id = row.file.parent;
            }
        }

        Self {
            current_root: root,
            overall_root: root,
            all_files,
            folder_sizes,
        }
    }

    pub fn get_children(&self, id: &Uuid) -> Vec<Node> {
        if !self.all_files.get(id).unwrap().file.is_folder() {
            return vec![];
        }
        let current_size = *self.folder_sizes.get(id).unwrap() as f64;
        let total_size = *self.folder_sizes.get(&self.current_root).unwrap() as f64;
        let children = self
            .all_files
            .values()
            .filter(|f| f.file.parent == *id && f.file.id != *id)
            .map(|f| Node {
                id: f.file.id,
                name: f.file.name.clone(),
                portion: current_size / total_size,
                children: self.get_children(&f.file.id),
            });
        let mut gathered_children = vec![];
        for child in children.into_iter() {
            gathered_children.push(child);
        }
        return gathered_children;
    }

    pub fn get_paint_order(&self) -> Vec<Node> {
        let mut paint_order_vec: Vec<Node> = vec![];
        //Planning to make a layer field in the Node struct. This will be populated recursively using get_children
        //I will get initial children of the current root and then add all nodes produced to master_vec.
        //I will then run get_children on children produced by children with a loop until nothing can be added to the vec
        //The master_vec will be sorted using sort_by in descending order in the terms of layer field and returned
        return paint_order_vec;
    }
}

#[cfg(test)]
mod test {
    use std::{collections::HashMap, fs};

    use eframe::egui::TextBuffer;
    use lb_rs::{File, Uuid};

    use crate::data::{FileRow, Node};

    use super::Data;

    //Test init: Root and two files. Verify order of files
    //Test get_children(): Root and two files
    //another variant of both of the above with a deeply nested folder structure

    //How should my program handle these? A current_root must always exist and only folders will be interactable
    //empty list
    //only one file
    //root missing

    #[test]
    fn init_root_checker() {
        let data: Vec<FileRow> = vec![
            FileRow {
                file: File {
                    id: Uuid::parse_str("8cac2286-87d0-4df3-b6f7-5c86c4fa928c").unwrap(),
                    parent: Uuid::parse_str("8cac2286-87d0-4df3-b6f7-5c86c4fa928c").unwrap(),
                    name: "Root".to_string(),
                    file_type: lb_rs::FileType::Folder,
                    last_modified: 1693063210788,
                    last_modified_by: "parth".to_string(),
                    shares: [].to_vec(),
                },
                size: 1000,
            },
            FileRow {
                file: File {
                    id: Uuid::parse_str("1c890596-1df9-4638-b0c1-ec77fdaa7a49").unwrap(),
                    parent: Uuid::parse_str("8cac2286-87d0-4df3-b6f7-5c86c4fa928c").unwrap(),
                    name: "file1".to_string(),
                    file_type: lb_rs::FileType::Document,
                    last_modified: 1693063210788,
                    last_modified_by: "parth".to_string(),
                    shares: [].to_vec(),
                },
                size: 800,
            },
            FileRow {
                file: File {
                    id: Uuid::parse_str("9b052bca-50b4-47b1-8f6a-8a51e3310d86").unwrap(),
                    parent: Uuid::parse_str("8cac2286-87d0-4df3-b6f7-5c86c4fa928c").unwrap(),
                    name: "file2".to_string(),
                    file_type: lb_rs::FileType::Document,
                    last_modified: 1693063210788,
                    last_modified_by: "parth".to_string(),
                    shares: [].to_vec(),
                },
                size: 600,
            },
        ];
        let hold = Data::init(data);
        let expected_root = Uuid::parse_str("8cac2286-87d0-4df3-b6f7-5c86c4fa928c").unwrap();
        assert_eq!(hold.current_root, expected_root);
        let root_size = (hold.folder_sizes.get(&expected_root)).unwrap().clone();
        assert_eq!(root_size, 2400);
    }

    //this test sometimes outputs in different orders
    #[test]
    fn get_children_root_two_files() {
        let data: Vec<FileRow> = vec![
            FileRow {
                file: File {
                    id: Uuid::parse_str("8cac2286-87d0-4df3-b6f7-5c86c4fa928c").unwrap(),
                    parent: Uuid::parse_str("8cac2286-87d0-4df3-b6f7-5c86c4fa928c").unwrap(),
                    name: "Root".to_string(),
                    file_type: lb_rs::FileType::Folder,
                    last_modified: 1693063210788,
                    last_modified_by: "parth".to_string(),
                    shares: [].to_vec(),
                },
                size: 1000,
            },
            FileRow {
                file: File {
                    id: Uuid::parse_str("1c890596-1df9-4638-b0c1-ec77fdaa7a49").unwrap(),
                    parent: Uuid::parse_str("8cac2286-87d0-4df3-b6f7-5c86c4fa928c").unwrap(),
                    name: "file1".to_string(),
                    file_type: lb_rs::FileType::Document,
                    last_modified: 1693063210788,
                    last_modified_by: "parth".to_string(),
                    shares: [].to_vec(),
                },
                size: 800,
            },
            FileRow {
                file: File {
                    id: Uuid::parse_str("9b052bca-50b4-47b1-8f6a-8a51e3310d86").unwrap(),
                    parent: Uuid::parse_str("8cac2286-87d0-4df3-b6f7-5c86c4fa928c").unwrap(),
                    name: "file2".to_string(),
                    file_type: lb_rs::FileType::Document,
                    last_modified: 1693063210788,
                    last_modified_by: "parth".to_string(),
                    shares: [].to_vec(),
                },
                size: 600,
            },
        ];
        let hold = Data::init(data);
        let actual_children = Data::get_children(&hold, &hold.current_root);
        let expected_children = vec![
            Node {
                id: Uuid::parse_str("1c890596-1df9-4638-b0c1-ec77fdaa7a49").unwrap(),
                name: "file1".to_string(),
                portion: 800.0 / 2400.0,
                children: vec![],
            },
            Node {
                id: Uuid::parse_str("9b052bca-50b4-47b1-8f6a-8a51e3310d86").unwrap(),
                name: "file2".to_string(),
                portion: 600.0 / 2400.0,
                children: vec![],
            },
        ];
        assert_eq!(expected_children, actual_children);
    }

    #[test]
    fn get_children_nested_folders() {
        let data: Vec<FileRow> = vec![
            FileRow {
                file: File {
                    id: Uuid::parse_str("8cac2286-87d0-4df3-b6f7-5c86c4fa928c").unwrap(),
                    parent: Uuid::parse_str("8cac2286-87d0-4df3-b6f7-5c86c4fa928c").unwrap(),
                    name: "Root".to_string(),
                    file_type: lb_rs::FileType::Folder,
                    last_modified: 1693063210788,
                    last_modified_by: "parth".to_string(),
                    shares: [].to_vec(),
                },
                size: 1000,
            },
            FileRow {
                file: File {
                    id: Uuid::parse_str("1c890596-1df9-4638-b0c1-ec77fdaa7a49").unwrap(),
                    parent: Uuid::parse_str("8cac2286-87d0-4df3-b6f7-5c86c4fa928c").unwrap(),
                    name: "Layer1".to_string(),
                    file_type: lb_rs::FileType::Folder,
                    last_modified: 1693063210788,
                    last_modified_by: "parth".to_string(),
                    shares: [].to_vec(),
                },
                size: 1000,
            },
            FileRow {
                file: File {
                    id: Uuid::parse_str("9b052bca-50b4-47b1-8f6a-8a51e3310d86").unwrap(),
                    parent: Uuid::parse_str("1c890596-1df9-4638-b0c1-ec77fdaa7a49").unwrap(),
                    name: "Layer2".to_string(),
                    file_type: lb_rs::FileType::Folder,
                    last_modified: 1693063210788,
                    last_modified_by: "parth".to_string(),
                    shares: [].to_vec(),
                },
                size: 1000,
            },
            FileRow {
                file: File {
                    id: Uuid::parse_str("fc50112e-5f9d-4ebf-b6a8-023ba619fd0f").unwrap(),
                    parent: Uuid::parse_str("9b052bca-50b4-47b1-8f6a-8a51e3310d86").unwrap(),
                    name: "file".to_string(),
                    file_type: lb_rs::FileType::Document,
                    last_modified: 1693063210788,
                    last_modified_by: "parth".to_string(),
                    shares: [].to_vec(),
                },
                size: 800,
            },
        ];
        let hold = Data::init(data);
        let actual_children = Data::get_children(&hold, &hold.current_root);
        let expected_children = vec![Node {
            id: Uuid::parse_str("1c890596-1df9-4638-b0c1-ec77fdaa7a49").unwrap(),
            name: "Layer1".to_string(),
            portion: 2800.0 / 3800.0,
            children: vec![Node {
                id: Uuid::parse_str("9b052bca-50b4-47b1-8f6a-8a51e3310d86").unwrap(),
                name: "Layer2".to_string(),
                portion: 1800.0 / 3800.0,
                children: vec![Node {
                    id: Uuid::parse_str("fc50112e-5f9d-4ebf-b6a8-023ba619fd0f").unwrap(),
                    name: "file".to_string(),
                    portion: 800.0 / 3800.0,
                    children: vec![],
                }],
            }],
        }];
        assert_eq!(expected_children, actual_children);
    }

    //This code will be finished when get paint order is finished
    #[test]
    fn nested_two_files_order() {
        let data: Vec<FileRow> = vec![
            FileRow {
                file: File {
                    id: Uuid::parse_str("8cac2286-87d0-4df3-b6f7-5c86c4fa928c").unwrap(),
                    parent: Uuid::parse_str("8cac2286-87d0-4df3-b6f7-5c86c4fa928c").unwrap(),
                    name: "Root".to_string(),
                    file_type: lb_rs::FileType::Folder,
                    last_modified: 1693063210788,
                    last_modified_by: "parth".to_string(),
                    shares: [].to_vec(),
                },
                size: 1000,
            },
            FileRow {
                file: File {
                    id: Uuid::parse_str("9b052bca-50b4-47b1-8f6a-8a51e3310d86").unwrap(),
                    parent: Uuid::parse_str("8cac2286-87d0-4df3-b6f7-5c86c4fa928c").unwrap(),
                    name: "leftlayer1".to_string(),
                    file_type: lb_rs::FileType::Folder,
                    last_modified: 1693063210788,
                    last_modified_by: "parth".to_string(),
                    shares: [].to_vec(),
                },
                size: 1000,
            },
            FileRow {
                file: File {
                    id: Uuid::parse_str("1c890596-1df9-4638-b0c1-ec77fdaa7a49").unwrap(),
                    parent: Uuid::parse_str("9b052bca-50b4-47b1-8f6a-8a51e3310d86").unwrap(),
                    name: "leftlayer2file".to_string(),
                    file_type: lb_rs::FileType::Document,
                    last_modified: 1693063210788,
                    last_modified_by: "parth".to_string(),
                    shares: [].to_vec(),
                },
                size: 800,
            },
            FileRow {
                file: File {
                    id: Uuid::parse_str("219df288-f08b-422b-adf6-59534df7ee91").unwrap(),
                    parent: Uuid::parse_str("8cac2286-87d0-4df3-b6f7-5c86c4fa928c").unwrap(),
                    name: "rightlayer1".to_string(),
                    file_type: lb_rs::FileType::Folder,
                    last_modified: 1693063210788,
                    last_modified_by: "parth".to_string(),
                    shares: [].to_vec(),
                },
                size: 1000,
            },
            FileRow {
                file: File {
                    id: Uuid::parse_str("1c890596-1df9-4638-b0c1-ec77fdaa7a49").unwrap(),
                    parent: Uuid::parse_str("219df288-f08b-422b-adf6-59534df7ee91").unwrap(),
                    name: "rightlayer2file1".to_string(),
                    file_type: lb_rs::FileType::Document,
                    last_modified: 1693063210788,
                    last_modified_by: "parth".to_string(),
                    shares: [].to_vec(),
                },
                size: 300,
            },
            FileRow {
                file: File {
                    id: Uuid::parse_str("fe777276-381f-408b-b41a-bac9b302b9cc").unwrap(),
                    parent: Uuid::parse_str("219df288-f08b-422b-adf6-59534df7ee91").unwrap(),
                    name: "rightlayer2file2".to_string(),
                    file_type: lb_rs::FileType::Document,
                    last_modified: 1693063210788,
                    last_modified_by: "parth".to_string(),
                    shares: [].to_vec(),
                },
                size: 300,
            },
        ];
    }
}
