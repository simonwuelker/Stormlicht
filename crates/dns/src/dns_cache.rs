use std::{
    collections::HashMap,
    net::IpAddr,
    sync::{LazyLock, Mutex},
    time::{Duration, Instant},
};

use crate::{DNSError, Domain};

const MAX_CACHE_SIZE: usize = 1000;

/// Caches recently resolved domains so we don't have to resolve them multiple times
pub static DNS_CACHE: LazyLock<Cache> = LazyLock::new(|| {
    log::info!("Initializing global DNS cache");
    Cache::default()
});

#[derive(Debug, Default)]
pub struct Cache {
    cache: Mutex<LockedCache>,
}

#[derive(Debug, Default)]
pub struct LockedCache(HashMap<Domain, CacheEntry>);

impl Cache {
    pub fn insert(&self, domain: Domain, ip: IpAddr, ttl: u32) {
        if ttl == 0 {
            return;
        }

        self.cache
            .lock()
            .expect("DNS Cache lock was poisoned")
            .insert(domain, ip, ttl);
    }

    /// Try to get an entry from the cache.
    ///
    /// If the entry is present but expired, it is deleted and `None` is returned.
    pub fn get(&self, domain: &Domain) -> Result<IpAddr, DNSError> {
        let now = Instant::now();
        let mut locked_cache = self.cache.lock().expect("DNS Cache lock was poisoned");
        let cached_entry = locked_cache.0.get_mut(domain);

        match cached_entry {
            Some(entry) if now < entry.expires_at => {
                // The entry is valid, great, nothing else to do
                entry.last_accessed = now;
                Ok(entry.ip)
            },
            _ => {
                // Resolving a domain is recursive, so to prevent deadlocks we drop the lock here
                drop(locked_cache);

                // We need to resolve the domain and put it in the cache
                let (ip, ttl) = domain.resolve()?;

                // A ttl of 0 means we shouldn't cache the entry at all
                if ttl == 0 {
                    return Ok(ip);
                }

                // It's safe to reacquire the lock now since we've done all lookups
                let mut locked_cache = self.cache.lock().expect("DNS Cache lock was poisoned");
                if locked_cache.len() >= MAX_CACHE_SIZE {
                    locked_cache.compact();

                    if locked_cache.len() > MAX_CACHE_SIZE {
                        let to_remove = locked_cache.len() - MAX_CACHE_SIZE;
                        locked_cache.evict_n_least_recently_used(to_remove);
                    }
                }

                locked_cache.insert(domain.clone(), ip, ttl);
                Ok(ip)
            },
        }
    }
}

impl LockedCache {
    /// Try to free space in the cache by clearing expired entrys
    fn compact(&mut self) {
        let now = Instant::now();
        self.0.retain(|_, entry| now < entry.expires_at);
    }

    fn evict_n_least_recently_used(&mut self, n: usize) {
        log::debug!("DNS cache is full, evicting {n} items");

        let mut items = Vec::with_capacity(self.0.len());
        for (key, value) in self.0.iter() {
            items.push((key.clone(), *value));
        }
        items.sort_by(|a, b| a.1.last_accessed.cmp(&b.1.last_accessed));

        // remove the first n elements from the cache
        for item in items.iter().take(n) {
            self.0.remove(&item.0);
        }
    }

    fn insert(&mut self, domain: Domain, ip: IpAddr, ttl: u32) {
        // FIXME: For some reason, opening an ipv6 connection with udp fails on linux.
        //        For now, we can't use ipv6 with dns at all. Maybe AAAA records are only
        //        useful for things like DNS over HTTPS?
        if ip.is_ipv6() {
            return;
        }

        self.0.insert(
            domain,
            CacheEntry {
                expires_at: Instant::now() + Duration::from_secs(ttl as u64),
                last_accessed: Instant::now(),
                ip,
            },
        );
    }

    #[must_use]
    fn len(&self) -> usize {
        self.0.len()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CacheEntry {
    expires_at: Instant,
    last_accessed: Instant,
    // In the future we might want to differentiate between IPv4 and IPv6 here
    ip: IpAddr,
}
