use std::sync::Arc;

pub trait IsEven {
    // 支持`Arc<Self>`，意味着允许跨线程使用`Self`
    fn is_even(self: Arc<Self>, n: u64) -> bool;

    // 使得IsEven不dyn safe
    fn emit_count<F>(&self, f: F)
    where
        F: FnOnce(usize);
}
