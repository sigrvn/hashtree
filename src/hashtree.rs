#![allow(dead_code)]
use std::collections::VecDeque;
use std::path::PathBuf;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, Result};
use std::fmt;

const BLOCK_SIZE: usize = 4096;

#[derive(Debug, Clone)]
pub struct HashTreeNode {
    block_tag: String,
    hash: md5::Digest,
    left: Option<Box<HashTreeNode>>,
    right: Option<Box<HashTreeNode>>,
}

impl HashTreeNode {
    pub fn new(block_tag: String, hash: md5::Digest) -> Self {
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
        write!(f, "[Block {}]\nhash: {:?}", self.block_tag, self.hash)
    }
}

pub struct HashTree {
    root: Option<Box<HashTreeNode>>,
    num_nodes: usize,
    num_blocks: usize,
}

impl HashTree {
    pub fn new(num_blocks: usize) -> Self {
        Self {
            root: None,
            num_nodes: 0,
            num_blocks,
        }
    }

    pub fn from_file(path: PathBuf) -> Result<Self> {
        let file = File::open(path)?;

        let mut buf: [u8; BLOCK_SIZE] = [0u8; BLOCK_SIZE];
        let mut reader = BufReader::new(file);

        let mut nodes = VecDeque::<Box<HashTreeNode>>::new();
        let mut block_num: usize = 0;
        let mut total: usize = 0;

        let mut done = false;
        loop {
            match reader.read(&mut buf) {
                Ok(bytes_read) => {
                    if bytes_read < BLOCK_SIZE { 
                        println!("last read of size {} bytes", bytes_read);
                        done = true;
                    } else {
                        println!("{} bytes read into buf", bytes_read);
                    }

                    let block_tag = format!("B{}", block_num);
                    let hash = md5::compute(buf);
                    println!("hash for block {}: {:?}", block_tag, hash);
                    let node = Box::new(HashTreeNode::new(block_tag, hash));
                    nodes.push_back(node);

                    total += bytes_read; 
                    block_num += 1; 

                    if done {
                        break;
                    }
                },
                Err(error) => {
                    eprintln!("{}", error);
                    break;
                }
            }
        } 

        println!("Read a total of {} blocks ({} bytes) from file", block_num, total);
        println!("Building HashTree...");
        println!("# blocks: {}", block_num);

        let mut hashtree = HashTree::new(block_num);
        hashtree.num_nodes += nodes.len();
        hashtree.build(nodes);
        Ok(hashtree)
    }

    fn build(&mut self, mut nodes: VecDeque::<Box<HashTreeNode>>) {
        let nodes_to_process = nodes.len();
        if nodes_to_process == 1 {
            return;
        }

        let mut parents = VecDeque::<Box<HashTreeNode>>::new();
        let mut processed = 0;
        while processed < nodes_to_process {
            //println!("processed: {}, nodes_to_process: {}", processed, nodes_to_process);
            let n1 = nodes.pop_front().unwrap();
            let n2 = nodes.pop_front().unwrap_or(n1.clone());

            let block_tag = format!("{} + {}", n1.block(), n2.block());
            let merged_hash = format!("{:?}{:?}", n1.hash(), n2.hash());
            let parent_hash = md5::compute(merged_hash.as_bytes());
            //println!("hash for block {}: {:?}", block_tag, parent_hash);

            let mut parent = Box::new(HashTreeNode::new(block_tag, parent_hash));
            parent.left = Some(n1);
            parent.right = Some(n2);

            parents.push_back(parent);
            processed += 2;
            self.num_nodes += 1;
        }

        return self.build(parents);
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
