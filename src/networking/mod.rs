//! The main task of this module is to manage all network interactions between
//! the client and the simulator.
//!
//! This is done by maintaining a in viewer representation of the World and
//! updating it dynamically, which will then be rendered by different
//! components of the viewer.

use cache::{CacheConfig, CacheError, TerrainCache};
use chashmap::CHashMap;
use data::TerrainPatch;
use futures::{future, Async, Future, Poll};
use nalgebra::Vector2;
use opensim_networking::simulator::Simulator;
use opensim_networking::services::terrain;
use std::collections::HashMap;
use std::thread;
use std::sync::{mpsc, Arc, Mutex};
use std::path::PathBuf;
use uuid::Uuid;

/// Manages the interaction between Viewer and Region.
pub struct RegionManager {
    simulators: HashMap<Uuid, Simulator>,

    terrain_manager: TerrainManager,
}

impl RegionManager {
    fn start() -> Self {
        // TODO: Remove the expect.
        let terrain_manager = TerrainManager::start().expect("Setting up terrain_manager failed.");

        RegionManager {
            simulators: HashMap::new(),
            terrain_manager,
        }
    }

    fn setup_sim(&mut self, sim: Simulator) {
        let region_id = sim.region_info().region_id.clone();
        let terrain_receivers = sim.services().terrain.receivers().unwrap();
        self.simulators.insert(region_id, sim);
    }
}

type PatchHandle = (Uuid, Vector2<u8>);

struct TerrainManagerInner {
    cache: Mutex<TerrainCache>,
    receivers: Mutex<Vec<(Uuid, terrain::Receivers)>>,
}

impl TerrainManagerInner {
    // TODO: For performance reasons this method should be aware of what we are
    // looking for and not be called when it is available, and if it is not
    // available upfront it should immediately return once the relevant data is
    // encountered even if new data is available.
    //
    // (Maybe it's safer to only do the second check, since otherwise we might end
    // up  using outdated cache data.)
    /// Extracts all available messages from the receiver queues.
    fn extract_queues(&self) -> Result<(), CacheError> {
        let receivers = self.receivers.lock().unwrap();
        for &(ref region_id, ref receiver) in receivers.iter() {
            loop {
                match receiver.land_patches.try_recv() {
                    Ok(patches) => {
                        let mut cache = self.cache.lock().unwrap();
                        for patch in &patches {
                            let pos = patch.patch_position();
                            let pos = Vector2::new(pos.0 as u8, pos.1 as u8);

                            let patch_handle = (region_id.clone(), pos);
                            cache.put(
                                &patch_handle,
                                &TerrainPatch {
                                    position: pos,
                                    region: region_id.clone(),
                                    // TODO: this resize should be checked.
                                    land_heightmap: patch.data().clone().fixed_resize(-1.),
                                },
                            )?;
                        }
                    }
                    Err(mpsc::TryRecvError::Empty) => {
                        break;
                    }
                    Err(mpsc::TryRecvError::Disconnected) => {
                        // TODO: Delete the receiver.
                        break;
                    }
                }
            }
        }
        Ok(())
    }
}

pub struct TerrainManager {
    inner: Arc<TerrainManagerInner>,
}

impl TerrainManager {
    fn start() -> Result<Self, CacheError> {
        thread::spawn(|| loop {});

        let config = CacheConfig {
            // 1 GiB
            max_bytes: 1 * 1024 * 1024 * 1024,
        };

        // Make configurable.
        let path = "target/cache/terrain".into();
        let cache = TerrainCache::initialize(path, config)?;

        let inner = Arc::new(TerrainManagerInner {
            cache: Mutex::new(cache),
            receivers: Mutex::new(Vec::new()),
        });

        Ok(TerrainManager { inner })
    }

    pub fn get_patch(
        &self,
        patch_handle: PatchHandle,
    ) -> Box<Future<Item = TerrainPatch, Error = GetPatchError>> {
        // Extract queue entries.
        match self.inner.extract_queues() {
            Err(e) => return Box::new(future::err(GetPatchError::CacheError(e))),
            _ => {}
        }

        // Check if it is in the cache.
        let cache_item = {
            let mut cache = self.inner.cache.lock().unwrap();
            cache.get(&patch_handle)
        };

        match cache_item {
            Ok(Some(item)) => Box::new(future::ok(item)),
            Ok(None) => Box::new(PendingPatch {
                terrain_manager: Arc::clone(&self.inner),
                patch_handle,
            }),
            Err(e) => Box::new(future::err(GetPatchError::CacheError(e))),
        }
    }

    fn register_receivers(&self, region_id: Uuid, receivers: terrain::Receivers) {
        let mut all = self.inner.receivers.lock().unwrap();
        all.push((region_id, receivers));
    }
}

struct PendingPatch {
    terrain_manager: Arc<TerrainManagerInner>,
    patch_handle: PatchHandle,
}

impl Future for PendingPatch {
    type Item = TerrainPatch;
    type Error = GetPatchError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.terrain_manager.extract_queues() {
            Err(e) => return Err(GetPatchError::CacheError(e)),
            _ => {}
        }
        let item = {
            let mut cache = self.terrain_manager.cache.lock().unwrap();
            cache
                .get(&self.patch_handle)
                .map_err(|e| GetPatchError::CacheError(e))?
        };

        if let Some(patch) = item {
            Ok(Async::Ready(patch))
        } else {
            Ok(Async::NotReady)
        }
    }
}

pub enum GetPatchError {
    NotAvailable,
    CacheError(CacheError),
}
