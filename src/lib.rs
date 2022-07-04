pub mod tree;
pub use tree::HashTree;

#[cfg(test)]
mod tests {
    use crate::HashTree;

    #[test]
    fn one_byte_block_size() {
        const BLOCK_SIZE: usize = 1;
        let data = vec![0u8, 1u8];
        let tree = HashTree::new(BLOCK_SIZE).from_data(&mut data.as_slice()).unwrap();
        assert!(tree.num_blocks() == 2);
        assert!(tree.num_nodes() == 3);
        assert_eq!(tree.root_hash().unwrap(), 
            "30e1867424e66e8b6d159246db94e3486778136f7e386ff5f001859d6b8484ab");
    }

    #[test]
    fn one_byte_clone_compare() {
        const BLOCK_SIZE: usize = 1;
        let data = vec![0u8, 1u8];
        if let Ok(tree) = HashTree::new(BLOCK_SIZE).from_data(&mut data.as_slice()) {
            assert!(tree.num_blocks() == 2);
            assert!(tree.num_nodes() == 3);
            assert_eq!(tree.root_hash().unwrap(), 
                "30e1867424e66e8b6d159246db94e3486778136f7e386ff5f001859d6b8484ab");

            let tree_clone = tree.clone();
            assert!(tree == tree_clone);
        }
    }

    #[test]
    fn odd_block_count() {
        const BLOCK_SIZE: usize = 1000;
        let data = vec![42u8; 3000];
        if let Ok(tree) = HashTree::new(BLOCK_SIZE).from_data(&mut data.as_slice()) {
            assert!(tree.num_blocks() == 3);
            assert!(tree.num_nodes() == 7);
        }
    }
}

