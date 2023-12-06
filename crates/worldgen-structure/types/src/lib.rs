use std::borrow::Cow;

use bns_core::BlockInstance;
use glam::IVec3;

/// An edition that a structure can apply.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StructureEdit {
    /// The relative position of the block that must be inserted.
    ///
    /// This position is relative ot the structure's origin.
    pub position: IVec3,
    /// The block that must be inserted.
    pub block: BlockInstance,
}

/// A structure that's made of [`StructureEdit`]s.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Structure<'a> {
    /// The minimum bound of the structure.
    pub min: IVec3,
    /// The maximum bound of the structure.
    pub max: IVec3,
    /// The editions that the structure applies.
    pub edits: Cow<'a, [StructureEdit]>,
}
