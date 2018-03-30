//! This module contains the types which represent the data that represents the
//! state of the simulator and is to be rendered on the screen.

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use types::nalgebra::{Matrix, MatrixVec};
use types::{DMatrix, Matrix4, Quaternion, UnitQuaternion, Uuid, Vector2, Vector3};

pub mod config {
    use std::path::PathBuf;

    pub struct Paths {}

    impl Paths {
        pub fn terrain_cache(&self) -> PathBuf {
            // TODO
            "target/cache/terrain".into()
        }
    }
}

/// Managment of the various identifiers, often UUIDs are mapped to usize values
/// so they can be used in other places to save memory.
pub mod ids {
    use types::Uuid;

    // TODO replace by u32, u64 or usize
    pub type RegionId = Uuid;

    /// To be used in caches.
    pub type PersistentRegionId = Uuid;
}

pub mod avatar;
pub mod terrain;

/// Contains the various storages for the various entities.
///
/// Note: Using Arc inside this struct has the advantages, that where needed
///       that storage can be directly referenced instead of having to always
///       use the full qualified name.
#[derive(Clone)]
pub struct Storage {
    pub terrain: Arc<terrain::TerrainStorage>,
    pub region: Arc<region::RegionStorage>,
    pub client_avatar: Arc<RwLock<avatar::ClientAvatar>>,
}

pub mod region {
    use data::ids;
    use failure::Error;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use types::Uuid;
    use util::bimap::BiMap;

    #[derive(Debug, Fail)]
    pub enum StorageError {
        #[fail(display = "Region not registered at all: {}", 0)]
        NotRegistered(ids::RegionId),
    }

    pub struct RegionStorage {
        regions: Mutex<HashMap<ids::RegionId, Arc<Connection>>>,
    }

    impl RegionStorage {
        pub fn new() -> Self {
            RegionStorage {
                regions: Mutex::new(HashMap::new()),
            }
        }

        /// Warning: Don't store the results, if you want updated values, you
        /// have to call this method again.
        pub fn get(&self, id: &ids::RegionId) -> Result<Arc<Connection>, StorageError> {
            let regions = self.regions.lock().unwrap();
            regions
                .get(id)
                .map(Arc::clone)
                .ok_or_else(|| StorageError::NotRegistered(id.clone()))
        }

        pub fn put(&self, id: ids::RegionId, connection: Connection) {
            let mut regions = self.regions.lock().unwrap();
            regions.insert(id, Arc::new(connection));
        }

        /*
        pub fn get_or_create(&self, id: &ids::RegionId) -> Arc<Mutex<Connection>> {
            let mut regions = self.regions.lock().unwrap();
            if let Some(region) = regions.get(id) {
                return Arc::clone(region);
            }

            let region = Arc::new(Mutex::new(Connection::Pending));
            regions.insert(id.clone(), Arc::clone(&region));
            region
        }
        */
    }

    pub struct Region {
        /// The UUID of the region (on the sim).
        uuid: Uuid,

        /// The id of the region (in our representation).
        id: ids::RegionId,

        /// Dimensions of this region.
        dimensions: RegionDimensions,
    }

    impl Region {
        pub fn new(uuid: Uuid, id: ids::RegionId, dimensions: RegionDimensions) -> Self {
            Region {
                uuid,
                id,
                dimensions,
            }
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct RegionDimensions {
        /// Side length of the region in meteres.
        pub side_meters: u32,

        /// Number of patches per side.
        pub patches_per_side: u8,
    }

    pub enum Connection {
        /// The connection to the region is not yet established.
        ///
        /// This should be displayed to the user if needed.
        Pending,

        /// Connecting to the region failed.
        ///
        /// TODO: This should be matchable in the future so the error can
        ///       be displayed in a nice way to the user.
        Failed(Error),

        /// The connection to the region is established.
        Connected(Region),

        /// The connection to the region was dropped.
        Disconnected,
    }
}

/*
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

#[deprecated]
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

#[deprecated]
pub struct Region {
    /// The unique ID of the region.
    pub id: Uuid,

    /// Side length of the region in meters.
    pub size: u32,

    /// The location of the region on the grid.
    pub grid_location: Vector2<u32>,
}
*/

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
    /// (Justification: Maybe 512 might have been a bit more efficient, but it
    /// would have made things more complicated as 256 size regions would
    /// have to be handled differently.)
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
