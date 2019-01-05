use prometheus::{Counter, Encoder, Gauge, Opts, Registry, TextEncoder};
use std::boxed::Box;

pub struct Metrics {
    cache_hits: Counter,
    cache_miss: Counter,
    last_cache_check: Gauge,
    registry: Registry,
}

impl Metrics {
    pub fn inc_cache_hits(&mut self) {
        self.cache_hits.inc();
    }

    pub fn inc_cache_miss(&mut self) {
        self.cache_miss.inc();
    }

    pub fn cache_check(&mut self, timestamp: u64) {
        self.last_cache_check.set(timestamp as f64);
    }

    pub fn metrics(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        encoder.encode(&metric_families, &mut buffer).unwrap();
        buffer
    }
}

impl Default for Metrics {
    fn default() -> Self {
        let cache_hits_opts = Opts::new("cache_hits", "Counts the cache hits");
        let cache_hits = Counter::with_opts(cache_hits_opts).unwrap();

        let cache_miss_opts = Opts::new("cache_miss", "Counts the cache misses");
        let cache_miss = Counter::with_opts(cache_miss_opts).unwrap();

        let last_cache_check_opts = Opts::new(
            "last_cache_check",
            "Last time the ttl of all entries in the cache where checked",
        );
        let last_cache_check = Gauge::with_opts(last_cache_check_opts).unwrap();

        let registry = Registry::new();
        registry.register(Box::new(cache_hits.clone())).unwrap();
        registry.register(Box::new(cache_miss.clone())).unwrap();
        registry
            .register(Box::new(last_cache_check.clone()))
            .unwrap();

        Self {
            cache_hits,
            cache_miss,
            last_cache_check,
            registry,
        }
    }
}
