#![allow(dead_code)]
use std::collections::VecDeque;
use std::io::prelude::*;
use std::io::Result;
use std::fmt::{Debug, Display};

type NodePtr<T> = Box<HashTreeNode<T>>;

/// A node from the `HashTree`.
#[derive(Debug, Clone)]
pub struct HashTreeNode<T>
where
    T: Debug + Clone + PartialEq 
{
    hash: T,
    left: Option<NodePtr<T>>,
    right: Option<NodePtr<T>>,
}

impl<T> HashTreeNode<T> 
where
    T: Debug + Clone + PartialEq 
{
    pub fn new(hash: T) -> Self {
        Self {
            hash,
            left: None,
            right: None,
        }
    }

    pub fn hash(&self) -> &T {
        &self.hash
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

impl<T> Display for HashTreeNode<T> 
where
    T: Debug + Clone + PartialEq
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[hash {:?}]", self.hash)
    }
}

/// A struct that defines how the `HashTree` should partition and hash
/// the blocks of data.
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

/// A Merkle-tree.
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
    /// Constructs a new `HashTree<T>` from a mutable object
    /// that implements the `Read` trait and a `HashStrategy<T, F>`.
    /// Returns an `Error` value if the function failed to read from
    /// the given object.
    ///
    /// # Examples
    ///
    /// ```
    /// #![allow(dead_code)]
    /// use hashtree::{HashTree, HashStrategy};
    /// use md5::*;
    ///
    /// fn main() {
    ///     let mut data = vec![0u8, 1u8];
    ///     let tree = HashTree::new(
    ///         &mut &data[..],
    ///         HashStrategy::new(1, |x| md5::compute(x))
    ///     );
    /// }
    /// ```
    /// The example above uses a `HashStrategy` that splits the data
    /// into 1-byte blocks and computes their MD5 digests for the
    /// `HashTree`.
    pub fn new<R, F>(reader: &mut R, strategy: HashStrategy<T, F>) -> Result<Self>
    where 
        R: Read,
        F: Fn(&[u8]) -> T,
    {
        let mut buf = vec![0u8; strategy.block_size];
        let mut nodes = VecDeque::<NodePtr<T>>::new();
        let mut block_num: usize = 0;

        let mut done = false;
        while !done {
            let bytes_read = reader.read(&mut buf)?;

            if bytes_read < strategy.block_size {
                done = true;
            }

            let hash = (strategy.hash_function)(&buf);
            let node = Box::new(HashTreeNode::new(hash));
            nodes.push_back(node);

            block_num += 1; 
        }

        let mut hashtree = HashTree::<T>{
            root: None,
            num_nodes: nodes.len(),
            num_blocks: block_num,
        };

        hashtree.build(nodes, strategy);
        Ok(hashtree)
    }

    fn build<F>(&mut self, mut nodes: VecDeque<NodePtr<T>>, strategy: HashStrategy<T, F>)
    where
        F: Fn(&[u8]) -> T 
    {
        let nodes_to_process = nodes.len();
        if nodes_to_process == 1 {
            self.root = nodes.pop_front();
            return;
        }

        let mut parents = VecDeque::<NodePtr<T>>::new();
        let mut processed = 0;
        while processed < nodes_to_process {
            let n1 = nodes.pop_front().unwrap();
            let n2 = nodes.pop_front().unwrap_or(n1.clone());

            let merged_hash = format!("{:?}{:?}", n1.hash(), n2.hash());
            let parent_hash = (strategy.hash_function)(merged_hash.as_bytes());

            let mut parent = Box::new(HashTreeNode::new(parent_hash));
            parent.left = Some(n1);
            parent.right = Some(n2);

            parents.push_back(parent);
            processed += 2;
            self.num_nodes += 1;
        }

        return self.build(parents, strategy);
    }

    /// Returns the root of the `HashTree` as an `Option<&Box<HashTreeNode<T>>>`.
    pub fn root(&self) -> Option<&NodePtr<T>> {
        if let Some(ref root) = self.root {
            return Some(root)
        }
        None
    }

    pub fn find(&self, hash: T) -> Option<&NodePtr<T>> {
        if let Some(ref root) = self.root {
            return Some(root)
        }
        None
    }

    /// Returns the number of nodes in the `HashTree`.
    pub fn nodes(&self) -> usize {
        self.num_nodes
    }

    /// Returns the number of blocks that were used to
    /// construct the `HashTree`.
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
            if let Some(other_root) = &other.root {
                return root.hash == other_root.hash
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
