// TODO: For now the goal of this is to be just a demo rendering a terrain,
// then wander around on the map, and only then actual networking code
// will be added.

// TODO: Remove at some later time.
#![allow(dead_code, unused_imports, unused_variables)]
#![feature(proc_macro, generators)]

extern crate addressable_queue;
extern crate alga;
extern crate chashmap;
extern crate crossbeam_channel;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate futures_await as futures;
#[macro_use]
extern crate glium;
#[macro_use]
extern crate lazy_static;
extern crate multiqueue;
extern crate opensim_networking;
extern crate opensim_types as types;
extern crate parking_lot;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate simple_disk_cache;
#[macro_use]
extern crate slog;
extern crate specs;
extern crate tokio;
extern crate tokio_core;
extern crate toml;
extern crate typed_rwlock;
extern crate typenum;

pub mod cache;
pub mod config;
pub mod data;
pub mod networking;
pub mod render;
pub mod util;

fn main() {
    use futures::Future;
    use networking::RegionManager;
    use opensim_networking::circuit::message_handlers::Handlers;
    use opensim_networking::logging::{Log, LogLevel};
    use opensim_networking::login::{hash_password, LoginRequest};
    use opensim_networking::simulator::Simulator;
    use parking_lot::RwLock;
    use std::sync::{mpsc, Arc, Mutex};
    use std::thread;
    use tokio_core::reactor::Core;
    use typed_rwlock;
    use types::Vector2;

    // Perform the login.
    let cfg = config::get_config("remote_sim.toml").expect("no config");
    let login_request = LoginRequest {
        first_name: cfg.user.first_name,
        last_name: cfg.user.last_name,
        password_hash: hash_password(cfg.user.password_plain.as_str()),
        start: "last".to_string(),
    };
    let login_response = login_request
        .perform(cfg.sim.loginuri.as_str())
        .expect("Login failure.");

    // Setup logging.
    let log = Log::new_dir("target/log", LogLevel::Debug).unwrap();
    let connect_info = login_response.into();
    let handlers = Handlers::default();

    // Setup storage managers.
    let paths = data::config::Paths {};
    let client_avatar = Arc::new(RwLock::new(data::avatar::ClientAvatar::new(None)));
    let storage = data::Storage {
        terrain: Arc::new(
            data::terrain::TerrainStorage::new(&paths, Arc::clone(&client_avatar))
                .expect("setup terrain storage failed"),
        ),
        region: Arc::new(data::region::RegionStorage::new()),
        client_avatar,
    };

    // Connect to the simulator.
    //
    // Note: With the default stack size of 2 MiB this code overflows the stack.
    // However in general I don't really like this solution of just making
    // the stack bigger.
    let builder = thread::Builder::new().stack_size(16 * 1024 * 1024);
    let storage_ = storage.clone();
    builder
        .spawn(move || {
            let mut region_manager = Box::new(RegionManager::start(log.clone(), &storage_));
            let mut reactor = Core::new().unwrap();
            let handle = reactor.handle();

            println!("connecting sim");
            let sim = reactor
                .run(Simulator::connect(connect_info, handlers, handle, log))
                .unwrap();
            println!("connecting sim finished");

            // Notify region storage of the connected region.
            let region_uuid = sim.region_info().region_id.clone();
            let region_dims = data::region::RegionDimensions {
                // TODO
                patches_per_side: 16,
                // TODO
                patch_size_axis: 16,
                // TODO (VarRegions extension?)
                side_meters: 256,
            };
            // TODO: grid_location
            // TODO: patches_size
            let region =
                data::region::Region::new(region_uuid.clone(), region_uuid.clone(), region_dims);
            storage_.region.put(
                region_uuid.clone(),
                data::region::Connection::Connected(region),
            );

            // Notify the client storage about the current region.
            storage_
                .client_avatar
                .write()
                .set_current_region(Some(region_uuid));

            // Setup the region manager so terrain data is downloaded etc.
            region_manager.setup_sim(sim);

            loop {
                reactor.turn(None);
            }
        })
        .unwrap();

    render::render_world(storage);
}
