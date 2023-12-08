use bns_core::BlockId;

use super::structures;
use crate::make_standard_biome;

make_standard_biome! {
    /// The desert biome.
    pub struct Desert(|biome| {
        biome.set_surface_block(BlockId::Sand.into());
        biome.set_dirt(BlockId::Sandstone.into(), 6, 8);
        biome.set_underground(BlockId::Stone.into());
        biome.set_base_height(8.0);
        biome.add_height_noise(4.0, 0.01);
        biome.add_prop(BlockId::Pebbles.into(), 5000);
    });
}

make_standard_biome! {
    /// The oak forest biome.
    pub struct OakForest(|biome| {
        biome.set_surface_block(BlockId::Grass.into());
        biome.set_dirt(BlockId::Dirt.into(), 6, 8);
        biome.set_underground(BlockId::Stone.into());
        biome.set_base_height(10.0);
        biome.add_height_noise( 2.0, 0.06);
        biome.add_prop(BlockId::Pebbles.into(), 100);
        biome.add_prop(BlockId::Daffodil.into(), 300);
        biome.add_structure(structures::OAK_TREES, 100);
    });
}

make_standard_biome! {
    /// The plains biome.
    pub struct Plains(|biome| {
        biome.set_surface_block(BlockId::Grass.into());
        biome.set_dirt(BlockId::Dirt.into(), 6, 8);
        biome.set_underground(BlockId::Stone.into());
        biome.set_base_height(5.0);
        biome.add_height_noise(2.0, 0.03);
        biome.add_height_noise(1.0, 0.015);
        biome.add_prop(BlockId::Pebbles.into(), 600);
        biome.add_prop(BlockId::Daffodil.into(), 600);
        biome.add_structure(structures::OAK_TREES, 3000);
        biome.add_structure(structures::BOULDERS, 6000);
        biome.add_structure(structures::LIL_HOUSES, 200000);
        biome.add_structure(std::slice::from_ref(&&structures::VILLAGE), 500000);
    });
}

make_standard_biome! {
    /// The pine forest biome.
    pub struct PineForest(|biome| {
        biome.set_surface_block(BlockId::Podzol.into());
        biome.set_dirt(BlockId::Dirt.into(), 6, 8);
        biome.set_underground(BlockId::Stone.into());
        biome.set_base_height(4.0);
        biome.add_height_noise(0.02, 0.01);
        biome.add_prop(BlockId::Pebbles.into(), 400);
        biome.add_structure(structures::PINE_TREES, 200);
        biome.add_structure(structures::BOULDERS, 500);
    });
}

make_standard_biome! {
    /// The moutain biome.
    pub struct Mountain(|biome| {
        biome.set_surface_block(BlockId::Stone.into());
        biome.set_dirt(BlockId::Stone.into(), 3, 2);
        biome.set_underground(BlockId::Stone.into());
        biome.set_base_height(40.0);
        biome.add_height_noise(20.0, 0.01);
        biome.add_height_noise(5.0, 0.03);
        biome.add_height_noise(2.0, 0.1);
    });
}
