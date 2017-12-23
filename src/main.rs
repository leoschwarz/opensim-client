// TODO: For now the goal of this is to be just a demo rendering a terrain,
// then wander around on the map, and only then actual networking code
// will be added.

// TODO: Remove at some later time.
#![allow(dead_code, unused_imports, unused_variables)]

extern crate alga;
extern crate futures;
#[macro_use]
extern crate glium;
#[macro_use]
extern crate lazy_static;
extern crate multiqueue;
extern crate nalgebra;
extern crate opensim_networking;
#[macro_use]
extern crate serde_derive;
extern crate tokio_core;
extern crate toml;
extern crate typenum;

pub mod config;
pub mod data;
pub mod networking;
pub mod render;

fn main() {
    use opensim_networking::login::{hash_password, LoginRequest};

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

    // Connect to the simulator.


    render::render_world();
}
