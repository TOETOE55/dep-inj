use even_api::IsEven;
use odd_api::IsOdd;
use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
};

#[derive(Default, Debug, dep_inj::DepInj)]
#[target(OddApp)]
pub struct OddState {
    count: Mutex<usize>,
}

pub(crate) trait OddAppDeps: AsRef<OddState> + IsEven + Send + Sync + 'static {}
impl<T: AsRef<OddState> + IsEven + Send + Sync + 'static> OddAppDeps for T {}

#[allow(unused)]
pub(crate) type DynOddApp = OddApp<dyn OddAppDeps>;

impl<Ctx: OddAppDeps> IsOdd for OddApp<Ctx> {
    #[inline]
    fn is_odd(self: Arc<Self>, n: u64) -> bool {
        is_odd_impl(self, n)
    }
}

// 这里需要到泛型方法`emit_count`，所以这里不能用`DynOddApp`
fn is_odd_impl<Ctx: OddAppDeps>(app: Arc<OddApp<Ctx>>, n: u64) -> bool {
    *app.count.lock().unwrap() += 1;

    if n == 0 {
        return false;
    }

    app.deps_ref().emit_count(|even_count| {
        if even_count > 100 {
            println!("IsEven::is_even was called over {even_count} times");
        }
    });
    app.deps_arc().is_even(n - 1)
}
