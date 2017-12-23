use data::TerrainPatch;
use futures::{Future, Sink, Stream};
use multiqueue::{self, MPMCFutReceiver, MPMCFutSender};
use std::sync::mpsc::RecvError;

/// Managing the connection in the client code.
//#[derive(Clone)]
pub struct RegionConnection {
    recv: MPMCFutReceiver<EventRecv>,
    send: MPMCFutSender<EventSend>,
}

/// The internal manager of the connection in the networking code (in the
/// networking thread).
pub struct RegionConnectionInternal {
    recv: MPMCFutReceiver<EventSend>,
    send: MPMCFutSender<EventRecv>,
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
    pub fn send(&self, event: EventSend) {
        self.send.clone().send(event).map(|_| ()).wait();
        // TODO: The above line should not require the EventSend to be Clone,
        //       but the only alternative we have is try_send which can fail
        //       if the buffer is full and will require us to implement a waiting
        //       strategy here. → Ideally we can formulate this somehow using
        //       a Future and .wait()
    }

    pub fn recv(&self) -> Result<EventRecv, RecvError> {
        self.recv.recv()
    }
}

pub enum EventRecv {
    TerrainPatch(TerrainPatch),
}

#[derive(Clone)]
pub enum EventSend {}
