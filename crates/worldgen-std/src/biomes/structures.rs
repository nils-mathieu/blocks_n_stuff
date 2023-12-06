use bns_worldgen_structure::{include_structure, Structure as S};

pub const OAK_TREE_1: S = include_structure!("structures/oak_tree_1.ron");
pub const OAK_TREE_2: S = include_structure!("structures/oak_tree_2.ron");

pub const OAK_TREES: &[&S] = &[&OAK_TREE_1, &OAK_TREE_2];