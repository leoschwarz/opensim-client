use cache::TerrainCache;
use data::avatar::ClientAvatar;
use data::{config, ids};
use failure::Error;
use parking_lot::RwLock;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use types::{DMatrix, Uuid, Vector2};

pub type PatchPosition = Vector2<u8>;
// TODO: Use and check the patch size where appropriate.
pub type PatchSize = usize;
pub type PatchHandle = (ids::RegionId, PatchPosition);

#[derive(Debug, Fail)]
pub enum StorageError {
    #[fail(display = "Patch was not found.")]
    NotFound,

    #[fail(display = "Cache error: {}", 0)]
    Cache(::simple_disk_cache::CacheError),
}

/// The terrain storage manages both the terrain data for patches close
/// to the client avatar position, and a disk cache for patches further
/// away.
pub struct TerrainStorage {
    client_avatar: Arc<RwLock<ClientAvatar>>,
    // TODO Remove entries once they are too far away from the avatar.
    //      This could also be implemented in a dedicated method to be
    //      called from the client update functionality.
    mem_storage: Mutex<HashMap<PatchHandle, TerrainPatch>>,
    disk_storage: Mutex<TerrainCache>,
}

impl TerrainStorage {
    /// Returns true if a patch is withing the relevant distance from client
    /// avatar to be kept in memory.
    fn within_range(&self, p_pos: &PatchPosition) -> bool {
        // TODO implement!!!
        true
    }

    pub fn new(
        paths: &config::Paths,
        client_avatar: Arc<RwLock<ClientAvatar>>,
    ) -> Result<Self, Error> {
        use simple_disk_cache as sdc;

        // Setup disk cache.
        let config = sdc::config::CacheConfig {
            // 128 MiB (TODO)
            max_bytes: 128 * 1024 * 1024,
            encoding: sdc::config::DataEncoding::Bincode,
            strategy: sdc::config::CacheStrategy::LRU,
            subdirs_per_level: 20,
        };
        let disk_storage = TerrainCache::initialize(paths.terrain_cache(), config)?;

        Ok(TerrainStorage {
            client_avatar,
            mem_storage: Mutex::new(HashMap::new()),
            disk_storage: Mutex::new(disk_storage),
        })
    }

    pub fn put_patch(
        &self,
        region: ids::RegionId,
        patch_pos: PatchPosition,
        patch: TerrainPatch,
    ) -> Result<(), StorageError> {
        // Store to disk in any case.
        {
            let mut storage = self.disk_storage.lock().unwrap();
            storage
                .put(&(region, patch_pos), &patch)
                .map_err(|e| StorageError::Cache(e))?;
        }

        // Store in memory if within relevant distance from avatar.
        if self.within_range(&patch_pos) {
            let mut storage = self.mem_storage.lock().unwrap();
            storage.insert((region, patch_pos), patch);
        }

        Ok(())
    }

    pub fn get_patch(
        &self,
        patch_handle: &PatchHandle,
        /* TODO */
        /* patch_size: &PatchSize, */
    ) -> Result<TerrainPatch, StorageError> {
        // Check in memory storage first.
        {
            let storage = self.mem_storage.lock().unwrap();
            if let Some(patch) = storage.get(patch_handle) {
                // TODO: Avoid this clone. Maybe the mem_storage should
                // hold Arcs (or better Rcs) to the actual data instead of the patches
                // directly?
                return Ok(patch.clone());
            }
        }

        // Check disk storage if it was not found in memory.
        let mut storage = self.disk_storage.lock().unwrap();
        let res = storage
            .get(patch_handle)
            .map_err(|e| StorageError::Cache(e))?;

        match res {
            Some(patch) => Ok(patch),
            None => Err(StorageError::NotFound),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TerrainPatch {
    region: ids::RegionId,
    size: PatchSize,
    position: PatchPosition,
    land_heightmap: DMatrix<f32>,
}

impl TerrainPatch {
    pub fn new(
        region: ids::RegionId,
        size: PatchSize,
        position: PatchPosition,
        land_heightmap: DMatrix<f32>,
    ) -> Self {
        TerrainPatch {
            region,
            size,
            position,
            land_heightmap,
        }
    }

    /// Returns the size of the patch.
    pub fn size(&self) -> &PatchSize {
        &self.size
    }

    /// Returns the position of the patch.
    pub fn position(&self) -> &PatchPosition {
        &self.position
    }

    pub fn land_heightmap(&self) -> &DMatrix<f32> {
        &self.land_heightmap
    }

    #[deprecated]
    pub fn dummy() -> Self {
        let raw_data = include!("./layer_land.png.txt");
        TerrainPatch {
            position: Vector2::new(0, 0),
            region: Uuid::nil(),
            size: 256,
            land_heightmap: DMatrix::from_fn(256, 256, |x, y| raw_data[x][y]),
        }
    }
}
