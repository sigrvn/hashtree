#![allow(dead_code)]
use std::collections::VecDeque;
use std::io::prelude::*;
use sha2::Digest;

/// A node from the `HashTree`.
#[derive(Debug, Clone)]
struct Node {
    pub hash: String,
    pub index: usize,
    pub left: Option<usize>,
    pub right: Option<usize>,
}

// The structure of the HashTree is as follows:
// The `nodes` VecDeque contains the nodes in this order:
// * The first `num_blocks` indices represent the actual nodes
// of the blocks of the file that was hashed.
// * The subsequent blocks until the last index contain the
// node parents of the blocks
// * The last index holds the root of the tree.

/// A Merkle-tree.
#[derive(Debug, Clone)]
pub struct HashTree {
    nodes: VecDeque<Node>,
    num_blocks: usize,
    block_size: usize,
}

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
    /// let tree = HashTree::new(BLOCK_SIZE);
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
    /// const BLOCK_SIZE: usize = 1;
    /// let mut data = vec![0u8, 1u8];
    /// if let Ok(tree) = HashTree::new(BLOCK_SIZE).from_data(&mut data.as_slice()) {
    ///     assert!(tree.num_blocks() == 2);
    ///     assert!(tree.num_nodes() == 3);
    /// };
    /// ```
    /// The example above splits the data into 1-byte blocks and computes 
    /// their SHA256 digests.
    pub fn from_data<R: Read>(mut self, data: &mut R) -> Result<Self, Box<dyn std::error::Error>> {
        let mut buf = Vec::with_capacity(self.block_size);
        let mut index = 0;

        loop {
            let mut chunk = data.take(self.block_size as u64);
            if chunk.read_to_end(&mut buf)? == 0 { break; }

            let mut hasher = sha2::Sha256::new();
            hasher.update(&buf);
            let hash = String::from_utf8(hasher.finalize().to_vec())?;

            let node = Node { hash, index, left: None, right: None };
            self.nodes.push_back(node);
            index += 1;

            buf.clear();
        }

        // If there are an odd number of blocks, we need to clone the last block in order to 
        // build the tree properly
        if self.nodes.len() % 2 == 1 {
            self.nodes.push_back(self.nodes.back().unwrap().clone());
        }

        self.build(self.nodes.clone())?;
        Ok(self)
    }

    fn build(&mut self, mut unprocessed_nodes: VecDeque<Node>) -> Result<(), Box<dyn std::error::Error>> {
        let mut nodes_to_process = match unprocessed_nodes.len() {
            1 => return Ok(()), // We only have the root left
            n => n,
        };

        let mut parents = VecDeque::<Node>::new();
        while nodes_to_process > 0 {
            let index = self.nodes.len();
            let n1 = unprocessed_nodes.pop_front().unwrap();
            let n2 = unprocessed_nodes.pop_front().unwrap();
            let merged_hash = format!("{:x?}{:x?}", n1.hash, n2.hash);

            let mut hasher = sha2::Sha256::new();
            hasher.update(merged_hash);
            let parent_hash = String::from_utf8(hasher.finalize().to_vec())?;

            let parent = Node { 
                hash: parent_hash, 
                index, 
                left: Some(n1.index),
                right: Some(n2.index)
            };

            parents.push_back(parent.clone());
            self.nodes.push_back(parent);
            nodes_to_process -= 1;
        }

        self.build(parents)
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
    /// let tree = HashTree::new(BLOCK_SIZE);
    /// assert_eq!(tree.is_empty(), true);
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
            return Some(&root.hash)
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

        my_root == other_root
    }
}
