#![allow(dead_code)]
use std::collections::VecDeque;
use std::path::PathBuf;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, Result};
use std::fmt;

const BLOCK_SIZE: usize = 4096;

pub struct HashTreeNode {
    block: usize,
    hash: md5::Digest,
    left: Option<Box<HashTreeNode>>,
    right: Option<Box<HashTreeNode>>,
}

impl HashTreeNode {
    pub fn new(block: usize, hash: md5::Digest) -> Self {
        Self {
            block,
            hash,
            left: None,
            right: None,
        }
    }

    pub fn block(&self) -> usize {
        return self.block
    }

    pub fn hash(&self) -> &md5::Digest {
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

impl fmt::Display for HashTreeNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[Block {}]\nhash: {:?}", self.block, self.hash)
    }
}

pub struct HashTree {
    root: Option<Box<HashTreeNode>>,
    num_nodes: usize,
    num_blocks: usize,
}

impl HashTree {
    pub fn new() -> Self {
        Self {
            root: None,
            num_nodes: 0,
            num_blocks: 0,
        }
    }

    pub fn from_file(path: PathBuf) -> Result<Self> {
        let file = File::open(path)?;

        let mut buf: [u8; BLOCK_SIZE] = [0u8; BLOCK_SIZE];
        let mut reader = BufReader::new(file);

        let mut nodes_queue = VecDeque::<Box<HashTreeNode>>::new();
        let mut blocks: usize = 0;
        let mut total: usize = 0;

        loop {
            match reader.read(&mut buf) {
                Ok(bytes_read) => {
                   if bytes_read < BLOCK_SIZE { 
                        println!("last read of size {} bytes", bytes_read);
                        break; 
                    } else {
                        println!("{} bytes read into buf", bytes_read);
                    }

                    blocks += 1; 
                    total += bytes_read; 

                    let hash = md5::compute(buf);
                    println!("hash for block {}: {:?}", blocks, hash);
                    let node = Box::new(HashTreeNode::new(blocks, hash));
                    nodes_queue.push_back(node);
                },
                Err(error) => {
                    eprintln!("{}", error);
                    break;
                }
            }
        } 

        println!("Read a total of {} blocks ({} bytes) from file", blocks, total);
        println!("Building HashTree...");
        println!("# blocks: {}", blocks);

        let mut nodes: usize = nodes_queue.len();
        while nodes_queue.len() != 1 {
            println!("# nodes_queue to process: {}", nodes_queue.len());

            let n1 = nodes_queue.pop_front().unwrap();
            let n2 = nodes_queue.pop_front().unwrap();

            let merged_hash = format!("{:?}{:?}", n1.hash(), n2.hash());
            println!("merged hash: {:?}", merged_hash);

            let parent_hash = md5::compute(merged_hash.as_bytes());
            println!("parent hash: {:?}", parent_hash);

            let mut parent = Box::new(HashTreeNode::new(0, parent_hash));
            parent.left = Some(n1);
            parent.right = Some(n2);

            nodes_queue.push_back(parent);
            nodes += 1;
        }
        Ok(Self { root: nodes_queue.pop_front(), num_nodes: nodes, num_blocks: blocks })
    }

    pub fn root(&self) -> Option<&Box<HashTreeNode>> {
        match self.root {
            Some(ref root) => {
                Some(root) 
            },
            None => {
                None
            }
        } 
    }

    pub fn find(&self, hash: md5::Digest) {

    }

    pub fn nodes(&self) -> usize {
        self.num_nodes
    }

    pub fn blocks(&self) -> usize {
        self.num_blocks
    }

    pub fn is_empty(&self) -> bool {
        match self.root {
            Some(_) => {
                false 
            },
            None => {
                true 
            }
        } 
    }
}
