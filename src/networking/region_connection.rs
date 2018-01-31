use data::TerrainPatch;
use futures::{Async, Future, Poll, Sink, Stream};
use futures::sync::mpsc;
//use multiqueue::{self, mpsc::Receiver, mpsc::Sender};
use std::sync::mpsc::{RecvError, SendError};

/// Managing the connection in the client code.
pub struct RegionConnection {
    recv: mpsc::Receiver<EventRecv>,
    send: mpsc::Sender<EventSend>,
}

/// The internal manager of the connection in the networking code (in the
/// networking thread).
pub struct RegionConnectionInternal {
    pub recv: mpsc::Receiver<EventSend>,
    pub send: mpsc::Sender<EventRecv>,
}

pub fn new_pair() -> (RegionConnection, RegionConnectionInternal) {
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

pub enum EventRecv {
    TerrainPatch(TerrainPatch),
    ConnectResult(Result<(), ()>),
}

pub enum EventSend {}
