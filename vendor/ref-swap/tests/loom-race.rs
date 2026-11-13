#![cfg(loom)]

use loom::sync::atomic::AtomicU32;
use loom::thread;

use ref_swap::RefSwap;

#[allow(deprecated)]
use std::sync::atomic::ATOMIC_BOOL_INIT;
use std::sync::atomic::{AtomicBool, Ordering::*};

const BRANCHES_COUNT: usize = 2;
#[allow(deprecated)]
static BRANCHES_USED: [AtomicBool; BRANCHES_COUNT] = [ATOMIC_BOOL_INIT; BRANCHES_COUNT];

struct Dropper<T> {
    ptr: *mut T,
}

impl<T> Drop for Dropper<T> {
    fn drop(&mut self) {
        drop(unsafe { Box::from_raw(self.ptr) })
    }
}

impl<T: 'static> Dropper<T> {
    unsafe fn new(ptr: *mut T) -> Self {
        Self { ptr }
    }
}

#[test]
fn loom_refswap() {
    loom::model(|| {
        // thread closures must be 'static
        let static_init = Box::into_raw(Box::new(AtomicU32::new(0)));
        let dropper1 = unsafe { Dropper::new(static_init) };
        let second_thread_value = Box::into_raw(Box::new(AtomicU32::new(1)));
        let dropper2 = unsafe { Dropper::new(second_thread_value) };
        let r = Box::into_raw(Box::new(RefSwap::new(unsafe { &*static_init })));
        let dropper3 = unsafe { Dropper::new(r) };

        let handle1 = thread::spawn(move || first_thread(unsafe { &*r }));
        let handle2 =
            thread::spawn(move || second_thread(unsafe { &*r }, unsafe { &*second_thread_value }));
        let res1 = handle1.join();
        let res2 = handle2.join();

        // Avoid memory leak
        drop((dropper1, dropper2, dropper3));

        res1.unwrap();
        res2.unwrap();
    });

    for b in &BRANCHES_USED {
        assert!(b.load(Relaxed));
    }
}

fn first_thread(r: &RefSwap<'_, AtomicU32>) {
    match r.load(Relaxed).load(Relaxed) {
        0 => BRANCHES_USED[0].store(true, Relaxed),
        // This should not be reachable. Assuming that the below value.store(2, Relaxed) is a
        // non-atomic mutation, it *must* be observed otherwise a wrong state could be observed
        1 => unreachable!(
            "Write from second thread prior to storing to first thread must be observed"
        ),
        2 => BRANCHES_USED[1].store(true, Relaxed),
        _ => panic!(),
    }
}

fn second_thread<'a>(r: &RefSwap<'a, AtomicU32>, value: &'a AtomicU32) {
    // Assuming this is a non-atomic write. But this is done manually so that loom is "aware" of this mutation and can reorder its observation.
    value.store(2, Relaxed);
    r.store(value, Relaxed);
}
