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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub struct HashTree<T, F> 
where
    T: Debug + Clone + PartialEq,
    F: Fn(&[u8]) -> T,
{
    root: Option<NodePtr<T>>,
    num_nodes: usize,
    num_blocks: usize,
    strategy: HashStrategy<T, F>,
}

impl<T, F> HashTree<T, F> 
where
    T: Debug + Clone + PartialEq,
    F: Fn(&[u8]) -> T,
{
    /// Constructs a new empty `HashTree<T>` with a given
    /// `HashStrategy<T, F>`.
    ///
    /// # Examples
    ///
    /// ```
    /// #![allow(dead_code)]
    /// use hashtree::{HashTree, HashStrategy};
    /// use md5::*;
    ///
    /// const BLOCK_SIZE: usize = 4096;
    ///
    /// fn main() {
    ///     let tree = HashTree::new(
    ///         HashStrategy::new(BLOCK_SIZE, |data| {
    ///             md5::compute(data)
    ///         })
    ///     );
    /// }
    /// ```
    pub fn new(strategy: HashStrategy<T, F>) -> Self {
        Self {
            root: None,
            num_nodes: 0,
            num_blocks: 0,
            strategy,
        }
    }

    /// Constructs a new `HashTree<T, F>` from a mutable object
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
    ///     let tree = HashTree::create(
    ///         &mut data.as_slice(),
    ///         HashStrategy::new(1, |x| md5::compute(x))
    ///     );
    /// }
    /// ```
    /// The example above uses a `HashStrategy` that splits the data
    /// into 1-byte blocks and computes their MD5 digests for the
    /// `HashTree`.
    pub fn create<R>(data: &mut R, strategy: HashStrategy<T, F>) -> Result<Self>
    where 
        R: Read,
    {
        let mut buf = vec![0u8; strategy.block_size];
        let mut nodes = VecDeque::<NodePtr<T>>::new();
        let mut block_num: usize = 0;

        let mut done = false;
        while !done {
            let bytes_read = data.read(&mut buf)?;

            if bytes_read < strategy.block_size {
                done = true;
            }

            let hash = (strategy.hash_function)(&buf);
            let node = Box::new(HashTreeNode::new(hash));
            nodes.push_back(node);

            block_num += 1; 
        }

        let mut hashtree = HashTree::<T, F>{
            root: None,
            num_nodes: nodes.len(),
            num_blocks: block_num,
            strategy,
        };

        hashtree.build(nodes);
        Ok(hashtree)
    }

    fn build(&mut self, mut nodes: VecDeque<NodePtr<T>>) {
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
            let parent_hash = (self.strategy.hash_function)(merged_hash.as_bytes());

            let mut parent = Box::new(HashTreeNode::new(parent_hash));
            parent.left = Some(n1);
            parent.right = Some(n2);

            parents.push_back(parent);
            processed += 2;
            self.num_nodes += 1;
        }

        return self.build(parents);
    }

    /// 
    pub fn insert<R>(&mut self, data: &mut R) 
    where 
        R: Read
    {
    }

    /// Returns `true` if the `HashTree` is empty and `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// #![allow(dead_code)]
    /// use hashtree::{HashTree, HashStrategy};
    /// use md5::*;
    /// 
    /// const BLOCK_SIZE: usize = 4096;
    ///
    /// fn main() {
    ///     let tree = HashTree::new(
    ///         HashStrategy::new(BLOCK_SIZE, |data| {
    ///             md5::compute(data)
    ///         })
    ///     );
    ///
    ///     assert_eq!(tree.is_empty(), true);
    /// }
    /// ```
    pub fn is_empty(&self) -> bool {
        if let Some(_) = self.root {
            return false
        }
        true
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

    /// Returns the number of blocks that were used to construct the `HashTree`.
    pub fn blocks(&self) -> usize {
        self.num_blocks
    }
}

impl<T, F> PartialEq for HashTree<T, F> 
where 
    T: Debug + Clone + PartialEq,
    F: Fn(&[u8]) -> T,
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
