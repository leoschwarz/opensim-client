//! This module contains the types which represent the data that represents the
//! state of the simulator and is to be rendered on the screen.

// TODO: For now the solution with the typed_rwlock around World is good enough.
// However in the future there should be a lot more granular control about
// locking because for example if a terrain patch is received it makes no sense
// to also lock something completely different.
// This could have a huge performance impact.
// What would be really nice to explore would be how far currently locked things
// could be skipped and other rendering work be performed, before it is
// unlocked again.

use types::{DMatrix, Matrix4, Quaternion, UnitQuaternion, Vector2, Vector3};
use types::nalgebra::{Matrix, MatrixVec};
use types::nalgebra::core::dimension::U256;
pub use opensim_networking::types::Uuid;
use std::collections::HashMap;

mod ecs {
    use specs::{Component, BTreeStorage, System, World};

    pub struct Terrain {

    }

    impl Component for Terrain {
        // TODO: In the future, reconsider all storage choices.
        type Storage = BTreeStorage<Self>;
    }

    pub struct Region {

    }

    impl Component for Region {
        type Storage = BTreeStorage<Self>;
    }

    pub fn new_world() -> World {
        let mut world = World::new();
        world.register::<Terrain>();
        world.register::<Region>();
        world
    }
}

pub mod avatar;

/*  */
// (old notes)
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
//
/// Provides access to the current state of the whole world, to be rendered.
///
/// # State management (TODO)
///
/// In this case it is a multi-producer single consumer problem.
///
/// The networking thread and the GUI thread shall write,
/// the rendering thread shall only read.
///
/// For this reason the `typed_rwlock` crate is used.
/// Currently this does not provide any advantage but maybe in the future
/// this could be utilized to use an even better synchronization primitive.
/// (TODO)
pub struct World {
    pub current_region: RegionConnection,
    /* TODO */
    /* pub client_avatar: RwLock<avatar::ClientAvatar>, */
}

pub enum RegionConnection {
    /// The connection to the region is not yet established.
    ///
    /// This should be displayed to the user if needed.
    Pending,

    /// The connection to the region is established.
    Connected(Region),

    /// The connection to the region was dropped.
    Disconnected,
}

pub struct Region {
    /// The unique ID of the region.
    pub id: Uuid,

    /// Side length of the region in meters.
    pub size: u32,

    /// The location of the region on the grid.
    pub grid_location: Vector2<u32>,

    /// The (currently available) terrain data.
    pub terrain: Terrain,
}

// TODO:
// - How to store region data?
//   For a 8192x8192 with 64e6 f32 values we already have 240 MB of data
//   just for the land height map, obviously we don't want to store everything
//   in memory.
//   - Make TerrainPatch serializable and implement a disk caching strategy,
//     where patches around the player are kept in memory.
//   - For now store everything in RAM.
pub struct Terrain {
    patches: HashMap<Vector2<u8>, Option<TerrainPatch>>,
}

impl Terrain {
    pub fn empty(patches_per_side: u8) -> Terrain {
        let mut patches = HashMap::new();
        for i in 0..patches_per_side {
            for j in 0..patches_per_side {
                patches.insert(Vector2::new(i, j), None);
            }
        }
        Terrain { patches: patches }
    }

    pub fn insert_patch(&mut self, patch: TerrainPatch) {
        self.patches.insert(patch.position.clone(), Some(patch));
    }

    pub fn get_patch(&self, position: &Vector2<u8>) -> &Option<TerrainPatch> {
        self.patches.get(position).expect("invalid patch position")
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TerrainPatch {
    pub position: Vector2<u8>,
    pub region: Uuid,
    pub land_heightmap: DMatrix<f32>,
}

impl TerrainPatch {
    pub fn land_heightmap(&self) -> &DMatrix<f32> {
        &self.land_heightmap
    }

    /// TODO: Remove
    pub fn dummy() -> Self {
        let raw_data = include!("./layer_land.png.txt");
        TerrainPatch {
            position: Vector2::new(0, 0),
            region: Uuid::nil(),
            land_heightmap: DMatrix::from_fn(256, 256, |x, y| raw_data[x][y]),
        }
    }
}

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
