// TODO: For now the goal of this is to be just a demo rendering a terrain,
// then wander around on the map, and only then actual networking code
// will be added.

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
use self::data::*;
use self::data::client_avatar::ClientAvatar;

pub mod networking;
pub mod render;

fn main() {
    render::render_world();
}
