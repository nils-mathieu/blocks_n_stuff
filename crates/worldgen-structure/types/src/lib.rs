use std::borrow::Cow;

use bns_core::BlockInstance;
use glam::IVec3;

/// An edition that a structure can apply.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
pub struct Structure<'a> {
    /// The name of the structure.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub name: Option<String>,
    /// The maximum bound of the structure.
    pub bounds: IVec3,
    /// The editions that the structure applies.
    pub edits: Cow<'a, [StructureEdit]>,
}
