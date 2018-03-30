//! The main task of this module is to manage all network interactions between
//! the client and the simulator.
//!
//! This is done by maintaining a in viewer representation of the World and
//! updating it dynamically, which will then be rendered by different
//! components of the viewer.

use chashmap::CHashMap;
use crossbeam_channel;
use data::ids;
use data::terrain::{self, PatchHandle, TerrainPatch, TerrainStorage};
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

    terrain_receivers: Arc<Mutex<services::terrain::Receivers>>,
    terrain_storage: Arc<TerrainStorage>,
}

impl RegionManager {
    pub fn start(log: Log, terrain_storage: Arc<TerrainStorage>) -> Self {
        let terrain_receivers = Arc::new(Mutex::new(services::terrain::Receivers::new()));

        let terrain_receivers_ = Arc::clone(&terrain_receivers);
        let terrain_storage_ = Arc::clone(&terrain_storage);

        thread::spawn(move || {
            let terrain_receivers = Arc::clone(&terrain_receivers_);
            let terrain_storage = Arc::clone(&terrain_storage_);

            // TODO !!! Make better
            loop {
                {
                    let mut recv = terrain_receivers.lock().unwrap();
                    recv.receive_patches(|region_id, patch| {
                        let p = patch.patch_position();
                        let patch_pos = Vector2::new(p.0 as u8, p.1 as u8);
                        let data_matrix = patch.to_data();
                        // TODO fail gracefully
                        assert_eq!(data_matrix.nrows(), data_matrix.ncols());
                        terrain_storage
                            .put_patch(
                                region_id.clone(),
                                patch_pos,
                                TerrainPatch::new(
                                    region_id.clone(),
                                    data_matrix.nrows(),
                                    patch_pos,
                                    data_matrix,
                                ),
                            )
                            .unwrap();
                    });
                }
                thread::sleep(::std::time::Duration::from_millis(50));
            }
        });

        RegionManager {
            simulators: HashMap::new(),
            log,
            terrain_storage,
            terrain_receivers,
        }
    }

    pub fn setup_sim(&mut self, sim: Simulator) {
        let region_id = sim.region_info().region_id.clone();
        // TODO: handle potential errors
        self.terrain_receivers
            .lock()
            .unwrap()
            .register(region_id, &sim.services().terrain)
            .unwrap();
        self.simulators.insert(region_id, sim);
    }
}
