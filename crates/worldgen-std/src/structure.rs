use std::hash::{BuildHasherDefault, Hash};

use bns_core::{Chunk, ChunkPos};
use bns_worldgen_structure::Structure;

use glam::IVec3;
use hashbrown::HashMap;
use rustc_hash::FxHasher;

/// A unique identifier for a structure to be spawned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Key {
    /// The world-space position of the structure.
    position: IVec3,
    /// An additional identifier
    id: u32,
}

impl Key {
    /// Returns the [`Key`] that corresponds to the provided [`PendingStructure`].
    #[inline]
    pub fn of_pending_structure(pending: &PendingStructure) -> Self {
        Self {
            position: pending.position,
            id: pending.id,
        }
    }
}

impl Hash for Key {
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

/// A unique identifier used to discriminate between two structures wanting to be spawned
/// at the same location.
pub type StructureRegistryId = u32;

/// A structure that hasn't been inserted into the world completely yet.
#[derive(Debug)]
pub struct PendingStructure {
    /// The position of the insertion (the origin of the spawned structure in world-space).
    pub position: IVec3,
    /// The ID of the structure.
    pub id: StructureRegistryId,
    /// The structure itself.
    pub contents: Structure<'static>,
}

impl PendingStructure {
    /// The bounds of the inserted structure in world-space.
    pub fn bounds(&self) -> (IVec3, IVec3) {
        let min = self.position;
        let max = self.position + self.contents.bounds;
        (min, max)
    }

    /// Returns whether this structure is at least partially part of the chunk at the provided
    /// position.
    pub fn is_part_of_chunk(&self, pos: ChunkPos) -> bool {
        let (min, max) = self.bounds();

        if min == max {
            return false;
        }

        let min = ChunkPos::from_world_pos_i(min);
        let max = ChunkPos::from_world_pos_i(max - IVec3::ONE);

        min.x <= pos.x
            && pos.x <= max.x
            && min.z <= pos.z
            && pos.z <= max.z
            && min.y <= pos.y
            && pos.y <= max.y
    }

    /// Writes the part of the structure that's in the provided chunk.
    pub fn write_to(&self, pos: ChunkPos, chunk: &mut Chunk) {
        for edit in self.contents.edits.iter() {
            if let Some(pos) = pos.checked_local_pos(self.position + edit.position) {
                chunk.set_block(pos, edit.block.clone());
            }
        }
    }
}

/// Stores the structure that haven't been inserted into the world yet.
#[derive(Debug, Default)]
pub struct StructureRegistry {
    /// The list of pending structures.
    pending: HashMap<Key, PendingStructure, BuildHasherDefault<FxHasher>>,
}

impl StructureRegistry {
    /// Write the provided chunk with the structures currently stored in the registry.
    pub fn write_chunk(&self, pos: ChunkPos, chunk: &mut Chunk) {
        self.pending
            .values()
            .filter(|p| p.is_part_of_chunk(pos))
            .for_each(|p| p.write_to(pos, chunk));
    }

    /// Inserts a structure into the registry.
    pub fn insert(&mut self, structure: PendingStructure) {
        self.pending
            .insert(Key::of_pending_structure(&structure), structure);
    }
}
