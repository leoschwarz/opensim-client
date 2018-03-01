//! The main task of this module is to manage all network interactions between
//! the client and the simulator.
//!
//! This is done by maintaining a in viewer representation of the World and
//! updating it dynamically, which will then be rendered by different
//! components of the viewer.

// TODO: Actually we probably don't need the connection communicators.
// What we really need is someone who manages our instance of world data, provides ways to request
// more when needed, and caches it away to save memory.

use {data, std};
use chashmap::CHashMap;
use futures::prelude::*;
use futures::stream::FuturesUnordered;
use futures::sync::mpsc;
use futures::{self, Future, Sink, Stream};
use opensim_networking::logging::Log;
use opensim_networking::simulator::{ConnectInfo, MessageHandlers, Simulator};
use std::cell::RefCell;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::mpsc::SendError;
use std::thread::{self, JoinHandle};
use tokio::reactor::{Handle, Reactor};
use uuid::Uuid;

pub mod region_connection;
pub use self::region_connection::RegionConnection;
use self::region_connection::RegionConnectionInternal;

type RegionId = Uuid;

/// Main manager of networking resources in the client.
///
/// There should be only one instance of this struct held by the viewer,
/// it exposes an interface for establishing new connections to different simulators.
/// These connections can then be used to communicate with the relevant simulators.
pub struct Networking {
    /// The thread where the networking code runs.
    net_thread: JoinHandle<()>,

    /// Log instance to write to.
    /// TODO: Remove if not needed.
    log: Log,

    /// Used to register a RegionConnection in the networking thread.
    setup_conn: mpsc::Sender<(RegionConnectionInternal, RegionId)>,
}

impl Networking {
    pub fn new(log: Log) -> Self {
        let (setup_conn_tx, setup_conn_rx) = mpsc::channel(1);
        let thread_handle = thread::spawn(move || {
            use tokio::executor::current_thread;

            let mut reactor = Reactor::new().unwrap();
            let conns: Arc<CHashMap<RegionId, RegionConnectionInternal>> =
                Arc::new(CHashMap::new());

            let do_setup_conn = setup_conn_rx.map(move |(conn_int, region_id)| {
                conns.insert(region_id, conn_int);
            });
            let fut = do_setup_conn.into_future().map(|_| ()).map_err(|_| ());

            current_thread::run(|_| {
                current_thread::spawn(fut);
            });
        });

        Networking {
            net_thread: thread_handle,
            log: log,
            setup_conn: setup_conn_tx,
        }
    }

    pub fn connect_region(
        &self,
        region_id: RegionId,
    ) -> impl Future<Item = RegionConnection, Error = ()> {
        let setup_conn = self.setup_conn.clone();
        async_block! {
            let (conn, conn_int) = region_connection::new_pair();
            await!(setup_conn.send((conn_int, region_id)));
            // TODO: make sure the connection is actually established.
            Ok(conn)
        }
    }
}

/*
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

/// Connection remote to a simulator.
struct Connection {
    /// The communicator.
    comm: RegionConnectionInternal,

    /// The simulator connection.
    sim: Simulator,
}



fn setup_connection(
    conns: Rc<CHashMap<RegionId, Connection>>,
    connect_info: ConnectInfo,
    core_handle: Handle,
    log: Log,
    region_id: RegionId,
    conn_internal: RegionConnectionInternal,
) -> impl Future<Item = (), Error = mpsc::SendError<EventRecv>> {
    let sim_future = Simulator::connect(connect_info, MessageHandlers::default(), core_handle, log);
    let send_future = sim_future.then(move |sim_result| {
        if let Ok(sim) = sim_result {
            let send = conn_internal.send.clone();
            conns.insert(
                region_id,
                Connection {
                    comm: conn_internal,
                    sim: sim,
                },
            );
            send.send(EventRecv::ConnectResult(Ok(())))
        } else {
            conn_internal.send.send(EventRecv::ConnectResult(Err(())))
        }
    });
    send_future.map(|_| ())
}

impl Networking {


    //    #[async]
    pub fn connect_region(
        self: Box<Self>,
        region_id: RegionId,
    ) -> impl Future<Item = RegionConnection, Error = ConnectError>
//Result<RegionConnection, ConnectError>
    {
        let (conn, conn_internal) = region_connection::new_pair();
        //let conn = Rc::new(RefCell::new(conn));

        let send_setup = self.setup_conn
            .clone()
            .send((conn_internal, region_id))
            .map_err(|_| ConnectError::SendError);

        send_setup.and_then(move |_| {
            conn.recv()
                .map_err(|_| ConnectError::RecvError)
                .map(|_| conn)
        })

        //handshake.map(move |_| conn)
        //await!(self.connect_region_internal(region_id))
        //    .map(|c| c.into_inner())

        /*
        let conn2 = Rc::clone(&conn);
        let handshake = await!(RegionConnection::recv(conn2.borrow_mut())).map_err(|_| ConnectError::RecvError)?;
        */
        /*
        let conn2 = Rc::clone(&conn);
        let handshake = async_block! {
            let recv = conn2.borrow_mut().recv();
            await!(recv).map_err(|_| ConnectError::RecvError)
        };
        //let handshake = await!(conn.borrow_mut().recv()).map_err(|_| ConnectError::RecvError)?;
        */

        //Rc::try_unwrap(conn).map(RefCell::into_inner).map_err(|_| unreachable!())
        //conn.map(|c| c.into_inner()).map_err(|_| unreachable!())
        //
    }
}

pub enum ConnectError {
    SendError,
    RecvError,
}

impl<T> From<mpsc::SendError<T>> for ConnectError {
    fn from(_: mpsc::SendError<T>) -> Self {
        ConnectError::SendError
    }
}
*/
