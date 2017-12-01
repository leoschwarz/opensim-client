//! This module contains the types which represent the data that represents the
//! state of the simulator and is to be rendered on the screen.

pub use nalgebra::{Vector2, Vector3};

pub mod terrain;

/// Universal Region Locator, points to a specific region on a specific grid.
#[derive(Clone, Debug)]
pub struct RegionLocator {
    // TODO: This should probably be an URI
    pub grid: String,
    pub reg_pos: Vector2<u32>,
}

/// Locates a patch in a region.
///
/// Each region is sliced into 256x256 size patches for these purposes.
/// (Justification: Maybe 512 might have been a bit more efficient, but it would have made
///  things more complicated as 256 size regions would have to be handled differently.)
#[derive(Clone, Debug)]
pub struct PatchLocator {
    pub region: RegionLocator,
    pub patch_pos: Vector2<u8>,
}

// TODO
// Maybe this should be made into trait so it can be used efficiently without allocating
// or updating in places where positions are to be iterated. (But only do this if such a need
// really arises and not just for the sake of a nice abstraction... :'))
#[derive(Clone, Debug)]
pub struct PointLocator {
    pub region: RegionLocator,
    /// Relative position inside of the region.
    pub rel_pos: Vector3<f32>,
}
