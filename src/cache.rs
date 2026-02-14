use moka::future::Cache;
use samael::metadata::EntityDescriptor;
use std::time::Duration;

pub struct MdqCache {
    inner: Cache<String, EntityDescriptor>,
}

impl MdqCache {
    pub fn new(max_entries: u64, ttl: Duration) -> Self {
        Self {
            inner: Cache::builder()
                .max_capacity(max_entries)
                .time_to_live(ttl)
                .build(),
        }
    }

    pub async fn get(&self, key: &str) -> Option<EntityDescriptor> {
        self.inner.get(key).await
    }

    pub async fn insert(&self, key: String, value: EntityDescriptor) {
        self.inner.insert(key, value).await;
    }

    pub async fn invalidate(&self, key: &str) {
        self.inner.invalidate(key).await;
    }
}

impl Default for MdqCache {
    fn default() -> Self {
        Self::new(1000, Duration::from_secs(3600))
    }
}
