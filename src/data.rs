use lb_rs::model::file::File;
use lb_rs::Uuid;
use serde::Deserialize;
use std::{collections::HashMap, fs};

#[derive(Debug)]
pub struct Data {
    pub current_root: Uuid,
    pub all_files: HashMap<Uuid, FileRow>,
    pub folder_sizes: HashMap<Uuid, u64>,
    pub overall_root: Uuid,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Node {
    pub id: Uuid,
    pub name: String,
    pub portion: f32,
    pub children: Vec<Node>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct NodeLayer {
    pub id: Uuid,
    pub name: String,
    pub portion: f32,
    pub layer: u64,
}

#[derive(Debug, Deserialize, Clone, Hash, PartialEq, Eq)]
pub struct FileRow {
    pub file: File,
    pub size: u64,
}

impl Data {
    pub fn from_file(file: String) -> Vec<FileRow> {
        let file_contents = fs::read_to_string(file).expect("Couldn't read file");
        return serde_json::from_str(&file_contents).expect("Json not formatted well");
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
                folder_sizes.insert(datum.file.id, datum.size); //change to datum.size when metadata is accounted for
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
        let total_size = *self.folder_sizes.get(&self.current_root).unwrap() as f32;
        let children = self
            .all_files
            .values()
            .filter(|f| f.file.parent == *id && f.file.id != *id)
            .map(|f| {
                let mut current_size = f.size as f32;
                if f.file.is_folder() {
                    current_size = *self.folder_sizes.get(&f.file.id).unwrap() as f32;
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
        gathered_children.sort_by(|a, b| {
            let a_size = (a.portion * 10000.0) as u32;
            let b_size = (b.portion * 10000.0) as u32;
            b_size.cmp(&a_size)
        });
        return gathered_children;
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
                    if raw_layers.contains(&item) {
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
        paint_order_vec.sort_by(|a, b| a.layer.cmp(&b.layer));
        return paint_order_vec;
    }
}

//Some of these tests may not work due to change in logic - will be fixed soon
#[cfg(test)]
mod test {
    use super::Data;
    use crate::data::{FileRow, Node, NodeLayer};
    use lb_rs::model::file::File;
    use lb_rs::model::file_metadata::FileType;
    use lb_rs::Uuid;

    fn get_root_two_files() -> Vec<FileRow> {
        return vec![
            FileRow {
                file: File {
                    id: Uuid::parse_str("8cac2286-87d0-4df3-b6f7-5c86c4fa928c").unwrap(),
                    parent: Uuid::parse_str("8cac2286-87d0-4df3-b6f7-5c86c4fa928c").unwrap(),
                    name: "Root".to_string(),
                    file_type: FileType::Folder,
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
                    file_type: FileType::Document,
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
                    file_type: FileType::Document,
                    last_modified: 1693063210788,
                    last_modified_by: "parth".to_string(),
                    shares: [].to_vec(),
                },
                size: 600,
            },
        ];
    }

    #[test]
    fn init_root_checker() {
        let hold = Data::init(get_root_two_files());
        let expected_root = Uuid::parse_str("8cac2286-87d0-4df3-b6f7-5c86c4fa928c").unwrap();
        assert_eq!(hold.current_root, expected_root);
        let root_size = (hold.folder_sizes.get(&expected_root)).unwrap().clone();
        assert_eq!(root_size, 2400);
    }

    //this test sometimes outputs in different orders
    #[test]
    fn get_children_root_two_files() {
        let hold = Data::init(get_root_two_files());
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
                    file_type: FileType::Folder,
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
                    file_type: FileType::Folder,
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
                    file_type: FileType::Folder,
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
                    file_type: FileType::Document,
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
                    file_type: FileType::Folder,
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
                    file_type: FileType::Folder,
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
                    file_type: FileType::Document,
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
                    file_type: FileType::Folder,
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
                    file_type: FileType::Document,
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
                    file_type: FileType::Document,
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
                id: Uuid::parse_str("fe777276-381f-408b-b41a-bac9b302b9cc").unwrap(),
                name: "rightlayer2file2".to_string(),
                portion: 300.0 / 4400.0,
                layer: 2,
            },
            NodeLayer {
                id: Uuid::parse_str("f2c90c41-4aea-44be-a79d-caea3f0306aa").unwrap(),
                name: "rightlayer2file1".to_string(),
                portion: 300.0 / 4400.0,
                layer: 2,
            },
            NodeLayer {
                id: Uuid::parse_str("1c890596-1df9-4638-b0c1-ec77fdaa7a49").unwrap(),
                name: "leftlayer2file".to_string(),
                portion: 800.0 / 4400.0,
                layer: 2,
            },
            NodeLayer {
                id: Uuid::parse_str("219df288-f08b-422b-adf6-59534df7ee91").unwrap(),
                name: "rightlayer1".to_string(),
                portion: 1600.0 / 4400.0,
                layer: 1,
            },
            NodeLayer {
                id: Uuid::parse_str("9b052bca-50b4-47b1-8f6a-8a51e3310d86").unwrap(),
                name: "leftlayer1".to_string(),
                portion: 1800.0 / 4400.0,
                layer: 1,
            },
            NodeLayer {
                id: Uuid::parse_str("8cac2286-87d0-4df3-b6f7-5c86c4fa928c").unwrap(),
                name: "Root".to_string(),
                portion: 1.0,
                layer: 0,
            },
        ];
        let actual_order = Data::get_paint_order(&hold);
        assert_eq!(
            expected_order, actual_order,
            "\nExpected: \n{:?}\nActual:\n{:?}\n",
            expected_order, actual_order
        );
    }

    #[test]
    fn jumbled_input_uneven_tree() {
        let data: Vec<FileRow> = vec![
            FileRow {
                file: File {
                    id: Uuid::parse_str("fc50112e-5f9d-4ebf-b6a8-023ba619fd0f").unwrap(),
                    parent: Uuid::parse_str("9b052bca-50b4-47b1-8f6a-8a51e3310d86").unwrap(),
                    name: "Left3".to_string(),
                    file_type: FileType::Document,
                    last_modified: 1693063210788,
                    last_modified_by: "parth".to_string(),
                    shares: [].to_vec(),
                },
                size: 800,
            },
            FileRow {
                file: File {
                    id: Uuid::parse_str("8cac2286-87d0-4df3-b6f7-5c86c4fa928c").unwrap(),
                    parent: Uuid::parse_str("8cac2286-87d0-4df3-b6f7-5c86c4fa928c").unwrap(),
                    name: "Root".to_string(),
                    file_type: FileType::Folder,
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
                    name: "Left1".to_string(),
                    file_type: FileType::Folder,
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
                    name: "Left2".to_string(),
                    file_type: FileType::Folder,
                    last_modified: 1693063210788,
                    last_modified_by: "parth".to_string(),
                    shares: [].to_vec(),
                },
                size: 1000,
            },
            FileRow {
                file: File {
                    id: Uuid::parse_str("6c1cb978-7c4e-4d83-825a-477287f89c69").unwrap(),
                    parent: Uuid::parse_str("1c890596-1df9-4638-b0c1-ec77fdaa7a49").unwrap(),
                    name: "Right2".to_string(),
                    file_type: FileType::Document,
                    last_modified: 1693063210788,
                    last_modified_by: "parth".to_string(),
                    shares: [].to_vec(),
                },
                size: 2000,
            },
        ];
        let hold = Data::init(data);
        let actual_order = Data::get_paint_order(&hold);
        let expected_order: Vec<NodeLayer> = vec![
            NodeLayer {
                id: Uuid::parse_str("fc50112e-5f9d-4ebf-b6a8-023ba619fd0f").unwrap(),
                name: "Left3".to_string(),
                portion: 800.0 / 5800.0,
                layer: 3,
            },
            NodeLayer {
                id: Uuid::parse_str("9b052bca-50b4-47b1-8f6a-8a51e3310d86").unwrap(),
                name: "Left2".to_string(),
                portion: 1800.0 / 5800.0,
                layer: 2,
            },
            NodeLayer {
                id: Uuid::parse_str("6c1cb978-7c4e-4d83-825a-477287f89c69").unwrap(),
                name: "Right2".to_string(),
                portion: 2000.0 / 5800.0,
                layer: 2,
            },
            NodeLayer {
                id: Uuid::parse_str("1c890596-1df9-4638-b0c1-ec77fdaa7a49").unwrap(),
                name: "Left1".to_string(),
                portion: 4800.0 / 5800.0,
                layer: 1,
            },
            NodeLayer {
                id: Uuid::parse_str("8cac2286-87d0-4df3-b6f7-5c86c4fa928c").unwrap(),
                name: "Root".to_string(),
                portion: 1.0,
                layer: 0,
            },
        ];
        assert_eq!(
            expected_order, actual_order,
            "\nExpected: \n{:?}\nActual:\n{:?}\n",
            expected_order, actual_order
        );
    }
}
