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
    use networking::RegionManager;
    use tokio_core::reactor::Core;
    use std::thread;
    use std::sync::{Arc, Mutex};

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

    thread::spawn(move || {
        let mut region_manager = RegionManager::start();
        let mut reactor = Core::new().unwrap();
        let handle = reactor.handle();

        let sim = Simulator::connect(connect_info, handlers, handle, log)
            .wait()
            .unwrap();
        region_manager.setup_sim(sim);

        loop {
            reactor.turn(None);
        }
    });

    render::render_world();
}
