use dep_inj::DepInj;
use even_api::IsEven;
use odd_api::IsOdd;
use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
};

#[derive(Default, Debug, DepInj)]
#[target(EvenProxy)]
pub struct EvenState {
    count: Mutex<usize>,
}

// just an alias for easier coding
// `trait EvenDeps = AsRef<EvenState> + IsOdd + Send + Sync + 'static;`
pub(crate) trait EvenDeps: AsRef<EvenState> + IsOdd + Send + Sync + 'static {}
impl<T: AsRef<EvenState> + IsOdd + Send + Sync + 'static> EvenDeps for T {}

// dyn may benefit compilation speed
pub(crate) type DynEvenApp = EvenProxy<dyn EvenDeps>;

impl<Ctx: EvenDeps> IsEven for EvenProxy<Ctx> {
    fn is_even(self: Arc<Self>, n: u64) -> bool {
        is_even_impl(self, n)
    }

    fn emit_count<F>(&self, f: F)
    where
        F: FnOnce(usize),
    {
        // emit call count
        f(*self.count.lock().unwrap());
    }
}

fn is_even_impl(app: Arc<DynEvenApp>, n: u64) -> bool {
    *app.count.lock().unwrap() += 1;

    (n == 0) || app.prj_arc().is_odd(n - 1)
}
