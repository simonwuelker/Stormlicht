#![feature(lazy_cell)]

mod loader;

use loader::{LoadCompletion, ResourceLoadRequest, ResourceLoader};
use sl_std::oneshot;

use std::{
    fmt,
    sync::{mpsc, LazyLock},
    thread,
};

use url::URL;

// FIXME: All the resource-related stuff should live in this crate
pub use mime::{Resource, ResourceLoadError};

pub static RESOURCE_LOADER: LazyLock<ResourceThreadHandle> = LazyLock::new(|| {
    let (tx, rx) = mpsc::channel();

    let thread_handle = thread::Builder::new()
        .name("ResourceLoader".to_string())
        .spawn(|| ResourceLoader::start(rx))
        .expect("Failed to spawn ResourceLoader thread");

    let resource_loader = ResourceThreadHandle {
        thread_handle,
        sender: tx,
    };

    resource_loader
});

/// A handle held by the main thread to communicate
/// with the resource thread
pub struct ResourceThreadHandle {
    /// Handle for the ResourceLoader thread
    thread_handle: thread::JoinHandle<()>,

    /// Channel to forward incoming requests to the ResourceLoader
    sender: mpsc::Sender<ResourceLoadRequest>,
}

/// Indicates that a message could not be sent because the resource thread
/// disconnected.
pub struct ResourceLoaderDisconnected;

/// Handle held by the user
pub struct PendingLoad {
    receiver: oneshot::Receiver<LoadCompletion>,
}

impl PendingLoad {
    #[must_use]
    pub fn block(self) -> LoadCompletion {
        // Don't propagate the error to the user because this is never
        // expected to happen.
        self.receiver
            .receive_blocking()
            .expect("Failed to receive response")
    }
}

impl ResourceThreadHandle {
    #[must_use]
    pub fn thread_handle(&self) -> &thread::JoinHandle<()> {
        &self.thread_handle
    }

    pub fn try_schedule_load(&self, url: URL) -> Result<PendingLoad, ResourceLoaderDisconnected> {
        let (sender, receiver) = oneshot::Channel::create();

        let client = ResourceLoadRequest::new(url, sender);

        // We ignore the send error and propagate an opaque ResourceLoaderDisconnected since
        // the error only contains the request itself, which we don't care about from the outside.
        self.sender
            .send(client)
            .map_err(|_| ResourceLoaderDisconnected)?;

        let load_handle = PendingLoad { receiver };

        Ok(load_handle)
    }

    /// Request a resource to be loaded
    ///
    /// Called from the main thread.
    ///
    /// # Panics
    ///
    /// Panics if the communication with the resource thread failed.
    /// If you want to handle the error gracefully instead, use [Self::try_schedule_load].
    #[must_use]
    pub fn schedule_load(&self, url: URL) -> PendingLoad {
        self.try_schedule_load(url)
            .expect("Failed to schedule load request")
    }
}

impl fmt::Debug for ResourceLoaderDisconnected {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ResourceLoader disconnected")
    }
}
