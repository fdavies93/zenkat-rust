use serde::{Deserialize, Serialize};

use crate::common::node::{Node, NodeData, NodeType};
use std::collections::{HashMap, VecDeque};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tree {
    pub path: String,
    pub root_node: String,
    pub nodes: HashMap<String, Node>,
}

impl Tree {
    pub fn new() -> Tree {
        return Tree {
            path: String::new(),
            root_node: String::new(),
            nodes: HashMap::new(),
        };
    }

    pub async fn load(path: String, traverse_symbolic: bool) -> Option<Tree> {
        let mut tree: Tree = Tree::new();
        tree.path = path.clone();
        let mut queue: VecDeque<String> = VecDeque::new();
        let mut parents: HashMap<String, String> = HashMap::new();

        queue.push_back(path.clone());

        while queue.len() > 0 {
            let path_str = queue.pop_front()?;
            let cur_path = Path::new(&path_str);
            let mut cur_node: Node = Node::new(NodeType::None);
            let cur_node_id: String = cur_node.id.clone();
            println!("{}", path_str);
            if !cur_path.exists() {
                continue;
            } else if cur_path.is_symlink() && !traverse_symbolic {
                continue;
            } else if cur_path.is_file() {
                if cur_path.extension()? != "md" {
                    continue;
                }
                cur_node.node_type = NodeType::DOCUMENT;
                cur_node.data = NodeData::DocumentData {
                    path: cur_path.to_str()?.into(),
                    loaded: false,
                };
                tree.nodes.insert(cur_node_id.clone(), cur_node);
            } else if cur_path.is_dir() {
                cur_node.node_type = NodeType::DIRECTORY;
                cur_node.data = NodeData::DirectoryData {
                    path: cur_path.to_str()?.into(),
                };

                let children = cur_path.read_dir().ok()?;
                for child in children {
                    let c_path = child.ok()?.path();
                    let c_path_str: String = c_path.to_str()?.into();
                    queue.push_back(c_path_str.clone());
                    parents.insert(c_path_str, cur_node_id.clone());
                }
                tree.nodes.insert(cur_node_id.clone(), cur_node);
            }
            // link the parents to the children by ID using hash table
            let parent_id = parents.get(&path_str);
            match parent_id {
                Some(id) => {
                    let parent = tree.nodes.get_mut(id)?;
                    parent.children.push(cur_node_id.clone());
                }
                None => tree.root_node = cur_node_id.clone(),
            }
        }
        return Some(tree);
    }
}
