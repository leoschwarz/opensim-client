//! The main task of this module is to manage all network interactions between
//! the client and the simulator.
//!
//! This is done by maintaining a in viewer representation of the World and
//! updating it dynamically, which will then be rendered by different
//! components of the viewer.

// TODO

use {data, std};
use futures::{self, Future, Sink, Stream};
use futures::prelude::*;
use futures::stream::FuturesUnordered;
use futures::sync::mpsc;
use opensim_networking::logging::Log;
use opensim_networking::simulator::{ConnectInfo, MessageHandlers, Simulator};
use std::collections::HashMap;
use std::thread::{self, JoinHandle};
use std::sync::Arc;
use std::sync::mpsc::SendError;
use std::hash::{Hash, Hasher};
use tokio_core::reactor::{Core, Handle};

pub mod region_connection;
use self::region_connection::{EventRecv, RegionConnection, RegionConnectionInternal};

#[derive(Clone, Debug)]
pub struct RegionId {
    /// The unique id in the networking struct.
    id: u32,

    /// ConnectInfo, only here for informational purposes, not used in
    /// comparison.
    connect_info: Arc<ConnectInfo>,
}

impl Hash for RegionId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for RegionId {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for RegionId {}

struct Connection {
    /// The communicator.
    comm: RegionConnectionInternal,

    /// The simulator connection.
    sim: Simulator,
}

pub struct Networking {
    /// The thread where the networking code runs.
    net_thread: JoinHandle<()>,
    log: Log,

    /// Used to register a RegionConnection in the networking thread.
    setup_conn: mpsc::Sender<(RegionConnectionInternal, RegionId)>,
}

fn setup_connection(conns: &mut HashMap<RegionId, Connection>,
                    connect_info: ConnectInfo,
                    core_handle: Handle,
                    log: Log,
                    region_id: RegionId,
                    conn_internal: RegionConnectionInternal) 
    -> Box<Future<Item=(), Error=SendError<EventRecv>>> {
    let fut = async_block! {
        let sim_result = await!(Simulator::connect(
                connect_info,
                MessageHandlers::default(),
                core_handle,
                log
                ));

        let send = if let Ok(sim) = sim_result {
            conns.insert(
                region_id,
                Connection {
                    comm: conn_internal.clone(),
                    sim: sim,
                },
            );
            conn_internal.send.send(EventRecv::ConnectResult(Ok(())))
        } else {
            conn_internal.send.send(EventRecv::ConnectResult(Err(())))
        };
        await!(send).map(|_| ())
    };
    Box::new(fut)
}

impl Networking {
    pub fn new(log: Log) -> Self {
        let (setup_conn_tx, setup_conn_rx) = mpsc::channel(1);
        let log_copy = log.clone();

        let thread_handle = thread::spawn(move || {
            let mut core = Core::new().unwrap();
            let mut conns: HashMap<RegionId, Connection> = HashMap::new();

            let core_handle = core.handle();
            let setup_conn_handler = setup_conn_rx.map_err(|_| "MPMC recv error");
            let setup_conn_handler = setup_conn_handler.and_then(|tuple| {
                // TODO: Why are type annotations required here?
                let (conn_internal, region_id): (
                    RegionConnectionInternal,
                    RegionId,
                ) = tuple;

                let send =async_block!{ await!(setup_connection(&mut conns,
                                                   (*region_id.connect_info).clone(),
                                                   core_handle.clone(),
                                                   log_copy.clone(),
                                                   region_id,
                                                   conn_internal.clone()))};
                send.map_err(|_| "MPMC send error.")
            });

            core.run(setup_conn_handler.into_future());
        });

        Networking {
            net_thread: thread_handle,
            log: log,
            setup_conn: setup_conn_tx,
        }
    }

    pub fn connect_region(&self, region_id: RegionId) -> RegionConnection {
        let (conn, conn_internal) = region_connection::new_pair();
        // TODO: Consider whether we really want to unwrap here?
        self.setup_conn
            .clone()
            .send((conn_internal, region_id))
            .wait()
            .unwrap();
        conn
    }
}
