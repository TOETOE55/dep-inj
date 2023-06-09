use even_api::IsEven;
use odd_api::IsOdd;
use std::{
    fmt::Debug,
    ops::Deref,
    sync::{Arc, Mutex},
};

#[derive(Default, Debug)]
pub struct OddState {
    count: Mutex<usize>,
}

// 解释同even_impl
#[repr(transparent)]
pub struct OddApp<Ctx: ?Sized>(Ctx);
mod macro_expended {
    use super::{Arc, Deref, OddApp, OddState};
    use even_api::IsEven;
    // 防止用户添加实现
    impl<Ctx: ?Sized> Drop for OddApp<Ctx> {
        fn drop(&mut self) {
            // 不应该有任何实现
        }
    }

    impl<Ctx: ?Sized> OddApp<Ctx> {
        #[inline]
        pub fn from_ref(ctx: &Ctx) -> &Self {
            unsafe { &*(ctx as *const Ctx as *const _) }
        }

        #[inline]
        pub fn into_ref(&self) -> &Ctx {
            unsafe { &*(self as *const Self as *const _) }
        }

        #[inline]
        pub fn from_arc(ctx: Arc<Ctx>) -> Arc<Self> {
            unsafe { Arc::from_raw(Arc::into_raw(ctx) as *const Self) }
        }

        #[inline]
        pub fn into_arc(self: Arc<Self>) -> Arc<Ctx> {
            unsafe { Arc::from_raw(Arc::into_raw(self) as *const Ctx) }
        }
    }

    impl<Ctx: AsRef<OddState> + ?Sized> Deref for OddApp<Ctx> {
        type Target = OddState;
        #[inline]
        fn deref(&self) -> &Self::Target {
            self.0.as_ref()
        }
    }

    impl<Ctx: IsEven + ?Sized> IsEven for OddApp<Ctx> {
        #[inline]
        fn is_even(self: Arc<Self>, n: u64) -> bool {
            self.into_arc().is_even(n)
        }
        #[inline]
        fn emit_count<F>(&self, f: F)
        where
            F: FnOnce(usize),
        {
            self.into_ref().emit_count(f)
        }
    }
}

// 解释同even_impl
pub trait OddAppCtx: AsRef<OddState> + IsEven + Send + Sync + 'static {}
impl<T: AsRef<OddState> + IsEven + Send + Sync + 'static> OddAppCtx for T {}

impl<Ctx: OddAppCtx> IsOdd for OddApp<Ctx> {
    #[inline]
    fn is_odd(self: Arc<Self>, n: u64) -> bool {
        is_odd_impl(self, n)
    }
}

// 这里需要到泛型方法`emit_count`，所以这里不能用`OddApp<dyn OddAppCtx>`
fn is_odd_impl<Ctx: OddAppCtx>(app: Arc<OddApp<Ctx>>, n: u64) -> bool {
    *app.count.lock().unwrap() += 1;

    if n == 0 {
        return false;
    }

    app.emit_count(|even_count| {
        println!("IsEven::is_even was called {even_count} times");
    });
    app.is_even(n - 1)
}
