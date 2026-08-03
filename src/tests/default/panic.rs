use crate::{MonoWaitGroup, WaitGroup};

#[cfg_attr(not(loom), test, should_panic)]
#[allow(unreachable_code, reason = "should panic")]
pub fn test_wg_panic_both() {
    let (_wg, _token) = WaitGroup::new();
    panic!();
    drop((_wg, _token));
}

#[cfg_attr(not(loom), test, should_panic)]
#[allow(unreachable_code, reason = "should panic")]
pub fn test_wg_panic_wg() {
    let (_wg, token) = WaitGroup::new();
    drop(token);
    panic!();
    drop(_wg);
}

#[cfg_attr(not(loom), test, should_panic)]
#[allow(unreachable_code, reason = "should panic")]
pub fn test_wg_panic_handle() {
    let (wg, _token) = WaitGroup::new();
    drop(wg);
    panic!();
    drop(_token);
}

#[cfg_attr(not(loom), test, should_panic)]
#[allow(unreachable_code, reason = "should panic")]
pub fn test_mono_wg_panic_both() {
    let (_wg, _token) = MonoWaitGroup::new();
    panic!();
    drop((_wg, _token));
}

#[cfg_attr(not(loom), test, should_panic)]
#[allow(unreachable_code, reason = "should panic")]
pub fn test_mono_wg_panic_wg() {
    let (_wg, token) = MonoWaitGroup::new();
    drop(token);
    panic!();
    drop(_wg);
}

#[cfg_attr(not(loom), test, should_panic)]
#[allow(unreachable_code, reason = "should panic")]
pub fn test_mono_wg_panic_handle() {
    let (wg, _token) = MonoWaitGroup::new();
    drop(wg);
    panic!();
    drop(_token);
}
