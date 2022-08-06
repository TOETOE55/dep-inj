use std::sync::Arc;
use the_world::{ContainerRef, TheWorld};

#[derive(Default)]
struct ContainerInner {
    even_state: even_impl::EvenState,
    odd_state: odd_impl::OddState,
}

#[derive(Default, Clone)]
struct Container(Arc<ContainerInner>);

impl Container {
    fn as_ref<Component>(&self) -> ContainerRef<Component>
    where
        Self: TheWorld<Component>
    {
        ContainerRef::new(dyn_clone::clone_box(self))
    }
}

// todo: if possible, generating it by macro;
mod boilerplate {
    use the_world::TheWorld;
    use crate::Container;

    impl even_api::Even for Container {
        fn is_even(&self, n: u64) -> bool {
            even_impl::EvenApp(self.as_ref()).is_even(n)
        }
    }

    impl odd_api::Odd for Container {
        fn is_odd(&self, n: u64) -> bool {
            odd_impl::OddApp(self.as_ref()).is_odd(n)
        }
    }

    impl TheWorld<odd_impl::OddState> for Container {
        fn project(&self) -> &odd_impl::OddState {
            &self.0.odd_state
        }
    }

    impl TheWorld<even_impl::EvenState> for Container {
        fn project(&self) -> &even_impl::EvenState {
            &self.0.even_state
        }
    }
}

impl Container {
    fn main_loop(&mut self, _inbox: ()) {
        loop {
            // dispatch events into different submodules
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Container;
    use even_api::Even;
    use odd_api::Odd;

    #[test]
    fn it_works() {
        let ctx = Container::default();
        assert!(ctx.is_odd(11));
        assert_eq!(ctx.0.odd_state.count(), 6);
        assert_eq!(ctx.0.even_state.count(), 5);

        assert!(ctx.is_even(12));
        assert_eq!(ctx.0.odd_state.count(), 12);
        assert_eq!(ctx.0.even_state.count(), 11);
    }
}
