use crossbeam_channel as channel;
pub use crossbeam_channel::{RecvError, SendError};
use data::TerrainPatch;

/// Managing the connection in the client code.
pub struct RegionConnection {
    recv: channel::Receiver<EventRecv>,
    send: channel::Sender<EventSend>,
}

/// The internal manager of the connection in the networking code (in the
/// networking thread).
pub struct RegionConnectionInternal {
    recv: channel::Receiver<EventSend>,
    send: channel::Sender<EventRecv>,
}

pub fn new_pair() -> (RegionConnection, RegionConnectionInternal) {
    let max_buffer = 256;
    let (send1, recv1) = channel::bounded(max_buffer);
    let (send2, recv2) = channel::bounded(max_buffer);
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
        self.send.send(event)
    }

    pub fn recv(&self) -> Result<EventRecv, RecvError> {
        self.recv.recv()
    }
}

pub enum EventRecv {
    TerrainPatch(TerrainPatch),
}

pub enum EventSend {}
