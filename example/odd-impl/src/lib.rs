use even_api::IsEven;
use odd_api::IsOdd;
use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
};

#[derive(Default, Debug, dep_inj::DepInj)]
#[target(OddProxy)]
pub struct OddState {
    count: Mutex<usize>,
}

pub(crate) trait OddDeps: AsRef<OddState> + IsEven + Send + Sync + 'static {}
impl<T: AsRef<OddState> + IsEven + Send + Sync + 'static> OddDeps for T {}

#[allow(unused)]
pub(crate) type DynOddProxy = OddProxy<dyn OddDeps>;

impl<Ctx: OddDeps> IsOdd for OddProxy<Ctx> {
    #[inline]
    fn is_odd(self: Arc<Self>, n: u64) -> bool {
        is_odd_impl(self, n)
    }
}

fn is_odd_impl<Ctx: OddDeps>(app: Arc<OddProxy<Ctx>>, n: u64) -> bool {
    *app.count.lock().unwrap() += 1;

    if n == 0 {
        return false;
    }

    // `DynOddProxy` cannot call `emit_count`
    app.prj_ref().emit_count(|even_count| {
        if even_count > 100 {
            println!("IsEven::is_even was called over {even_count} times");
        }
    });
    app.prj_arc().is_even(n - 1)
}
