use std::{collections::HashMap, fs};
//model::file::File
use lb_rs::{shared::file::File, shared::file_metadata::FileType, Uuid};
use serde::Deserialize;
use std::cmp::Ordering;

#[derive(Debug)]
pub struct Data {
    current_root: Uuid,
    all_files: HashMap<Uuid, FileRow>,
    folder_sizes: HashMap<Uuid, u64>,
    overall_root: Uuid,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Node {
    pub id: Uuid,
    pub name: String,
    pub portion: f64,
    pub children: Vec<Node>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct NodeLayer {
    pub id: Uuid,
    pub name: String,
    pub portion: f64,
    pub layer: u64,
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
        //Initial for loop for folders is necessary to give folders starting value as we need to go over folders again to update sizes
        for datum in data.clone() {
            if datum.file.is_folder() {
                folder_sizes.insert(datum.file.id, datum.size);
            }
        }
        for datum in data {
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
        let total_size = *self.folder_sizes.get(&self.current_root).unwrap() as f64;
        let children = self
            .all_files
            .values()
            .filter(|f| f.file.parent == *id && f.file.id != *id)
            .map(|f| {
                let mut current_size = f.size as f64;
                if f.file.is_folder() {
                    current_size = *self.folder_sizes.get(&f.file.id).unwrap() as f64;
                }
                Node {
                    id: f.file.id,
                    name: f.file.name.clone(),
                    portion: current_size / total_size,
                    children: self.get_children(&f.file.id),
                }
            });
        let mut gathered_children = vec![];
        for child in children.into_iter() {
            gathered_children.push(child);
        }
        return gathered_children;
    }

    pub fn reset_root(&mut self) {
        self.current_root = self.overall_root;
    }

    fn set_layers(
        tree: &Vec<Node>,
        current_layer: u64,
        mut raw_layers: Vec<NodeLayer>,
    ) -> Vec<NodeLayer> {
        for slice in tree {
            raw_layers.push(NodeLayer {
                id: slice.id,
                name: slice.name.clone(),
                portion: slice.portion,
                layer: current_layer,
            });
            if !slice.children.is_empty() {
                let hold = Data::set_layers(&slice.children, current_layer + 1, raw_layers.clone());
                for item in hold {
                    if raw_layers.contains(&item){
                        continue;
                    }
                    raw_layers.push(item.clone());
                }
            }
        }
        return raw_layers;
    }

    pub fn get_paint_order(&self) -> Vec<NodeLayer> {
        //maybe add paint order to a field of self so that it only calls set if nothing is present/current root is changed

        let tree = self.get_children(&self.current_root);
        let mut paint_order_vec = Data::set_layers(&tree, 1, vec![]);
        paint_order_vec.push(NodeLayer {
            id: self.current_root,
            name: self
                .all_files
                .get(&self.current_root)
                .unwrap()
                .file
                .name
                .clone(),
            portion: 1.0,
            layer: 0,
        });
        paint_order_vec.sort_by(|a, b| a.layer.cmp(&b.layer));
        return paint_order_vec;
    }
}

#[cfg(test)]
mod test {
    use std::{collections::HashMap, fs};

    use eframe::egui::TextBuffer;
    use lb_rs::{File, Uuid};

    use crate::data::{FileRow, Node, NodeLayer};

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
                    id: Uuid::parse_str("f2c90c41-4aea-44be-a79d-caea3f0306aa").unwrap(),
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
        let hold = Data::init(data);
        let expected_order: Vec<NodeLayer> = vec![
            NodeLayer {
                id: Uuid::parse_str("8cac2286-87d0-4df3-b6f7-5c86c4fa928c").unwrap(),
                name: "Root".to_string(),
                portion: 1.0,
                layer: 0,
            },
            NodeLayer {
                id: Uuid::parse_str("9b052bca-50b4-47b1-8f6a-8a51e3310d86").unwrap(),
                name: "leftlayer1".to_string(),
                portion: 1800.0 / 4400.0,
                layer: 1,
            },
            NodeLayer {
                id: Uuid::parse_str("219df288-f08b-422b-adf6-59534df7ee91").unwrap(),
                name: "rightlayer1".to_string(),
                portion: 1600.0 / 4400.0,
                layer: 1,
            },
            NodeLayer {
                id: Uuid::parse_str("1c890596-1df9-4638-b0c1-ec77fdaa7a49").unwrap(),
                name: "leftlayer2file".to_string(),
                portion: 800.0 / 4400.0,
                layer: 2,
            },
            NodeLayer {
                id: Uuid::parse_str("f2c90c41-4aea-44be-a79d-caea3f0306aa").unwrap(),
                name: "rightlayer2file1".to_string(),
                portion: 300.0 / 4400.0,
                layer: 2,
            },
            NodeLayer {
                id: Uuid::parse_str("fe777276-381f-408b-b41a-bac9b302b9cc").unwrap(),
                name: "rightlayer2file2".to_string(),
                portion: 300.0 / 4400.0,
                layer: 2,
            },
        ];
        let actual_order = Data::get_paint_order(&hold);
        assert_eq!(expected_order, actual_order, "\nExpected: \n{:?}\nActual:\n{:?}\n",expected_order,actual_order);
    }
}
