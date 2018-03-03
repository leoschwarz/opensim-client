// TODO: For now the goal of this is to be just a demo rendering a terrain,
// then wander around on the map, and only then actual networking code
// will be added.

// TODO: Remove at some later time.
#![allow(dead_code, unused_imports, unused_variables)]
#![feature(proc_macro, conservative_impl_trait, generators)]

extern crate addressable_queue;
extern crate alga;
extern crate chashmap;
#[macro_use]
extern crate futures_await as futures;
#[macro_use]
extern crate glium;
#[macro_use]
extern crate lazy_static;
extern crate multiqueue;
extern crate nalgebra;
extern crate opensim_networking;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate slog;
extern crate tokio;
extern crate tokio_core;
extern crate toml;
extern crate typenum;
extern crate uuid;

pub mod cache;
pub mod config;
pub mod data;
pub mod networking;
pub mod render;

fn main() {
    use futures::Future;
    use opensim_networking::logging::{Log, LogLevel};
    use opensim_networking::login::{hash_password, LoginRequest};
    use opensim_networking::simulator::Simulator;
    use opensim_networking::circuit::message_handlers::Handlers;
    use nalgebra::Vector2;
    use networking::RegionManager;
    use tokio_core::reactor::Core;
    use std::thread;
    use std::sync::{mpsc, Arc, Mutex};

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

    // Connect to the simulator.
    let (terrain_manager_tx, terrain_manager_rx) = mpsc::channel();
    let (region_id_tx, region_id_rx) = mpsc::channel();

    // Note: With the default stack size of 2 MiB this code overflows the stack.
    //       However in general I don't really like this solution of just making the stack bigger.
    let builder = thread::Builder::new().stack_size(8 * 1024 * 1024);
    builder.spawn(move || {
        let mut region_manager = Box::new(RegionManager::start(log.clone()));
        let mut reactor = Core::new().unwrap();
        let handle = reactor.handle();

        println!("connecting sim");
        let sim = reactor.run(Simulator::connect(connect_info, handlers, handle, log)).unwrap();
        println!("connecting sim finished");
        region_id_tx.send(sim.region_info().region_id.clone()).unwrap();
        let region_id = sim.region_info().region_id.clone();
        region_manager.setup_sim(sim);

        terrain_manager_tx.send(region_manager.terrain_manager.clone()).unwrap();

        let patch_handle = (region_id, Vector2::new(0, 0));
        let fut = region_manager.terrain_manager.get_patch(patch_handle);
        let patch = reactor.run(fut).unwrap();

        println!("patch: {:?}", patch);

        loop {
            reactor.turn(None);
        }
    }).unwrap();

    let terrain_manager = terrain_manager_rx.recv().unwrap();
    let region_id = region_id_rx.recv().unwrap();
    let patch_handle = (region_id, Vector2::new(0, 0));
    //let mut reactor = Core::new().unwrap();
    //let patch = reactor.run(terrain_manager.get_patch(patch_handle)).unwrap();

    //println!("patch: {:?}", patch);

    render::render_world();
}
