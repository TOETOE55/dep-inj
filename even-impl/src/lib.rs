use crate::macro_expended::{DynEvenApp, EvenAppCtx};
use even_api::IsEven;
use odd_api::IsOdd;
use std::{
    fmt::Debug,
    ops::Deref,
    sync::{Arc, Mutex},
};

#[derive(Default, Debug)]
pub struct EvenState {
    count: Mutex<usize>,
}

// 通过`Ctx`注入需要的Service，以及自己实际的状态
//
// 假设有一个过程宏，用于注入Service，比如说EvenApp注入`IsOdd` trait
// #[derive(Delegate)]
// #[inj(IsOdd)]
//
// 甚至能 #[cfg_attr(windows, required(IsOdd))]
#[repr(transparent)]
pub struct EvenApp<Ctx: ?Sized>(
    // * 如果需要依赖不dyn safe的service，则Ctx需要使用泛型
    // * 如果依赖的的都是dyn safe的service，则Ctx可以取dyn XXXCtx
    Ctx,
);

// 展开成:
#[allow(dead_code)]
mod macro_expended {
    use super::{Arc, Deref, EvenApp, EvenState, IsOdd};
    // 防止用户添加实现
    impl<Ctx: ?Sized> Drop for EvenApp<Ctx> {
        fn drop(&mut self) {
            // 不应该有任何实现
        }
    }

    impl<Ctx: ?Sized> EvenApp<Ctx> {
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

    impl<Ctx: AsRef<EvenState> + ?Sized> Deref for EvenApp<Ctx> {
        type Target = EvenState;
        #[inline]
        fn deref(&self) -> &Self::Target {
            self.0.as_ref()
        }
    }

    // 注入依赖
    impl<Ctx: IsOdd + ?Sized> IsOdd for EvenApp<Ctx> {
        #[inline]
        fn is_odd(self: Arc<Self>, n: u64) -> bool {
            self.into_arc().is_odd(n)
        }
    }

    // 为了方便使用的一个Ctx，会包含所有注入的依赖
    pub(crate) trait EvenAppCtx: AsRef<EvenState> + IsOdd + Send + Sync + 'static {}
    impl<T: AsRef<EvenState> + IsOdd + Send + Sync + 'static> EvenAppCtx for T {}

    pub(crate) type DynEvenApp = EvenApp<dyn EvenAppCtx>;
}

impl<Ctx: EvenAppCtx> IsEven for EvenApp<Ctx> {
    fn is_even(self: Arc<Self>, n: u64) -> bool {
        is_even_impl(self, n)
    }

    fn emit_count<F>(&self, f: F)
    where
        F: FnOnce(usize),
    {
        // 输出一下调用次数
        f(*self.count.lock().unwrap());
    }
}

// 此处不依赖带泛型方法的service，使用DynApp有利于编译速度
fn is_even_impl(app: Arc<DynEvenApp>, n: u64) -> bool {
    *app.count.lock().unwrap() += 1;

    (n == 0) || app.is_odd(n - 1) // 因为app已经注入了IsOdd，所以可以直接使用is_odd方法
}
