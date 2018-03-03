use data::TerrainPatch;
use nalgebra::Vector2;
use simple_disk_cache::SimpleCache;
pub use simple_disk_cache::CacheError;
pub use simple_disk_cache::config::CacheConfig;
use uuid::Uuid;

pub type TerrainCache = SimpleCache<(Uuid, Vector2<u8>), TerrainPatch>;
