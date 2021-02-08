mod hashtree;

#[cfg(test)]
mod test {
    use crate::hashtree::HashTree;

    #[test]
    fn is_empty_test() {
        let my_tree = HashTree::new(); 
        assert_eq!(my_tree.is_empty(), true);
    }

    #[test]
    fn from_file_big_test() {
        use std::path::PathBuf;

        let mut test_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_file_path.push("target/debug/examples/lorem.txt");

        match HashTree::from_file(test_file_path) {
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
    fn from_file_medium_test() {
        use std::path::PathBuf;

        let mut test_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_file_path.push("target/debug/examples/8096.txt");

        match HashTree::from_file(test_file_path) {
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
