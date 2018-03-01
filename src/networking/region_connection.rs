//! RegionConnection exposes actions to be performed through the networking
//! thread, for communication with specific simulators.

use data::TerrainPatch;
use futures::{Async, Future, Poll, Sink, Stream};
use futures::sync::mpsc;
use std::sync::mpsc::{RecvError, SendError};

/// The main handle to perform communication with a region, on the networking thread
/// managed by the Networking struct.
pub struct RegionConnection {
    recv: mpsc::Receiver<EventRecv>,
    send: mpsc::Sender<EventSend>,
}

/// Internal counterpart of `RegionConnection`, this is what the networking thread
/// uses to communicate with the rest of the client code.
pub(super) struct RegionConnectionInternal {
    pub recv: mpsc::Receiver<EventSend>,
    pub send: mpsc::Sender<EventRecv>,
}

/// Creates a new
pub(super) fn new_pair() -> (RegionConnection, RegionConnectionInternal) {
    let max_buffer = 256;
    let (send1, recv1) = mpsc::channel(max_buffer);
    let (send2, recv2) = mpsc::channel(max_buffer);
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

/// The events that can be sent out from the viewer to the region.
pub(super) enum EventSend {

}

/// The events that can be received from the region by the viewer.
pub(super) enum EventRecv {
    TerrainPatch(TerrainPatch), // ConnectResult(Result<(), ()>)
}

/*
pub struct Recv<'a> {
    recv: &'a mut mpsc::Receiver<EventRecv>,
}

impl<'a> Future for Recv<'a> {
    type Item = EventRecv;
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.recv.poll() {
            Ok(Async::Ready(Some(val))) => Ok(Async::Ready(val)),
            Ok(Async::Ready(None)) => Err(()),
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(e) => Err(()),
        }
    }
}

TODO
impl RegionConnection {
    pub fn send(&self, event: EventSend) -> Result<(), mpsc::SendError<EventSend>> {
        self.send.clone().send(event).map(|_| ()).wait()
    }

    pub fn recv<'a>(&'a mut self) -> Recv<'a> {
        Recv {
            recv: &mut self.recv,
        }
    }
}
*/
