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

    // Setup world representation.
    let world = data::World {
        current_region: data::RegionConnection::Pending,
    };
    let (world_reader, world_writer) = typed_rwlock::new(world);

    // Connect to the simulator.
    //
    // Note: With the default stack size of 2 MiB this code overflows the stack.
    // However in general I don't really like this solution of just making
    // the stack bigger.
    let builder = thread::Builder::new().stack_size(16 * 1024 * 1024);
    builder
        .spawn(move || {
            let paths = data::config::Paths {};
            let terrain_storage = Arc::new(
                data::terrain::TerrainStorage::new(&paths).expect("setup terrain storage failed"),
            );
            let mut region_manager = Box::new(RegionManager::start(log.clone(), terrain_storage));
            let mut reactor = Core::new().unwrap();
            let handle = reactor.handle();

            println!("connecting sim");
            let sim = reactor
                .run(Simulator::connect(connect_info, handlers, handle, log))
                .unwrap();
            println!("connecting sim finished");

            {
                let patches_per_side = 16; // TODO
                let region = data::Region {
                    id: sim.region_info().region_id.clone(),
                    // TODO !!!
                    size: 256,
                    // TODO !!!
                    grid_location: Vector2::new(0, 0),
                };
                let mut world = world_writer.write();
                world.current_region = data::RegionConnection::Connected(region);
            }

            let region_id = sim.region_info().region_id.clone();
            region_manager.setup_sim(sim);

            let patch_handle = (region_id, Vector2::new(0, 0));
            let fut = region_manager.terrain_manager.get_patch(patch_handle);
            let patch = reactor.run(fut).unwrap();

            loop {
                reactor.turn(None);
            }
        })
        .unwrap();

    render::render_world(world_reader);
}
