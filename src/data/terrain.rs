use cache::TerrainCache;
use data::{config, ids};
use failure::Error;
use std::sync::Mutex;
use types::{DMatrix, Uuid, Vector2};

pub type PatchPosition = Vector2<u8>;
pub type PatchSize = usize;

#[derive(Debug, Fail)]
pub enum StorageError {
    #[fail(display = "Patch was not found.")]
    NotFound,

    #[fail(display = "Cache error: {}", 0)]
    Cache(::simple_disk_cache::CacheError),
}

pub struct TerrainStorage {
    cache: Mutex<TerrainCache>,
}

impl TerrainStorage {
    pub fn new(paths: &config::Paths) -> Result<Self, Error> {
        use simple_disk_cache as sdc;

        let config = sdc::config::CacheConfig {
            // 128 MiB
            max_bytes: 128 * 1024 * 1024,
            encoding: sdc::config::DataEncoding::Bincode,
            strategy: sdc::config::CacheStrategy::LRU,
            subdirs_per_level: 20,
        };
        let cache = TerrainCache::initialize(paths.terrain_cache(), config)?;

        Ok(TerrainStorage {
            cache: Mutex::new(cache),
        })
    }

    pub fn put_patch(
        &self,
        region: &ids::RegionId,
        patch_pos: &PatchPosition,
        patch: &TerrainPatch,
    ) -> Result<(), StorageError> {
        let mut cache = self.cache.lock().unwrap();
        cache
            .put(&(*region, *patch_pos), patch)
            .map_err(|e| StorageError::Cache(e))
    }

    pub fn get_patch(
        &self,
        region: &ids::RegionId,
        // TODO
        //patch_size: &PatchSize,
        patch_pos: &PatchPosition,
    ) -> Result<TerrainPatch, StorageError> {
        let mut cache = self.cache.lock().unwrap();
        let res = cache
            .get(&(*region, *patch_pos))
            .map_err(|e| StorageError::Cache(e))?;
        res.ok_or_else(|| StorageError::NotFound)
    }
}

#[derive(Serialize, Deserialize)]
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
