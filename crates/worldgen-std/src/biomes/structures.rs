use bns_worldgen_structure::{include_structure, Structure as S};

pub const OAK_TREE_1: S = include_structure!("structures/oak_tree_1.ron");
pub const OAK_TREE_2: S = include_structure!("structures/oak_tree_2.ron");
pub const OAK_TREE_3: S = include_structure!("structures/oak_tree_3.ron");
pub const OAK_TREE_4: S = include_structure!("structures/oak_tree_4.ron");

pub const OAK_TREES: &[&S] = &[&OAK_TREE_1, &OAK_TREE_2, &OAK_TREE_3, &OAK_TREE_4];

pub const PINE_TREE_1: S = include_structure!("structures/pine_tree_1.ron");
pub const PINE_TREE_2: S = include_structure!("structures/pine_tree_2.ron");

pub const PINE_TREES: &[&S] = &[&PINE_TREE_1, &PINE_TREE_2];

pub const BOULDER_1: S = include_structure!("structures/boulder_1.ron");
pub const BOULDER_2: S = include_structure!("structures/boulder_2.ron");
pub const BOULDER_3: S = include_structure!("structures/boulder_3.ron");

pub const BOULDERS: &[&S] = &[&BOULDER_1, &BOULDER_2, &BOULDER_3];

pub const LIL_HOUSE_1: S = include_structure!("structures/lil_house_1.ron");
pub const LIL_HOUSE_2: S = include_structure!("structures/lil_house_2.ron");

pub const LIL_HOUSES: &[&S] = &[&LIL_HOUSE_1, &LIL_HOUSE_2];

pub const VILLAGE: S = include_structure!("structures/village.ron");
