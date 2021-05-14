#![allow(dead_code)]
use std::collections::VecDeque;
use std::io::prelude::*;
use std::io::Result;
use std::fmt::Debug;

type NodePtr<T> = Box<HashTreeNode<T>>;

#[derive(Debug, Clone)]
pub struct HashTreeNode<T>
where
    T: Debug + Clone + PartialEq 
{
    block_tag: String,
    hash: T,
    left: Option<NodePtr<T>>,
    right: Option<NodePtr<T>>,
}

impl<T> HashTreeNode<T> 
where
    T: Debug + Clone + PartialEq 
{
    pub fn new(block_tag: String, hash: T) -> Self {
        Self {
            block_tag,
            hash,
            left: None,
            right: None,
        }
    }

    pub fn block(&self) -> &String {
        return &self.block_tag
    }

    pub fn hash(&self) -> &T {
        return &self.hash
    }

    pub fn print_inorder(&self) {
        if let Some(ref left) = self.left {
            left.print_inorder();
        }
        println!("{}", self);
        if let Some(ref right) = self.right {
            right.print_inorder();
        }
    }

    pub fn print_preorder(&self) {
        println!("{}", self);
        if let Some(ref left) = self.left {
            left.print_preorder();
        }
        if let Some(ref right) = self.right {
            right.print_preorder();
        }
    }

    pub fn print_postorder(&self) {
        if let Some(ref left) = self.left {
            left.print_postorder();
        }
        if let Some(ref right) = self.right {
            right.print_postorder();
        }
        println!("{}", self);
    }
}

impl<T> std::fmt::Display for HashTreeNode<T> 
where
    T: Debug + Clone + PartialEq
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[Block {}]\nhash: {:?}", self.block_tag, self.hash)
    }
}

#[derive(Debug)]
pub struct HashStrategy<T, F> 
where
    T: Debug + Clone + PartialEq,
    F: Fn(&[u8]) -> T,
{
    block_size: usize,
    hash_function: F,
}

// TODO: Implement custom hash function strategy through 
// a closure to HashStrategy::new()
impl<T, F> HashStrategy<T, F>
where
    T: Debug + Clone + PartialEq,
    F: Fn(&[u8]) -> T,
{
    pub fn new(block_size: usize, hash_function: F) -> Self {
        Self {
            block_size,
            hash_function
        }
    }
}

#[derive(Clone)]
pub struct HashTree<T> 
where
    T: Debug + Clone + PartialEq,
{
    root: Option<NodePtr<T>>,
    num_nodes: usize,
    num_blocks: usize,
}

impl<T> HashTree<T> 
where
    T: Debug + Clone + PartialEq,
{
    pub fn new<R, F>(reader: &mut R, strategy: HashStrategy<T, F>) -> Result<Self>
    where 
        R: Read,
        F: Fn(&[u8]) -> T,
    {
        let mut buf = vec![0u8; strategy.block_size];
        let mut nodes = VecDeque::<NodePtr<T>>::new();
        let mut block_num: usize = 0;
        let mut total: usize = 0;

        let mut done = false;
        while !done {
            let bytes_read = reader.read(&mut buf)?;
            println!("bytes_read: {}", bytes_read);
            //let bytes_read = buf.len();

            // Last read from reader here
            if bytes_read < strategy.block_size {
                println!("last read of size {} bytes", bytes_read);
                done = true;
            } else {
                println!("{} bytes read into buf", bytes_read);
            }

            let block_tag = format!("{}", block_num);
            let hash = (strategy.hash_function)(&buf);
            println!("hash for block {}: {:?}", block_tag, hash);
            let node = Box::new(HashTreeNode::new(block_tag, hash));
            nodes.push_back(node);

            total += buf.len(); 
            block_num += 1; 
        }

        println!("Read a total of {} blocks ({} bytes) from file", block_num, total);
        println!("Building HashTree...");
        println!("# blocks: {}", block_num);

        let mut hashtree = HashTree::<T>{
            root: None,
            num_nodes: nodes.len(),
            num_blocks: 0,
        };

        hashtree.build(nodes, &strategy);
        Ok(hashtree)
    }

    fn build<F>(&mut self, mut nodes: VecDeque::<NodePtr<T>>, strategy: &HashStrategy<T, F>)
    where
        F: Fn(&[u8]) -> T 
    {
        let nodes_to_process = nodes.len();
        if nodes_to_process == 1 {
            self.root = nodes.pop_front();
            return;
        }

        let hs = strategy;
        let mut parents = VecDeque::<NodePtr<T>>::new();
        let mut processed = 0;
        while processed < nodes_to_process {
            println!("processed: {}, nodes_to_process: {}", processed, nodes_to_process);
            let n1 = nodes.pop_front().unwrap();
            let n2 = nodes.pop_front().unwrap_or(n1.clone());

            let block_tag = format!("{} + {}", n1.block(), n2.block());
            let merged_hash = format!("{:?}{:?}", n1.hash(), n2.hash());
            let parent_hash = (hs.hash_function)(merged_hash.as_bytes());
            println!("hash for block {}: {:?}", block_tag, parent_hash);

            let mut parent = Box::new(HashTreeNode::new(block_tag, parent_hash));
            parent.left = Some(n1);
            parent.right = Some(n2);

            parents.push_back(parent);
            processed += 2;
            self.num_nodes += 1;
        }

        return self.build(parents, strategy);
    }

    pub fn root(&self) -> Option<&NodePtr<T>> {
        match self.root {
            Some(ref root) => {
                Some(root) 
            },
            None => {
                None
            }
        } 
    }

    pub fn find(&self, hash: T) -> Option<&NodePtr<T>> {
        match self.root {
            Some(ref root) => {
                Some(root)
            },
            None => None
        }
    }

    pub fn nodes(&self) -> usize {
        self.num_nodes
    }

    pub fn blocks(&self) -> usize {
        self.num_blocks
    }
}

impl<T> PartialEq for HashTree<T> 
where 
    T: Debug + Clone + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        if let Some(root) = &self.root {
            if let Some(other_root) = other.root() {
                return root.hash() == other_root.hash()
            }
        }
        false
    }
}

#[cfg(test)]
mod test {
    use std::fs::File;
    use std::path::PathBuf;
    use crate::*;

    #[test]
    fn new_big_test() {
        let mut test_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_file_path.push("examples/lorem.txt");
        let mut file = File::open(test_file_path).unwrap();

        match HashTree::new(
            &mut file, 
            HashStrategy::new(4096, |x| md5::compute(x))
        ) {
            Ok(tree) => {
                println!("Tree has {} blocks and {} nodes", tree.blocks(), tree.nodes());
                match tree.root() {
                    Some(root) => {
                        println!("ROOT HASH: {:?}", root.hash());
                        root.print_postorder();
                    },
                    None => {}
                } 
            },
            Err(error) => {
                eprintln!("{}", error);
            }
        }
    }

    #[test]
    fn compare_hashtrees() {
        let mut test_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_file_path.push("examples/8096.txt");
        let mut file = File::open(test_file_path).unwrap();

        let tree = match HashTree::new(
            &mut file,
            HashStrategy::new(4096, |x| md5::compute(x))
        ) {
            Ok(tree) => tree,
            Err(error) => {
                eprintln!("{}", error);
                std::process::exit(1);
            }
        };

        let tree_clone = tree.clone();
        if tree == tree_clone {
            println!("Trees are the same!");
        } else {
            println!("Trees are different!");
        }
    }

    #[test]
    fn new_medium_test() {
        let mut test_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_file_path.push("examples/8096.txt");
        let mut file = File::open(test_file_path).unwrap();

        match HashTree::new(
            &mut file, 
            HashStrategy::new(4096, |x| md5::compute(x))
        ) {
            Ok(tree) => {
                println!("Tree has {} blocks and {} nodes", tree.blocks(), tree.nodes());
                match tree.root() {
                    Some(root) => {
                        println!("ROOT HASH: {:?}", root.hash());
                        root.print_postorder();
                    },
                    None => {}
                } 
            },
            Err(error) => {
                eprintln!("{}", error);
            }
        }
    }
}
