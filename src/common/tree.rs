use serde::{Deserialize, Serialize};

use crate::common::node::{Node, NodeData, NodeType};
use std::collections::{HashMap, VecDeque};
use std::path::Path;
use std::time::Instant;
use tokio::process::Command;
use tokio::task::JoinSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tree {
    pub name: String,
    pub path: String,
    pub root_node: String,
    pub nodes: HashMap<String, Node>,
}

impl Tree {
    // new functions:
    // splice (replace data of node and add children)
    // add_as_child (just make a child of an existing node)
    pub fn new() -> Tree {
        return Tree {
            name: String::new(),
            path: String::new(),
            root_node: String::new(),
            nodes: HashMap::new(),
        };
    }

    pub async fn load(name: String, path: String, traverse_symbolic: bool) -> Option<Tree> {
        let mut tree: Tree = Tree::new();
        tree.name = name.clone();
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

    pub async fn load_document(path: String, parser: String) -> Tree {
        let output = Command::new(parser.as_str())
            .arg(path)
            .output()
            .await
            .expect("");

        let parsed_json = String::from_utf8(output.stdout).expect("");
        let parsed_tree: Tree = serde_json::from_str(parsed_json.as_str()).unwrap();
        return parsed_tree;
    }

    pub async fn load_all_unloaded_docs(&mut self, parser: String) {
        // trigger full document load
        let mut set = JoinSet::new();
        let mut path_to_id: HashMap<String, String> = HashMap::new();

        for (_, node) in self.nodes.iter() {
            match node.data.clone() {
                NodeData::DocumentData { path, loaded } => {
                    if !loaded {
                        path_to_id.insert(path.clone(), node.id.clone());
                        set.spawn(Tree::load_document(path.clone(), parser.clone()));
                    }
                }
                _ => {}
            }
        }

        let mut counted = 0;
        let before = Instant::now();

        while let Some(res) = set.join_next().await {
            counted += 1;

            let mut doc_tree = Tree::new();
            match res {
                Ok(res_ok) => {
                    doc_tree = res_ok;
                }
                Err(_) => {
                    continue;
                }
            }

            let root_id = doc_tree.root_node.clone();
            let new_root = doc_tree.nodes.get(&root_id).unwrap();
            let path = doc_tree.path.clone();
            // copy data to original node, rather than replacing it (so we don't need to recalculate parent links)
            let og_node_id = path_to_id.get(&path).unwrap();
            let og_node = self.nodes.get_mut(og_node_id).unwrap();
            og_node.children = new_root.children.clone();
            match og_node.data.clone() {
                NodeData::DocumentData { path, loaded: _ } => {
                    og_node.data = NodeData::DocumentData { path, loaded: true };
                }
                _ => {}
            }

            for (node_id, node) in doc_tree.nodes.iter() {
                if node_id.clone() == root_id {
                    continue;
                }
                self.nodes.insert(node_id.clone(), node.clone());
            }
        }
        println!("Loaded {} documents in {:.4?}.", counted, before.elapsed());
    }
}
