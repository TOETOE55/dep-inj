
use std::sync::atomic::{AtomicU64, Ordering};
use the_world::ContainerRef;

#[derive(Default)]
pub struct EvenState {
    count: AtomicU64,
}

impl EvenState {
    pub fn count(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
    }
}


/// workaround for cyclic deps between the crates
#[derive(Clone)]
pub struct EvenApp(pub ContainerRef<EvenState>);


impl even_api::Even for EvenApp {
    fn is_even(&self, n: u64) -> bool {
        self.0.count.fetch_add(1, Ordering::SeqCst);

        (n == 0) || self.0.the_world().is_odd(n - 1)
    }
}

