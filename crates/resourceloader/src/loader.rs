use std::{
    collections::HashMap,
    mem,
    sync::{mpsc, Arc},
};

use sl_std::oneshot;
use url::URL;

pub struct ResourceLoader {
    receiver: mpsc::Receiver<ResourceLoadRequest>,
    cache: HashMap<URL, Arc<mime::Resource>>,
    pending_loads: Vec<ResourceLoadRequest>,
}

/// A handle to a resource being fetched
///
/// A [ResourceClient] is unable to write to the resource and can only
/// read from it.
///
/// A resource is shared and can cheaply be cloned.
pub struct ResourceLoadRequest {
    /// The location of the resource that should be loaded
    pub url: URL,

    pub sender: oneshot::Sender<LoadCompletion>,
}

pub type LoadCompletion = Result<Arc<mime::Resource>, mime::ResourceLoadError>;

impl ResourceLoadRequest {
    #[must_use]
    pub fn new(url: URL, sender: oneshot::Sender<LoadCompletion>) -> Self {
        Self { url, sender }
    }
}

impl ResourceLoader {
    /// Starts a [ResourceLoader] instance on the current thread
    pub fn start(receiver: mpsc::Receiver<ResourceLoadRequest>) {
        log::info!("Starting ResourceLoader thread");

        let mut loader = Self {
            receiver,
            cache: HashMap::default(),
            pending_loads: Vec::default(),
        };

        loader.run();
    }

    fn run(&mut self) {
        loop {
            if self.pending_loads.is_empty() {
                self.wait_for_request();
            }

            // FIXME: make this concurrent
            self.handle_pending_loads();
        }
    }

    fn wait_for_request(&mut self) {
        let request = self.receiver.recv().expect("Main thread disconnected");
        self.handle_incoming_request(request);

        loop {
            match self.receiver.try_recv() {
                Ok(request) => {
                    self.handle_incoming_request(request);
                },
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => panic!("Main thread disconnected"),
            }
        }
    }

    /// Looks up a cache entry for a given request and adds it to the list of
    /// pending loads if no cache entry is present.
    fn handle_incoming_request(&mut self, request: ResourceLoadRequest) {
        if let Some(cached_resource) = self.cache.get(&request.url) {
            let response = Ok(cached_resource.clone());
            let was_sent = request.sender.send(response).is_ok();
            assert!(was_sent, "Receiver disconnected");
            return;
        }

        // This request is not in the cache, create a new handle for it
        self.pending_loads.push(request);
    }

    fn handle_pending_loads(&mut self) {
        for pending_load in mem::take(&mut self.pending_loads) {
            let completion = mime::Resource::load(&pending_load.url).map(Arc::new);

            if let Ok(resource) = &completion {
                self.cache.insert(pending_load.url, resource.clone());
            }

            let was_sent = pending_load.sender.send(completion).is_ok();
            assert!(was_sent, "Receiver disconnected");
        }
    }
}
