use data::TerrainPatch;
use futures::{Future, Sink, Stream};
use multiqueue::{self, MPMCFutReceiver, MPMCFutSender};
use std::sync::mpsc::{RecvError, SendError};

/// Managing the connection in the client code.
#[derive(Clone)]
pub struct RegionConnection {
    recv: MPMCFutReceiver<EventRecv>,
    send: MPMCFutSender<EventSend>,
}

/// The internal manager of the connection in the networking code (in the
/// networking thread).
#[derive(Clone)]
pub struct RegionConnectionInternal {
    pub recv: MPMCFutReceiver<EventSend>,
    pub send: MPMCFutSender<EventRecv>,
}

pub fn new_pair() -> (RegionConnection, RegionConnectionInternal) {
    let max_buffer = 256;
    let (send1, recv1) = multiqueue::mpmc_fut_queue(max_buffer);
    let (send2, recv2) = multiqueue::mpmc_fut_queue(max_buffer);
    let conn1 = RegionConnection {
        recv: recv1,
        send: send2,
    };
    let conn2 = RegionConnectionInternal {
        recv: recv2,
        send: send1,
    };
    (conn1, conn2)
}

impl RegionConnection {
    pub fn send(&self, event: EventSend) -> Result<(), SendError<EventSend>> {
        self.send.clone().send(event).map(|_| ()).wait()
    }

    pub fn recv(&self) -> Result<EventRecv, RecvError> {
        self.recv.recv()
    }
}

pub enum EventRecv {
    TerrainPatch(TerrainPatch),
    ConnectResult(Result<(), ()>),
}

pub enum EventSend {}
