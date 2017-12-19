// TODO: For now the goal of this is to be just a demo rendering a terrain,
// then wander around on the map, and only then actual networking code
// will be added.

// TODO: Remove at some later time.
#![allow(dead_code,unused_imports,unused_variables)]

extern crate alga;
extern crate futures;
#[macro_use]
extern crate glium;
#[macro_use]
extern crate lazy_static;
extern crate nalgebra;
extern crate opensim_networking;
extern crate tokio_core;
extern crate typenum;

pub mod data;

pub mod networking;
pub mod render;

fn main() {
    render::render_world();
}
