pub mod tree;
pub use tree::HashTree;

#[cfg(test)]
mod tests {
    use crate::HashTree;

    // Quick Sanity Check
    // SHA256 Hash for 0: 5feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9
    // SHA256 Hash for 1: 6b86b273ff34fce19d6b804eff5a3f5747ada4eaa22f1d49c01e52ddb7875b4b
    // SHA256 Merged Hash of above: fa13bb36c022a6943f37c638126a2c88fc8d008eb5a9fe8fcde17026807feae4
    #[test]
    fn one_byte_block_size() {
        const BLOCK_SIZE: usize = 1;
        let data = vec![0u8, 1u8];
        if let Ok(tree) = HashTree::new(BLOCK_SIZE).from_data(&mut data.as_slice()) {
            assert!(tree.num_blocks() == 2);
            assert!(tree.num_nodes() == 3);
            assert!(tree.root_hash().unwrap() == "fa13bb36c022a6943f37c638126a2c88fc8d008eb5a9fe8fcde17026807feae4");
        }
    }

    #[test]
    fn one_byte_clone_compare() {
        const BLOCK_SIZE: usize = 1;
        let data = vec![0u8, 1u8];
        if let Ok(tree) = HashTree::new(BLOCK_SIZE).from_data(&mut data.as_slice()) {
            assert!(tree.num_blocks() == 2);
            assert!(tree.num_nodes() == 3);
            assert!(tree.root_hash().unwrap() == "fa13bb36c022a6943f37c638126a2c88fc8d008eb5a9fe8fcde17026807feae4");

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

