use std::thread::{Thread, thread};
use tokio_core::reactor::Core;
use opensim_networking::logging::Log;
use opensim_networking::simulator::manager::SimManager;

/// This a `Send + Sync` interface to the actual networking code
/// running in a dedicated thread.
pub struct Networking {
    thread_handle: Thread,
}

impl Networking {
    pub fn new(log: Log) -> Self {
        let thread_handle = thread::spawn(move || {
            let core = Core::new();
            let sim_manager = SimManager::new(core.handle(), log);


        });

        Networking {
            thread_handle: thread_handle,
        }
    }
}
