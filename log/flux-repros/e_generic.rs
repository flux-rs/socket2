#[cfg(flux)] extern crate flux_core;
pub fn make<T>() -> T { panic!() }
// Must fail, and does.
pub fn control() -> usize { core::hint::black_box(0usize) - 1 }
// Must fail, and doesn't: `x`'s type gets a kvar that nothing constrains, so it is
// solved to false and everything after is dead.
pub fn silent() -> usize {
    let _x: u32 = make::<u32>();
    core::hint::black_box(0usize) - 1
}
pub fn silent_zeroed() -> usize {
    let _x: u32 = unsafe { core::mem::zeroed() };
    core::hint::black_box(0usize) - 1
}
#[flux_rs::spec(fn() -> i32[1])]
pub fn proves_anything() -> i32 {
    let _x: u32 = make::<u32>();
    2
}
