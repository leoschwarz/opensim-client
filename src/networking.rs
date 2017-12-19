//! The main task of this module is to manage all network interactions between
//! the client and the simulator.
//!
//! This is done by maintaining a in viewer representation of the World and updating
//! it dynamically, which will then be rendered by different components of the viewer.

// TODO

use std;
use std::thread::{JoinHandle, self};
use tokio_core::reactor::Core;
use opensim_networking::logging::Log;

pub struct Networking {
    thread_handle: JoinHandle<()>,
}

impl Networking {
    pub fn new(log: Log) -> Self {
        let thread_handle = thread::spawn(move || {
            let core = Core::new().unwrap();

        });

        Networking {
            thread_handle: thread_handle,
        }
    }
}

