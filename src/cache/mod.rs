pub use simple_disk_cache::CacheError;
pub use simple_disk_cache::config::CacheConfig;
use data::TerrainPatch;
use simple_disk_cache::SimpleCache;
use types::{Uuid, Vector2};

pub type TerrainCache = SimpleCache<(Uuid, Vector2<u8>), TerrainPatch>;
