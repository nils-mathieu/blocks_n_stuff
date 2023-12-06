use std::hash::{BuildHasherDefault, Hash};

use bns_core::{Chunk, ChunkPos};
use bns_worldgen_structure::Structure;

use glam::IVec3;
use hashbrown::HashMap;
use rustc_hash::FxHasher;

/// A unique identifier for a structure to be spawned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructureId {
    /// The world-space position of the structure.
    pub position: IVec3,
    /// An additional identifier
    pub id: u32,
}

impl Hash for StructureId {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        #[cfg(target_pointer_width = "64")]
        {
            state.write_usize((self.position.x as usize) << 32 | self.position.y as usize);
            state.write_usize((self.position.z as usize) << 32 | self.id as usize);
        }

        #[cfg(target_pointer_width = "32")]
        {
            self.position.hash(state);
            self.id.hash(state);
        }
    }
}

/// A structure that hasn't been inserted into the world completely yet.
#[derive(Debug)]
struct PendingStructure {
    /// The position of the insertion (the origin of the spawned structure in world-space).
    position: IVec3,
    /// The structure itself.
    contents: Structure<'static>,
}

impl PendingStructure {
    /// The bounds of the inserted structure in world-space.
    pub fn bounds(&self) -> (IVec3, IVec3) {
        let min = self.position;
        let max = self.position + self.contents.bounds;
        (min, max)
    }
}

/// Stores the structure that haven't been inserted into the world yet.
#[derive(Debug, Default)]
pub struct StructureRegistry {
    /// The list of pending structures.
    pending: HashMap<StructureId, PendingStructure, BuildHasherDefault<FxHasher>>,
}

impl StructureRegistry {
    /// Write the provided chunk with the structures currently stored in the registry.
    pub fn write_chunk(&self, pos: ChunkPos, chunk: &mut Chunk) {
        todo!();
    }

    /// Inserts a structure into the registry.
    pub fn insert(&mut self, id: StructureId, position: IVec3, contents: Structure<'static>) {
        self.pending
            .insert(id, PendingStructure { position, contents });
    }
}
