//! The main task of this module is to manage all network interactions between
//! the client and the simulator.
//!
//! This is done by maintaining a in viewer representation of the World and
//! updating it dynamically, which will then be rendered by different
//! components of the viewer.

use chashmap::CHashMap;
use data::TerrainPatch;
use nalgebra::Vector2;
use opensim_networking::simulator::Simulator;
use opensim_networking::services::terrain;
use std::collections::HashMap;
use std::thread;
use std::sync::Arc;
use uuid::Uuid;

/// Manages the interaction between Viewer and Region.
pub struct RegionManager {
    simulators: HashMap<Uuid, Simulator>,

    terrain_manager: TerrainManager,
}

impl RegionManager {
    fn start() -> Self {
        let terrain_manager = TerrainManager::start();

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

pub struct TerrainManager {}

impl TerrainManager {
    fn start() -> Self {
        thread::spawn(|| loop {});

        TerrainManager {}
    }

    pub fn get_patch(&self, region: Uuid, pos: Vector2<u8>) -> Result<TerrainPatch, GetPatchError> {
        unimplemented!()

        /*
        Ok(TerrainPatch {
            position: pos,
            region,
            land_heightmap: ...,
        })
        */
    }

    fn register_receivers(&self, region_id: Uuid, receivers: terrain::Receivers) {}
}

pub enum GetPatchError {
    NotAvailable,
}
