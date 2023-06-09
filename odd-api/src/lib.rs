use std::sync::Arc;
pub trait IsOdd {
    fn is_odd(self: Arc<Self>, n: u64) -> bool;
}
