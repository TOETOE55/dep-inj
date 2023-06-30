// use crate::macro_expended::{DynEvenApp, EvenAppCtx};
use dep_inj::DepInj;
use even_api::IsEven;
use odd_api::IsOdd;
use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
};

#[derive(Default, Debug, DepInj)]
#[target(EvenApp)]
pub struct EvenState {
    count: Mutex<usize>,
}

// just an alias for easier coding
// trait EvenAppDeps = AsRef<EvenState> + IsOdd + Send + Sync + 'static;
//
// * 不同模块若依赖不同的trait可以定义不同的Deps
pub(crate) trait EvenAppDeps: AsRef<EvenState> + IsOdd + Send + Sync + 'static {}
impl<T: AsRef<EvenState> + IsOdd + Send + Sync + 'static> EvenAppDeps for T {}

// 使用dyn有利于编译速度
pub(crate) type DynEvenApp = EvenApp<dyn EvenAppDeps>;

impl<Ctx: EvenAppDeps> IsEven for EvenApp<Ctx> {
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

    (n == 0) || app.prj_arc().is_odd(n - 1)
}
