use std;
use std::thread::{JoinHandle, self};
use futures::sync::mpsc;
use tokio_core::reactor::{Core, Remote};
use opensim_networking::logging::Log;
use opensim_networking::simulator::SimLocator;
use opensim_networking::simulator::manager::SimManager;
use opensim_networking::circuit::SendMessage;
use opensim_networking::messages::MessageInstance;

/// Holds all the senders which are found in a SimRemote.
#[derive(Clone)]
struct Senders {
    message: mpsc::Sender<RequestSendMessage>
}

struct RequestSendMessage {
    instance: MessageInstance,
    reliable: bool,
    target: SimLocator,
}

/// This a `Send + Sync` interface to the actual networking code
/// running in a dedicated thread.
pub struct Networking {
    thread_handle: JoinHandle<()>,
    senders: Senders,
    remote: Remote,
}

impl Networking {
    pub fn new(log: Log) -> Self {
        let (message_tx, message_rx) = mpsc::channel(128);
        let (remote_tx, remote_rx) = std::sync::mpsc::channel();

        let thread_handle = thread::spawn(move || {
            let core = Core::new().unwrap();
            remote_tx.send(core.remote()).unwrap();
            let sim_manager = SimManager::new(core.handle(), log);

        });

        let remote = remote_rx.recv().unwrap();

        Networking {
            thread_handle: thread_handle,
            senders: Senders {
                message: message_tx,
            },
            remote: remote,
        }
    }

    /// In some instances it might be nesecssary to have the remote from the networking thread.
    pub fn core_remote(&self) -> &Remote {
        &self.remote
    }

    pub fn setup_remote(&self, locator: &SimLocator) -> Result<SimRemote, ()> {
        unimplemented!()
    }
}

/// An interface for communicating with a simulator,
/// it's called remote because it can be used like a remote without a hold
/// of the actual Networking struct.
pub struct SimRemote {
    // TODO: Right now this is copied for every request which is very expensive.
    // In the future we might want to hold a table of all connected sims and give them session
    // unique ids, so that only a u32 has to be copied on every request instead of this whole
    // struct.
    locator: SimLocator,
}

impl SimRemote {
    pub fn send_message<M: Into<MessageInstance>>(
        &self,
        message: M,
        reliable: bool
    ) -> SendMessage {
        unimplemented!()
    }
}
