use crate::common::node::{Node, NodeType};
use std::collections::VecDeque;
use std::path::Path;
use tokio::process::Command;

#[derive(Debug)]
pub struct TreeStore {
    trees: Vec<Node>,
}

impl TreeStore {
    pub fn load_tree(path: &Path, traverse_symbolic: bool) -> Option<Node> {
        if path.is_symlink() && !traverse_symbolic {
            return None;
        }
        let mut output: Node = Node::new(String::new(), NodeType::DOCUMENT);

        if path.is_dir() {
            output.block_type = NodeType::DIRECTORY;
            let children = path.read_dir().ok()?;
            let mut child_nodes = vec![];

            for child in children {
                let c_path = child.ok()?;
                let node_choice = TreeStore::load_tree(c_path.path().as_path(), traverse_symbolic);

                if node_choice.is_some() {
                    child_nodes.push(node_choice.unwrap());
                }
            }

            output.blocks = child_nodes;
        }

        if path.is_file() && path.extension().unwrap() != "md" {
            return None;
        }

        let path_str = String::from(path.as_os_str().to_str().unwrap());
        output.data.insert(String::from("path"), path_str);

        return Some(output);
    }

    pub fn get_trees(&self) -> &Vec<Node> {
        return &self.trees;
    }

    pub fn load(paths: Vec<String>, traverse_symbolic: bool) -> TreeStore {
        let mut store: TreeStore = TreeStore { trees: vec![] };

        for path_str in paths {
            let path = Path::new(&path_str);

            if !path.try_exists().is_ok_and(|x| x) {
                println!(
                    "Couldn't access path at {} when loading tree store.",
                    path_str.as_str()
                );
                continue;
            }

            let tree_option = TreeStore::load_tree(path, traverse_symbolic);

            if tree_option.is_some() {
                store.trees.push(tree_option.unwrap());
            }
        }

        return store;
    }

    // likely needs to work on references
    pub fn get_all_documents_mut(&mut self) -> Vec<&mut Node> {
        let mut docs: Vec<&mut Node> = vec![];

        let mut remaining: VecDeque<&mut Node> = VecDeque::new();
        for root in (self.trees).iter_mut() {
            remaining.push_back(root);
        }

        loop {
            if remaining.len() <= 0 {
                return docs;
            }
            let next = remaining.pop_front().unwrap();
            if next.block_type == NodeType::DOCUMENT {
                docs.push(next);
                continue;
            }

            for block in (next.blocks).iter_mut() {
                remaining.push_back(block)
            }
        }
    }

    pub fn query(&mut self, query: &String) -> Vec<&Node> {
        let mut collected = vec![];
        let mut queue = VecDeque::new();
        for tree in self.trees.iter() {
            queue.push_back(tree);
        }
        while queue.len() > 0 {
            let next = queue.pop_front().unwrap();
            if next.type_as_string() == query.as_str() {
                collected.push(next)
            }
            for child in &next.blocks {
                queue.push_back(child);
            }
        }
        return collected;
    }

    pub async fn hydrate_node(node: &mut Node, parser: &String) -> Result<(), std::io::ErrorKind> {
        if node.block_type != NodeType::DOCUMENT {
            return Err(std::io::ErrorKind::InvalidInput);
        }

        let output = Command::new(parser.as_str())
            .arg(node.data.get("path").unwrap())
            .output()
            .await
            .expect("");

        let parsed_json = String::from_utf8(output.stdout).expect("");
        let parsed_obj: Node = serde_json::from_str(parsed_json.as_str()).unwrap();

        node.raw = parsed_obj.raw;
        node.blocks = parsed_obj.blocks;

        return Ok(());
    }

    pub async fn hydrate_docs(docs: Vec<&mut Node>, processes: usize, parser: &String) {
        let mut children: VecDeque<_> = VecDeque::new();
        let mut pending_doc: VecDeque<&mut Node> = VecDeque::new();
        let mut remaining_docs: Vec<&mut Node> = vec![];

        for doc in docs {
            remaining_docs.push(doc);
        }

        loop {
            if remaining_docs.len() == 0 && children.len() == 0 {
                break;
            }
            loop {
                if children.len() == processes || remaining_docs.len() == 0 {
                    break;
                }
                let next_doc = remaining_docs.pop().expect("");

                let parser_clone = parser.clone();
                let next_doc_path = next_doc.data.get("path").unwrap().clone();

                pending_doc.push_back(next_doc);

                let child = tokio::spawn(Command::new(parser_clone).arg(next_doc_path).output());

                children.push_back(child);
            }
            let output = children.pop_front().unwrap().await.unwrap().unwrap();

            let parsed_json = String::from_utf8(output.stdout).expect("");
            let parsed_obj: Node = serde_json::from_str(parsed_json.as_str()).unwrap();

            let finished_doc = pending_doc.pop_front().unwrap();

            finished_doc.blocks = parsed_obj.blocks;
            finished_doc.raw = parsed_obj.raw;
        }
    }
}
