//! The main task of this module is to manage all network interactions between
//! the client and the simulator.
//!
//! This is done by maintaining a in viewer representation of the World and
//! updating it dynamically, which will then be rendered by different
//! components of the viewer.

use chashmap::CHashMap;
use crossbeam_channel;
use data::terrain::{self, TerrainPatch, TerrainStorage};
use futures::{future, task, Async, Future, Poll};
use opensim_networking::logging::Log;
use opensim_networking::services;
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
    pub fn start(log: Log, terrain_storage: Arc<TerrainStorage>) -> Self {
        let terrain_manager = TerrainManager::start(log.clone(), terrain_storage);

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
    storage: Arc<TerrainStorage>,
    receivers: Mutex<Vec<(Uuid, services::terrain::Receivers)>>,
}

impl TerrainManagerInner {
    /// Extracts some (but not necessarily all) available messages from the
    /// receiver queues.
    fn extract_queues(&self) -> Result<(), terrain::StorageError> {
        let receivers = self.receivers.lock().unwrap();
        //let mut iter_count = 0;
        for &(ref region_id, ref receiver) in receivers.iter() {
            match receiver.land_patches.try_recv() {
                Ok(patches) => {
                    debug!(
                        self.logger,
                        "TerrainManager::extract_queues received patches"
                    );
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
                        self.storage.put_patch(
                            region_id,
                            &pos,
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
    fn start(log: Log, storage: Arc<TerrainStorage>) -> Self {
        let inner = Arc::new(TerrainManagerInner {
            logger: Logger::root(log, o!("component" => "TerrainManager")),
            storage,
            receivers: Mutex::new(Vec::new()),
        });

        TerrainManager { inner }
    }

    pub fn get_patch(
        &self,
        patch_handle: PatchHandle,
    ) -> Box<Future<Item = TerrainPatch, Error = terrain::StorageError>> {
        // Extract queue entries.
        match self.inner.extract_queues() {
            Err(e) => return Box::new(future::err(e)),
            _ => {}
        }

        // Check if it is in the storage.
        let storage_item = self.inner
            .storage
            .get_patch(&patch_handle.0, &patch_handle.1);
        match storage_item {
            Ok(item) => Box::new(future::ok(item)),
            Err(terrain::StorageError::NotFound) => Box::new(PendingPatch {
                terrain_manager: Arc::clone(&self.inner),
                patch_handle,
            }),
            Err(e) => Box::new(future::err(e)),
        }
    }

    fn register_receivers(&self, region_id: Uuid, receivers: services::terrain::Receivers) {
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
    type Error = terrain::StorageError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.terrain_manager.extract_queues()?;
        let item = self.terrain_manager
            .storage
            .get_patch(&self.patch_handle.0, &self.patch_handle.1);

        match item {
            Ok(patch) => Ok(Async::Ready(patch)),
            Err(terrain::StorageError::NotFound) => {
                task::current().notify();
                Ok(Async::NotReady)
            }
            Err(e) => Err(e),
        }
    }
}
