# dep-inj

A tool for easier dependency injection.

# Example

When you write a component(named `OddState` here) that need be injected some dependency(named `Even` here) from other crates, `dep_inj` can help you.


```ignore
//! odd component implementation


use dep_inj::DepInj;

#[derive(Default, DepInj)]
// to generator `OddProxy`
#[target(OddProxy)]
pub struct OddState {
    count: usize,
}

// `OddState` provides `Odd` 
impl<Deps> Odd for OddProxy<Deps> 
where
    // `is_odd` depends on `Even`
    Deps: Even
{
    fn is_odd(&mut self, n: u64) -> bool {
        // `OddProxy` can be deref to `OddState`
        self.count += 1;

        (n == 1) || self
            .prj_ref_mut()
            // `OddProxy` can be projected to its dependency `Even`
            // where
            // ```
            // trait Even {
            //     fn is_even(&mut self, n: u64) -> bool;
            // }
            // ```
            .is_even(n)
    }
}
```

and you can inject the dependency in the top crate:

```ignore

#[derive(Default)]
struct GlobalState {
    odd_state: OddState,
    even_state: EvenState,
}

// boilerplate
// `impl AsRef<OddState> for GlobalState`
// `impl AsMut<OddState> for GlobalState`
// `impl AsRef<EvenState> for GlobalState`
// `impl AsMut<EvenState> for GlobalState`

impl Even for GlobalState {
    fn is_even(&mut self, n: u64) -> bool {
        // inject `GlobalState` which is impl `Odd` for `EvenState` component
        EvenProxy::inj_ref_mut(self).is_even(n)
    }
}

impl Odd for GlobalState {
    fn is_odd(&mut self, n: u64) -> bool {
        // inject `GlobalState` which is impl `Even` for `OddState` component
        OddProxy::inj_ref_mut(self).is_odd(n)
    }
}

fn main() {
    let mut state = GlobalState::default();
    assert!(state.is_odd(11));
}
```

See more at [Exploring Design Patterns in Rust Inter-Component Interface Invocation](./doc/Exploring%20Design%20Patterns%20in%20Rust%20Inter-Component%20Interface%20Invocation.md)
