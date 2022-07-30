#[derive(Default)]
pub struct EvenState {
    count: u64,
}

impl EvenState {
    pub fn count(&self) -> u64 {
        self.count
    }
}

/// workaround for cyclic deps between the crates
pub struct EvenApp<Ctx>(pub Ctx);


impl<Ctx: AsMut<EvenState>> even_api::Even for EvenApp<&mut Ctx>
where
    // addition dependency
    Ctx: odd_api::Odd
{
    fn is_even(&mut self, n: u64) -> bool {
        self.0.as_mut().count += 1;

        (n == 0) || self.0.is_odd(n - 1)
    }
}

