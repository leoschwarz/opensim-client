use data::TerrainPatch;
use data::{ids, terrain};
pub use simple_disk_cache::CacheError;
use simple_disk_cache::SimpleCache;
pub use simple_disk_cache::config::CacheConfig;
use types::{Uuid, Vector2};

pub type TerrainCache =
    SimpleCache<(ids::PersistentRegionId, terrain::PatchPosition), TerrainPatch>;
