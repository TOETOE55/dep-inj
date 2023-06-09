use std::sync::Arc;

use even_api::IsEven;
use even_impl::{EvenApp, EvenState};
use odd_api::IsOdd;
use odd_impl::{OddApp, OddState};

// 将所有的State管理起来
// 实现所有的Service，以及AsRef<XXState>
#[derive(Default, Debug)]
pub struct GlobalStruct {
    // 可以是lazy的field
    odd_state: OddState,
    even_state: EvenState,
}

mod macro_expanded {
    use super::{Arc, EvenApp, EvenState, GlobalStruct, IsEven, IsOdd, OddApp, OddState};
    impl AsRef<OddState> for GlobalStruct {
        fn as_ref(&self) -> &OddState {
            &self.odd_state
        }
    }

    impl AsRef<EvenState> for GlobalStruct {
        fn as_ref(&self) -> &EvenState {
            &self.even_state
        }
    }

    impl IsOdd for GlobalStruct {
        #[inline]
        fn is_odd(self: Arc<Self>, n: u64) -> bool {
            OddApp::from_arc(self).is_odd(n)
        }
    }

    impl IsEven for GlobalStruct {
        #[inline]
        fn is_even(self: Arc<Self>, n: u64) -> bool {
            EvenApp::from_arc(self).is_even(n)
        }

        #[inline]
        fn emit_count<F>(&self, f: F)
        where
            F: FnOnce(usize),
        {
            EvenApp::from_ref(self).emit_count(f)
        }
    }
}

fn main() {
    let global = Arc::new(GlobalStruct::default());

    assert!(global.clone().is_even(100));
    assert!(global.clone().is_odd(101));

    dbg!(global);
}
