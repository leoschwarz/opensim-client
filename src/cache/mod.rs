use addressable_queue::fifo::Queue;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use data::TerrainPatch;
use nalgebra::Vector2;
use uuid::Uuid;
use serde_json;
use std::path::PathBuf;
use std::hash::Hash;
use std::io::{self, Write};
use std::fs::{self, File};
use std::marker::PhantomData;

#[derive(Clone, Debug)]
pub struct CacheConfig {
    /// Maximum size of the cache in bytes.
    pub max_bytes: u64,
}

#[derive(Serialize, Deserialize)]
struct CacheData<I>
where
    I: Clone + Eq + Hash,
{
    current_size: u64,
    counter: u64,
    entries: Queue<I, CacheEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CacheEntry {
    size: u64,
    id: u64,
}

/// Generic implementation of a LRU disk cache with a specificable maximum size.
pub struct SimpleCache<Value, Id>
where
    Id: Clone + Eq + Hash + Serialize + DeserializeOwned,
{
    config: CacheConfig,
    data: CacheData<Id>,
    data_dir: PathBuf,
    _phantom: PhantomData<Value>,
}

impl<V, I> SimpleCache<V, I>
where
    V: DeserializeOwned + Serialize,
    I: DeserializeOwned + Serialize + Clone + Eq + Hash,
{
    pub fn initialize(data_dir: PathBuf, config: CacheConfig) -> Result<Self, CacheError> {
        if !data_dir.exists() {
            fs::create_dir_all(&data_dir).map_err(|e| CacheError::CreateDir(e))?;
        }

        let data_file = data_dir.join("cache_data.json");
        let data = if data_file.exists() {
            let file = File::open(data_file).map_err(|e| CacheError::ReadDataFile(e))?;
            serde_json::from_reader(file).map_err(|e| CacheError::ParseDataFile(e))?
        } else {
            CacheData {
                current_size: 0,
                entries: Queue::new(),
                counter: 0,
            }
        };

        Ok(SimpleCache {
            config,
            data,
            data_dir,
            _phantom: PhantomData,
        })
    }

    pub fn get(&mut self, id: &I) -> Result<Option<V>, CacheError> {
        if let Some(item) = self.data.entries.remove_key(id) {
            // Read the value from the disk.
            let file_id = item.id;
            let file_name = self.data_file_path(item.id);
            let file = File::open(file_name).map_err(|e| CacheError::ReadCacheFile(e))?;
            let value = serde_json::from_reader(file).map_err(|e| CacheError::ParseCacheFile(e))?;

            // Insert the item again at the end of the queue.
            self.data.entries.insert(id.clone(), item);
            Ok(Some(value))
        } else {
            // The cache does not store a relevant entry.
            Ok(None)
        }
    }

    pub fn put(&mut self, id: &I, item: &V) -> Result<(), CacheError> {
        let data = serde_json::to_vec(item).map_err(|e| CacheError::EncodeCacheFile(e))?;
        let bytes = data.len() as u64;

        let entry_id = self.data.counter;
        self.data.counter += 1;

        // Write the file.
        let mut file =
            File::create(self.data_file_path(entry_id)).map_err(|e| CacheError::CreateFile(e))?;
        file.write(&data[..]).map_err(|e| CacheError::WriteFile(e))?;

        // Put the entry into the data struct.
        self.data.entries.insert(
            id.clone(),
            CacheEntry {
                size: bytes,
                id: entry_id,
            },
        );
        self.data.current_size += bytes;

        // Cleanup entries if needed.
        self.cleanup()?;

        Ok(())
    }

    fn data_file_path(&self, entry_id: u64) -> PathBuf {
        self.data_dir.join(format!("data_{}.json", entry_id))
    }

    /// Deletes as many cache entries as needed until the maximum storage is
    /// free again.
    fn cleanup(&mut self) -> Result<(), CacheError> {
        while self.data.current_size > self.config.max_bytes {
            let (entry_id, entry) = self.data.entries.remove_head().unwrap();
            self.data.current_size -= entry.size;
            let path = self.data_file_path(entry.id);
            fs::remove_file(path).map_err(|e| CacheError::RemoveFile(e))?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum CacheError {
    ReadDataFile(io::Error),
    ParseDataFile(serde_json::Error),
    ReadCacheFile(io::Error),
    ParseCacheFile(serde_json::Error),
    EncodeCacheFile(serde_json::Error),
    CreateDir(io::Error),
    CreateFile(io::Error),
    WriteFile(io::Error),
    RemoveFile(io::Error),
}

pub type TerrainCache = SimpleCache<TerrainPatch, (Uuid, Vector2<u8>)>;
