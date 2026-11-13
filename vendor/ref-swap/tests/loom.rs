#![cfg(loom)]

use loom::thread;

use ref_swap::{OptionRefSwap, RefSwap};

#[allow(deprecated)]
use std::sync::atomic::ATOMIC_BOOL_INIT;
use std::sync::atomic::{AtomicBool, Ordering::*};

const BRANCHES_COUNT: usize = 3;
#[allow(deprecated)]
static BRANCHES_USED: [AtomicBool; BRANCHES_COUNT] = [ATOMIC_BOOL_INIT; BRANCHES_COUNT];

const STATIC_INIT: u32 = 0;

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
        let r = Box::into_raw(Box::new(RefSwap::new(&STATIC_INIT)));
        let dropper = unsafe { Dropper::new(r) };

        let handle1 = thread::spawn(move || first_thread(unsafe { &*r }));
        let handle2 = thread::spawn(move || second_thread(unsafe { &*r }));
        let res1 = handle1.join();
        let res2 = handle2.join();

        // Avoid memory leak
        drop(dropper);

        res1.unwrap();
        res2.unwrap();
    });

    for b in &BRANCHES_USED {
        assert!(b.load(Relaxed));
    }
}

fn first_thread(r: &RefSwap<'_, u32>) {
    match r.load(Relaxed) {
        0 => BRANCHES_USED[0].store(true, Relaxed),
        1 => BRANCHES_USED[1].store(true, Relaxed),
        2 => BRANCHES_USED[2].store(true, Relaxed),
        _ => panic!(),
    }
}

fn second_thread(r: &RefSwap<'_, u32>) {
    static ONE: u32 = 1;
    static OTHER_ONE: u32 = 1;
    r.store(&ONE, Relaxed);
    // Fails because the equality is a ptr equality
    r.compare_exchange(&OTHER_ONE, &2, Relaxed, Relaxed)
        .unwrap_err();
    r.compare_exchange(&ONE, &2, Relaxed, Relaxed).unwrap();
}

const OPTION_BRANCHES_COUNT: usize = 3;
#[allow(deprecated)]
static OPTIONS_BRANCHES_USED: [AtomicBool; OPTION_BRANCHES_COUNT] =
    [ATOMIC_BOOL_INIT; OPTION_BRANCHES_COUNT];

const OPTION_STATIC_INIT: Option<&u32> = None;

#[test]
fn loom_optionrefswap() {
    loom::model(|| {
        // thread closures must be 'static
        let r = Box::into_raw(Box::new(OptionRefSwap::new(OPTION_STATIC_INIT)));
        let dropper = unsafe { Dropper::new(r) };

        let handle1 = thread::spawn(move || option_first_thread(unsafe { &*r }));
        let handle2 = thread::spawn(move || option_second_thread(unsafe { &*r }));
        let res1 = handle1.join();
        let res2 = handle2.join();

        // Avoid memory leak
        drop(dropper);

        res1.unwrap();
        res2.unwrap();
    });

    for b in &OPTIONS_BRANCHES_USED {
        assert!(b.load(Relaxed));
    }
}

fn option_first_thread(r: &OptionRefSwap<'_, u32>) {
    match r.load(Relaxed) {
        None => OPTIONS_BRANCHES_USED[0].store(true, Relaxed),
        Some(1) => OPTIONS_BRANCHES_USED[1].store(true, Relaxed),
        Some(2) => OPTIONS_BRANCHES_USED[2].store(true, Relaxed),
        _ => panic!(),
    }
}

fn option_second_thread(r: &OptionRefSwap<'_, u32>) {
    static ONE: u32 = 1;
    static OTHER_ONE: u32 = 1;
    r.store(Some(&ONE), Relaxed);
    // Fails because the equality is a ptr equality
    r.compare_exchange(Some(&OTHER_ONE), Some(&2), Relaxed, Relaxed)
        .unwrap_err();
    r.compare_exchange(Some(&ONE), Some(&2), Relaxed, Relaxed)
        .unwrap();
}
