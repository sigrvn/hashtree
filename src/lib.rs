#![allow(dead_code)]
use std::collections::VecDeque;
use std::io::prelude::*;
use std::fmt::{Debug, Display};
use sha2::Digest;

/// A node from the `HashTree`.
#[derive(Debug, Clone)]
struct Node {
    hash: String,
    index: usize,
    left: Option<usize>,
    right: Option<usize>,
}

impl Node {
    pub fn new(hash: String, index: usize) -> Self {
        Self {
            hash,
            index,
            left: None,
            right: None,
        }
    }

    pub fn hash(&self) -> &str {
        &self.hash
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn left(&self) -> Option<usize> {
        self.left
    }

    pub fn right(&self) -> Option<usize> {
        self.right
    }
}

/// A Merkle-tree.
#[derive(Debug, Clone)]
pub struct HashTree {
    nodes: VecDeque<Node>,
    num_blocks: usize,
    block_size: usize,
}

// The `nodes` VecDeque contains the nodes in this order:
// * The first `num_blocks` indices represent the actual nodes
// of the blocks of the file that was hashed.
// * The subsequent blocks until the last index contain the
// node parents of the blocks
// * The last index holds the root of the tree.

impl HashTree {
    /// Constructs a new empty `HashTree`.
    ///
    /// # Examples
    ///
    /// ```
    /// #![allow(dead_code)]
    /// use hashtree::HashTree;
    ///
    /// const BLOCK_SIZE: usize = 4096;
    ///
    /// fn main() {
    ///     let tree = HashTree::new(BLOCK_SIZE);
    /// }
    /// ```
    pub fn new(block_size: usize) -> Self {
        Self {
            nodes: VecDeque::new(), 
            num_blocks: 0,
            block_size,
        }
    }

    /// Constructs a new `HashTree` from a mutable object
    /// that implements the `Read` trait.
    /// Returns an `Error` value if the function failed to read from
    /// the given object.
    ///
    /// # Examples
    ///
    /// ```
    /// #![allow(dead_code)]
    /// use hashtree::HashTree;
    ///
    /// fn main() {
    ///     let block_size = 1;
    ///     let mut data = vec![0u8, 1u8];
    ///     if let Ok(tree) = HashTree::create(block_size, &mut data.as_slice()) {
    ///         assert!(tree.num_blocks() == 2);
    ///         assert!(tree.num_nodes() == 3);
    ///     };
    /// }
    ///
    /// ```
    /// The example above splits the data into 1-byte blocks and computes 
    /// their SHA256 digests.
    pub fn create<R: Read>(block_size: usize, data: &mut R) -> Result<Self, Box<dyn std::error::Error>> {
        let mut buf = vec![];
        let mut nodes = VecDeque::<Node>::new();
        let mut index = 0;

        loop {
            let mut chunk = data.take(block_size as u64);
            let n = chunk.read_to_end(&mut buf)?;
            if n == 0 {
                break;
            }

            let mut hasher = sha2::Sha256::new();
            hasher.update(&buf);
            let hash_as_bytes = hasher.finalize().to_vec();
            let hash = String::from_utf8(hash_as_bytes)?;

            let node = Node::new(hash, index);
            nodes.push_back(node);
            index += 1;

            buf.clear();
        }

        // If there are an odd number of blocks, we need to clone the last block in order to 
        // build the tree properly
        if nodes.len() % 2 == 1 {
            nodes.push_back(nodes.back().unwrap().clone());
        }

        let mut hashtree = HashTree {
            nodes: nodes.clone(), 
            num_blocks: index,
            block_size,
        };

        hashtree.build(nodes)?;
        Ok(hashtree)
    }

    fn build(&mut self, mut unprocessed_nodes: VecDeque<Node>) -> Result<(), Box<dyn std::error::Error>> {
        let nodes_to_process = unprocessed_nodes.len();
        if nodes_to_process == 1 {
            return Ok(());
        }

        let mut parents = VecDeque::<Node>::new();
        let mut processed = 0;
        while processed < nodes_to_process {
            let index = self.nodes.len();
            let n1 = unprocessed_nodes.pop_front().unwrap();
            let n2 = unprocessed_nodes.pop_front().unwrap();

            let merged_hash = format!("{:x?}{:x?}", n1.hash(), n2.hash());

            let mut hasher = sha2::Sha256::new();
            hasher.update(merged_hash);
            let parent_hash_as_bytes = hasher.finalize().to_vec();
            let parent_hash = String::from_utf8(parent_hash_as_bytes)?;

            let mut parent = Node::new(parent_hash, index);
            parent.left = Some(n1.index());
            parent.right = Some(n2.index());

            parents.push_back(parent.clone());
            self.nodes.push_back(parent);
            processed += 2;
        }

        return self.build(parents);
    }

    // TODO: Implement ability to add data manually and reconstruct HashTree on the fly 
    pub fn insert<R: Read>(&mut self, data: &mut R) {
        unimplemented!();
    }

    /// Recomputes the hashes and nodes of the `HashTree`. This method should be called
    /// after you are done manually inserting data via the `insert` method.
    pub fn update(&mut self) {
        unimplemented!();
    }

    /// Returns `true` if the `HashTree` is empty and `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// #![allow(dead_code)]
    /// use hashtree::HashTree;
    /// 
    /// const BLOCK_SIZE: usize = 4096;
    ///
    /// fn main() {
    ///     let tree = HashTree::new(BLOCK_SIZE);
    ///     assert_eq!(tree.is_empty(), true);
    /// }
    /// ```
    pub fn is_empty(&self) -> bool {
        if self.nodes.len() == 0 {
            return true
        }
        false
    }

    /// Returns the root of the `HashTree` as an `Option<&str>`.
    pub fn root_hash(&self) -> Option<&str> {
        if let Some(root) = self.nodes.back() {
            return Some(root.hash())
        };
        None
    }

    /// Returns the number of nodes in the `HashTree`.
    pub fn num_nodes(&self) -> usize {
        self.nodes.len()
    }

    /// Returns the number of blocks that were used to construct the `HashTree`.
    pub fn num_blocks(&self) -> usize {
        self.num_blocks
    }
}

impl PartialEq for HashTree {
    fn eq(&self, other: &Self) -> bool {
        let my_root = match self.root_hash() {
            Some(v) => v,
            None => {
                match other.root_hash() {
                    Some(_) => { return false },
                    None => { return true }
                };
            }
        };

        let other_root = match other.root_hash() {
            Some(v) => v,
            None => { return false; }
        };

        return my_root == other_root;
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

#[cfg(test)]
mod tests {
    use crate::HashTree;

    // Quick Sanity Check
    // SHA256 Hash for 0: 5feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9
    // SHA256 Hash for 1: 6b86b273ff34fce19d6b804eff5a3f5747ada4eaa22f1d49c01e52ddb7875b4b
    // SHA256 Merged Hash of above: fa13bb36c022a6943f37c638126a2c88fc8d008eb5a9fe8fcde17026807feae4
    #[test]
    fn one_byte_test() {
        let block_size = 1;
        let data = vec![0u8, 1u8];
        if let Ok(tree) = HashTree::create(block_size, &mut data.as_slice()) {
            println!("Tree has {} blocks with {} nodes", tree.num_blocks(), tree.num_nodes());
            assert!(tree.num_blocks() == 2);
            assert!(tree.num_nodes() == 3);
            assert!(tree.root_hash().unwrap() == "fa13bb36c022a6943f37c638126a2c88fc8d008eb5a9fe8fcde17026807feae4");
        }
    }

    #[test]
    fn one_byte_clone_test() {
        let block_size = 1;
        let data = vec![0u8, 1u8];
        if let Ok(tree) = HashTree::create(block_size, &mut data.as_slice()) {
            println!("Tree has {} blocks with {} nodes", tree.num_blocks(), tree.num_nodes());
            assert!(tree.num_blocks() == 2);
            assert!(tree.num_nodes() == 3);
            assert!(tree.root_hash().unwrap() == "fa13bb36c022a6943f37c638126a2c88fc8d008eb5a9fe8fcde17026807feae4");

            let tree_clone = tree.clone();
            assert!(tree == tree_clone);
        }
    }

    #[test]
    fn four_k_test() {
        let block_size = 1000;
        let data = vec![42u8; 3000];
        if let Ok(tree) = HashTree::create(block_size, &mut data.as_slice()) {
            assert!(tree.num_blocks() == 3);
            assert!(tree.num_nodes() == 7);
            println!("Root Hash: {}", tree.root_hash().unwrap());
        }
    }
}
