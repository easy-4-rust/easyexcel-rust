//! Regression coverage for the upstream ref-swap 0.1.3 security fix.
//!
//! ref-swap 0.1.3 makes the reference lifetime invariant and makes
//! OptionRefSwap::into_inner handle a null pointer safely. Keep these tests in
//! the consumer workspace so they remain exercised without vendoring the crate.

use ref_swap::{OptionRefSwap, RefSwap};
use std::sync::atomic::Ordering;

#[test]
fn ref_swap_try_update_preserves_cas_semantics() {
    let initial = 1_u8;
    let next = 2_u8;
    let swap = RefSwap::new(&initial);

    let result = swap.try_update(Ordering::AcqRel, Ordering::Acquire, |current| {
        (*current == initial).then_some(&next)
    });

    assert!(matches!(result, Ok(previous) if *previous == initial));
    assert_eq!(*swap.load(Ordering::Acquire), next);
}

#[test]
fn option_ref_swap_try_update_preserves_none_to_some_transition() {
    let next = 2_u8;
    let swap = OptionRefSwap::new(None);

    let result = swap.try_update(Ordering::AcqRel, Ordering::Acquire, |current| {
        current.is_none().then_some(Some(&next))
    });

    assert!(matches!(result, Ok(None)));
    assert_eq!(swap.load(Ordering::Acquire).copied(), Some(next));
    assert_eq!(swap.into_inner().copied(), Some(next));
}

#[test]
fn option_ref_swap_into_inner_handles_none_without_dereferencing_null() {
    let swap = OptionRefSwap::<u8>::new(None);

    assert!(swap.into_inner().is_none());
}
