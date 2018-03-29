//! The main task of this module is to manage all network interactions between
//! the client and the simulator.
//!
//! This is done by maintaining a in viewer representation of the World and
//! updating it dynamically, which will then be rendered by different
//! components of the viewer.

use cache::{CacheConfig, CacheError, TerrainCache};
use chashmap::CHashMap;
use crossbeam_channel;
use data::terrain::TerrainPatch;
use futures::{future, task, Async, Future, Poll};
use opensim_networking::logging::Log;
use opensim_networking::services::terrain;
use opensim_networking::simulator::Simulator;
use simple_disk_cache::config::{CacheStrategy, DataEncoding};
use slog::{Drain, Logger};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use types::Uuid;
use types::{DMatrix, Vector2};

/// Manages the interaction between Viewer and Region.
pub struct RegionManager {
    simulators: HashMap<Uuid, Simulator>,
    log: Log,

    pub terrain_manager: TerrainManager,
}

impl RegionManager {
    pub fn start(log: Log) -> Self {
        // TODO: Remove the expect.
        let terrain_manager =
            TerrainManager::start(log.clone()).expect("Setting up terrain_manager failed.");

        RegionManager {
            simulators: HashMap::new(),
            log,
            terrain_manager,
        }
    }

    pub fn setup_sim(&mut self, sim: Simulator) {
        let region_id = sim.region_info().region_id.clone();
        let terrain_receivers = sim.services().terrain.receivers().unwrap();

        let mut all_receivers = self.terrain_manager.inner.receivers.lock().unwrap();
        all_receivers.push((region_id.clone(), terrain_receivers));

        self.simulators.insert(region_id, sim);
    }
}

type PatchHandle = (Uuid, Vector2<u8>);

struct TerrainManagerInner {
    logger: Logger,
    cache: Mutex<TerrainCache>,
    receivers: Mutex<Vec<(Uuid, terrain::Receivers)>>,
}

impl TerrainManagerInner {
    /// Extracts some (but not necessarily all) available messages from the
    /// receiver queues.
    fn extract_queues(&self) -> Result<(), CacheError> {
        let receivers = self.receivers.lock().unwrap();
        //let mut iter_count = 0;
        for &(ref region_id, ref receiver) in receivers.iter() {
            match receiver.land_patches.try_recv() {
                Ok(patches) => {
                    debug!(
                        self.logger,
                        "TerrainManager::extract_queues received patches"
                    );
                    let mut cache = self.cache.lock().unwrap();
                    for patch in patches {
                        let pos = patch.patch_position();
                        let pos = Vector2::new(pos.0 as u8, pos.1 as u8);

                        let patch_handle = (region_id.clone(), pos);
                        debug!(
                            self.logger,
                            "Received terrain patch for: {:?}", patch_handle
                        );
                        let data_matrix = patch.to_data();
                        // TODO fail gracefully
                        assert_eq!(data_matrix.nrows(), data_matrix.ncols());
                        cache.put(
                            &patch_handle,
                            &TerrainPatch::new(
                                region_id.clone(),
                                data_matrix.nrows(),
                                pos,
                                data_matrix,
                            ),
                        )?;
                        debug!(self.logger, "Cached terrain patch for: {:?}", patch_handle);
                    }
                }
                Err(crossbeam_channel::TryRecvError::Empty) => {}
                Err(crossbeam_channel::TryRecvError::Disconnected) => {
                    debug!(self.logger, "Channel disconnected.");
                    // TODO: Delete the receiver.
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct TerrainManager {
    inner: Arc<TerrainManagerInner>,
}

impl TerrainManager {
    fn start(log: Log) -> Result<Self, CacheError> {
        thread::spawn(|| loop {});

        let config = CacheConfig {
            // 1 GiB
            max_bytes: 1 * 1024 * 1024 * 1024,
            encoding: DataEncoding::Bincode,
            strategy: CacheStrategy::LRU,
            subdirs_per_level: 20,
        };

        // Make configurable.
        let path = "target/cache/terrain";
        let cache = TerrainCache::initialize(path, config)?;

        let inner = Arc::new(TerrainManagerInner {
            logger: Logger::root(log, o!("component" => "TerrainManager")),
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
        if let Err(e) = self.terrain_manager.extract_queues() {
            return Err(GetPatchError::CacheError(e));
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
            task::current().notify();
            Ok(Async::NotReady)
        }
    }
}

#[derive(Debug)]
pub enum GetPatchError {
    NotAvailable,
    CacheError(CacheError),
}
