//! The main task of this module is to manage all network interactions between
//! the client and the simulator.
//!
//! This is done by maintaining a in viewer representation of the World and
//! updating it dynamically, which will then be rendered by different
//! components of the viewer.

// TODO

use {data, std};
use futures::{self, Future, Sink, Stream};
use futures::sync::mpsc;
use opensim_networking::logging::Log;
use opensim_networking::simulator::{ConnectInfo, MessageHandlers, Simulator};
use std::collections::HashMap;
use std::thread::{self, JoinHandle};
use tokio_core::reactor::Core;

pub mod region_connection;
use self::region_connection::{RegionConnection, RegionConnectionInternal};

pub struct Networking {
    thread_handle: JoinHandle<()>,
    connect_tx: mpsc::Sender<(RegionConnectionInternal, ConnectInfo)>,
    log: Log,
}

impl Networking {
    pub fn new(log: Log) -> Self {
        let (connect_tx, connect_rx) = mpsc::channel(1);
        let thread_handle = thread::spawn(move || {
            // let mut connections = HashMap::new();

            // TODO: probably need to use and_then and map_err here in the future.
            let connect_handler = connect_rx.map(|conn| {
                let (c, c_info) = conn;
                // let simulator = Simulator::connect(c_info);
                // unimplemented!()
            });
            let connect_handler = connect_handler.map_err(|_| "");

            let mut core = Core::new().unwrap();
            core.run(connect_handler.into_future());
        });

        Networking {
            thread_handle: thread_handle,
            connect_tx: connect_tx,
            log: log,
        }
    }

    pub fn connect_region(&self, c_info: ConnectInfo) -> RegionConnection {
        let (conn, conn_internal) = region_connection::new_pair();
        // TODO: Consider whether we really want to unwrap here?
        self.connect_tx
            .clone()
            .send((conn_internal, c_info))
            .wait()
            .unwrap();
        conn
    }
}
