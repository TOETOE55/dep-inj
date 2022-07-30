
#[derive(Default)]
pub struct OddState {
    count: u64,
}

impl OddState {
    pub fn count(&self) -> u64 {
        self.count
    }
}

/// workaround for cyclic deps between the crates
pub struct OddApp<Ctx>(pub Ctx);


impl<Ctx: AsMut<OddState>> odd_api::Odd for OddApp<&mut Ctx>
where
    // addition dependency
    Ctx: even_api::Even
{
    fn is_odd(&mut self, n: u64) -> bool {
        self.is_odd_impl(n)
    }
}

impl<Ctx: AsMut<OddState>> OddApp<&mut Ctx>
where
    Ctx: even_api::Even
{
    fn is_odd_impl(&mut self, n: u64) -> bool {
        self.0.as_mut().count += 1;

        (n == 1) || self.0.is_even(n - 1)
    }
}
