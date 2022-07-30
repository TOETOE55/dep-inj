use viu::Views;

#[derive(Default)]
// generate view type for partial borrowing
#[derive(Views)]
#[view_as(view_mutable_even_and_odd)]
struct GlobalStruct {
    #[mut_in(view_mutable_even_and_odd)]
    even_state: even_impl::EvenState,
    #[mut_in(view_mutable_even_and_odd)]
    odd_state: odd_impl::OddState,
}

// todo: if possible, generating it by macro;
mod boilerplate {
    use super::view_mutable_even_and_odd;

    impl even_api::Even for view_mutable_even_and_odd<'_, '_> {
        fn is_even(&mut self, n: u64) -> bool {
            even_impl::EvenApp(self).is_even(n)
        }
    }

    impl odd_api::Odd for view_mutable_even_and_odd<'_, '_> {
        fn is_odd(&mut self, n: u64) -> bool {
            odd_impl::OddApp(self).is_odd(n)
        }
    }

    impl AsMut<even_impl::EvenState> for view_mutable_even_and_odd<'_, '_> {
        fn as_mut(&mut self) -> &mut even_impl::EvenState {
            &mut self.even_state
        }
    }

    impl AsMut<odd_impl::OddState> for view_mutable_even_and_odd<'_, '_> {
        fn as_mut(&mut self) -> &mut odd_impl::OddState {
            &mut self.odd_state
        }
    }
}

impl GlobalStruct {
    fn main_loop(&mut self, _inbox: (/* external events */)) {
        loop {
            // dispatch events into different submodules
        }
    }
}

#[cfg(test)]
mod tests {
    use even_api::Even;
    use odd_api::Odd;
    use super::{GlobalStruct, view_mutable_even_and_odd};

    #[test]
    fn it_works() {
        let mut ctx = GlobalStruct::default();
        let mut view = view_mutable_even_and_odd_ctor!(ctx);
        assert!(view.is_odd(11));
        assert_eq!(view.odd_state.count(), 6);
        assert_eq!(view.even_state.count(), 5);

        assert!(view.is_even(12));
        assert_eq!(view.odd_state.count(), 12);
        assert_eq!(view.even_state.count(), 11);
    }
}
