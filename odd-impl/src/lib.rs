use std::sync::atomic::{AtomicU64, Ordering};
use odd_api::Odd;
use the_world::ContainerRef;

#[derive(Default)]
pub struct OddState {
    count: AtomicU64,
}

impl OddState {
    pub fn count(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
    }
}

/// workaround for cyclic deps between the crates
#[derive(Clone)]
pub struct OddApp(pub ContainerRef<OddState>);

impl odd_api::Odd for OddApp {
    fn is_odd(&self, n: u64) -> bool {
        self.is_odd_impl(n)
    }
}

impl OddApp {
    fn is_odd_impl(&self, n: u64) -> bool {
        self.0.count.fetch_add(1, Ordering::SeqCst);

        (n == 1) || self.0.the_world().is_even(n - 1)
    }

    fn _can_be_share_between_threads(&self) {
        let cloned = self.clone();
        std::thread::spawn(move || {
           assert!(!cloned.is_odd(10));
        });
    }
}