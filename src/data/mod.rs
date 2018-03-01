//! This module contains the types which represent the data that represents the
//! state of the simulator and is to be rendered on the screen.

pub use nalgebra::{DMatrix, Matrix4, MatrixN, Quaternion, UnitQuaternion, Vector2, Vector3};
pub use opensim_networking::types::Uuid;
use typenum;

pub mod avatar;
pub mod entities;

// TODO:
// - Should manage the current region and the ones adjacent to it.
// - For compatibility regions are generally required to be of the same
//   size as their neighbours, however maybe the code could actually be
//   written in such a way that in the future different layouts are also
//   possible.
//   This could be achieved in the following way:
//   - Determine a numbering for 256x256 (normal region) surrounding regions
//     of the current region no matter how big it is.
//     i. e.
//     +---+---+---+---+
//     | 0 | 1 | 2 | 3 |
//     +---+---+---+---+
//     |11 |       | 4 |
//     +---+       +---+
//     |10 |       | 5 |
//     +---+---+---+---+
//     | 9 | 8 | 7 | 6 |
//     +---+---+---+---+
//   - Map these values to actual regions in a many-to-one fashion.
//   - Load the regions in direct proximity of the viewer.
// - This should probably be implemented with an inner struct which can be
// updated   by the networking thread and use a mutex inside.
//   (I would prefer RwLock but writer starvation is a big problem for us.)
pub struct World {}

pub struct Region {
    /// Side length of the region in meters.
    size: u32,

    /// The unique ID of the region.
    id: Uuid,

    /// The location of the region on the grid.
    grid_location: Vector2<u32>,
}

impl Region {
    /// Side length of the region in meters.
    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn terrain(&self) -> Terrain {
        unimplemented!()
    }
}

// TODO:
// - How to store region data?
//   For a 8192x8192 with 64e6 f32 values we already have 240 MB of data
//   just for the land height map, obviously we don't want to store everything
//   in memory.
//   - Make TerrainPatch serializable and implement a disk caching strategy,
//     where patches around the player are kept in memory.
//   - For now store everything in RAM.
pub struct Terrain {}

impl Terrain {
    pub fn get_patch(&mut self, pos: Vector2<u8>) -> Result<TerrainPatch, ()> {
        unimplemented!()
    }
}

pub struct TerrainPatch {
    land_heightmap: PatchMatrix<f32>,
}

impl TerrainPatch {
    pub fn land_heightmap(&self) -> &PatchMatrix<f32> {
        &self.land_heightmap
    }

    /// TODO: Remove
    pub fn dummy() -> Self {
        let raw_data = include!("./layer_land.png.txt");
        TerrainPatch {
            land_heightmap: PatchMatrix::from_fn(|x, y| raw_data[x][y]),
        }
    }
}

pub type PatchMatrix<S> = MatrixN<S, typenum::U256>;

// TODO: I'm very unhappy with these.
pub mod locators {
    use super::*;

    /// Universal Region Locator, points to a specific region on a specific
    /// grid.
    #[derive(Clone, Debug)]
    pub struct RegionLocator {
        // TODO: This should probably be an URI
        // TODO: This will probably be copied around a lot, so consider whether
        // it might not be too wasteful to every time make a new heap copy of the string.
        // Maybe a better type could be used here. (Arc<String>?)
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

    /// Universal Point Locator, points to a specific point in a specific
    /// region on
    /// a specific grid.
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
}
use self::locators::*;
